# Industrial Protocols Rust

Kumpulan protokol komunikasi industri Rust — arsitektur mikrokernel + SDK protokol, mencakup 38 protokol industri.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Bahasa / Languages

| Bahasa | Dokumentasi |
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

Mengacu pada kumpulan fitur versi PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), didesain ulang sesuai kebiasaan ekosistem Rust. Seluruh 42 protokol versi PHP tercakup.

## Arsitektur

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

**Konsep inti:** Kernel hanya mendefinisikan «apa itu protokol» (6 trait), tanpa menyertakan implementasi protokol konkret. Protokol dimuat sebagai crate terpisah sesuai kebutuhan, terhubung ke kernel dengan mengimplementasikan trait SDK yang seragam.

## Memulai Cepat

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
    // 1. Bangun kernel
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

    // 2. Nyalakan (validasi registrasi protokol)
    kernel.boot().expect("boot failed");

    // 3. Lihat protokol yang terdaftar
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Matikan
    kernel.shutdown().expect("shutdown failed");
}
```

## Protokol yang Didukung (38)

### Ethernet Industri

| Protokol | Crate | Deskripsi |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Tumpukan protokol Binary, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Fieldbus

| Protokol | Crate | Deskripsi |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 polling master-slave, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | SCADA kelistrikan, polling Class 0, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | Gardu induk MMS, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, frame SD2, gateway CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Serial point-to-point, parameter ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Versi Ethernet industri, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, register geser ring network |

### IoT/Pesan

| Protokol | Crate | Deskripsi |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART over TCP/UDP, port 5094 |

### Bus Otomotif

| Protokol | Crate | Deskripsi |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Bus bodi kendaraan, UART 19200 bps |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | Dua saluran 10 Mbps, pemicu waktu |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Multimedia serat optik, 25/50/150 Mbps |

### Bangunan/Pencahayaan

| Protokol | Crate | Deskripsi |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, chip Neuron, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, kontrol siaran |

### Jembatan Perangkat Keras

| Protokol | Crate | Deskripsi |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, Datagram BRD/APRD |
| POWERLINK | `industrial-protocols-powerlink` | Frame SoC/PReq/PRes, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | Telegram MDT/AT, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | Token passing 5 Mbps, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Pengkodean Manchester, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Ring fiber Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | Token peer-to-peer SA85/BM85 |
| ISA100 | `industrial-protocols-isa100` | Mesh nirkabel, gateway YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, gateway Emerson 1420 |

### Bus Sistem

| Protokol | Crate | Deskripsi |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Ruang konfigurasi + MMIO, jembatan kernel libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Pengalamatan geografis, Eurocard 3U/6U |

## Konsep Inti / Referensi API

Arsitektur kernel dan seluruh API publik (6 Trait Protocol SDK, komponen Kernel, abstraksi transport Bridge, rantai kesalahan Error, lapisan Async) lihat [dokumen Referensi API](API.md).

## Struktur Proyek

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # mikrokernel (10 modules)
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
├── async-layer/            # pembungkus async tokio
└── examples/               # simple-modbus
```

## Skala Proyek

| Metrik | Nilai |
|------|------|
| Total crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| Rangkaian pengujian | 137 passing |
| Modul kernel | 10 |
| SDK traits | 6 |
| Strategi koneksi | 3 (Simple/Persistent/Pooled) |
| Strategi retry | 4 (Fixed/Exponential/Linear/None) |
| Tipe kesalahan | 10 varian + rantai konteks |
| Rust minimal | 1.75+ (edition 2024) |

## Keputusan Teknis

| Keputusan | Pilihan | Alasan |
|------|------|------|
| Arsitektur | Mikrokernel + SDK protokol | Kernel minimal, protokol terdekopling |
| Organisasi paket | Cargo workspace + crate terpisah | Kompilasi mandiri, versi mandiri |
| Async | Kernel sinkron + tokio opsional | Pilihan biaya nol |
| Kesalahan | thiserror + context() | Terstruktur + rantai konteks |
| Konfigurasi | TOML + Builder | Berbasis file + override kode |
| Koneksi | Abstraksi trait + 3 strategi | Adaptasi fleksibel sesuai skenario |
| Perangkat keras | Transport trait + FFI | Antarmuka terpadu perangkat lunak/perangkat keras |
| trait | Associated type + segregasi antarmuka | Keamanan tipe saat kompilasi |

## Prinsip Desain

1. **YAGNI** — kernel hanya menyediakan infrastruktur yang dibutuhkan bersama oleh protokol
2. **Segregasi antarmuka** — 6 trait independen, diimplementasikan sesuai kebutuhan
3. **Keamanan tipe** — associated type membedakan alamat/nilai antar protokol saat kompilasi
4. **Opsional biaya nol** — lapisan async independen, tanpa runtime paksa
5. **Transparansi kesalahan** — rantai konteks mempertahankan jalur kegagalan lengkap

## Donasi

Jika proyek ini bermanfaat bagi Anda, silakan berikan dukungan donasi (jumlah bebas, terima kasih kepada setiap pendukung).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Transfer Global (SWIFT Bank Transfer)

| Item | Informasi |
|------|------|
| Nama penerima | WANG KEXUN |
| Nomor rekening penerima | 881015918251 |
| Bank penerima | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| Kode bank | 387 |
| Alamat bank | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Bank perantara transfer lintas negara (correspondent bank)**: Perlu diperhatikan, ini adalah informasi bank perantara transfer lintas negara, **bukan informasi bank penerima**. Silakan tanyakan kepada bank pengirim apakah perlu menyertakan informasi bank perantara transfer lintas negara.

- **Setoran dalam Dolar Hong Kong, RMB, dan Dolar AS** (bank perantara Citibank):
  - Nama bank: Citibank N.A. Hong Kong
  - SWIFT Code: CITIHKHXXXX
  - Kode bank: 006
  - Nama cabang: Hong Kong Branch
  - Kode cabang: 391
  - Alamat bank: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Setoran dalam mata uang lainnya** (bank perantara BNY Mellon):
  - Nama bank: THE BANK OF NEW YORK MELLON
  - SWIFT Code: IRVTUS3NXXX
  - Alamat bank: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
