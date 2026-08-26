# Industrial Protocols Rust

Rust-Protokollsatz für industrielle Kommunikation — Mikrokernel + Protokoll-SDK-Architektur, abdeckend 38 industrielle Protokolle.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Sprachen / Languages

| Sprache | Dokumentation |
|------|------|
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

Angelehnt an den Funktionsumfang der PHP-Version [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), neu gestaltet nach den Gewohnheiten des Rust-Ökosystems. Alle 42 Protokolle der PHP-Version sind abgedeckt.

## Architektur

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

**Kernidee:** Der Kernel definiert nur, „was ein Protokoll ist" (6 Traits), und enthält keine konkrete Protokollimplementierung. Protokolle werden als eigenständige Crates nach Bedarf eingebunden und über die einheitlichen SDK-Traits an den Kernel angebunden.

## Schnellstart

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
    // 1. Kernel aufbauen
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

    // 2. Starten (prüft die Protokollregistrierung)
    kernel.boot().expect("boot failed");

    // 3. Registrierte Protokolle anzeigen
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Herunterfahren
    kernel.shutdown().expect("shutdown failed");
}
```

## Unterstützte Protokolle (38)

### Industrielles Ethernet

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary-Protokollstapel, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Feldbus

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 Master-Slave-Polling, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | Energie-SCADA, Class-0-Polling, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS-Umspannwerk, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, SD2-Frames, CP-5611-Gateway |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP über CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Punkt-zu-Punkt seriell, ISDU-Parameter |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Industrielle-Ethernet-Version, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, Ring-Schieberegister |

### IoT/Nachrichten

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART über TCP/UDP, Port 5094 |

### Fahrzeugbusse

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Karosseriebus, 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps, Dual-Kanal, zeitgesteuert |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Glasfaser-Multimedia, 25/50/150 Mbps |

### Gebäude/Beleuchtung

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, Neuron-Chip, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, Broadcast-Steuerung |

### Hardware-Brücken

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD-Datagramme |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes-Frames, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT-Telegramme, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps Token-Passing, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Manchester-Kodierung, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff Glasfaser-Ring |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 Token-Peer-to-Peer |
| ISA100 | `industrial-protocols-isa100` | Drahtloses Mesh, YFGW410-Gateway |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, Emerson-1420-Gateway |

### Systembusse

| Protokoll | Crate | Beschreibung |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Konfigurationsraum + MMIO, libpci-Kernel-Brücke |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Geografische Adressierung, 3U/6U Eurocard |

## Kernkonzepte / API-Referenz

Kernarchitektur und alle öffentlichen APIs (die 6 Traits des Protocol SDK, Kernel-Komponenten, Bridge-Transportabstraktion, Error-Fehlerkette, Async-Schicht) siehe [API-Referenzdokument](API.md).

## Projektstruktur

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # Mikrokernel (10 Module)
│   ├── sdk/                # 6 Core-Traits + Typen
│   ├── registry.rs         # ProtocolRegistry
│   ├── connection.rs       # ConnectionManager
│   ├── config.rs           # ConfigRepository
│   ├── gateway.rs          # GatewayEngine
│   ├── circuit.rs          # CircuitBreaker
│   ├── event.rs            # EventBus
│   ├── retry.rs            # RetryConfig
│   ├── error.rs            # ProtocolError
│   └── bridge.rs           # Transport-Trait
├── bridge/
│   ├── socket/             # TCP/UDP
│   ├── serial/             # Seriell (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Hersteller-Presets
├── protocols/              # 38 Protokoll-Crates
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 mehr)
├── async-layer/            # tokio-Async-Wrapper
└── examples/               # simple-modbus
```

## Projektumfang

| Kennzahl | Wert |
|------|------|
| Crates gesamt | 45 (kernel + 5 bridge + 38 protocols + async) |
| Testsuite | 137 bestanden |
| Kernel-Module | 10 |
| SDK-Traits | 6 |
| Verbindungsstrategien | 3 (Simple/Persistent/Pooled) |
| Retry-Strategien | 4 (Fixed/Exponential/Linear/None) |
| Fehlertypen | 10 Varianten + Kontextkette |
| Mindest-Rust | 1.75+ (2024 edition) |

## Technische Entscheidungen

| Entscheidung | Wahl | Begründung |
|------|------|------|
| Architektur | Mikrokernel + Protokoll-SDK | Kernel minimal, Protokolle entkoppelt |
| Paketorganisation | Cargo-Workspace + eigenständige Crates | Unabhängig kompilierbar, unabhängig versioniert |
| Async | Synchroner Kernel + optionales tokio | Null-Kosten-Wahl |
| Fehler | thiserror + context() | Strukturiert + Kontextkette |
| Konfiguration | TOML + Builder | Dateigesteuert + Code-überschreibbar |
| Verbindung | Trait-Abstraktion + 3 Strategien | Flexible Szenario-Anpassung |
| Hardware | Transport-Trait + FFI | Einheitliche Software/Hardware-Schnittstelle |
| Traits | Assoziierte Typen + Schnittstellenisolation | Typensicherheit zur Kompilierzeit |

## Designprinzipien

1. **YAGNI** — Der Kernel bietet nur die von allen Protokollen benötigte Infrastruktur
2. **Schnittstellenisolation** — 6 unabhängige Traits, nach Bedarf implementierbar
3. **Typsicherheit** — Assoziierte Typen unterscheiden Adressen/Werte verschiedener Protokolle zur Kompilierzeit
4. **Null-Kosten-Optionalität** — Die Async-Schicht ist unabhängig, kein erzwungenes Laufzeitsystem
5. **Fehlertransparenz** — Die Kontextkette bewahrt den vollständigen Fehlerpfad

## Spenden / Donations

Wenn dieses Projekt für dich hilfreich ist, freuen wir uns über eine Spende (beliebiger Betrag, Dank an jeden Unterstützer).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Internationale Überweisung (SWIFT Banküberweisung)

| Posten | Information |
|------|------|
| Empfängername | WANG KEXUN |
| Kontonummer des Empfängers | 881015918251 |
| Empfängerbank | ZA Bank Limited |
| SWIFT-Code | AABLHKHHXXX |
| Bankleitzahl | 387 |
| Bankadresse | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Interbank-Agent der grenzüberschreitenden Überweisung (Zwischenbank)**: Bitte beachten, dass es sich hierbei um Informationen zur Interbank-Agent der grenzüberschreitenden Überweisung (Zwischenbank) handelt, **nicht um Informationen zur Empfängerbank**. Bitte erfrage bei deiner überweisenden Bank, ob Angaben zur Interbank-Agent der grenzüberschreitenden Überweisung erforderlich sind.

- **Überweisung in Hongkong-Dollar, RMB und US-Dollar** (Agent: Citibank):
  - Bankname: Citibank N.A. Hong Kong
  - SWIFT-Code: CITIHKHXXXX
  - Bankleitzahl: 006
  - Filialname: Hong Kong Branch
  - Filialnummer: 391
  - Bankadresse: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Überweisung in anderen Währungen** (Agent: BNY Mellon):
  - Bankname: THE BANK OF NEW YORK MELLON
  - SWIFT-Code: IRVTUS3NXXX
  - Bankadresse: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## Lizenz

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
