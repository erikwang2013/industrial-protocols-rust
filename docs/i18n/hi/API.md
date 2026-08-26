# API संदर्भ — Industrial Protocols Rust

यह दस्तावेज़ औद्योगिक प्रोटोकॉल संग्रह के मुख्य सार्वजनिक APIs का वर्णन करता है: Protocol SDK Traits, कर्नेल घटक, ब्रिज परत, त्रुटि प्रकार और async परत।

वापस [README](../README.md) पर जाएं (बहुभाषा संस्करणों के लिए [भाषा निर्देशिका](../README.md#多语言--languages))।

## 1. Protocol SDK — 6 Traits

प्रत्येक प्रोटोकॉल निम्नलिखित traits को लागू करके कर्नेल से जुड़ता है:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // प्रोटोकॉल-विशिष्ट पता प्रकार
    type Value: ValueLike;      // प्रोटोकॉल-विशिष्ट मान प्रकार
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

**संबद्ध प्रकार** पते और मान को टाइप-सुरक्षित बनाते हैं: Modbus `u16` का उपयोग करता है, OPC UA `String`, BACnet `f32`। **Trait विभाजन** प्रोटोकॉल को आवश्यकता अनुसार लागू करने देता है — केवल-पठन सेंसर को `ProtocolWrite` की आवश्यकता नहीं होती।

## 2. Kernel — माइक्रो-कर्नेल

कर्नेल कोई प्रोटोकॉल लागू नहीं करता, केवल साझा बुनियादी ढांचा प्रदान करता है:

| घटक | ज़िम्मेदारी | मुख्य API |
|------|------|----------|
| `ProtocolRegistry` | प्रोटोकॉल पंजीकरण/खोज/वर्गीकरण | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | डिवाइस कनेक्शन जीवनचक्र | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML कॉन्फ़िगरेशन + रनटाइम ओवरराइड | `load()`, `get_device()`, `set_override()` |
| `EventBus` | इवेंट प्रकाशन/सदस्यता | `subscribe()`, `publish()` |
| `CircuitBreaker` | सर्किट ब्रेकर Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | फिक्स्ड/एक्सपोनेंशियल/लीनियर/कोई नहीं | `run()`, `start()` |
| `GatewayEngine` | प्रोटोकॉल-से-प्रोटोकॉल डेटा फ़ॉरवर्डिंग नियम | `add_rule()`, `start()/stop()` |

कनेक्शन प्रबंधन तीन रणनीतियों का समर्थन करता है:
```rust
pub enum ConnectionStrategy {
    Simple,      // हर बार नया, उपयोग के बाद मुक्त
    Persistent,  // एकल कनेक्शन पुन: उपयोग, डिस्कनेक्ट पर स्वचालित पुनः-कनेक्ट
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge परत — परिवहन अमूर्तता

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | कार्यान्वयन | निर्भरता |
|--------|------|------|
| `bridge-socket` | TCP कनेक्शन + UDP भेजें/प्राप्त करें | std |
| `bridge-serial` | वास्तविक सीरियल पोर्ट, वैकल्पिक `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | वेंडर C SDK डायनेमिक लोडिंग | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider प्रीसेट | — |

## 4. Error — संरचित त्रुटि श्रृंखला

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 अमान्य पता"
    CircuitOpen { device, retry_after },  // सर्किट ब्रेकर खुला है
    Context { context, source },  // संदर्भ जोड़ें, त्रुटि श्रृंखला बनाएं
    // ...
}
```

त्रुटि श्रृंखला उदाहरण:
```
PLC-001 होल्डिंग रजिस्टर 40001 पढ़ना विफल
  → टाइमआउट: device=plc-001, timeout=3000ms
    → कनेक्शन विफल: 192.168.1.10:502: Connection refused
```

## 5. Async परत — वैकल्पिक tokio रैपर

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// async इवेंट स्ट्रीम
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
