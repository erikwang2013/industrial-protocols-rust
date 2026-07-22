# Industrial Protocols Rust

Industrial communication protocol suite for Rust — micro-kernel + protocol SDK architecture, covering 38 industrial protocols.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

[中文文档](README.md)

Port of the PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) library, redesigned with Rust idioms. All 42 PHP protocols covered.

## Architecture

```
┌─────────────────────────────────────────────────┐
│               User Application                  │
├─────────────────────────────────────────────────┤
│          Async Layer (tokio, optional)           │
│        AsyncKernel · AsyncEventStream           │
├─────────────────────────────────────────────────┤
│                 Micro-Kernel                     │
│  ┌─────────────┐ ┌──────────────┐ ┌──────────┐ │
│  │Protocol     │ │Connection    │ │Config    │ │
│  │Registry     │ │Manager       │ │Repository│ │
│  ├─────────────┤ ├──────────────┤ ├──────────┤ │
│  │Gateway      │ │Circuit       │ │Event     │ │
│  │Engine       │ │Breaker       │ │Bus       │ │
│  ├─────────────┤ ├──────────────┤ ├──────────┤ │
│  │Retry        │ │Error(Context) │ │          │ │
│  └─────────────┘ └──────────────┘ └──────────┘ │
├─────────────────────────────────────────────────┤
│         Protocol SDK — 6 Core Traits            │
│  Protocol · Read · Write · Subscribe            │
│  Discover · Health · Diagnostics                │
├─────────────────────────────────────────────────┤
│  38 Protocol Crates  │  5 Bridge Transports     │
│                      │  socket · serial         │
│                      │  can · ffi · vendor      │
└─────────────────────────────────────────────────┘
```

**Core idea:** The kernel defines "what a protocol is" (6 traits) without containing any protocol implementations. Protocol crates are independently versioned and pulled in on demand. Each crate implements the unified SDK traits to plug into the kernel.

## Quick Start

```toml
# Cargo.toml
[dependencies]
industrial-protocols-kernel = "0.1"
industrial-protocols-modbus = "0.1"
```

```rust
use industrial_protocols_kernel::*;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_modbus::ModbusProtocol;

fn main() {
    // 1. Build the kernel
    let mut kernel = Kernel::builder()
        .config_from_str(r#"
            [devices.plc-001]
            protocol = "modbus"
            variant = "tcp"
            host = "192.168.1.10"
            port = 502
            unit_id = 1
            timeout_ms = 3000
        "#)
        .with_protocol(Box::new(ModbusProtocol::tcp()))
        .build();

    // 2. Boot (validates protocol registrations)
    kernel.boot().expect("boot failed");

    // 3. Inspect registered protocols
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Shutdown
    kernel.shutdown().expect("shutdown failed");
}
```

## Supported Protocols (38)

### Industrial Ethernet (5)

| Protocol | Crate | Description |
|----------|-------|-------------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary stack, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Fieldbus (12)

| Protocol | Crate | Description |
|----------|-------|-------------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 master-slave, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | Power SCADA, Class 0 poll, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS substation, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, SD2 frames, CP 5611 gateway |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Point-to-point, ISDU params |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Industrial Ethernet variant, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, ring shift register |

### IoT / Messaging (2)

| Protocol | Crate | Description |
|----------|-------|-------------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART over TCP/UDP, port 5094 |

### Automotive Bus (5)

| Protocol | Crate | Description |
|----------|-------|-------------|
| LIN | `industrial-protocols-lin` | Body bus, 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps dual-channel, time-triggered |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Fiber optic multimedia, 25/50/150 Mbps |

### Building / Lighting (2)

| Protocol | Crate | Description |
|----------|-------|-------------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, Neuron chip, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, broadcast |

### Hardware Bridge (9)

| Protocol | Crate | Description |
|----------|-------|-------------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes frames, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps token-passing, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Manchester encoded, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff fiber ring |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 token peer-to-peer |
| ISA100 | `industrial-protocols-isa100` | Wireless mesh, YFGW410 gateway |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, Emerson 1420 gateway |

### System Bus (3)

| Protocol | Crate | Description |
|----------|-------|-------------|
| PCI/PCIe | `industrial-protocols-pci` | Config space + MMIO, libpci bridge |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Geographic addressing, 3U/6U |

## Core Concepts

### 1. Protocol SDK — 6 Traits

Every protocol plugs into the kernel by implementing these traits:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // protocol-specific address type
    type Value: ValueLike;       // protocol-specific value type
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

**Associated types** make address and value types type-safe: Modbus uses `u16`, OPC UA uses `String`, BACnet uses `f32`. **Trait separation** means a read-only sensor doesn't need `ProtocolWrite`.

### 2. Kernel — Shared Infrastructure

The kernel provides no protocol logic — only shared plumbing:

| Component | Purpose | Key API |
|-----------|---------|---------|
| `ProtocolRegistry` | Register, find, list protocols | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Device connection lifecycle | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML config + runtime overrides | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Pub/sub events | `subscribe()`, `publish()` |
| `CircuitBreaker` | Protective Closed->Open->HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fixed/Exponential/Linear/None | `run()`, `start()` |
| `GatewayEngine` | Cross-protocol data forwarding | `add_rule()`, `start()/stop()` |

Three connection strategies:
```rust
pub enum ConnectionStrategy {
    Simple,      // New connection per use, closed after
    Persistent,  // Single reuse, auto-reconnect
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

### 3. Bridge Layer — Transport Abstraction

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementation | Deps |
|--------|---------------|------|
| `bridge-socket` | TCP connect + UDP send/recv | std |
| `bridge-serial` | Real serial port, `serial` feature opt-in | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | Vendor C SDK dlopen/dlsym | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider presets | — |

### 4. Error — Structured Error Chains

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),                       // host:port refused
    Timeout { device: String, timeout_ms: u64 },
    Parse { protocol: String, reason: String },
    DeviceError { device: String, code: u16, message: String },
    CircuitOpen { device: String, retry_after: Duration },
    Config(String),
    ProtocolSpecific(Box<dyn StdError + Send + Sync>),
    Context { context: String, source: Box<ProtocolError> },
    // ...
}
```

Error chains preserve the full failure path:
```
failed to read holding register 40001 from PLC-001
  -> timeout: device=plc-001, timeout=3000ms
    -> connection failed: 192.168.1.10:502: Connection refused
```

### 5. Async Layer — Optional tokio Wrapper

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

## Project Structure

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # micro-kernel (10 modules)
│   ├── sdk/                # 6 core traits + types
│   ├── registry.rs         # ProtocolRegistry
│   ├── connection.rs       # ConnectionManager
│   ├── config.rs           # ConfigRepository
│   ├── gateway.rs          # GatewayEngine
│   ├── circuit.rs          # CircuitBreaker
│   ├── event.rs            # EventBus
│   ├── retry.rs            # RetryConfig
│   ├── error.rs            # ProtocolError
│   └── bridge.rs           # Transport trait
├── bridge/
│   ├── socket/             # TCP/UDP
│   ├── serial/             # Serial (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Vendor presets
├── protocols/              # 38 protocol crates
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 more)
├── async-layer/            # tokio async wrapper
└── examples/               # simple-modbus
```

## Project Scale

| Metric | Value |
|--------|-------|
| Total crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| Test suites | 137 passing |
| Kernel modules | 10 |
| SDK traits | 6 |
| Connection strategies | 3 (Simple/Persistent/Pooled) |
| Retry strategies | 4 (Fixed/Exponential/Linear/None) |
| Error variants | 10 + context chain |
| MSRV | 1.75+ (2024 edition) |

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Architecture | Micro-kernel + protocol SDK | Minimal kernel, decoupled protocols |
| Packaging | Cargo workspace + independent crates | Per-crate compilation and versioning |
| Async | Sync core + optional tokio | Zero-cost choice, no forced runtime |
| Errors | thiserror + context() | Structured matching + error chains |
| Config | TOML files + Builder | File-driven with code overrides |
| Connections | trait + 3 strategies | Flexible per-device configuration |
| Hardware | Transport trait + FFI | Unified software/hardware interface |
| Trait design | Associated types + ISP | Compile-time type safety, implement only what you need |

## Design Principles

1. **YAGNI** — Kernel provides only what all protocols share
2. **Interface Segregation** — 6 independent traits, opt-in per protocol
3. **Type Safety** — Associated types prevent address/value confusion at compile time
4. **Zero-Cost Optional** — Async layer is independent, no forced runtime
5. **Error Transparency** — Context chains preserve full failure paths

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
