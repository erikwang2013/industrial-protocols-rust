# API রেফারেন্স — Industrial Protocols Rust

এই ডকুমেন্টে ইন্ডাস্ট্রিয়াল প্রোটোকল স্যুটের মূল পাবলিক API রয়েছে: Protocol SDK Trait, কার্নেল কম্পোনেন্ট, ব্রিজ লেয়ার, এরর টাইপ ও অ্যাসিঙ্ক লেয়ার।

[README](../README.md)-এ ফিরে যান (বহুভাষা সংস্করণ দেখুন [ভাষা তালিকা](../README.md#多語言--languages))।

## 1. Protocol SDK — ৬টি Trait

প্রতিটি প্রোটোকল নিচের trait বাস্তবায়নের মাধ্যমে কার্নেলে যুক্ত হয়:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // প্রোটোকল-নির্দিষ্ট ঠিকানা টাইপ
    type Value: ValueLike;      // প্রোটোকল-নির্দিষ্ট মান টাইপ
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

**অ্যাসোসিয়েটেড টাইপ** ঠিকানা ও মানকে টাইপ-সেফ করে: Modbus-এ `u16`, OPC UA-তে `String`, BACnet-এ `f32`। **Trait বিচ্ছিন্নতা** প্রোটোকলকে প্রয়োজন অনুযায়ী বাস্তবায়নের সুযোগ দেয় — শুধু-পঠনযোগ্য সেন্সরের `ProtocolWrite` লাগে না।

## 2. Kernel — মাইক্রো-কার্নেল

কার্নেল কোনো প্রোটোকল বাস্তবায়ন করে না, শুধু শেয়ার্ড ইনফ্রাস্ট্রাকচার সরবরাহ করে:

| কম্পোনেন্ট | দায়িত্ব | মূল API |
|------|------|----------|
| `ProtocolRegistry` | প্রোটোকল নিবন্ধন/অনুসন্ধান/শ্রেণিবিভাগ | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | ডিভাইস সংযোগ জীবনচক্র | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML কনফিগ + রানটাইম ওভাররাইড | `load()`, `get_device()`, `set_override()` |
| `EventBus` | ইভেন্ট প্রকাশ/সাবস্ক্রাইব | `subscribe()`, `publish()` |
| `CircuitBreaker` | সার্কিট ব্রেকার Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | ফিক্সড/এক্সপোনেনশিয়াল/লিনিয়ার/না | `run()`, `start()` |
| `GatewayEngine` | প্রোটোকলের মধ্যে ডেটা ফরওয়ার্ডিং নিয়ম | `add_rule()`, `start()/stop()` |

সংযোগ ব্যবস্থাপনা তিনটি কৌশল সমর্থন করে:
```rust
pub enum ConnectionStrategy {
    Simple,      // প্রতি বার নতুন করে, ব্যবহার শেষে মুক্ত
    Persistent,  // একক সংযোগ পুনঃব্যবহার, বিচ্ছিন্ন হলে স্বয়ংক্রিয় পুনঃসংযোগ
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — ট্রান্সপোর্ট অ্যাবস্ট্রাকশন

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | বাস্তবায়ন | নির্ভরতা |
|--------|------|------|
| `bridge-socket` | TCP সংযোগ + UDP পাঠ/গ্রহণ | std |
| `bridge-serial` | প্রকৃত সিরিয়াল পোর্ট, ঐচ্ছিক `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | ভেন্ডর C SDK ডায়নামিক লোডিং | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider প্রিসেট | — |

## 4. Error — স্ট্রাকচার্ড এরর চেইন

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 অবৈধ ঠিকানা"
    CircuitOpen { device, retry_after },  // সার্কিট ব্রেকার খোলা
    Context { context, source },  // প্রসঙ্গ সংযুক্ত করে এরর চেইন গঠন
    // ...
}
```

এরর চেইন উদাহরণ:
```
PLC-001 হোল্ডিং রেজিস্টার 40001 পড়া ব্যর্থ
  → টাইমআউট: device=plc-001, timeout=3000ms
    → সংযোগ ব্যর্থ: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — ঐচ্ছিক tokio র‍্যাপার

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// অ্যাসিঙ্ক ইভেন্ট স্ট্রিম
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
