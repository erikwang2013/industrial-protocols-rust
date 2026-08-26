# Industrial Protocols Rust

مجموعة بروتوكولات الاتصال الصناعية بلغة Rust — بنية نواة مصغّرة (Micro-Kernel) + SDK بروتوكولات، تغطي 38 بروتوكولًا صناعيًا.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## اللغات / Languages

| اللغة | الوثيقة |
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

أُعيد تصميمه وفقًا لمجموعة ميزات نسخة PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols)، باتباع عادات النظام البيئي لـ Rust. يغطي جميع بروتوكولات نسخة PHP البالغ عددها 42.

## البنية

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

**الفكرة الأساسية:** تعرّف النواة فقط «ما هو البروتوكول» (6 Traits)، ولا تتضمن أي تنفيذ محدد للبروتوكولات. تُحمَّل البروتوكولات كـ crates مستقلة حسب الحاجة، وتتصل بالنواة عبر تنفيذ Traits موحّدة من SDK.

## البداية السريعة

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
    // 1. بناء النواة
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

    // 2. التشغيل (التحقق من تسجيل البروتوكول)
    kernel.boot().expect("boot failed");

    // 3. عرض البروتوكولات المسجلة
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. الإغلاق
    kernel.shutdown().expect("shutdown failed");
}
```

## البروتوكولات المدعومة (38)

### الإيثرنت الصناعية

| البروتوكول | Crate | الوصف |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII، FC 01-06/10/15، CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP، Who-Is/I-Am، ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag، RegisterSession |
| OPC UA | `industrial-protocols-opcua` | مكدس Binary، Hello/Ack، SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT، DCP Identify، Record Data Read |

### الناقل الميداني

| البروتوكول | Crate | الوصف |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA، Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | استعلام رئيسي-تابع عبر RS-485، CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | SCADA كهربائي، استعلام Class 0، CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS محطة فرعية، TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS، إطارات SD2، بوابة CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301، NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN، Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE، NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3، Data Exchange |
| IO-Link | `industrial-protocols-iolink` | تسلسلي من نقطة إلى نقطة، معاملات ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | نسخة الإيثرنت الصناعية، Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS، سجلات إزاحة حلقة الشبكة |

### إنترنت الأشياء / الرسائل

| البروتوكول | Crate | الوصف |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1، CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART عبر TCP/UDP، المنفذ 5094 |

### ناقل السيارات

| البروتوكول | Crate | الوصف |
|------|-------|------|
| LIN | `industrial-protocols-lin` | ناقل هيكل السيارة، 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II، ISO 9141/14230، KWP2000 |
| FlexRay | `industrial-protocols-flexray` | قناتان مزدوجتان 10 Mbps، تحفيز زمني |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM، CRC-8 |
| MOST | `industrial-protocols-most` | ألياف ضوئية متعددة الوسائط، 25/50/150 Mbps |

### المباني / الإضاءة

| البروتوكول | Crate | الوصف |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk، شريحة Neuron، NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386، ARC Power، تحكم بثّي |

### الجسر العتادي

| البروتوكول | Crate | الوصف |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT، BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | إطارات SoC/PReq/PRes، EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram، Service Channel |
| ControlNet | `industrial-protocols-controlnet` | تمرير رمز 5 Mbps، 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | ترميز مانشستر، FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | حلقة ألياف ضوئية من Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | نظير-لنظير بالرمز SA85/BM85 |
| ISA100 | `industrial-protocols-isa100` | Mesh لاسلكي، بوابة YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591، بوابة Emerson 1420 |

### ناقل النظام

| البروتوكول | Crate | الوصف |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | مساحة التهيئة + MMIO، جسر نواة libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32، AM Codes |
| CompactPCI | `industrial-protocols-cpci` | عنونة جغرافية، 3U/6U Eurocard |

## المفاهيم الأساسية / مرجع API

تُوثَّق بنية النواة وجميع واجهات API العامة (6 Traits في Protocol SDK، ومكونات Kernel، وتجريد النقل Bridge، وسلسلة الأخطاء Error، والطبقة غير المتزامنة Async) في [وثيقة مرجع API](API.md).

## بنية المشروع

```
industrial-protocols-rust/
├── Cargo.toml              # جذر workspace
├── kernel/                 # النواة المصغّرة (10 وحدات)
│   ├── sdk/                # 6 traits أساسية + أنواع
│   ├── registry.rs         # ProtocolRegistry
│   ├── connection.rs       # ConnectionManager
│   ├── config.rs           # ConfigRepository
│   ├── gateway.rs          # GatewayEngine
│   ├── circuit.rs          # CircuitBreaker
│   ├── event.rs            # EventBus
│   ├── retry.rs            # RetryConfig
│   ├── error.rs            # ProtocolError
│   └── bridge.rs           # trait النقل
├── bridge/
│   ├── socket/             # TCP/UDP
│   ├── serial/             # المنفذ التسلسلي (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # إعدادات مسبقة للموردين
├── protocols/              # 38 crates بروتوكولات
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 more)
├── async-layer/            # غلاف tokio غير المتزامن
└── examples/               # simple-modbus
```

## حجم المشروع

| المؤشر | القيمة |
|------|------|
| إجمالي crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| مجموعة الاختبارات | 137 passing |
| وحدات النواة | 10 |
| Traits في SDK | 6 |
| استراتيجيات الاتصال | 3 (Simple/Persistent/Pooled) |
| استراتيجيات إعادة المحاولة | 4 (Fixed/Exponential/Linear/None) |
| أنواع الأخطاء | 10 متغيرات + سلسلة سياق |
| أقل إصدار Rust | 1.75+ (2024 edition) |

## القرارات التقنية

| القرار | الخيار | السبب |
|------|------|------|
| البنية | نواة مصغّرة + SDK بروتوكولات | نواة في أبسط صورها، بروتوكولات مفكوكة |
| تنظيم الحزم | Cargo workspace + crates مستقلة | ترجمة وإصدار مستقلان |
| عدم التزامن | نواة متزامنة + tokio اختياري | خيار بدون تكلفة |
| الأخطاء | thiserror + context() | منظم + سلسلة سياق |
| الإعدادات | TOML + Builder | مدفوعة بالملفات + تغطية بالكود |
| الاتصال | تجريد trait + 3 استراتيجيات | مرونة لمختلف السيناريوهات |
| العتاد | trait النقل + FFI | واجهة موحّدة للبرمجيات والعتاد |
| traits | أنواع مرتبطة + عزل الواجهات | أمان أنواع في زمن الترجمة |

## مبادئ التصميم

1. **YAGNI** — توفر النواة فقط البنية الأساسية المشتركة التي تحتاجها البروتوكولات
2. **عزل الواجهات** — 6 traits مستقلة، تُنفَّذ حسب الحاجة
3. **أمان الأنواع** — أنواع مرتبطة تميّز عناوين وقيم البروتوكولات المختلفة في زمن الترجمة
4. **اختياري بدون تكلفة** — الطبقة غير المتزامنة مستقلة، ولا تفرض وقت تشغيل إلزاميًا
5. **شفافية الأخطاء** — سلسلة السياق تحافظ على مسار الفشل الكامل

## التبرعات / Donations

إذا كان هذا المشروع مفيدًا لك، فنرحب بتبرعك لدعمه (المبلغ حسب رغبتك، شكرًا لكل داعم).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### التحويلات العالمية (حوالة مصرفية SWIFT)

| البند | المعلومات |
|------|------|
| اسم المستفيد | WANG KEXUN |
| رقم حساب المستفيد | 881015918251 |
| البنك المستفيد | ZA Bank Limited |
| رمز SWIFT | AABLHKHHXXX |
| رقم البنك | 387 |
| عنوان البنك | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **البنك الوسيط للتحويلات عبر الحدود**: يرجى ملاحظة أن هذه معلومات البنك الوسيط (المُمرِّر) للتحويلات عبر الحدود، **وليست معلومات البنك المستفيد**. يُرجى الاستفسار من البنك المُرسِل عما إذا كان يتطلب تقديم معلومات البنك الوسيط للتحويلات عبر الحدود.

- **لإيداع الدولار الهونغ كونغي واليوان الصيني والدولار الأمريكي** (البنك الوسيط Citibank):
  - اسم البنك: Citibank N.A. Hong Kong
  - رمز SWIFT: CITIHKHXXXX
  - رقم البنك: 006
  - اسم الفرع: Hong Kong Branch
  - رقم الفرع: 391
  - عنوان البنك: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **للعملات الأخرى** (البنك الوسيط BNY Mellon):
  - اسم البنك: THE BANK OF NEW YORK MELLON
  - رمز SWIFT: IRVTUS3NXXX
  - عنوان البنك: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
