# Industrial Protocols Rust

Conjunto de protocolos de comunicação industrial em Rust — arquitetura de microkernel + SDK de protocolos, cobrindo 38 protocolos industriais.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## 多语言 / Languages

| Língua | Documento |
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

Referência ao conjunto de funcionalidades da versão PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), reprojetado conforme as convenções do ecossistema Rust. Todos os 42 protocolos da versão PHP são cobertos.

## Arquitetura

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

**Ideia central:** o kernel apenas define «o que é um protocolo» (6 traits), sem incluir implementações concretas de protocolos. Os protocolos são crates independentes, dependidos conforme a necessidade, e integram-se ao kernel implementando os traits unificados do SDK.

## Início rápido

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
    // 1. 构建内核
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

    // 2. 启动（校验协议注册）
    kernel.boot().expect("boot failed");

    // 3. 查看已注册协议
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. 关闭
    kernel.shutdown().expect("shutdown failed");
}
```

## Protocolos suportados (38)

### Ethernet industrial

| Protocolo | Crate | Descrição |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Pilha de protocolo Binary, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Fieldbus

| Protocolo | Crate | Descrição |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | Polling mestre-escravo RS-485, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | SCADA elétrico, polling Class 0, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | Subestação MMS, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, quadro SD2, gateway CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP sobre CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Serial ponto a ponto, parâmetros ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Versão Ethernet industrial, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, registrador de deslocamento em anel |

### IoT/Mensagens

| Protocolo | Crate | Descrição |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART sobre TCP/UDP, porta 5094 |

### Barramentos automotivos

| Protocolo | Crate | Descrição |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Barramento de carroceria, UART 19200 bps |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | Canal duplo 10 Mbps, acionado por tempo |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Multimídia por fibra óptica, 25/50/150 Mbps |

### Edifícios/Iluminação

| Protocolo | Crate | Descrição |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, chip Neuron, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, controle por broadcast |

### Ponte de hardware

| Protocolo | Crate | Descrição |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, Datagram BRD/APRD |
| POWERLINK | `industrial-protocols-powerlink` | Quadros SoC/PReq/PRes, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | Telegramas MDT/AT, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | Token passing 5 Mbps, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Codificação Manchester, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Anel de fibra óptica Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | Paridade por token SA85/BM85 |
| ISA100 | `industrial-protocols-isa100` | Mesh sem fio, gateway YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, gateway Emerson 1420 |

### Barramentos de sistema

| Protocolo | Crate | Descrição |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Espaço de configuração + MMIO, ponte de kernel libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Endereçamento geográfico, Eurocard 3U/6U |

## Conceitos Centrais / Referência de API

A arquitetura do kernel e todas as APIs públicas (os 6 traits do Protocol SDK, componentes do Kernel, abstração de transporte Bridge, cadeia de erros Error, camada assíncrona Async) estão no [documento de Referência de API](API.md).

## Estrutura do projeto

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # microkernel (10 modules)
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
│   ├── serial/             # porta serial (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # predefinições de fornecedores
├── protocols/              # 38 crates de protocolo
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 more)
├── async-layer/            # wrapper assíncrono tokio
└── examples/               # simple-modbus
```

## Dimensão do projeto

| Métrica | Valor |
|------|------|
| Total de crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| Suíte de testes | 137 passing |
| Módulos do kernel | 10 |
| Traits do SDK | 6 |
| Estratégias de conexão | 3 (Simple/Persistent/Pooled) |
| Estratégias de retry | 4 (Fixed/Exponential/Linear/None) |
| Tipos de erro | 10 variantes + cadeia de contexto |
| Rust mínimo | 1.75+ (edition 2024) |

## Decisões técnicas

| Decisão | Escolha | Motivo |
|------|------|------|
| Arquitetura | Microkernel + SDK de protocolos | Kernel minimalista, protocolos desacoplados |
| Organização de pacotes | Cargo workspace + crates independentes | Compilação e versionamento independentes |
| Assíncrono | Kernel síncrono + tokio opcional | Escolha com custo zero |
| Erros | thiserror + context() | Estruturado + cadeia de contexto |
| Configuração | TOML + Builder | Orientado a arquivo + sobreposição por código |
| Conexão | Abstração de trait + 3 estratégias | Adaptação flexível a cenários |
| Hardware | Transport trait + FFI | Interface unificada software/hardware |
| Trait | Tipos associados + segregação de interfaces | Segurança de tipos em tempo de compilação |

## Princípios de design

1. **YAGNI** — o kernel fornece apenas a infraestrutura comum a todos os protocolos
2. **Segregação de interfaces** — 6 traits independentes, implementados conforme a necessidade
3. **Segurança de tipos** — tipos associados distinguem endereços/valores de protocolos diferentes em tempo de compilação
4. **Opcional com custo zero** — a camada async é independente, sem impor runtime obrigatório
5. **Erros transparentes** — a cadeia de contexto preserva o caminho completo da falha

## Doações / Donations

Se este projeto for útil para você, sinta-se à vontade para apoiá-lo com uma doação (o valor fica a seu critério — obrigado a cada apoiador).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Transferência Global (remessa bancária SWIFT)

| Item | Informação |
|------|------|
| Nome do beneficiário | WANG KEXUN |
| Número da conta do beneficiário | 881015918251 |
| Banco do beneficiário | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| Código do banco | 387 |
| Endereço do banco | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Banco intermediário da remessa internacional (banco correspondente)**: observe que estas são informações do banco intermediário da remessa internacional (banco correspondente), **não do banco do beneficiário**. Consulte o banco remetente para saber se as informações do banco intermediário da remessa internacional são necessárias.

- **Remessas em dólar de Hong Kong, renminbi e dólar americano** (banco correspondente Citibank):
  - Nome do banco: Citibank N.A. Hong Kong
  - SWIFT Code：CITIHKHXXXX
  - Código do banco: 006
  - Nome da agência: Hong Kong Branch
  - Código da agência: 391
  - Endereço do banco: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Remessas em outras moedas** (banco correspondente BNY Mellon):
  - Nome do banco: THE BANK OF NEW YORK MELLON
  - SWIFT Code：IRVTUS3NXXX
  - Endereço do banco: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
