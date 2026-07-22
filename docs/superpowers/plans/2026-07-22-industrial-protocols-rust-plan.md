# Industrial Protocols Rust — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust industrial communication protocols library with micro-kernel + protocol SDK architecture, starting with 5 core protocols (Modbus, OPC UA, BACnet, EtherNet/IP, Profinet) in a Cargo workspace.

**Architecture:** Layered design — kernel defines 6 protocol traits and provides shared infrastructure (registry, connection management, config, circuit breaker, event bus, retry). Protocol crates implement the traits. Bridge layer abstracts transport (socket, serial). Pure synchronous core with optional tokio async wrapper.

**Tech Stack:** Rust 1.75+ (2024 edition), thiserror, serde + toml, Cargo workspace

---

## Build Order

```
Phase 1  → Task  1-2    Workspace + SDK traits
Phase 2  → Task  3-9    Kernel infrastructure
Phase 3  → Task 10-11   Bridge layer
Phase 4  → Task 12      Kernel assembly
Phase 5  → Task 13-15   Protocol crates
Phase 6  → Task 16-18   Skeletons + example + final verification
```

---

### Task 1: Workspace scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `kernel/Cargo.toml`
- Create: `kernel/src/lib.rs`
- Create: `kernel/src/sdk/mod.rs`
- Create: `kernel/src/error.rs`

- [ ] **Step 1: Create root workspace Cargo.toml**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "kernel",
    "bridge/socket",
    "bridge/serial",
    "bridge/can",
    "bridge/ffi",
    "bridge/vendor",
    "protocols/modbus",
    "protocols/opcua",
    "protocols/bacnet",
    "protocols/ethernet-ip",
    "protocols/profinet",
    "async-layer",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/erikwang2013/industrial-protocols-rust"

[workspace.dependencies]
thiserror = "2"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
log = "0.4"
```

- [ ] **Step 2: Create kernel Cargo.toml**

```toml
# kernel/Cargo.toml
[package]
name = "industrial-protocols-kernel"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
thiserror.workspace = true
serde.workspace = true
toml.workspace = true
log.workspace = true
```

- [ ] **Step 3: Verify workspace builds**

Run: `cargo check`
Expected: compiles successfully (kernel/src/lib.rs is empty)

- [ ] **Step 4: Commit**

```bash
git init && git add Cargo.toml kernel/Cargo.toml kernel/src/lib.rs && git commit -m "chore: scaffold Cargo workspace with kernel crate"
```

---

### Task 2: Protocol SDK — 6 core traits

**Files:**
- Create: `kernel/src/sdk/mod.rs`
- Create: `kernel/src/sdk/types.rs`
- Modify: `kernel/src/lib.rs`

- [ ] **Step 1: Write the SDK types (AddressLike, ValueLike, ProtocolCategory, helper structs)**

Create `kernel/src/sdk/types.rs`:

```rust
use std::fmt::Debug;

pub trait AddressLike: Clone + Debug + Send + Sync {}
impl<T: Clone + Debug + Send + Sync> AddressLike for T {}

pub trait ValueLike: Clone + Debug + Send + Sync {}
impl<T: Clone + Debug + Send + Sync> ValueLike for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolCategory {
    IndustrialEthernet,
    Fieldbus,
    IoTMessaging,
    AutomotiveBus,
    BuildingLighting,
    HardwareBridge,
    SystemBus,
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub device_id: String,
    pub protocol: String,
    pub address: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub state: HealthState,
    pub latency_ms: u64,
    pub last_seen: std::time::Instant,
    pub details: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsReport {
    pub device_id: String,
    pub uptime_secs: Option<u64>,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionUpdate {
    pub subscription_id: String,
    pub address: String,
    pub value: String,
    pub timestamp: std::time::Instant,
}
```

- [ ] **Step 2: Write the 6 SDK traits**

Replace `kernel/src/sdk/mod.rs`:

```rust
pub mod types;

use crate::error::Result;
use crate::connection::Connection;
use crate::bridge::Transport;
pub use types::*;

pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn read(&self, conn: &dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>>;
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn write(&self, conn: &dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()>;
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()>;
}

pub trait ProtocolSubscribe: Protocol {
    type SubscriptionId: Clone + Send + Sync;
    type Address: AddressLike;
    fn subscribe(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Self::SubscriptionId>;
    fn unsubscribe(&self, conn: &dyn Connection, sub_id: &Self::SubscriptionId) -> Result<()>;
    fn poll(&self, conn: &dyn Connection) -> Result<Vec<SubscriptionUpdate>>;
}

pub trait ProtocolDiscover {
    fn discover(&self, transport: &dyn Transport) -> Result<Vec<DiscoveredDevice>>;
}

pub trait ProtocolHealth: Protocol {
    fn health(&self, conn: &dyn Connection) -> Result<HealthStatus>;
}

pub trait ProtocolDiagnostics: Protocol {
    fn diagnostics(&self, conn: &dyn Connection) -> Result<DiagnosticsReport>;
}
```

- [ ] **Step 3: Write tests for trait objects**

Create `kernel/tests/sdk_tests.rs`:

```rust
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use std::io;

struct TestProtocol;
impl Protocol for TestProtocol {
    fn name(&self) -> &'static str { "test" }
    fn version(&self) -> &'static str { "0.1.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

impl ProtocolRead for TestProtocol {
    type Address = u16;
    type Value = u16;
    fn read(&self, _conn: &dyn Connection, _addr: &Self::Address) -> Result<Self::Value> { Ok(42) }
    fn read_batch(&self, _conn: &dyn Connection, _addrs: &[Self::Address]) -> Result<Vec<Self::Value>> { Ok(vec![1, 2, 3]) }
    fn read_range(&self, _conn: &dyn Connection, _start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> { Ok(vec![0; count as usize]) }
}

impl ProtocolHealth for TestProtocol {
    fn health(&self, _conn: &dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus { state: HealthState::Healthy, latency_ms: 1, last_seen: std::time::Instant::now(), details: "ok".into() })
    }
}

struct TestConnection;
impl Connection for TestConnection {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> { Ok(vec![]) }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
    fn is_alive(&self) -> bool { true }
}

#[test]
fn test_protocol_as_trait_object() {
    let p: Box<dyn Protocol> = Box::new(TestProtocol);
    assert_eq!(p.name(), "test");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}

#[test]
fn test_protocol_read_as_trait_object() {
    let p = TestProtocol;
    let _: u16 = ProtocolRead::read(&p, &TestConnection, &1).unwrap();
}
```

- [ ] **Step 4: Create placeholder modules so tests compile**

Add to `kernel/src/lib.rs`:

```rust
pub mod sdk;
pub mod error;
pub mod connection;
pub mod bridge;
```

Create `kernel/src/error.rs`:

```rust
pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug)]
pub enum ProtocolError {
    Connection(String),
    Timeout { device: String, timeout_ms: u64 },
    Parse { protocol: String, reason: String },
    DeviceError { device: String, code: u16, message: String },
    DeviceNotFound(String),
    Unsupported { protocol: String, operation: String },
    CircuitOpen { device: String, retry_after: std::time::Duration },
    Config(String),
    Other(Box<dyn std::error::Error + Send + Sync>),
}
```

Create `kernel/src/connection.rs`:

```rust
use std::io;
pub trait Connection: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn close(&mut self) -> io::Result<()>;
    fn is_alive(&self) -> bool;
}
```

Create `kernel/src/bridge.rs`:

```rust
use std::io;
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

- [ ] **Step 5: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add kernel/ && git commit -m "feat(kernel): add 6 core protocol SDK traits with tests"
```

---

### Task 3: Error type with thiserror + context chain

**Files:**
- Modify: `kernel/Cargo.toml` (thiserror already listed)
- Rewrite: `kernel/src/error.rs`
- Create: `kernel/tests/error_tests.rs`

- [ ] **Step 1: Write error tests**

Create `kernel/tests/error_tests.rs`:

```rust
use industrial_protocols_kernel::error::ProtocolError;
use std::time::Duration;

#[test]
fn test_connection_error_display() {
    let err = ProtocolError::Connection("192.168.1.10:502: connection refused".into());
    let msg = err.to_string();
    assert!(msg.contains("connection refused"));
}

#[test]
fn test_timeout_error_display() {
    let err = ProtocolError::Timeout { device: "plc-001".into(), timeout_ms: 3000 };
    let msg = err.to_string();
    assert!(msg.contains("plc-001"));
    assert!(msg.contains("3000"));
}

#[test]
fn test_context_chain_display() {
    let inner = ProtocolError::Timeout { device: "plc-001".into(), timeout_ms: 3000 };
    let outer = inner.context("failed to read holding register 40001 from PLC-001");
    let msg = outer.to_string();
    assert!(msg.contains("failed to read"));
    assert!(msg.contains("plc-001"));
}

#[test]
fn test_device_error_with_hex_code() {
    let err = ProtocolError::DeviceError { device: "plc-001".into(), code: 0x02, message: "illegal data address".into() };
    let msg = err.to_string();
    assert!(msg.contains("0x02"));
    assert!(msg.contains("illegal data address"));
}

#[test]
fn test_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = ProtocolError::from(io_err);
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn test_circuit_open_error() {
    let err = ProtocolError::CircuitOpen { device: "robot-003".into(), retry_after: Duration::from_secs(30) };
    let msg = err.to_string();
    assert!(msg.contains("robot-003"));
}
```

- [ ] **Step 2: Run tests — expect failure (old error type incompatible)**

Run: `cargo test -p industrial-protocols-kernel --test error_tests`
Expected: compile errors

- [ ] **Step 3: Rewrite error.rs with thiserror and context chain**

Rewrite `kernel/src/error.rs`:

```rust
use std::time::Duration;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("timeout: device={device}, timeout={timeout_ms}ms")]
    Timeout { device: String, timeout_ms: u64 },

    #[error("frame parse error: protocol={protocol}, reason={reason}")]
    Parse { protocol: String, reason: String },

    #[error("device error: device={device}, code={code:#04X}, message={message}")]
    DeviceError { device: String, code: u16, message: String },

    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("operation unsupported: protocol={protocol}, operation={operation}")]
    Unsupported { protocol: String, operation: String },

    #[error("circuit breaker open: device={device}, retry_after={retry_after:?}")]
    CircuitOpen { device: String, retry_after: Duration },

    #[error("config error: {0}")]
    Config(String),

    #[error("protocol-specific error: {0}")]
    ProtocolSpecific(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<ProtocolError>,
    },
}

impl ProtocolError {
    pub fn context(self, ctx: impl Into<String>) -> Self {
        ProtocolError::Context {
            context: ctx.into(),
            source: Box::new(self),
        }
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        ProtocolError::Connection(e.to_string())
    }
}
```

- [ ] **Step 4: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add kernel/src/error.rs kernel/tests/error_tests.rs && git commit -m "feat(kernel): add thiserror-based ProtocolError with context chain"
```

---

### Task 4: RetryConfig with 4 strategies

**Files:**
- Create: `kernel/src/retry.rs`
- Create: `kernel/tests/retry_tests.rs`
- Modify: `kernel/src/lib.rs` (add `pub mod retry;`)

- [ ] **Step 1: Write retry tests**

Create `kernel/tests/retry_tests.rs`:

```rust
use industrial_protocols_kernel::retry::*;
use std::time::Duration;

#[test]
fn test_fixed_retry_always_same_delay() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 100 } };
    let mut state = config.start();
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), None);
}

#[test]
fn test_exponential_backoff() {
    let config = RetryConfig { max_attempts: 4, strategy: RetryStrategy::Exponential { base_ms: 100, max_ms: 2000 } };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays.len(), 3);
    assert!(delays[0] >= Duration::from_millis(100));
    assert!(delays[1] >= delays[0]);
    assert!(delays[2] <= Duration::from_millis(2000));
}

#[test]
fn test_linear_backoff() {
    let config = RetryConfig { max_attempts: 4, strategy: RetryStrategy::Linear { base_ms: 100, step_ms: 50 } };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays[0], Duration::from_millis(100));
    assert_eq!(delays[1], Duration::from_millis(150));
    assert_eq!(delays[2], Duration::from_millis(200));
}

#[test]
fn test_no_retry() {
    let config = RetryConfig { max_attempts: 1, strategy: RetryStrategy::None };
    let delays: Vec<_> = config.start().collect();
    assert!(delays.is_empty());
}

#[test]
fn test_run_succeeds_first_try() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 10 } };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        Ok::<i32, &str>(42)
    });
    assert_eq!(result, Ok(42));
    assert_eq!(calls, 1);
}

#[test]
fn test_run_retries_then_fails() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 1 } };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        Err::<i32, &str>("fail")
    });
    assert!(result.is_err());
    assert_eq!(calls, 3);
}

#[test]
fn test_run_retries_then_succeeds() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 1 } };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        if calls < 2 { Err("fail") } else { Ok(99) }
    });
    assert_eq!(result, Ok(99));
    assert_eq!(calls, 2);
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test retry_tests`
Expected: compile error

- [ ] **Step 3: Implement RetryConfig**

Create `kernel/src/retry.rs`:

```rust
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub strategy: RetryStrategy,
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Fixed { delay_ms: u64 },
    Exponential { base_ms: u64, max_ms: u64 },
    Linear { base_ms: u64, step_ms: u64 },
    None,
}

impl RetryConfig {
    pub fn start(&self) -> RetryState {
        RetryState {
            config: self.clone(),
            attempt: 0,
            current_delay: self.initial_delay(),
        }
    }

    fn initial_delay(&self) -> Duration {
        match &self.strategy {
            RetryStrategy::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            RetryStrategy::Exponential { base_ms, .. } => Duration::from_millis(*base_ms),
            RetryStrategy::Linear { base_ms, .. } => Duration::from_millis(*base_ms),
            RetryStrategy::None => Duration::ZERO,
        }
    }

    pub fn run<F, T, E>(&self, mut f: F) -> std::result::Result<T, E>
    where
        F: FnMut() -> std::result::Result<T, E>,
    {
        let mut state = self.start();
        loop {
            match f() {
                Ok(val) => return Ok(val),
                Err(e) => match state.next_delay() {
                    Some(delay) => std::thread::sleep(delay),
                    None => return Err(e),
                },
            }
        }
    }
}

pub struct RetryState {
    config: RetryConfig,
    attempt: u32,
    current_delay: Duration,
}

impl RetryState {
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempt >= self.config.max_attempts.saturating_sub(1) {
            return None;
        }
        let delay = self.current_delay;
        self.advance();
        Some(delay)
    }

    fn advance(&mut self) {
        self.attempt += 1;
        self.current_delay = match &self.config.strategy {
            RetryStrategy::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            RetryStrategy::Exponential { base_ms, max_ms } => {
                let next = self.current_delay.as_millis() as u64 * 2;
                Duration::from_millis(next.min(*max_ms))
            }
            RetryStrategy::Linear { base_ms, step_ms } => {
                Duration::from_millis(base_ms + step_ms * (self.attempt as u64 + 1))
            }
            RetryStrategy::None => Duration::ZERO,
        };
    }
}

impl Iterator for RetryState {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> {
        self.next_delay()
    }
}
```

- [ ] **Step 4: Add `pub mod retry;` to `kernel/src/lib.rs`**

```rust
pub mod retry;
```

- [ ] **Step 5: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add kernel/src/retry.rs kernel/tests/retry_tests.rs kernel/src/lib.rs && git commit -m "feat(kernel): add RetryConfig with Fixed/Exponential/Linear/None strategies"
```

---

### Task 5: EventBus — publish/subscribe

**Files:**
- Create: `kernel/src/event.rs`
- Create: `kernel/tests/event_tests.rs`
- Modify: `kernel/src/lib.rs` (add `pub mod event;`)

- [ ] **Step 1: Write event tests**

Create `kernel/tests/event_tests.rs`:

```rust
use industrial_protocols_kernel::event::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_subscribe_and_publish() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Connection, move |event| {
        r.lock().unwrap().push(format!("{:?}", event));
    });
    let event = ProtocolEvent::Connected {
        device_id: "plc-001".into(),
        protocol: "modbus".into(),
        timestamp: std::time::Instant::now(),
    };
    bus.publish(event);
    std::thread::sleep(std::time::Duration::from_millis(10));
    let msgs = received.lock().unwrap();
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("plc-001"));
}

#[test]
fn test_error_event() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Error, move |event| {
        r.lock().unwrap().push(format!("{:?}", event));
    });
    bus.publish(ProtocolEvent::Error {
        device_id: "plc-001".into(),
        error: "timeout".into(),
        severity: Severity::Critical,
    });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(received.lock().unwrap().len(), 1);
}

#[test]
fn test_health_changed_event() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Health, move |event| {
        r.lock().unwrap().push(format!("{:?}", event));
    });
    bus.publish(ProtocolEvent::HealthChanged {
        device_id: "plc-001".into(),
        from: HealthState::Healthy,
        to: HealthState::Unhealthy,
    });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(received.lock().unwrap().len(), 1);
}

#[test]
fn test_event_topic_routing() {
    let event = ProtocolEvent::ReadCompleted { device_id: "plc-001".into(), address: "40001".into(), latency_ms: 5 };
    assert_eq!(event.topic(), EventTopic::Connection);
    let event = ProtocolEvent::CircuitBreakerTriggered { device_id: "robot".into(), failures: 5 };
    assert_eq!(event.topic(), EventTopic::Error);
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test event_tests`
Expected: compile error

- [ ] **Step 3: Implement EventBus**

Create `kernel/src/event.rs`:

```rust
use std::sync::{Arc, Mutex};

pub type EventHandler = Box<dyn Fn(ProtocolEvent) + Send + 'static>;

pub struct EventBus {
    handlers: Arc<Mutex<Vec<(EventTopic, EventHandler)>>>,
}

impl EventBus {
    pub fn new(_capacity: usize) -> Self {
        EventBus { handlers: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn subscribe(&self, topic: EventTopic, handler: impl Fn(ProtocolEvent) + Send + 'static) {
        self.handlers.lock().unwrap().push((topic, Box::new(handler)));
    }

    pub fn publish(&self, event: ProtocolEvent) {
        let topic = event.topic();
        let handlers = self.handlers.lock().unwrap();
        for (t, handler) in handlers.iter() {
            if *t == topic {
                handler(event.clone());
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTopic {
    Connection,
    Error,
    Health,
}

#[derive(Debug, Clone)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    Connected { device_id: String, protocol: String, timestamp: std::time::Instant },
    Disconnected { device_id: String, reason: String },
    ReadCompleted { device_id: String, address: String, latency_ms: u64 },
    WriteCompleted { device_id: String, address: String, latency_ms: u64 },
    Error { device_id: String, error: String, severity: Severity },
    HealthChanged { device_id: String, from: HealthState, to: HealthState },
    CircuitBreakerTriggered { device_id: String, failures: u32 },
}

impl ProtocolEvent {
    pub fn topic(&self) -> EventTopic {
        match self {
            ProtocolEvent::Connected { .. }
            | ProtocolEvent::Disconnected { .. }
            | ProtocolEvent::ReadCompleted { .. }
            | ProtocolEvent::WriteCompleted { .. } => EventTopic::Connection,
            ProtocolEvent::Error { .. }
            | ProtocolEvent::CircuitBreakerTriggered { .. } => EventTopic::Error,
            ProtocolEvent::HealthChanged { .. } => EventTopic::Health,
        }
    }
}
```

- [ ] **Step 4: Add `pub mod event;` to `kernel/src/lib.rs`**

```rust
pub mod event;
```

- [ ] **Step 5: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add kernel/src/event.rs kernel/tests/event_tests.rs kernel/src/lib.rs && git commit -m "feat(kernel): add EventBus with publish/subscribe and 3 topics"
```

---

### Task 6: CircuitBreaker

**Files:**
- Create: `kernel/src/circuit.rs`
- Create: `kernel/tests/circuit_tests.rs`
- Modify: `kernel/src/lib.rs` (add `pub mod circuit;`)

- [ ] **Step 1: Write circuit breaker tests**

Create `kernel/tests/circuit_tests.rs`:

```rust
use industrial_protocols_kernel::circuit::*;
use std::time::Duration;

#[test]
fn test_starts_closed() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_opens_after_threshold_failures() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_success_resets_closed() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1");
    cb.record_success("dev1");
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_call_success() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 2, cooldown: Duration::from_secs(30) });
    let result = cb.call("dev1", || Ok::<i32, &str>(42));
    assert_eq!(result, Ok(42));
}

#[test]
fn test_call_trips_breaker() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 2, cooldown: Duration::from_millis(100) });
    assert!(cb.call("dev1", || Err::<i32, &str>("fail1")).is_err());
    assert!(cb.call("dev1", || Err::<i32, &str>("fail2")).is_err());
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    let result = cb.call("dev1", || Ok::<i32, &str>(42));
    assert!(result.is_err());
}

#[test]
fn test_independent_devices() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 2, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    assert_eq!(cb.state("dev2"), CircuitState::Closed);
}

#[test]
fn test_half_open_after_cooldown() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 1, cooldown: Duration::from_millis(10) });
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(cb.state("dev1"), CircuitState::HalfOpen);
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test circuit_tests`
Expected: compile error

- [ ] **Step 3: Implement CircuitBreaker**

Create `kernel/src/circuit.rs`:

```rust
use crate::error::{ProtocolError, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CircuitConfig {
    pub threshold: u32,
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct DeviceCircuit {
    failures: u32,
    state: CircuitState,
    opened_at: Option<Instant>,
}

pub struct CircuitBreaker {
    config: CircuitConfig,
    devices: Mutex<HashMap<String, DeviceCircuit>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitConfig) -> Self {
        CircuitBreaker { config, devices: Mutex::new(HashMap::new()) }
    }

    pub fn state(&self, device_id: &str) -> CircuitState {
        let mut devices = self.devices.lock().unwrap();
        let entry = devices.entry(device_id.to_string()).or_insert(DeviceCircuit {
            failures: 0, state: CircuitState::Closed, opened_at: None,
        });
        self.transition_if_ready(entry);
        entry.state
    }

    pub fn record_success(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        if let Some(entry) = devices.get_mut(device_id) {
            entry.failures = 0;
            entry.state = CircuitState::Closed;
            entry.opened_at = None;
        }
    }

    pub fn record_failure(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        let entry = devices.entry(device_id.to_string()).or_insert(DeviceCircuit {
            failures: 0, state: CircuitState::Closed, opened_at: None,
        });
        entry.failures += 1;
        if entry.failures >= self.config.threshold {
            entry.state = CircuitState::Open;
            entry.opened_at = Some(Instant::now());
        }
    }

    pub fn call<F, T>(&self, device_id: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        match self.state(device_id) {
            CircuitState::Open => {
                let devices = self.devices.lock().unwrap();
                let retry_after = devices.get(device_id)
                    .and_then(|e| e.opened_at)
                    .map(|t| self.config.cooldown.saturating_sub(t.elapsed()))
                    .unwrap_or(self.config.cooldown);
                Err(ProtocolError::CircuitOpen { device: device_id.into(), retry_after })
            }
            CircuitState::Closed | CircuitState::HalfOpen => match f() {
                Ok(val) => { self.record_success(device_id); Ok(val) }
                Err(e) => { self.record_failure(device_id); Err(e) }
            },
        }
    }

    fn transition_if_ready(&self, entry: &mut DeviceCircuit) {
        if entry.state == CircuitState::Open {
            if let Some(opened_at) = entry.opened_at {
                if opened_at.elapsed() >= self.config.cooldown {
                    entry.state = CircuitState::HalfOpen;
                }
            }
        }
    }
}
```

- [ ] **Step 4: Add `pub mod circuit;` to `kernel/src/lib.rs`**

```rust
pub mod circuit;
```

- [ ] **Step 5: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add kernel/src/circuit.rs kernel/tests/circuit_tests.rs kernel/src/lib.rs && git commit -m "feat(kernel): add CircuitBreaker with closed/open/half-open states"
```

---

### Task 7: ConfigRepository — TOML loading + DeviceConfig

**Files:**
- Create: `kernel/src/config.rs`
- Create: `kernel/tests/config_tests.rs`
- Create: `kernel/tests/fixtures/sample.toml`
- Modify: `kernel/src/lib.rs`

- [ ] **Step 1: Create test fixture**

Create `kernel/tests/fixtures/sample.toml`:

```toml
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "192.168.1.10"
port = 502
unit_id = 1
timeout_ms = 3000
retry = { max = 3, strategy = "exponential", base_ms = 100, max_ms = 5000 }
connection_strategy = "persistent"
health_check = true

[devices.chiller-002]
protocol = "bacnet"
variant = "ip"
host = "192.168.1.20"
port = 47808
device_instance = 1001
timeout_ms = 5000

[defaults]
timeout_ms = 5000
retry_max = 3
retry_strategy = "exponential"
health_check_interval_s = 30
circuit_breaker_threshold = 5
circuit_breaker_cooldown_s = 30
```

- [ ] **Step 2: Write config tests**

Create `kernel/tests/config_tests.rs`:

```rust
use industrial_protocols_kernel::config::*;

#[test]
fn test_load_toml_from_file() {
    let repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    let dev = repo.get_device("plc-001").unwrap();
    assert_eq!(dev.protocol, "modbus");
    assert_eq!(dev.variant, "tcp");
    assert_eq!(dev.host.as_deref(), Some("192.168.1.10"));
    assert_eq!(dev.port, Some(502));
    assert_eq!(dev.unit_id, 1);
    assert_eq!(dev.timeout_ms, 3000);
    assert_eq!(dev.retry.max, 3);
    assert_eq!(dev.connection_strategy, ConnectionStrategyConfig::Persistent);
    assert!(dev.health_check);
}

#[test]
fn test_device_not_found() {
    let repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    assert!(repo.get_device("nonexistent").is_none());
}

#[test]
fn test_list_all_devices() {
    let repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    let ids = repo.device_ids();
    assert!(ids.contains(&"plc-001".to_string()));
    assert!(ids.contains(&"chiller-002".to_string()));
}

#[test]
fn test_default_values() {
    let repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    assert_eq!(repo.defaults().timeout_ms, 5000);
    assert_eq!(repo.defaults().circuit_breaker_threshold, 5);
}

#[test]
fn test_set_override() {
    let mut repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    repo.set_override("plc-001", "timeout_ms", toml::Value::Integer(9999));
    let dev = repo.get_device("plc-001").unwrap();
    assert_eq!(dev.timeout_ms, 9999);
}

#[test]
fn test_extras_preserve_protocol_specific_fields() {
    let repo = ConfigRepository::load("kernel/tests/fixtures/sample.toml").unwrap();
    let dev = repo.get_device("chiller-002").unwrap();
    assert_eq!(dev.extras.get("device_instance").unwrap().as_integer(), Some(1001));
}
```

- [ ] **Step 3: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test config_tests`
Expected: compile error

- [ ] **Step 4: Implement ConfigRepository**

Create `kernel/src/config.rs`:

```rust
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub devices: HashMap<String, DeviceConfigRaw>,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfigRaw {
    pub protocol: String,
    #[serde(default)]
    pub variant: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub serial_port: Option<String>,
    pub baud_rate: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub unit_id: Option<u16>,
    pub retry: Option<RetryConfigRaw>,
    pub connection_strategy: Option<ConnectionStrategyConfig>,
    pub health_check: Option<bool>,
    #[serde(flatten)]
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfigRaw {
    pub max: u32,
    pub strategy: String,
    #[serde(default)]
    pub base_ms: u64,
    #[serde(default)]
    pub max_ms: u64,
    #[serde(default)]
    pub step_ms: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStrategyConfig {
    Simple,
    Persistent,
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_max")]
    pub retry_max: u32,
    #[serde(default = "default_retry_strategy")]
    pub retry_strategy: String,
    #[serde(default = "default_health_interval")]
    pub health_check_interval_s: u64,
    #[serde(default = "default_cb_threshold")]
    pub circuit_breaker_threshold: u32,
    #[serde(default = "default_cb_cooldown")]
    pub circuit_breaker_cooldown_s: u64,
}

fn default_timeout() -> u64 { 5000 }
fn default_retry_max() -> u32 { 3 }
fn default_retry_strategy() -> String { "exponential".into() }
fn default_health_interval() -> u64 { 30 }
fn default_cb_threshold() -> u32 { 5 }
fn default_cb_cooldown() -> u64 { 30 }

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub protocol: String,
    pub variant: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub serial_port: Option<String>,
    pub baud_rate: Option<u32>,
    pub timeout_ms: u64,
    pub unit_id: u16,
    pub retry: RetryConfigResolved,
    pub connection_strategy: ConnectionStrategyConfig,
    pub health_check: bool,
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct RetryConfigResolved {
    pub max: u32,
    pub strategy: String,
    pub base_ms: u64,
    pub max_ms: u64,
    pub step_ms: u64,
}

pub struct ConfigRepository {
    raw: ConfigFile,
    overrides: HashMap<String, HashMap<String, toml::Value>>,
}

impl ConfigRepository {
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let raw: ConfigFile = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ConfigRepository { raw, overrides: HashMap::new() })
    }

    pub fn from_str(toml_str: &str) -> std::io::Result<Self> {
        let raw: ConfigFile = toml::from_str(toml_str)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ConfigRepository { raw, overrides: HashMap::new() })
    }

    pub fn get_device(&self, id: &str) -> Option<DeviceConfig> {
        let raw = self.raw.devices.get(id)?;
        let overrides = self.overrides.get(id);
        let timeout_ms = overrides.and_then(|o| o.get("timeout_ms"))
            .and_then(|v| v.as_integer()).map(|v| v as u64)
            .unwrap_or(raw.timeout_ms.unwrap_or(self.raw.defaults.timeout_ms));
        let retry = raw.retry.clone().unwrap_or(RetryConfigRaw {
            max: self.raw.defaults.retry_max,
            strategy: self.raw.defaults.retry_strategy.clone(),
            base_ms: 100, max_ms: 5000, step_ms: 50,
        });
        Some(DeviceConfig {
            protocol: raw.protocol.clone(),
            variant: raw.variant.clone(),
            host: raw.host.clone(),
            port: raw.port,
            serial_port: raw.serial_port.clone(),
            baud_rate: raw.baud_rate,
            timeout_ms,
            unit_id: overrides.and_then(|o| o.get("unit_id"))
                .and_then(|v| v.as_integer()).map(|v| v as u16)
                .unwrap_or(raw.unit_id.unwrap_or(1)),
            retry: RetryConfigResolved {
                max: retry.max,
                strategy: retry.strategy,
                base_ms: retry.base_ms,
                max_ms: retry.max_ms,
                step_ms: retry.step_ms,
            },
            connection_strategy: raw.connection_strategy.clone().unwrap_or(ConnectionStrategyConfig::Persistent),
            health_check: raw.health_check.unwrap_or(true),
            extras: raw.extras.clone(),
        })
    }

    pub fn device_ids(&self) -> Vec<String> {
        self.raw.devices.keys().cloned().collect()
    }

    pub fn defaults(&self) -> &DefaultsConfig {
        &self.raw.defaults
    }

    pub fn set_override(&mut self, device_id: &str, key: &str, value: toml::Value) {
        self.overrides.entry(device_id.to_string()).or_default().insert(key.to_string(), value);
    }

    pub fn clear_overrides(&mut self, device_id: &str) {
        self.overrides.remove(device_id);
    }
}
```

- [ ] **Step 5: Add `pub mod config;` to `kernel/src/lib.rs`**

```rust
pub mod config;
```

- [ ] **Step 6: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add kernel/src/config.rs kernel/tests/config_tests.rs kernel/tests/fixtures/ kernel/src/lib.rs && git commit -m "feat(kernel): add ConfigRepository with TOML loading and runtime overrides"
```

---

### Task 8: ProtocolRegistry

**Files:**
- Create: `kernel/src/registry.rs`
- Create: `kernel/tests/registry_tests.rs`
- Modify: `kernel/src/lib.rs`

- [ ] **Step 1: Write registry tests**

Create `kernel/tests/registry_tests.rs`:

```rust
use industrial_protocols_kernel::registry::*;
use industrial_protocols_kernel::sdk::*;

struct ModbusProtocol;
impl Protocol for ModbusProtocol {
    fn name(&self) -> &'static str { "modbus" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp", "rtu", "ascii"] }
}

struct OpcuaProtocol;
impl Protocol for OpcuaProtocol {
    fn name(&self) -> &'static str { "opcua" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["binary"] }
}

#[test]
fn test_register_and_get() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    assert!(reg.get("modbus").is_some());
    assert!(reg.get("opcua").is_some());
    assert!(reg.get("bacnet").is_none());
}

#[test]
fn test_list_all() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    assert_eq!(reg.list_all().len(), 2);
}

#[test]
fn test_list_by_category() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    assert_eq!(reg.list_by_category(ProtocolCategory::IndustrialEthernet).len(), 2);
    assert_eq!(reg.list_by_category(ProtocolCategory::Fieldbus).len(), 0);
}

#[test]
fn test_protocol_names() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    let names = reg.protocol_names();
    assert!(names.contains(&"modbus".to_string()));
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test registry_tests`
Expected: compile error

- [ ] **Step 3: Implement ProtocolRegistry**

Create `kernel/src/registry.rs`:

```rust
use crate::sdk::{Protocol, ProtocolCategory};
use std::collections::HashMap;

pub struct ProtocolRegistry {
    protocols: HashMap<String, Box<dyn Protocol>>,
}

impl ProtocolRegistry {
    pub fn new() -> Self {
        ProtocolRegistry { protocols: HashMap::new() }
    }

    pub fn register(&mut self, protocol: Box<dyn Protocol>) {
        self.protocols.insert(protocol.name().to_string(), protocol);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Protocol> {
        self.protocols.get(name).map(|p| p.as_ref())
    }

    pub fn list_all(&self) -> Vec<&dyn Protocol> {
        self.protocols.values().map(|p| p.as_ref()).collect()
    }

    pub fn list_by_category(&self, category: ProtocolCategory) -> Vec<&dyn Protocol> {
        self.protocols.values().filter(|p| p.category() == category).map(|p| p.as_ref()).collect()
    }

    pub fn protocol_names(&self) -> Vec<String> {
        self.protocols.keys().cloned().collect()
    }
}

impl Default for ProtocolRegistry {
    fn default() -> Self { Self::new() }
}
```

- [ ] **Step 4: Add `pub mod registry;` to `kernel/src/lib.rs`**

```rust
pub mod registry;
```

- [ ] **Step 5: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add kernel/src/registry.rs kernel/tests/registry_tests.rs kernel/src/lib.rs && git commit -m "feat(kernel): add ProtocolRegistry for protocol registration and lookup"
```

---

### Task 9: ConnectionManager — 3 strategies

**Files:**
- Rewrite: `kernel/src/connection.rs` (full Connection trait + ConnectionManager)
- Create: `kernel/tests/connection_tests.rs`

- [ ] **Step 1: Write connection tests**

Create `kernel/tests/connection_tests.rs`:

```rust
use industrial_protocols_kernel::connection::*;
use std::io;
use std::sync::{Arc, Mutex};

struct MockTransport {
    alive: bool,
    send_fn: Arc<Mutex<Vec<Vec<u8>>>>,
    responses: Vec<Vec<u8>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockTransport {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockTransport { alive: true, send_fn: Arc::new(Mutex::new(Vec::new())), responses, call_count: Arc::new(Mutex::new(0)) }
    }
}

impl Connection for MockTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.send_fn.lock().unwrap().push(data.to_vec());
        let idx = *self.call_count.lock().unwrap();
        *self.call_count.lock().unwrap() += 1;
        if idx < self.responses.len() { Ok(self.responses[idx].clone()) } else { Ok(vec![]) }
    }
    fn close(&mut self) -> io::Result<()> { self.alive = false; Ok(()) }
    fn is_alive(&self) -> bool { self.alive }
}

#[test]
fn test_simple_strategy_creates_new_each_time() {
    let manager = ConnectionManager::new(ConnectionStrategy::Simple);
    let id = manager.connect("plc-001", || Ok(Box::new(MockTransport::new(vec![vec![1, 2, 3]])))).unwrap();
    assert_eq!(id, "plc-001");
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_persistent_strategy_reuses_connection() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager.connect("plc-001", || Ok(Box::new(MockTransport::new(vec![vec![1, 2, 3]])))).unwrap();
    assert!(manager.is_connected("plc-001"));
    let id2 = manager.connect("plc-001", || panic!("should not be called")).unwrap();
    assert_eq!(id2, "plc-001");
}

#[test]
fn test_pooled_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Pooled { min: 1, max: 2, timeout_ms: 5000 });
    manager.connect("plc-001", || Ok(Box::new(MockTransport::new(vec![vec![42]])))).unwrap();
    assert!(manager.is_connected("plc-001"));
    manager.disconnect("plc-001").unwrap();
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_health_check() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager.connect("plc-001", || Ok(Box::new(MockTransport::new(vec![])))).unwrap();
    let health = manager.health("plc-001").unwrap();
    assert!(health.latency_ms > 0);
}

#[test]
fn test_health_unknown_device() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    assert!(manager.health("nonexistent").is_none());
}

#[test]
fn test_shutdown_disconnects_all() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager.connect("plc-001", || Ok(Box::new(MockTransport::new(vec![])))).unwrap();
    manager.connect("plc-002", || Ok(Box::new(MockTransport::new(vec![])))).unwrap();
    manager.shutdown().unwrap();
    assert!(!manager.is_connected("plc-001"));
    assert!(!manager.is_connected("plc-002"));
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test connection_tests`
Expected: compile error (connection.rs currently has stub trait only)

- [ ] **Step 3: Implement full Connection trait + ConnectionManager**

Rewrite `kernel/src/connection.rs`:

```rust
use std::collections::HashMap;
use std::io;
use std::sync::Mutex;
use std::time::Instant;

pub trait Connection: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn close(&mut self) -> io::Result<()>;
    fn is_alive(&self) -> bool;
}

pub type ConnectionFactory = Box<dyn Fn() -> io::Result<Box<dyn Connection>> + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ConnectionStrategy {
    Simple,
    Persistent,
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}

struct ManagedConn {
    conn: Box<dyn Connection>,
}

pub struct ConnectionManager {
    strategy: ConnectionStrategy,
    connections: Mutex<HashMap<String, ManagedConn>>,
}

pub struct HealthInfo {
    pub connected: bool,
    pub latency_ms: u64,
}

impl ConnectionManager {
    pub fn new(strategy: ConnectionStrategy) -> Self {
        ConnectionManager { strategy, connections: Mutex::new(HashMap::new()) }
    }

    pub fn connect(&self, device_id: &str, factory: ConnectionFactory) -> io::Result<String> {
        let mut conns = self.connections.lock().unwrap();
        match self.strategy {
            ConnectionStrategy::Simple => {
                let mut conn = factory()?;
                conn.close()?;
                Ok(device_id.to_string())
            }
            ConnectionStrategy::Persistent | ConnectionStrategy::Pooled { .. } => {
                if conns.contains_key(device_id) {
                    return Ok(device_id.to_string());
                }
                let conn = factory()?;
                conns.insert(device_id.to_string(), ManagedConn { conn });
                Ok(device_id.to_string())
            }
        }
    }

    pub fn disconnect(&self, device_id: &str) -> io::Result<()> {
        let mut conns = self.connections.lock().unwrap();
        if let Some(mut managed) = conns.remove(device_id) {
            managed.conn.close()
        } else {
            Ok(())
        }
    }

    pub fn is_connected(&self, device_id: &str) -> bool {
        let conns = self.connections.lock().unwrap();
        conns.get(device_id).map(|c| c.conn.is_alive()).unwrap_or(false)
    }

    pub fn health(&self, device_id: &str) -> Option<HealthInfo> {
        let mut conns = self.connections.lock().unwrap();
        let managed = conns.get_mut(device_id)?;
        let start = Instant::now();
        let connected = managed.conn.is_alive();
        let latency_ms = start.elapsed().as_millis() as u64;
        Some(HealthInfo { connected, latency_ms })
    }

    pub fn send(&self, device_id: &str, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut conns = self.connections.lock().unwrap();
        let managed = conns.get_mut(device_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("device not connected: {device_id}")))?;
        managed.conn.send(data)
    }

    pub fn shutdown(&self) -> io::Result<()> {
        let mut conns = self.connections.lock().unwrap();
        for (_, mut managed) in conns.drain() {
            let _ = managed.conn.close();
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Run tests — should pass**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add kernel/src/connection.rs kernel/tests/connection_tests.rs && git commit -m "feat(kernel): add ConnectionManager with Simple/Persistent/Pooled strategies"
```

---

### Task 10: Bridge — socket transport

**Files:**
- Create: `bridge/socket/Cargo.toml`
- Create: `bridge/socket/src/lib.rs`

- [ ] **Step 1: Create bridge-socket crate**

Create `bridge/socket/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bridge-socket"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

- [ ] **Step 2: Implement TCP and UDP transports**

Create `bridge/socket/src/lib.rs`:

```rust
use industrial_protocols_kernel::bridge::Transport;
use std::io::{self, Read, Write};
use std::net::{TcpStream, UdpSocket, ToSocketAddrs};
use std::time::Duration;

pub struct TcpTransport {
    stream: TcpStream,
    timeout: Duration,
}

impl TcpTransport {
    pub fn connect(addr: impl ToSocketAddrs, timeout_ms: u64) -> io::Result<Self> {
        let timeout = Duration::from_millis(timeout_ms);
        let stream = TcpStream::connect_timeout(
            &addr.to_socket_addrs()?.next().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address resolved"))?,
            timeout,
        )?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(TcpTransport { stream, timeout })
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.stream.write_all(data)?;
        let mut buf = vec![0u8; 4096];
        let n = self.stream.read(&mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }

    fn is_open(&self) -> bool { true }

    fn close(&mut self) -> io::Result<()> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

pub struct UdpTransport {
    socket: UdpSocket,
    peer: std::net::SocketAddr,
}

impl UdpTransport {
    pub fn bind_and_connect(bind_addr: &str, peer_addr: &str, timeout_ms: u64) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        let peer = peer_addr.to_socket_addrs()?.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address resolved"))?;
        let timeout = Duration::from_millis(timeout_ms);
        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;
        Ok(UdpTransport { socket, peer })
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.socket.send_to(data, self.peer)?;
        let mut buf = vec![0u8; 4096];
        let (n, _src) = self.socket.recv_from(&mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }

    fn is_open(&self) -> bool { true }

    fn close(&mut self) -> io::Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_connect_refused() {
        let result = TcpTransport::connect("127.0.0.1:19999", 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_udp_bind_and_close() {
        let transport = UdpTransport::bind_and_connect("127.0.0.1:0", "127.0.0.1:19999", 100);
        assert!(transport.is_ok());
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p industrial-protocols-bridge-socket`
Expected: compiles

- [ ] **Step 4: Run unit tests**

Run: `cargo test -p industrial-protocols-bridge-socket`
Expected: tests PASS

- [ ] **Step 5: Commit**

```bash
git add bridge/socket/ && git commit -m "feat(bridge): add TCP and UDP socket transports"
```

---

### Task 11: Bridge — serial transport (stub)

**Files:**
- Create: `bridge/serial/Cargo.toml`
- Create: `bridge/serial/src/lib.rs`

- [ ] **Step 1: Create bridge-serial crate**

Create `bridge/serial/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bridge-serial"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

- [ ] **Step 2: Implement serial transport stub**

Create `bridge/serial/src/lib.rs`:

```rust
use industrial_protocols_kernel::bridge::Transport;
use std::io;

#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub stop_bits: u8,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum Parity { None, Even, Odd }

pub struct SerialTransport { config: SerialConfig }

impl SerialTransport {
    pub fn open(config: SerialConfig) -> io::Result<Self> { Ok(SerialTransport { config }) }
}

impl Transport for SerialTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Err(io::Error::new(io::ErrorKind::Unsupported, format!("Serial transport not yet implemented for port {}", self.config.port)))
    }
    fn is_open(&self) -> bool { false }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p industrial-protocols-bridge-serial`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add bridge/serial/ && git commit -m "feat(bridge): add serial transport stub"
```

---

### Task 12: Kernel assembly — Kernel struct + Builder

**Files:**
- Rewrite: `kernel/src/lib.rs`
- Create: `kernel/tests/kernel_tests.rs`

- [ ] **Step 1: Write kernel integration tests**

Create `kernel/tests/kernel_tests.rs`:

```rust
use industrial_protocols_kernel::*;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_kernel::connection::Connection;
use std::io;

struct TestProtocol;
impl Protocol for TestProtocol {
    fn name(&self) -> &'static str { "test" }
    fn version(&self) -> &'static str { "1.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

struct TestConnection;
impl Connection for TestConnection {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> { Ok(vec![42]) }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
    fn is_alive(&self) -> bool { true }
}

#[test]
fn test_kernel_builder_boot_and_shutdown() {
    let config_toml = r#"
[devices.test-001]
protocol = "test"
variant = "tcp"
host = "127.0.0.1"
port = 9999
timeout_ms = 1000
"#;

    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(TestProtocol))
        .build();

    assert!(kernel.boot().is_ok());
    assert!(kernel.shutdown().is_ok());
}

#[test]
fn test_kernel_registry_access() {
    let mut kernel = Kernel::builder()
        .with_protocol(Box::new(TestProtocol))
        .build();

    assert!(kernel.protocol_registry().get("test").is_some());
}

#[test]
fn test_kernel_get_device_config() {
    let config_toml = r#"
[devices.plc-001]
protocol = "test"
variant = "tcp"
host = "192.168.1.10"
port = 502
"#;

    let kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(TestProtocol))
        .build();

    let dev = kernel.config().get_device("plc-001").unwrap();
    assert_eq!(dev.protocol, "test");
    assert_eq!(dev.host.as_deref(), Some("192.168.1.10"));
    assert_eq!(dev.port, Some(502));
}

#[test]
fn test_boot_fails_when_protocol_not_registered() {
    let config_toml = r#"
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "192.168.1.10"
port = 502
"#;

    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .build(); // no ModbusProtocol registered

    let result = kernel.boot();
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-kernel --test kernel_tests`
Expected: compile error (old lib.rs is minimal)

- [ ] **Step 3: Implement Kernel struct and Builder**

Rewrite `kernel/src/lib.rs`:

```rust
pub mod bridge;
pub mod circuit;
pub mod config;
pub mod connection;
pub mod error;
pub mod event;
pub mod registry;
pub mod retry;
pub mod sdk;

use config::ConfigRepository;
use connection::{ConnectionManager, ConnectionStrategy};
use event::EventBus;
use registry::ProtocolRegistry;
use sdk::Protocol;
use std::io;

pub struct Kernel {
    registry: ProtocolRegistry,
    conn_manager: ConnectionManager,
    config: Option<ConfigRepository>,
    events: EventBus,
}

impl Kernel {
    pub fn builder() -> KernelBuilder {
        KernelBuilder::new()
    }

    pub fn boot(&mut self) -> io::Result<()> {
        if let Some(ref config) = self.config {
            for device_id in config.device_ids() {
                let dev = config.get_device(&device_id).unwrap();
                if self.registry.get(&dev.protocol).is_none() {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("protocol '{}' not registered (needed by device '{device_id}')", dev.protocol),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> io::Result<()> {
        self.conn_manager.shutdown()
    }

    pub fn protocol_registry(&self) -> &ProtocolRegistry { &self.registry }
    pub fn connection_manager(&self) -> &ConnectionManager { &self.conn_manager }
    pub fn config(&self) -> &ConfigRepository { self.config.as_ref().expect("config not loaded") }
    pub fn event_bus(&self) -> &EventBus { &self.events }
}

pub struct KernelBuilder {
    config: Option<ConfigRepository>,
    protocols: Vec<Box<dyn Protocol>>,
    connection_strategy: ConnectionStrategy,
    event_capacity: usize,
}

impl KernelBuilder {
    fn new() -> Self {
        KernelBuilder {
            config: None,
            protocols: Vec::new(),
            connection_strategy: ConnectionStrategy::Persistent,
            event_capacity: 64,
        }
    }

    pub fn config_from_str(mut self, toml_str: &str) -> Self {
        self.config = Some(ConfigRepository::from_str(toml_str).expect("invalid TOML config"));
        self
    }

    pub fn config_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.config = Some(ConfigRepository::load(path).expect("failed to load config file"));
        self
    }

    pub fn with_protocol(mut self, protocol: Box<dyn Protocol>) -> Self {
        self.protocols.push(protocol);
        self
    }

    pub fn with_connection_strategy(mut self, strategy: ConnectionStrategy) -> Self {
        self.connection_strategy = strategy;
        self
    }

    pub fn with_event_capacity(mut self, cap: usize) -> Self {
        self.event_capacity = cap;
        self
    }

    pub fn build(self) -> Kernel {
        let mut registry = ProtocolRegistry::new();
        for p in self.protocols {
            registry.register(p);
        }

        Kernel {
            registry,
            conn_manager: ConnectionManager::new(self.connection_strategy),
            config: self.config,
            events: EventBus::new(self.event_capacity),
        }
    }
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test -p industrial-protocols-kernel`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add kernel/src/lib.rs kernel/tests/kernel_tests.rs && git commit -m "feat(kernel): add Kernel struct with builder pattern, boot/shutdown"
```

---

### Task 13: Protocol — Modbus

**Files:**
- Create: `protocols/modbus/Cargo.toml`
- Create: `protocols/modbus/src/lib.rs`
- Create: `protocols/modbus/src/frame.rs`
- Create: `protocols/modbus/tests/integration_test.rs`

- [ ] **Step 1: Create modbus crate**

Create `protocols/modbus/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-modbus"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

- [ ] **Step 2: Write integration tests**

Create `protocols/modbus/tests/integration_test.rs`:

```rust
use industrial_protocols_kernel::*;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_modbus::ModbusProtocol;

#[test]
fn test_modbus_protocol_metadata() {
    let p = ModbusProtocol::tcp();
    assert_eq!(p.name(), "modbus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert!(p.supported_variants().contains(&"tcp"));
    assert!(p.supported_variants().contains(&"rtu"));
    assert!(p.supported_variants().contains(&"ascii"));
}

#[test]
fn test_modbus_in_kernel() {
    let config_toml = r#"
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "127.0.0.1"
port = 9999
timeout_ms = 100
"#;

    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(ModbusProtocol::tcp()))
        .build();

    let _ = kernel.boot();
    assert!(kernel.protocol_registry().get("modbus").is_some());
}

#[test]
fn test_modbus_read_holding_registers_frame() {
    use industrial_protocols_modbus::frame;
    let frame = frame::build_read_holding_registers(1, 0, 10);
    assert_eq!(frame[0], 0x00);
    assert_eq!(frame[1], 0x01);
    assert_eq!(frame[7], 0x06);
    assert_eq!(frame[8], 0x03);
    assert_eq!(frame[12], 0x0A);
}

#[test]
fn test_parse_read_holding_registers_response() {
    use industrial_protocols_modbus::frame;
    let response = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x01, 0x03, 0x04, 0x00, 0x2A, 0x00, 0x42];
    let values = frame::parse_read_response(&response).unwrap();
    assert_eq!(values, vec![42, 66]);
}

#[test]
fn test_parse_exception() {
    use industrial_protocols_modbus::frame;
    let err_resp = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, 0x83, 0x02];
    assert_eq!(frame::parse_exception(&err_resp), Some(0x02));
}
```

- [ ] **Step 3: Run tests — expect failure**

Run: `cargo test -p industrial-protocols-modbus`
Expected: compile error

- [ ] **Step 4: Implement Modbus frame layer**

Create `protocols/modbus/src/frame.rs`:

```rust
pub fn build_read_holding_registers(transaction_id: u16, start_addr: u16, quantity: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&0x0006u16.to_be_bytes());
    frame.push(0x01);
    frame.push(0x03);
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    frame
}

pub fn build_write_single_register(transaction_id: u16, addr: u16, value: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&0x0006u16.to_be_bytes());
    frame.push(0x01);
    frame.push(0x06);
    frame.extend_from_slice(&addr.to_be_bytes());
    frame.extend_from_slice(&value.to_be_bytes());
    frame
}

pub fn build_write_multiple_registers(transaction_id: u16, start_addr: u16, values: &[u16]) -> Vec<u8> {
    let byte_count = (values.len() * 2) as u8;
    let remaining = 5 + byte_count as u16;
    let mut frame = Vec::with_capacity(7 + remaining as usize);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&remaining.to_be_bytes());
    frame.push(0x01);
    frame.push(0x10);
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&(values.len() as u16).to_be_bytes());
    frame.push(byte_count);
    for val in values {
        frame.extend_from_slice(&val.to_be_bytes());
    }
    frame
}

pub fn build_read_coils(transaction_id: u16, start_addr: u16, quantity: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&0x0006u16.to_be_bytes());
    frame.push(0x01);
    frame.push(0x01);
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    frame
}

pub fn build_read_discrete_inputs(transaction_id: u16, start_addr: u16, quantity: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&0x0006u16.to_be_bytes());
    frame.push(0x01);
    frame.push(0x02);
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    frame
}

pub fn build_read_input_registers(transaction_id: u16, start_addr: u16, quantity: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(12);
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0x0000u16.to_be_bytes());
    frame.extend_from_slice(&0x0006u16.to_be_bytes());
    frame.push(0x01);
    frame.push(0x04);
    frame.extend_from_slice(&start_addr.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    frame
}

pub fn parse_read_response(data: &[u8]) -> Result<Vec<u16>, String> {
    if data.len() < 9 {
        return Err(format!("response too short: {} bytes", data.len()));
    }
    let byte_count = data[8] as usize;
    let expected_len = 9 + byte_count;
    if data.len() < expected_len {
        return Err(format!("response truncated: expected {} bytes, got {}", expected_len, data.len()));
    }
    let mut values = Vec::with_capacity(byte_count / 2);
    let mut i = 9;
    while i + 1 < data.len() && i < expected_len {
        values.push(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    Ok(values)
}

pub fn parse_exception(data: &[u8]) -> Option<u8> {
    if data.len() >= 9 && (data[7] & 0x80) != 0 {
        Some(data[8])
    } else {
        None
    }
}
```

- [ ] **Step 5: Implement ModbusProtocol**

Create `protocols/modbus/src/lib.rs`:

```rust
pub mod frame;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;
use std::sync::atomic::{AtomicU16, Ordering};

pub struct ModbusProtocol {
    variant: ModbusVariant,
    transaction_id: AtomicU16,
}

#[derive(Debug, Clone, Copy)]
pub enum ModbusVariant { Tcp, Rtu, Ascii }

impl ModbusProtocol {
    pub fn tcp() -> Self { ModbusProtocol { variant: ModbusVariant::Tcp, transaction_id: AtomicU16::new(1) } }
    pub fn rtu() -> Self { ModbusProtocol { variant: ModbusVariant::Rtu, transaction_id: AtomicU16::new(1) } }

    fn next_tid(&self) -> u16 { self.transaction_id.fetch_add(1, Ordering::SeqCst) }
}

impl Protocol for ModbusProtocol {
    fn name(&self) -> &'static str { "modbus" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp", "rtu", "ascii"] }
}

impl ProtocolRead for ModbusProtocol {
    type Address = u16;
    type Value = u16;

    fn read(&self, conn: &dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let frame = frame::build_read_holding_registers(self.next_tid(), *addr, 1);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError { device: "modbus-device".into(), code: exc as u16, message: format!("Modbus exception {exc:#04X}") });
        }
        frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })?
            .into_iter().next()
            .ok_or_else(|| ProtocolError::Parse { protocol: "modbus".into(), reason: "empty response".into() })
    }

    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        if addrs.is_empty() { return Ok(vec![]); }
        let min = addrs.iter().min().copied().unwrap();
        let max = addrs.iter().max().copied().unwrap();
        let count = max - min + 1;
        let frame = frame::build_read_holding_registers(self.next_tid(), min, count);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        let all = frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })?;
        Ok(addrs.iter().map(|a| all[(*a - min) as usize]).collect())
    }

    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        let frame = frame::build_read_holding_registers(self.next_tid(), *start, count);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })
    }
}

impl ProtocolWrite for ModbusProtocol {
    type Address = u16;
    type Value = u16;

    fn write(&self, conn: &dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        let frame = frame::build_write_single_register(self.next_tid(), *addr, *value);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError { device: "modbus-device".into(), code: exc as u16, message: format!("Modbus exception {exc:#04X}") });
        }
        Ok(())
    }

    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        if items.is_empty() { return Ok(()); }
        let start = items[0].0;
        let values: Vec<u16> = items.iter().map(|(_, v)| *v).collect();
        let frame = frame::build_write_multiple_registers(self.next_tid(), start, &values);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError { device: "modbus-device".into(), code: exc as u16, message: format!("Modbus exception {exc:#04X}") });
        }
        Ok(())
    }
}

impl ProtocolHealth for ModbusProtocol {
    fn health(&self, conn: &dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus { state: HealthState::Healthy, latency_ms: start.elapsed().as_millis() as u64, last_seen: std::time::Instant::now(), details: "modbus: read holding register 0 ok".into() }),
            Err(e) => Ok(HealthStatus { state: HealthState::Unhealthy, latency_ms: start.elapsed().as_millis() as u64, last_seen: std::time::Instant::now(), details: format!("modbus health check failed: {e}") }),
        }
    }
}

impl ProtocolDiagnostics for ModbusProtocol {
    fn diagnostics(&self, _conn: &dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: "modbus-device".into(), uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("variant".into(), format!("{:?}", self.variant)),
                ("transaction_id".into(), self.transaction_id.load(Ordering::SeqCst).to_string()),
            ]),
        })
    }
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p industrial-protocols-modbus`
Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add protocols/modbus/ && git commit -m "feat(modbus): implement Modbus TCP with FC 01/02/03/04/06/10"
```

---

### Task 14: Protocols — OPC UA, BACnet, EtherNet/IP, Profinet skeletons

**Files:**
- Create: `protocols/opcua/Cargo.toml`, `protocols/opcua/src/lib.rs`, `protocols/opcua/tests/integration_test.rs`
- Create: `protocols/bacnet/Cargo.toml`, `protocols/bacnet/src/lib.rs`, `protocols/bacnet/tests/integration_test.rs`
- Create: `protocols/ethernet-ip/Cargo.toml`, `protocols/ethernet-ip/src/lib.rs`, `protocols/ethernet-ip/tests/integration_test.rs`
- Create: `protocols/profinet/Cargo.toml`, `protocols/profinet/src/lib.rs`, `protocols/profinet/tests/integration_test.rs`

- [ ] **Step 1: Create OPC UA skeleton**

Create `protocols/opcua/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-opcua"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `protocols/opcua/src/lib.rs`:

```rust
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct OpcuaProtocol { endpoint: String }

impl OpcuaProtocol {
    pub fn new(endpoint: impl Into<String>) -> Self { OpcuaProtocol { endpoint: endpoint.into() } }
}

impl Protocol for OpcuaProtocol {
    fn name(&self) -> &'static str { "opcua" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["binary"] }
}

impl ProtocolRead for OpcuaProtocol {
    type Address = String;
    type Value = OPCUAValue;
    fn read(&self, _conn: &dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Err(ProtocolError::Unsupported { protocol: "opcua".into(), operation: "read — skeleton".into() })
    }
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        addrs.iter().map(|a| self.read(conn, a)).collect()
    }
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        (0..count).map(|_| self.read(conn, start)).collect()
    }
}

impl ProtocolWrite for OpcuaProtocol {
    type Address = String;
    type Value = OPCUAValue;
    fn write(&self, _conn: &dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "opcua".into(), operation: "write — skeleton".into() })
    }
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        for (a, v) in items { self.write(conn, a, v)?; }
        Ok(())
    }
}

impl ProtocolHealth for OpcuaProtocol {
    fn health(&self, _conn: &dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus { state: HealthState::Unknown, latency_ms: 0, last_seen: std::time::Instant::now(), details: "opcua skeleton".into() })
    }
}

impl ProtocolDiagnostics for OpcuaProtocol {
    fn diagnostics(&self, _conn: &dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport { device_id: self.endpoint.clone(), uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([("endpoint".into(), self.endpoint.clone()), ("status".into(), "skeleton".into())]) })
    }
}

#[derive(Debug, Clone)]
pub enum OPCUAValue { Boolean(bool), SByte(i8), Byte(u8), Int16(i16), UInt16(u16), Int32(i32), UInt32(u32), Float(f32), Double(f64), String(String), Null }
```

Create `protocols/opcua/tests/integration_test.rs`:

```rust
use industrial_protocols_opcua::OpcuaProtocol;
use industrial_protocols_kernel::sdk::*;

#[test]
fn test_opcua_metadata() {
    let p = OpcuaProtocol::new("opc.tcp://localhost:4840");
    assert_eq!(p.name(), "opcua");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}
```

- [ ] **Step 2: Create BACnet skeleton**

Create `protocols/bacnet/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bacnet"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `protocols/bacnet/src/lib.rs`:

```rust
use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct BacnetProtocol { device_instance: u32 }

impl BacnetProtocol {
    pub fn new(device_instance: u32) -> Self { BacnetProtocol { device_instance } }
}

impl Protocol for BacnetProtocol {
    fn name(&self) -> &'static str { "bacnet" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["ip", "ethernet"] }
}

impl ProtocolRead for BacnetProtocol {
    type Address = String;
    type Value = f32;
    fn read(&self, _conn: &dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Err(ProtocolError::Unsupported { protocol: "bacnet".into(), operation: "read — skeleton".into() })
    }
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> { addrs.iter().map(|a| self.read(conn, a)).collect() }
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> { (0..count).map(|_| self.read(conn, start)).collect() }
}

impl ProtocolWrite for BacnetProtocol {
    type Address = String;
    type Value = f32;
    fn write(&self, _conn: &dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "bacnet".into(), operation: "write — skeleton".into() })
    }
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> { for (a, v) in items { self.write(conn, a, v)?; } Ok(()) }
}

impl ProtocolDiscover for BacnetProtocol {
    fn discover(&self, _transport: &dyn Transport) -> Result<Vec<DiscoveredDevice>> {
        Err(ProtocolError::Unsupported { protocol: "bacnet".into(), operation: "discover — skeleton".into() })
    }
}

impl ProtocolHealth for BacnetProtocol {
    fn health(&self, _conn: &dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus { state: HealthState::Unknown, latency_ms: 0, last_seen: std::time::Instant::now(), details: "bacnet skeleton".into() })
    }
}

impl ProtocolDiagnostics for BacnetProtocol {
    fn diagnostics(&self, _conn: &dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport { device_id: format!("bacnet:{}", self.device_instance), uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([("device_instance".into(), self.device_instance.to_string())]) })
    }
}
```

Create `protocols/bacnet/tests/integration_test.rs`:

```rust
use industrial_protocols_bacnet::BacnetProtocol;
use industrial_protocols_kernel::sdk::*;

#[test]
fn test_bacnet_metadata() {
    let p = BacnetProtocol::new(1001);
    assert_eq!(p.name(), "bacnet");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}
```

- [ ] **Step 3: Create EtherNet/IP skeleton**

Create `protocols/ethernet-ip/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-ethernetip"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `protocols/ethernet-ip/src/lib.rs`:

```rust
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct EthernetIpProtocol { slot: u8 }
impl EthernetIpProtocol { pub fn new(slot: u8) -> Self { EthernetIpProtocol { slot } } }

impl Protocol for EthernetIpProtocol {
    fn name(&self) -> &'static str { "ethernet-ip" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

impl ProtocolRead for EthernetIpProtocol {
    type Address = String;
    type Value = EthernetIpValue;
    fn read(&self, _conn: &dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Err(ProtocolError::Unsupported { protocol: "ethernet-ip".into(), operation: "read — skeleton".into() })
    }
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> { addrs.iter().map(|a| self.read(conn, a)).collect() }
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> { (0..count).map(|_| self.read(conn, start)).collect() }
}

impl ProtocolWrite for EthernetIpProtocol {
    type Address = String;
    type Value = EthernetIpValue;
    fn write(&self, _conn: &dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "ethernet-ip".into(), operation: "write — skeleton".into() })
    }
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> { for (a, v) in items { self.write(conn, a, v)?; } Ok(()) }
}

impl ProtocolHealth for EthernetIpProtocol {
    fn health(&self, _conn: &dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus { state: HealthState::Unknown, latency_ms: 0, last_seen: std::time::Instant::now(), details: "ethernet-ip skeleton".into() })
    }
}

impl ProtocolDiagnostics for EthernetIpProtocol {
    fn diagnostics(&self, _conn: &dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport { device_id: format!("enip:slot{}", self.slot), uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([("slot".into(), self.slot.to_string())]) })
    }
}

#[derive(Debug, Clone)]
pub enum EthernetIpValue { Int(i16), DInt(i32), Real(f32), Bool(bool), String(String) }
```

Create `protocols/ethernet-ip/tests/integration_test.rs`:

```rust
use industrial_protocols_ethernetip::EthernetIpProtocol;
use industrial_protocols_kernel::sdk::*;

#[test]
fn test_eip_metadata() {
    let p = EthernetIpProtocol::new(0);
    assert_eq!(p.name(), "ethernet-ip");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}
```

- [ ] **Step 4: Create Profinet skeleton**

Create `protocols/profinet/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-profinet"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `protocols/profinet/src/lib.rs`:

```rust
use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct ProfinetProtocol;
impl ProfinetProtocol { pub fn new() -> Self { ProfinetProtocol } }

impl Protocol for ProfinetProtocol {
    fn name(&self) -> &'static str { "profinet" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["nrt"] }
}

impl ProtocolRead for ProfinetProtocol {
    type Address = String;
    type Value = Vec<u8>;
    fn read(&self, _conn: &dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Err(ProtocolError::Unsupported { protocol: "profinet".into(), operation: "read — skeleton".into() })
    }
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> { addrs.iter().map(|a| self.read(conn, a)).collect() }
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> { (0..count).map(|_| self.read(conn, start)).collect() }
}

impl ProtocolWrite for ProfinetProtocol {
    type Address = String;
    type Value = Vec<u8>;
    fn write(&self, _conn: &dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "profinet".into(), operation: "write — skeleton".into() })
    }
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> { for (a, v) in items { self.write(conn, a, v)?; } Ok(()) }
}

impl ProtocolDiscover for ProfinetProtocol {
    fn discover(&self, _transport: &dyn Transport) -> Result<Vec<DiscoveredDevice>> {
        Err(ProtocolError::Unsupported { protocol: "profinet".into(), operation: "discover — skeleton".into() })
    }
}

impl ProtocolHealth for ProfinetProtocol {
    fn health(&self, _conn: &dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus { state: HealthState::Unknown, latency_ms: 0, last_seen: std::time::Instant::now(), details: "profinet skeleton".into() })
    }
}

impl ProtocolDiagnostics for ProfinetProtocol {
    fn diagnostics(&self, _conn: &dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport { device_id: "pn-device".into(), uptime_secs: None, error_count: 0, last_error: None, custom_fields: Default::default() })
    }
}
```

Create `protocols/profinet/tests/integration_test.rs`:

```rust
use industrial_protocols_profinet::ProfinetProtocol;
use industrial_protocols_kernel::sdk::*;

#[test]
fn test_profinet_metadata() {
    let p = ProfinetProtocol::new();
    assert_eq!(p.name(), "profinet");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}
```

- [ ] **Step 5: Run all protocol tests**

Run: `cargo test -p industrial-protocols-opcua -p industrial-protocols-bacnet -p industrial-protocols-ethernetip -p industrial-protocols-profinet`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add protocols/opcua/ protocols/bacnet/ protocols/ethernet-ip/ protocols/profinet/ && git commit -m "feat: add OPC UA, BACnet, EtherNet/IP, Profinet protocol skeletons"
```

---

### Task 15: Bridge skeletons — CAN, FFI, Vendor

**Files:**
- Create: `bridge/can/Cargo.toml`, `bridge/can/src/lib.rs`
- Create: `bridge/ffi/Cargo.toml`, `bridge/ffi/src/lib.rs`
- Create: `bridge/vendor/Cargo.toml`, `bridge/vendor/src/lib.rs`

- [ ] **Step 1: Create CAN, FFI, Vendor bridge skeletons**

Create `bridge/can/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bridge-can"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `bridge/can/src/lib.rs`:

```rust
//! CAN bus transport — skeleton. Real implementation requires socketcan or similar.
use std::io;
pub struct CanTransport;
impl CanTransport {
    pub fn open(_interface: &str) -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "CAN transport not yet implemented"))
    }
}
```

Create `bridge/ffi/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bridge-ffi"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
```

Create `bridge/ffi/src/lib.rs`:

```rust
//! FFI bridge — safe wrapper template for vendor C SDK calls.
use industrial_protocols_kernel::bridge::Transport;
use std::io;
use std::path::PathBuf;

pub struct FfiConfig {
    pub library_path: PathBuf,
    pub vendor: String,
    pub init_params: std::collections::HashMap<String, String>,
}

pub struct SafeVendorTransport;

impl SafeVendorTransport {
    pub fn load(_config: FfiConfig) -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "FFI bridge not yet implemented"))
    }
}

impl Transport for SafeVendorTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "FFI send not yet implemented"))
    }
    fn is_open(&self) -> bool { false }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}
```

Create `bridge/vendor/Cargo.toml`:

```toml
[package]
name = "industrial-protocols-bridge-vendor"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
toml.workspace = true
```

Create `bridge/vendor/src/lib.rs`:

```rust
//! Vendor preset configurations for common industrial devices.
use std::collections::HashMap;

pub fn siemens_s7_1200() -> HashMap<String, toml::Value> {
    let mut extras = HashMap::new();
    extras.insert("vendor".into(), toml::Value::String("Siemens".into()));
    extras.insert("model".into(), toml::Value::String("S7-1200".into()));
    extras.insert("rack".into(), toml::Value::Integer(0));
    extras.insert("slot".into(), toml::Value::Integer(1));
    extras
}

pub fn allen_bradley_compactlogix() -> HashMap<String, toml::Value> {
    let mut extras = HashMap::new();
    extras.insert("vendor".into(), toml::Value::String("Allen-Bradley".into()));
    extras.insert("model".into(), toml::Value::String("CompactLogix".into()));
    extras
}

pub fn schneider_m340() -> HashMap<String, toml::Value> {
    let mut extras = HashMap::new();
    extras.insert("vendor".into(), toml::Value::String("Schneider Electric".into()));
    extras.insert("model".into(), toml::Value::String("Modicon M340".into()));
    extras
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siemens_preset() {
        let p = siemens_s7_1200();
        assert_eq!(p.get("vendor").unwrap().as_str(), Some("Siemens"));
    }

    #[test]
    fn test_ab_preset() {
        let p = allen_bradley_compactlogix();
        assert_eq!(p.get("vendor").unwrap().as_str(), Some("Allen-Bradley"));
    }

    #[test]
    fn test_schneider_preset() {
        let p = schneider_m340();
        assert_eq!(p.get("vendor").unwrap().as_str(), Some("Schneider Electric"));
    }
}
```

- [ ] **Step 2: Verify all compile**

Run: `cargo check --workspace`
Expected: all crates compile

- [ ] **Step 3: Commit**

```bash
git add bridge/can/ bridge/ffi/ bridge/vendor/ && git commit -m "feat(bridge): add CAN, FFI, and Vendor preset skeletons"
```

---

### Task 16: Example — simple Modbus usage

**Files:**
- Create: `examples/simple-modbus/Cargo.toml`
- Create: `examples/simple-modbus/src/main.rs`

- [ ] **Step 1: Create example**

Create `examples/simple-modbus/Cargo.toml`:

```toml
[package]
name = "simple-modbus-example"
version = "0.1.0"
edition = "2024"

[dependencies]
industrial-protocols-kernel = { path = "../../kernel" }
industrial-protocols-modbus = { path = "../../protocols/modbus" }
```

Create `examples/simple-modbus/src/main.rs`:

```rust
use industrial_protocols_kernel::*;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_modbus::ModbusProtocol;

fn main() {
    let config_toml = r#"
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "127.0.0.1"
port = 502
unit_id = 1
timeout_ms = 3000
"#;

    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(ModbusProtocol::tcp()))
        .build();

    match kernel.boot() {
        Ok(()) => println!("Kernel booted successfully"),
        Err(e) => {
            println!("Boot warning (no real device expected): {e}");
            println!("Kernel is still usable for protocol metadata.");
        }
    }

    let registry = kernel.protocol_registry();
    if let Some(proto) = registry.get("modbus") {
        println!("Loaded protocol: {} v{}", proto.name(), proto.version());
        println!("  Category: {:?}", proto.category());
        println!("  Variants: {:?}", proto.supported_variants());
    }

    let _ = kernel.shutdown();
    println!("Kernel shut down.");
}
```

- [ ] **Step 2: Verify it compiles and runs**

Run: `cargo run -p simple-modbus-example`
Expected: runs, prints protocol info (may show connection error on boot which is expected)

- [ ] **Step 3: Commit**

```bash
git add examples/ && git commit -m "docs: add simple-modbus example demonstrating kernel usage"
```

---

### Task 17: Run full workspace test suite + final verification

- [ ] **Step 1: Run all tests across all crates**

Run: `cargo test --workspace`
Expected: all tests PASS

- [ ] **Step 2: Run clippy (if available)**

Run: `cargo clippy --workspace -- -D warnings 2>/dev/null || echo "clippy not installed, skipping"`
Expected: no warnings (or skip message)

- [ ] **Step 3: Verify crate structure**

Run: `find . -name Cargo.toml -not -path './target/*' | sort`
Expected:
```
./Cargo.toml
./bridge/can/Cargo.toml
./bridge/ffi/Cargo.toml
./bridge/serial/Cargo.toml
./bridge/socket/Cargo.toml
./bridge/vendor/Cargo.toml
./examples/simple-modbus/Cargo.toml
./kernel/Cargo.toml
./protocols/bacnet/Cargo.toml
./protocols/ethernet-ip/Cargo.toml
./protocols/modbus/Cargo.toml
./protocols/opcua/Cargo.toml
./protocols/profinet/Cargo.toml
```

- [ ] **Step 4: Commit final state**

```bash
git add -A && git commit -m "chore: finalize Phase 1 workspace — all crates, tests, and example"
```
