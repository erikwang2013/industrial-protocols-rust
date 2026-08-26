# Industrial Protocols Rust

Rust औद्योगिक संचार प्रोटोकॉल संग्रह — माइक्रो-कर्नेल + प्रोटोकॉल SDK आर्किटेक्चर, 38 औद्योगिक प्रोटोकॉल को कवर करता है।

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## बहुभाषा / Languages

| भाषा | दस्तावेज़ |
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

PHP संस्करण [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) की कार्यक्षमता का संदर्भ लेते हुए, इसे Rust पारिस्थितिकी की आदतों के अनुसार पुनः डिज़ाइन किया गया है। PHP संस्करण के सभी 42 प्रोटोकॉल शामिल हैं।

## वास्तुकला

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

**मूल अवधारणा:** कर्नेल केवल यह परिभाषित करता है कि «प्रोटोकॉल क्या है» (6 traits), इसमें कोई विशिष्ट प्रोटोकॉल कार्यान्वयन शामिल नहीं है। प्रोटोकॉल स्वतंत्र crates के रूप में आवश्यकता अनुसार शामिल किए जाते हैं, और एक समान SDK trait को लागू करके कर्नेल से जुड़ते हैं।

## त्वरित आरंभ

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
    // 1. कर्नेल का निर्माण करें
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

    // 2. आरंभ करें (प्रोटोकॉल पंजीकरण सत्यापित करें)
    kernel.boot().expect("boot failed");

    // 3. पंजीकृत प्रोटोकॉल देखें
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. बंद करें
    kernel.shutdown().expect("shutdown failed");
}
```

## समर्थित प्रोटोकॉल (38)

### औद्योगिक ईथरनेट

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | बाइनरी प्रोटोकॉल स्टैक, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### फ़ील्डबस

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, कमांड 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 मास्टर-स्लेव पोलिंग, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | पावर SCADA, क्लास 0 पोलिंग, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS सबस्टेशन, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, SD2 फ्रेम, CP 5611 गेटवे |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CAN पर CIP, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, डेटा एक्सचेंज |
| IO-Link | `industrial-protocols-iolink` | पॉइंट-टू-पॉइंट सीरियल, ISDU पैरामीटर |
| CC-Link IE Field | `industrial-protocols-cclinkie` | औद्योगिक ईथरनेट संस्करण, साइक्लिक डेटा |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, रिंग नेटवर्क शिफ्ट रजिस्टर |

### IoT/मैसेजिंग

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | TCP/UDP पर HART, पोर्ट 5094 |

### ऑटोमोटिव बस

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| LIN | `industrial-protocols-lin` | बॉडी बस, 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps डुअल-चैनल, टाइम-ट्रिगर्ड |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | फाइबर ऑप्टिक मल्टीमीडिया, 25/50/150 Mbps |

### बिल्डिंग/लाइटिंग

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, Neuron चिप, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC पावर, ब्रॉडकास्ट नियंत्रण |

### हार्डवेयर ब्रिजिंग

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD डेटाग्राम |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes फ्रेम, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT टेलीग्राम, सर्विस चैनल |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps टोकन पासिंग, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | मैनचेस्टर एन्कोडिंग, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff फाइबर ऑप्टिक रिंग |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 टोकन पीयर-टू-पीयर |
| ISA100 | `industrial-protocols-isa100` | वायरलेस मेश, YFGW410 गेटवे |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, Emerson 1420 गेटवे |

### सिस्टम बस

| प्रोटोकॉल | Crate | विवरण |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | कॉन्फ़िगरेशन स्पेस + MMIO, libpci कर्नेल ब्रिजिंग |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM कोड |
| CompactPCI | `industrial-protocols-cpci` | जियोग्राफिक एड्रेसिंग, 3U/6U यूरोकार्ड |

## मुख्य अवधारणाएँ / API संदर्भ

कर्नेल आर्किटेक्चर और सभी सार्वजनिक APIs (Protocol SDK के 6 Traits, Kernel घटक, Bridge परिवहन अमूर्तता, Error श्रृंखला, Async परत) के लिए [API संदर्भ दस्तावेज़](docs/i18n/hi/API.md) देखें।

## प्रोजेक्ट संरचना

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # माइक्रो-कर्नेल (10 modules)
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
│   ├── serial/             # सीरियल पोर्ट (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Vendor presets
├── protocols/              # 38 प्रोटोकॉल crates
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 more)
├── async-layer/            # tokio async रैपर
└── examples/               # simple-modbus
```

## प्रोजेक्ट आकार

| मीट्रिक | मान |
|------|------|
| कुल crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| टेस्ट सूट | 137 passing |
| कर्नेल मॉड्यूल | 10 |
| SDK traits | 6 |
| कनेक्शन रणनीतियाँ | 3 (Simple/Persistent/Pooled) |
| रीट्राई रणनीतियाँ | 4 (Fixed/Exponential/Linear/None) |
| त्रुटि प्रकार | 10 वेरिएंट + संदर्भ श्रृंखला |
| न्यूनतम Rust | 1.75+ (2024 edition) |

## तकनीकी निर्णय

| निर्णय | चयन | कारण |
|------|------|------|
| वास्तुकला | माइक्रो-कर्नेल + प्रोटोकॉल SDK | कर्नेल न्यूनतम, प्रोटोकॉल डिकपल्ड |
| पैकेज संगठन | Cargo workspace + स्वतंत्र crate | स्वतंत्र संकलन, स्वतंत्र संस्करण |
| एसिंक्रोनस | सिंक्रोनस कर्नेल + वैकल्पिक tokio | शून्य-लागत विकल्प |
| त्रुटि | thiserror + context() | संरचित + संदर्भ श्रृंखला |
| कॉन्फ़िगरेशन | TOML + Builder | फ़ाइल-संचालित + कोड ओवरराइड |
| कनेक्शन | trait अमूर्तता + 3 रणनीतियाँ | लचीला परिदृश्य अनुकूलन |
| हार्डवेयर | Transport trait + FFI | सॉफ्टवेयर-हार्डवेयर एकीकृत इंटरफ़ेस |
| trait | संबद्ध प्रकार + इंटरफ़ेस अलगाव | संकलन-समय टाइप सुरक्षा |

## डिज़ाइन सिद्धांत

1. **YAGNI** — कर्नेल केवल वह बुनियादी ढांचा प्रदान करता है जिसकी सभी प्रोटोकॉल को आवश्यकता होती है
2. **इंटरफ़ेस अलगाव** — 6 स्वतंत्र traits, आवश्यकता अनुसार लागू करें
3. **टाइप सुरक्षा** — संबद्ध प्रकार संकलन-समय पर विभिन्न प्रोटोकॉल के पते/मान अलग करते हैं
4. **शून्य-लागत वैकल्पिक** — async परत स्वतंत्र है, कोई अनिवार्य रनटाइम नहीं
5. **त्रुटि पारदर्शिता** — संदर्भ श्रृंखला पूर्ण विफलता पथ को संरक्षित करती है

## दान / Donations

अगर यह प्रोजेक्ट आपके लिए उपयोगी है, तो कृपया दान करके समर्थन करें (राशि आपके विवेक पर, हर समर्थक का आभार)।

### वीचैट पे / अलीपे

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### वैश्विक स्थानांतरण (बैंक SWIFT रेमिटेंस)

| आइटम | जानकारी |
|------|------|
| प्राप्तकर्ता का नाम | WANG KEXUN |
| प्राप्तकर्ता खाता संख्या | 881015918251 |
| प्राप्तकर्ता बैंक | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| बैंक कोड | 387 |
| बैंक का पता | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **क्रॉस-बॉर्डर रेमिटेंस एजेंट बैंक (मध्यस्थ बैंक)**: कृपया ध्यान दें, यह क्रॉस-बॉर्डर रेमिटेंस एजेंट बैंक (मध्यस्थ बैंक) की जानकारी है, **प्राप्तकर्ता बैंक की जानकारी नहीं**। कृपया अपने रेमिटिंग बैंक से पूछें कि क्या क्रॉस-बॉर्डर रेमिटेंस एजेंट बैंक जानकारी प्रदान करना आवश्यक है।

- **हांगकांग डॉलर, चीनी युआन और अमेरिकी डॉलर में रेमिट करने के लिए** (एजेंट बैंक Citibank):
  - बैंक का नाम: Citibank N.A. Hong Kong
  - SWIFT Code: CITIHKHXXXX
  - बैंक कोड: 006
  - शाखा का नाम: Hong Kong Branch
  - शाखा कोड: 391
  - बैंक का पता: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **अन्य मुद्राओं में रेमिट करने के लिए** (एजेंट बैंक BNY Mellon):
  - बैंक का नाम: THE BANK OF NEW YORK MELLON
  - SWIFT Code: IRVTUS3NXXX
  - बैंक का पता: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
