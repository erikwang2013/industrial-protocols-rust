# Industrial Protocols Rust

Conjunto de protocolos de comunicación industrial en Rust — arquitectura de microkernel + SDK de protocolos, que cubre 38 protocolos industriales.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Multilingüe / Languages

| Idioma | Documento |
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

Referencia al conjunto de funciones de la versión PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), rediseñada según las convenciones del ecosistema Rust. Los 42 protocolos de la versión PHP están completamente cubiertos.

## Arquitectura

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

**Idea central:** el kernel solo define «qué es un protocolo» (6 traits), sin incluir implementaciones concretas de protocolos. Los protocolos son crates independientes que se incluyen según la necesidad y se conectan al kernel implementando los traits unificados del SDK.

## Inicio rápido

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
    // 1. Construir el kernel
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

    // 2. Arrancar (valida el registro de protocolos)
    kernel.boot().expect("boot failed");

    // 3. Consultar los protocolos registrados
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Cerrar
    kernel.shutdown().expect("shutdown failed");
}
```

## Protocolos compatibles (38)

### Ethernet industrial

| Protocolo | Crate | Descripción |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Pila binaria, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Bus de campo

| Protocolo | Crate | Descripción |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | Maestro-esclavo RS-485 por sondeo, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | SCADA eléctrico, sondeo Class 0, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | Subestaciones MMS, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, trama SD2, pasarela CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Serie punto a punto, parámetros ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Versión Ethernet industrial, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, registro de desplazamiento en anillo |

### IoT/Mensajería

| Protocolo | Crate | Descripción |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART sobre TCP/UDP, puerto 5094 |

### Bus de automoción

| Protocolo | Crate | Descripción |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Bus de carrocería, UART 19200 bps |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps doble canal, disparado por tiempo |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Fibra óptica multimedia, 25/50/150 Mbps |

### Edificios/Iluminación

| Protocolo | Crate | Descripción |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, chip Neuron, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, control por difusión |

### Puente de hardware

| Protocolo | Crate | Descripción |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, datagramas BRD/APRD |
| POWERLINK | `industrial-protocols-powerlink` | Tramas SoC/PReq/PRes, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | Telegramas MDT/AT, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | Paso de testigo a 5 Mbps, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Codificación Manchester, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Anillo de fibra óptica Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | Testigo par a par SA85/BM85 |
| ISA100 | `industrial-protocols-isa100` | Mesh inalámbrico, pasarela YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, pasarela Emerson 1420 |

### Bus de sistema

| Protocolo | Crate | Descripción |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Espacio de configuración + MMIO, puente de kernel libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Direccionamiento geográfico, Eurocard 3U/6U |

## Conceptos básicos / Referencia de API

La arquitectura del kernel y todas las API públicas (6 traits del Protocol SDK, componentes del Kernel, abstracción de transporte Bridge, cadena de errores Error, capa asíncrona Async) se describen en el [documento de referencia de API](API.md).

## Estructura del proyecto

```
industrial-protocols-rust/
├── Cargo.toml              # raíz del workspace
├── kernel/                 # microkernel (10 módulos)
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
│   ├── serial/             # puerto serie (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Presets de proveedor
├── protocols/              # 38 protocol crates
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 more)
├── async-layer/            # envoltura async tokio
└── examples/               # simple-modbus
```

## Tamaño del proyecto

| Métrica | Valor |
|------|------|
| Crates totales | 45 (kernel + 5 bridge + 38 protocols + async) |
| Suite de pruebas | 137 passing |
| Módulos del kernel | 10 |
| SDK traits | 6 |
| Estrategias de conexión | 3 (Simple/Persistent/Pooled) |
| Estrategias de reintento | 4 (Fixed/Exponential/Linear/None) |
| Tipos de error | 10 variantes + cadena de contexto |
| Rust mínimo | 1.75+ (edición 2024) |

## Decisiones técnicas

| Decisión | Elección | Motivo |
|------|------|------|
| Arquitectura | Microkernel + SDK de protocolos | Kernel mínimo, protocolos desacoplados |
| Organización de paquetes | Workspace Cargo + crate independiente | Compilación independiente, versionado independiente |
| Asincronía | Kernel síncrono + tokio opcional | Elección sin coste |
| Errores | thiserror + context() | Estructurado + cadena de contexto |
| Configuración | TOML + Builder | Impulsado por archivos + anulación en código |
| Conexión | Abstracción trait + 3 estrategias | Adaptación flexible a escenarios |
| Hardware | Transport trait + FFI | Interfaz unificada software/hardware |
| Traits | Tipos asociados + segregación de interfaces | Seguridad de tipos en tiempo de compilación |

## Principios de diseño

1. **YAGNI** — el kernel solo proporciona la infraestructura común que necesitan los protocolos
2. **Segregación de interfaces** — 6 traits independientes, implementación según necesidad
3. **Seguridad de tipos** — los tipos asociados distinguen en tiempo de compilación las direcciones/valores de distintos protocolos
4. **Opcional sin coste** — la capa async es independiente, no impone un runtime
5. **Errores transparentes** — la cadena de contexto conserva la ruta completa de fallo

## Donaciones / Donations

Si este proyecto te resulta útil, puedes apoyarlo con una donación (cualquier cantidad es bienvenida, gracias a cada colaborador).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Transferencia internacional (remesa bancaria SWIFT)

| Concepto | Información |
|------|------|
| Nombre del beneficiario | WANG KEXUN |
| Número de cuenta del beneficiario | 881015918251 |
| Banco beneficiario | ZA Bank Limited |
| Código SWIFT | AABLHKHHXXX |
| Código del banco | 387 |
| Dirección del banco | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Banco corresponsal de remesas transfronterizas (banco intermediario)**: ten en cuenta que esta es la información del banco corresponsal de remesas transfronterizas (banco intermediario), **no la del banco beneficiario**. Consulta con tu banco remitente si es necesario proporcionar la información del banco corresponsal de remesas transfronterizas.

- **Para remesas en dólares de Hong Kong, RMB y dólares estadounidenses** (banco corresponsal Citibank):
  - Nombre del banco: Citibank N.A. Hong Kong
  - Código SWIFT: CITIHKHXXXX
  - Código del banco: 006
  - Nombre de la sucursal: Hong Kong Branch
  - Código de sucursal: 391
  - Dirección del banco: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Para remesas en otras divisas** (banco corresponsal BNY Mellon):
  - Nombre del banco: THE BANK OF NEW YORK MELLON
  - Código SWIFT: IRVTUS3NXXX
  - Dirección del banco: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
