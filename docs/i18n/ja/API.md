# API リファレンス — Industrial Protocols Rust

このドキュメントには産業用プロトコル集の主要な公開 API（Protocol SDK Trait、カーネルコンポーネント、ブリッジレイヤー、エラー型、非同期レイヤー）を記載します。

[README](../README.md) に戻る（多言語版は[言語ディレクトリ](../README.md#多言語--languages)を参照）。

## 1. Protocol SDK — 6 つの Trait

各プロトコルは以下の trait を実装することでカーネルに組み込まれます：

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // プロトコル固有のアドレス型
    type Value: ValueLike;      // プロトコル固有の値型
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

**関連型**により、アドレスと値を型安全に扱えます：Modbus は `u16`、OPC UA は `String`、BACnet は `f32` を使用します。**Trait の分割**により、プロトコルは必要に応じて実装できます——読み取り専用センサーには `ProtocolWrite` は不要です。

## 2. Kernel — マイクロカーネル

カーネルはプロトコルを実装せず、共有インフラのみを提供します：

| コンポーネント | 役割 | 主要 API |
|------|------|----------|
| `ProtocolRegistry` | プロトコルの登録/検索/分類 | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | デバイス接続のライフサイクル | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML 設定 + 実行時上書き | `load()`, `get_device()`, `set_override()` |
| `EventBus` | イベントの公開/購読 | `subscribe()`, `publish()` |
| `CircuitBreaker` | サーキットブレーカー Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | 固定/指数/線形/なし | `run()`, `start()` |
| `GatewayEngine` | プロトコル間データ転送ルール | `add_rule()`, `start()/stop()` |

接続管理は 3 つの戦略をサポートします：
```rust
pub enum ConnectionStrategy {
    Simple,      // 毎回新規作成、使用後に解放
    Persistent,  // 単一接続を再利用、切断時は自動再接続
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — トランスポート抽象

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | 実装 | 依存 |
|--------|------|------|
| `bridge-socket` | TCP 接続 + UDP 送受信 | std |
| `bridge-serial` | 実シリアルポート、オプションの `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | ベンダー C SDK の動的ロード | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider プリセット | — |

## 4. Error — 構造化エラーチェーン

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 不正なアドレス"
    CircuitOpen { device, retry_after },  // サーキットブレーカーが開放済み
    Context { context, source },  // コンテキストを付加してエラーチェーンを形成
    // ...
}
```

エラーチェーンの例：
```
PLC-001 の保持レジスタ 40001 の読み取りに失敗
  → タイムアウト: device=plc-001, timeout=3000ms
    → 接続失敗: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — オプションの tokio ラッパー

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// 非同期イベントストリーム
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
