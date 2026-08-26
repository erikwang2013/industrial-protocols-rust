# Referensi API — Industrial Protocols Rust

Dokumen ini berisi API publik inti kumpulan protokol industri: Trait Protocol SDK, komponen kernel, lapisan bridge, tipe kesalahan, dan lapisan async.

Kembali ke [README](../README.md) (versi bahasa lain lihat [direktori bahasa](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Trait

Setiap protokol terhubung ke kernel dengan mengimplementasikan trait berikut:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // tipe alamat spesifik protokol
    type Value: ValueLike;      // tipe nilai spesifik protokol
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

**Associated type** membuat alamat dan nilai aman secara tipe: Modbus menggunakan `u16`, OPC UA menggunakan `String`, BACnet menggunakan `f32`. **Pemecahan trait** memungkinkan protokol diimplementasikan sesuai kebutuhan — sensor read-only tidak memerlukan `ProtocolWrite`.

## 2. Kernel — Mikrokernel

Kernel tidak mengimplementasikan protokol apa pun, hanya menyediakan infrastruktur bersama:

| Komponen | Tanggung jawab | API kunci |
|------|------|----------|
| `ProtocolRegistry` | Registrasi/pencarian/klasifikasi protokol | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Siklus hidup koneksi perangkat | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | Konfigurasi TOML + override saat runtime | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Publikasi/langganan event | `subscribe()`, `publish()` |
| `CircuitBreaker` | Pemutus sirkuit Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Tetap/eksponensial/linear/tanpa | `run()`, `start()` |
| `GatewayEngine` | Aturan penerusan data antar protokol | `add_rule()`, `start()/stop()` |

Manajemen koneksi mendukung tiga strategi:
```rust
pub enum ConnectionStrategy {
    Simple,      // dibuat baru setiap kali, dilepas setelah dipakai
    Persistent,  // koneksi tunggal dipakai ulang, otomatis terhubung ulang saat putus
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — Abstraksi Transport

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementasi | Ketergantungan |
|--------|------|------|
| `bridge-socket` | Koneksi TCP + kirim/terima UDP | std |
| `bridge-serial` | Port serial nyata, opsional fitur `serial` | `serialport` |
| `bridge-can` | SocketCAN Linux (CAN_RAW) | `libc` |
| `bridge-ffi` | Pemuatan dinamis SDK C vendor | `libloading` |
| `bridge-vendor` | Preset Siemens/AB/Schneider | — |

## 4. Error — Rantai Kesalahan Terstruktur

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 alamat tidak valid"
    CircuitOpen { device, retry_after },  // pemutus sirkuit terbuka
    Context { context, source },  // menambahkan konteks, membentuk rantai kesalahan
    // ...
}
```

Contoh rantai kesalahan:
```
Gagal membaca holding register 40001 dari PLC-001
  → Timeout: device=plc-001, timeout=3000ms
    → Gagal koneksi: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — Pembungkus tokio Opsional

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// Aliran event async
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
