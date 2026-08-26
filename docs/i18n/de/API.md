# API-Referenz — Industrial Protocols Rust

Dieses Dokument enthält die zentralen öffentlichen APIs des Industrie-Protokollsatzes: Protocol-SDK-Traits, Kernel-Komponenten, Brückenschicht, Fehlertypen und Async-Schicht.

Zurück zum [README](../README.md) (mehrsprachige Versionen siehe [Sprachverzeichnis](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Traits

Jedes Protokoll wird über die Implementierung der folgenden Traits an den Kernel angebunden:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // protokollspezifischer Adresstyp
    type Value: ValueLike;      // protokollspezifischer Werttyp
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

**Assoziierte Typen** machen Adress- und Werttypen typsicher: Modbus verwendet `u16`, OPC UA `String`, BACnet `f32`. **Trait-Aufteilung** erlaubt bedarfsgerechte Implementierung — ein reiner Lesesensor benötigt kein `ProtocolWrite`.

## 2. Kernel — Mikrokernel

Der Kernel implementiert keine Protokolle, sondern stellt nur gemeinsame Infrastruktur bereit:

| Komponente | Aufgabe | Wichtige APIs |
|------|------|----------|
| `ProtocolRegistry` | Protokoll-Registrierung/Suche/Klassifizierung | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Lebenszyklus der Geräteverbindungen | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML-Konfiguration + Laufzeit-Überschreibungen | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Ereignisse veröffentlichen/abonnieren | `subscribe()`, `publish()` |
| `CircuitBreaker` | Unterbrecher Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fest/Exponentiell/Linear/Keine | `run()`, `start()` |
| `GatewayEngine` | Datenweiterleitungsregeln zwischen Protokollen | `add_rule()`, `start()/stop()` |

Das Verbindungsmanagement unterstützt drei Strategien:
```rust
pub enum ConnectionStrategy {
    Simple,      // jedes Mal neu, nach Gebrauch freigeben
    Persistent,  // eine Verbindung wiederverwenden, automatische Wiederverbindung bei Trennung
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — Transportabstraktion

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementierung | Abhängigkeit |
|--------|------|------|
| `bridge-socket` | TCP-Verbindung + UDP-Senden/Empfangen | std |
| `bridge-serial` | Echte serielle Schnittstelle, optionales `serial`-Feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | Dynamisches Laden von Hersteller-C-SDKs | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider-Presets | — |

## 4. Error — Strukturierte Fehlerkette

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 unzulässige Adresse"
    CircuitOpen { device, retry_after },  // Unterbrecher offen
    Context { context, source },  // Kontext anhängen, bildet die Fehlerkette
    // ...
}
```

Beispiel einer Fehlerkette:
```
Lesen von PLC-001 Holding-Register 40001 fehlgeschlagen
  → Zeitüberschreitung: device=plc-001, timeout=3000ms
    → Verbindungsfehler: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — optionaler tokio-Wrapper

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// Asynchroner Ereignisstrom
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
