# Industrial Protocols Rust

রাস্ট ইন্ডাস্ট্রিয়াল কমিউনিকেশন প্রোটোকল স্যুট —— মাইক্রো-কার্নেল + প্রোটোকল SDK আর্কিটেকচার, ৩৮টি ইন্ডাস্ট্রিয়াল প্রোটোকল কভার করে।

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## বহুভাষা / Languages

| ভাষা | ডকুমেন্ট |
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

PHP সংস্করণ [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) ফিচার সেটকে ভিত্তি ধরে, রাস্ট ইকোসিস্টেমের রীতি অনুযায়ী পুনর্নকশা করা হয়েছে। PHP সংস্করণের ৪২টি প্রোটোকলই সম্পূর্ণ কভার করা হয়েছে।

## আর্কিটেকচার

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

**মূল ধারণা:** কার্নেল শুধু সংজ্ঞায়িত করে «প্রোটোকল কী» (৬টি trait), কোনো নির্দিষ্ট প্রোটোকল বাস্তবায়ন ধারণ করে না। প্রোটোকলগুলো আলাদা crate হিসেবে প্রয়োজন অনুযায়ী নির্ভরতা হিসাবে যুক্ত হয় এবং অভিন্ন SDK trait বাস্তবায়নের মাধ্যমে কার্নেলের সাথে সংযুক্ত হয়।

## দ্রুত শুরু

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
    // 1. কার্নেল তৈরি করুন
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

    // 2. বুট করুন (প্রোটোকল নিবন্ধন যাচাই)
    kernel.boot().expect("boot failed");

    // 3. নিবন্ধিত প্রোটোকল দেখুন
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. বন্ধ করুন
    kernel.shutdown().expect("shutdown failed");
}
```

## সমর্থিত প্রোটোকল (৩৮টি)

### ইন্ডাস্ট্রিয়াল ইথারনেট

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary প্রোটোকল স্ট্যাক, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### ফিল্ডবাস

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 মাস্টার-স্লেভ পোলিং, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | পাওয়ার SCADA, Class 0 পোলিং, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS সাবস্টেশন, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, SD2 ফ্রেম, CP 5611 গেটওয়ে |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CAN-এর উপর CIP, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | পয়েন্ট-টু-পয়েন্ট সিরিয়াল, ISDU প্যারামিটার |
| CC-Link IE Field | `industrial-protocols-cclinkie` | ইন্ডাস্ট্রিয়াল ইথারনেট সংস্করণ, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, রিং নেটওয়ার্ক শিফট রেজিস্টার |

### IoT/মেসেজিং

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | TCP/UDP-এর উপর HART, পোর্ট 5094 |

### অটোমোটিভ বাস

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| LIN | `industrial-protocols-lin` | বডি বাস, 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps ডুয়াল-চ্যানেল, টাইম-ট্রিগার্ড |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | ফাইবার মাল্টিমিডিয়া, 25/50/150 Mbps |

### বিল্ডিং/লাইটিং

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, Neuron চিপ, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, ব্রডকাস্ট কন্ট্রোল |

### হার্ডওয়্যার ব্রিজিং

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes ফ্রেম, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps টোকেন পাসিং, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | ম্যানচেস্টার এনকোডিং, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff ফাইবার রিং নেটওয়ার্ক |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 টোকেন পিয়ার-টু-পিয়ার |
| ISA100 | `industrial-protocols-isa100` | ওয়্যারলেস Mesh, YFGW410 গেটওয়ে |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, Emerson 1420 গেটওয়ে |

### সিস্টেম বাস

| প্রোটোকল | Crate | বিবরণ |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | কনফিগারেশন স্পেস + MMIO, libpci কার্নেল ব্রিজিং |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | জিওগ্রাফিক অ্যাড্রেসিং, 3U/6U Eurocard |

## মূল ধারণা / API রেফারেন্স

কার্নেল আর্কিটেকচার ও সমস্ত পাবলিক API (Protocol SDK-এর ৬টি Trait, Kernel কম্পোনেন্ট, Bridge ট্রান্সপোর্ট অ্যাবস্ট্রাকশন, Error এরর চেইন, Async অ্যাসিঙ্ক লেয়ার) সম্পর্কে দেখুন [API রেফারেন্স ডকুমেন্ট](API.md)।

## প্রজেক্ট স্ট্রাকচার

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # মাইক্রো-কার্নেল (10 মডিউল)
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
│   ├── serial/             # সিরিয়াল পোর্ট (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Vendor presets
├── protocols/              # 38 প্রোটোকল crates
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (আরও 28টি)
├── async-layer/            # tokio async wrapper
└── examples/               # simple-modbus
```

## প্রজেক্ট স্কেল

| মেট্রিক | মান |
|------|------|
| মোট crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| টেস্ট স্যুট | 137 passing |
| কার্নেল মডিউল | 10 |
| SDK traits | 6 |
| সংযোগ কৌশল | 3 (Simple/Persistent/Pooled) |
| রিট্রাই কৌশল | 4 (Fixed/Exponential/Linear/None) |
| এরর টাইপ | 10 ভেরিয়েন্ট + কনটেক্সট চেইন |
| ন্যূনতম রাস্ট | 1.75+ (2024 edition) |

## প্রযুক্তিগত সিদ্ধান্ত

| সিদ্ধান্ত | পছন্দ | কারণ |
|------|------|------|
| আর্কিটেকচার | মাইক্রো-কার্নেল + প্রোটোকল SDK | কার্নেল ন্যূনতম, প্রোটোকল ডিকপলড |
| প্যাকেজ সংগঠন | Cargo workspace + আলাদা crate | আলাদা কম্পাইল, আলাদা ভার্সন |
| অ্যাসিঙ্ক | সিঙ্ক্রোনাস কার্নেল + ঐচ্ছিক tokio | জিরো-কস্ট অপশন |
| এরর | thiserror + context() | স্ট্রাকচার্ড + কনটেক্সট চেইন |
| কনফিগারেশন | TOML + Builder | ফাইল-ড্রিভেন + কোড ওভাররাইড |
| সংযোগ | trait অ্যাবস্ট্রাকশন + ৩ কৌশল | নমনীয় পরিস্থিতি অভিযোজন |
| হার্ডওয়্যার | Transport trait + FFI | সফটওয়্যার-হার্ডওয়্যার অভিন্ন ইন্টারফেস |
| trait | অ্যাসোসিয়েটেড টাইপ + ইন্টারফেস আইসোলেশন | কম্পাইল-টাইম টাইপ সেফটি |

## ডিজাইন নীতি

1. **YAGNI** — কার্নেল শুধু প্রোটোকলগুলোর সাধারণ প্রয়োজনীয় ইনফ্রাস্ট্রাকচার সরবরাহ করে
2. **ইন্টারফেস আইসোলেশন** — ৬টি আলাদা trait, প্রয়োজন অনুযায়ী বাস্তবায়ন
3. **টাইপ সেফটি** — অ্যাসোসিয়েটেড টাইপ কম্পাইল-টাইমে বিভিন্ন প্রোটোকলের ঠিকানা/মান আলাদা করে
4. **জিরো-কস্ট ঐচ্ছিক** — async লেয়ার আলাদা, কোনো বাধ্যতামূলক রানটাইম আনে না
5. **এরর ট্রান্সপারেন্সি** — কনটেক্সট চেইন সম্পূর্ণ ব্যর্থতার পথ সংরক্ষণ করে

## দান / Donations

যদি এই প্রজেক্ট আপনার কাজে লাগে, দান করে সহায়তা করতে পারেন (যে কোনো পরিমাণ, প্রতিটি সমর্থককে ধন্যবাদ)।

### ওয়েচ্যাট পে / আলিপে

![ওয়েচ্যাট পে](weixinpay.png) ![আলিপে](alipay.png)

### আন্তর্জাতিক ট্রান্সফার (ব্যাংক SWIFT রেমিট্যান্স)

| আইটেম | তথ্য |
|------|------|
| প্রাপকের নাম | WANG KEXUN |
| প্রাপকের অ্যাকাউন্ট নম্বর | 881015918251 |
| প্রাপক ব্যাংক | ZA Bank Limited |
| SWIFT কোড | AABLHKHHXXX |
| ব্যাংক কোড | 387 |
| ব্যাংকের ঠিকানা | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **ক্রস-বর্ডার রেমিট্যান্স করেসপনডেন্ট ব্যাংক (মধ্যস্থ ব্যাংক)**: অনুগ্রহ করে লক্ষ্য করুন, এটি ক্রস-বর্ডার রেমিট্যান্স করেসপনডেন্ট ব্যাংক (মধ্যস্থ ব্যাংক) সংক্রান্ত তথ্য, **প্রাপক ব্যাংকের তথ্য নয়**। রেমিট্যান্স পাঠানোর ব্যাংককে জিজ্ঞাসা করুন, ক্রস-বর্ডার রেমিট্যান্স করেসপনডেন্ট ব্যাংক তথ্যের প্রয়োজন কিনা।

- **HKD, CNY ও USD রেমিট্যান্স** (করেসপনডেন্ট ব্যাংক Citibank):
  - ব্যাংকের নাম: Citibank N.A. Hong Kong
  - SWIFT কোড: CITIHKHXXXX
  - ব্যাংক কোড: 006
  - শাখার নাম: Hong Kong Branch
  - শাখা কোড: 391
  - ব্যাংকের ঠিকানা: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **অন্যান্য মুদ্রায় রেমিট্যান্স** (করেসপনডেন্ট ব্যাংক BNY Mellon):
  - ব্যাংকের নাম: THE BANK OF NEW YORK MELLON
  - SWIFT কোড: IRVTUS3NXXX
  - ব্যাংকের ঠিকানা: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
