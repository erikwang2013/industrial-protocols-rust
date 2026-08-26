# API 참조 — Industrial Protocols Rust

이 문서는 산업용 프로토콜 집합의 핵심 공개 API를 다룹니다: Protocol SDK Trait, 커널 컴포넌트, 브리지 레이어, 오류 타입과 비동기 레이어.

[README](../README.md)로 돌아가기 (다국어 버전은 [언어 목록](../README.md#언어--languages) 참조).

## 1. Protocol SDK — 6개 Trait

각 프로토콜은 다음 trait을 구현하여 커널에 연결됩니다:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // 프로토콜별 주소 타입
    type Value: ValueLike;      // 프로토콜별 값 타입
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, ...) -> Result<Vec<Self::Value>>;
    fn read_range(&self, ...) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol { ... }
pub trait ProtocolSubscribe: Protocol { ... }
pub trait ProtocolDiscover { ... }
pub trait ProtocolHealth: Protocol { ... }
pub trait ProtocolDiagnostics: Protocol { ... }
```

**연관 타입**은 주소와 값 타입을 안전하게 만듭니다: Modbus는 `u16`, OPC UA는 `String`, BACnet은 `f32`를 사용합니다. **Trait 분리**는 프로토콜이 필요에 따라 구현하도록 합니다 — 읽기 전용 센서는 `ProtocolWrite`가 필요 없습니다.

## 2. Kernel — 마이크로커널

커널은 어떤 프로토콜도 구현하지 않으며, 공유 인프라만 제공합니다:

| 컴포넌트 | 역할 | 핵심 API |
|------|------|----------|
| `ProtocolRegistry` | 프로토콜 등록/조회/분류 | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | 장치 연결 수명 주기 | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML 설정 + 런타임 오버라이드 | `load()`, `get_device()`, `set_override()` |
| `EventBus` | 이벤트 발행/구독 | `subscribe()`, `publish()` |
| `CircuitBreaker` | 서킷 브레이커 Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | 고정/지수/선형/없음 | `run()`, `start()` |
| `GatewayEngine` | 프로토콜 간 데이터 전달 규칙 | `add_rule()`, `start()/stop()` |

연결 관리는 세 가지 전략을 지원합니다:
```rust
pub enum ConnectionStrategy {
    Simple,      // 매번 새로 생성, 사용 후 해제
    Persistent,  // 단일 연결 재사용, 끊기면 자동 재연결
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — 전송 추상화

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | 구현 | 의존성 |
|--------|------|------|
| `bridge-socket` | TCP 연결 + UDP 송수신 | std |
| `bridge-serial` | 실제 직렬 포트, 선택적 `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | 벤더 C SDK 동적 로딩 | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider 프리셋 | — |

## 4. Error — 구조화된 오류 체인

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 잘못된 주소"
    CircuitOpen { device, retry_after },  // 서킷 브레이커가 열림
    Context { context, source },  // 상위 문맥을 추가하여 오류 체인 형성
    // ...
}
```

오류 체인 예시:
```
PLC-001 보유 레지스터 40001 읽기 실패
  → 타임아웃: device=plc-001, timeout=3000ms
    → 연결 실패: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — 선택적 tokio 래퍼

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// 비동기 이벤트 스트림
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
