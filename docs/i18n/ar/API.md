# مرجع API — Industrial Protocols Rust

توثّق هذه الوثيقة واجهات API العامة الأساسية لمجموعة البروتوكولات الصناعية: Traits في Protocol SDK، ومكونات النواة، وطبقة الجسر، وأنواع الأخطاء، والطبقة غير المتزامنة.

العودة إلى [README](../README.md) (الإصدارات متعددة اللغات في [دليل اللغات](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Traits

يُدمج كل بروتوكول في النواة عبر تنفيذ traits التالية:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // نوع عنوان خاص بالبروتوكول
    type Value: ValueLike;      // نوع قيمة خاص بالبروتوكول
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

**الأنواع المرتبطة** تجعل العناوين والقيم آمنة النوع: يستخدم Modbus `u16`، وOPC UA يستخدم `String`، وBACnet يستخدم `f32`. **تقسيم الـ traits** يتيح للبروتوكولات التنفيذ حسب الحاجة — فجهاز استشعار للقراءة فقط لا يحتاج `ProtocolWrite`.

## 2. Kernel — النواة المصغّرة

لا تنفّذ النواة أي بروتوكول، بل توفر فقط البنية الأساسية المشتركة:

| المكوّن | المسؤولية | API رئيسية |
|------|------|----------|
| `ProtocolRegistry` | تسجيل/بحث/تصنيف البروتوكولات | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | دورة حياة اتصالات الأجهزة | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | إعدادات TOML + تجاوز في زمن التشغيل | `load()`, `get_device()`, `set_override()` |
| `EventBus` | نشر/اشتراك الأحداث | `subscribe()`, `publish()` |
| `CircuitBreaker` | قاطع الدائرة Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | ثابت/أسي/خطي/بدون | `run()`, `start()` |
| `GatewayEngine` | قواعد إعادة توجيه البيانات بين البروتوكولات | `add_rule()`, `start()/stop()` |

تدعم إدارة الاتصال ثلاث استراتيجيات:
```rust
pub enum ConnectionStrategy {
    Simple,      // إنشاء جديد في كل مرة، تحرير بعد الاستخدام
    Persistent,  // إعادة استخدام اتصال واحد، إعادة اتصال تلقائية عند الانقطاع
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — تجريد النقل

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | التنفيذ | الاعتماديات |
|--------|------|------|
| `bridge-socket` | اتصال TCP + إرسال/استقبال UDP | std |
| `bridge-serial` | منفذ تسلسلي حقيقي، feature اختياري `serial` | `serialport` |
| `bridge-can` | SocketCAN لنظام لينكس (CAN_RAW) | `libc` |
| `bridge-ffi` | تحميل ديناميكي لـ C SDK من الموردين | `libloading` |
| `bridge-vendor` | إعدادات مسبقة من Siemens/AB/Schneider | — |

## 4. Error — سلسلة أخطاء منظمة

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 عنوان غير صالح"
    CircuitOpen { device, retry_after },  // قاطع الدائرة مفتوح
    Context { context, source },  // إرفاق السياق لتكوين سلسلة الأخطاء
    // ...
}
```

مثال على سلسلة الأخطاء:
```
فشل قراءة سجل الاحتفاظ 40001 من PLC-001
  → مهلة: device=plc-001, timeout=3000ms
    → فشل الاتصال: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — غلاف tokio اختياري

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// دفق الأحداث غير المتزامن
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
