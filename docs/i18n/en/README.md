# Industrial Protocols Rust

Industrial communication protocol suite for Rust — micro-kernel + protocol SDK architecture, covering 38 industrial protocols.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Languages / 多语言

| Language | Document |
|----------|----------|
| 简体中文 | [README.md](README.md) |
| English | [docs/i18n/en/README.md](docs/i18n/en/README.md) |
| 한국어 | [docs/i18n/ko/README.md](docs/i18n/ko/README.md) |
| Русский | [docs/i18n/ru/README.md](docs/i18n/ru/README.md) |
| Deutsch | [docs/i18n/de/README.md](docs/i18n/de/README.md) |
| Français | [docs/i18n/fr/README.md](docs/i18n/fr/README.md) |
| Español | [docs/i18n/es/README.md](docs/i18n/es/README.md) |
| Português | [docs/i18n/pt/README.md](docs/i18n/pt/README.md) |
| हिन्दी | [docs/i18n/hi/README.md](docs/i18n/hi/README.md) |
| العربية | [docs/i18n/ar/README.md](docs/i18n/ar/README.md) |
| বাংলা | [docs/i18n/bn/README.md](docs/i18n/bn/README.md) |
| Bahasa Indonesia | [docs/i18n/id/README.md](docs/i18n/id/README.md) |
| 日本語 | [docs/i18n/ja/README.md](docs/i18n/ja/README.md) |

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

## Core Concepts / API Reference

Kernel architecture and the full public API (Protocol SDK 6 traits, Kernel components, Bridge transport abstraction, Error chains, Async layer) are documented in the [API Reference](docs/i18n/en/API.md).

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

## Donations

If this project helps you, any donation is welcome (any amount — thanks to every supporter).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Global Wire Transfer (SWIFT)

| Field | Value |
|-------|-------|
| Beneficiary Name | WANG KEXUN |
| Beneficiary Account No. | 881015918251 |
| Beneficiary Bank | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| Bank Code | 387 |
| Bank Address | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Cross-border correspondent bank (intermediary)**: This is correspondent/intermediary bank information, **NOT the beneficiary bank**. Please check with your remitting bank whether correspondent bank information is required.

- **For HKD, CNY and USD** (correspondent: Citibank):
  - Bank name: Citibank N.A. Hong Kong
  - SWIFT Code: CITIHKHXXXX
  - Bank Code: 006
  - Branch Name: Hong Kong Branch
  - Branch Code: 391
  - Bank address: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **For other currencies** (correspondent: BNY Mellon):
  - Bank name: THE BANK OF NEW YORK MELLON
  - SWIFT Code: IRVTUS3NXXX
  - Bank address: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
