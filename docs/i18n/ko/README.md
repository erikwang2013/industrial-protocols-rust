# Industrial Protocols Rust

Rust 산업용 통신 프로토콜 모음 — 마이크로커널 + 프로토콜 SDK 아키텍처, 38가지 산업용 프로토콜 지원.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## 언어 / Languages

| 언어 | 문서 |
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

PHP 버전 [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) 의 기능 세트를 참고하여 Rust 생태계의 관례에 맞게 재설계했습니다. PHP 버전의 42개 프로토콜을 모두 포함합니다.

## 아키텍처

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

**핵심 개념:** 커널은 "프로토콜이 무엇인지"(6개 trait)만 정의하며, 구체적인 프로토콜 구현은 포함하지 않습니다. 프로토콜은 독립적인 crate로 필요에 따라 의존하며, 통일된 SDK trait을 구현하여 커널에 연결됩니다.

## 빠른 시작

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
    // 1. 커널 구축
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

    // 2. 시작 (프로토콜 등록 검증)
    kernel.boot().expect("boot failed");

    // 3. 등록된 프로토콜 확인
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. 종료
    kernel.shutdown().expect("shutdown failed");
}
```

## 지원 프로토콜 (38종)

### 산업용 이더넷

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary 프로토콜 스택, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### 필드버스

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 마스터/슬레이브 폴링, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | 전력 SCADA, Class 0 폴링, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS 변전소, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, SD2 프레임, CP 5611 게이트웨이 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CAN 기반 CIP, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | 점대점 직렬, ISDU 파라미터 |
| CC-Link IE Field | `industrial-protocols-cclinkie` | 산업용 이더넷 버전, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, 링 네트워크 시프트 레지스터 |

### IoT/메시징

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | TCP/UDP 기반 HART, 포트 5094 |

### 자동차 버스

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| LIN | `industrial-protocols-lin` | 차체 버스, 19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps 듀얼 채널, 시간 트리거 |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | 광섬유 멀티미디어, 25/50/150 Mbps |

### 빌딩/조명

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, Neuron 칩, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, 브로드캐스트 제어 |

### 하드웨어 브리지

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes 프레임, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps 토큰 패싱, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | 맨체스터 인코딩, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff 광섬유 링 네트워크 |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 토큰 피어 투 피어 |
| ISA100 | `industrial-protocols-isa100` | 무선 Mesh, YFGW410 게이트웨이 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, Emerson 1420 게이트웨이 |

### 시스템 버스

| 프로토콜 | Crate | 설명 |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | 구성 공간 + MMIO, libpci 커널 브리지 |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | 지리적 주소 지정, 3U/6U Eurocard |

## 핵심 개념 / API 참조

커널 아키텍처와 모든 공개 API(Protocol SDK 6개 Trait, Kernel 컴포넌트, Bridge 전송 추상화, Error 오류 체인, Async 비동기 레이어)는 [API 참조 문서](API.md)를 참조하세요.

## 프로젝트 구조

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # 마이크로커널 (10 modules)
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
│   ├── serial/             # 직렬 포트 (feature-gated)
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

## 프로젝트 규모

| 지표 | 값 |
|------|------|
| 총 crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| 테스트 스위트 | 137 passing |
| 커널 모듈 | 10 |
| SDK traits | 6 |
| 연결 전략 | 3 (Simple/Persistent/Pooled) |
| 재시도 전략 | 4 (Fixed/Exponential/Linear/None) |
| 오류 유형 | 10 variants + 컨텍스트 체인 |
| 최소 Rust | 1.75+ (2024 edition) |

## 기술 결정 사항

| 결정 | 선택 | 이유 |
|------|------|------|
| 아키텍처 | 마이크로커널 + 프로토콜 SDK | 커널 최소화, 프로토콜 분리 |
| 패키지 구성 | Cargo workspace + 독립 crate | 독립 컴파일, 독립 버전 관리 |
| 비동기 | 동기 커널 + 선택적 tokio | 제로 비용 선택 |
| 오류 | thiserror + context() | 구조화 + 컨텍스트 체인 |
| 설정 | TOML + Builder | 파일 기반 + 코드 오버라이드 |
| 연결 | trait 추상화 + 3가지 전략 | 다양한 시나리오 대응 |
| 하드웨어 | Transport trait + FFI | 소프트웨어/하드웨어 통일 인터페이스 |
| trait | 연관 타입 + 인터페이스 분리 | 컴파일 타임 타입 안전성 |

## 설계 원칙

1. **YAGNI** — 커널은 프로토콜이 공통으로 필요로 하는 인프라만 제공
2. **인터페이스 분리** — 6개의 독립 trait, 필요에 따라 구현
3. **타입 안전성** — 연관 타입으로 프로토콜별 주소/값을 컴파일 타임에 구분
4. **제로 비용 선택** — async 레이어는 독립적이며 강제 런타임을 도입하지 않음
5. **오류 투명성** — 컨텍스트 체인이 전체 장애 경로를 보존

## 후원 / Donations

이 프로젝트가 도움이 되셨다면 후원해 주시면 감사하겠습니다(금액은 자유입니다. 모든 후원자께 감사드립니다).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### 해외 송금 (은행 SWIFT 송금)

| 항목 | 정보 |
|------|------|
| 수취인 이름 | WANG KEXUN |
| 수취인 계좌번호 | 881015918251 |
| 수취은행 | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| 은행 번호 | 387 |
| 은행 주소 | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **해외 송금 대리은행(중개은행)**: 이것은 해외 송금 대리은행(중개은행) 정보이며, **수취은행 정보가 아닙니다**. 송금 은행에 해외 송금 대리은행 정보가 필요한지 문의하시기 바랍니다.

- **홍콩 달러, 위안화, 미국 달러 송금** (대리은행 Citibank):
  - 은행 이름: Citibank N.A. Hong Kong
  - SWIFT Code: CITIHKHXXXX
  - 은행 번호: 006
  - 지점 이름: Hong Kong Branch
  - 지점 번호: 391
  - 은행 주소: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **기타 통화 송금** (대리은행 BNY Mellon):
  - 은행 이름: THE BANK OF NEW YORK MELLON
  - SWIFT Code: IRVTUS3NXXX
  - 은행 주소: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
