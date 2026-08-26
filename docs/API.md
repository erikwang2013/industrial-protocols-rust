# API 参考 — Industrial Protocols Rust

本文档包含工业协议集的核心公开 API：Protocol SDK Trait、内核组件、桥接层、错误类型与异步层。

返回 [README](../README.md)（多语言版本见 [语言目录](../README.md#多语言--languages)）。

## 1. Protocol SDK — 6 个 Trait

每个协议通过实现以下 trait 接入内核：

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // 协议特定地址类型
    type Value: ValueLike;      // 协议特定值类型
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

**关联类型**让地址和值类型安全：Modbus 用 `u16`，OPC UA 用 `String`，BACnet 用 `f32`。**Trait 拆分**让协议按需实现——只读传感器不需要 `ProtocolWrite`。

## 2. Kernel — 微内核

内核不实现任何协议，只提供共享基础设施：

| 组件 | 职责 | 关键 API |
|------|------|----------|
| `ProtocolRegistry` | 协议注册/查找/分类 | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | 设备连接生命周期 | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML 配置 + 运行时覆盖 | `load()`, `get_device()`, `set_override()` |
| `EventBus` | 发布/订阅事件 | `subscribe()`, `publish()` |
| `CircuitBreaker` | 熔断 Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | 固定/指数/线性/无 | `run()`, `start()` |
| `GatewayEngine` | 协议间数据转发规则 | `add_rule()`, `start()/stop()` |

连接管理支持三种策略：
```rust
pub enum ConnectionStrategy {
    Simple,      // 每次新建，用完释放
    Persistent,  // 单连接复用，断开自动重连
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — 传输抽象

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | 实现 | 依赖 |
|--------|------|------|
| `bridge-socket` | TCP 连接 + UDP 收发 | std |
| `bridge-serial` | 真实串口，可选 `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | 厂商 C SDK 动态加载 | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider 预置 | — |

## 4. Error — 结构化错误链

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 非法地址"
    CircuitOpen { device, retry_after },  // 熔断器已断开
    Context { context, source },  // 附加上文，形成错误链
    // ...
}
```

错误链示例：
```
读取 PLC-001 保持寄存器 40001 失败
  → 超时: device=plc-001, timeout=3000ms
    → 连接失败: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — 可选 tokio 包装

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// 异步事件流
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
