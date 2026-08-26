# API Reference — Industrial Protocols Rust

This document covers the core public API of the industrial protocol suite: Protocol SDK traits, kernel components, bridge layer, error types, and the async layer.

Back to [README](../README.md) (multi-language versions in [Languages](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Traits

Each protocol plugs into the kernel by implementing the following traits:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // Protocol-specific address type
    type Value: ValueLike;      // Protocol-specific value type
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

**Associated types** make addresses and values type-safe: Modbus uses `u16`, OPC UA uses `String`, BACnet uses `f32`. **Trait segregation** lets protocols implement only what they need — a read-only sensor does not require `ProtocolWrite`.

## 2. Kernel — Micro-Kernel

The kernel implements no protocols; it provides only shared infrastructure:

| Component | Responsibility | Key API |
|-----------|----------------|---------|
| `ProtocolRegistry` | Protocol registration/lookup/categorization | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Device connection lifecycle | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML config + runtime overrides | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Publish/subscribe events | `subscribe()`, `publish()` |
| `CircuitBreaker` | Circuit breaking Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fixed/Exponential/Linear/None | `run()`, `start()` |
| `GatewayEngine` | Inter-protocol data forwarding rules | `add_rule()`, `start()/stop()` |

Connection management supports three strategies:
```rust
pub enum ConnectionStrategy {
    Simple,      // New connection each time, released after use
    Persistent,  // Single connection reused, auto-reconnect on drop
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — Transport Abstraction

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementation | Dependency |
|--------|----------------|------------|
| `bridge-socket` | TCP connection + UDP send/receive | std |
| `bridge-serial` | Real serial port, optional `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | Dynamic loading of vendor C SDKs | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider presets | — |

## 4. Error — Structured Error Chain

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 illegal address"
    CircuitOpen { device, retry_after },  // Circuit breaker is open
    Context { context, source },  // Attach context, forming an error chain
    // ...
}
```

Error chain example:
```
Failed to read holding register 40001 of PLC-001
  → timeout: device=plc-001, timeout=3000ms
    → connection failed: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — Optional tokio Wrapper

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// Async event stream
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
