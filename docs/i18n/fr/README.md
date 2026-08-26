# Industrial Protocols Rust

Suite de protocoles de communication industrielle en Rust — architecture micro-noyau + SDK de protocoles, couvrant 38 protocoles industriels.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Multilingue / Languages

| Langue | Documentation |
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

Inspirée de la version PHP [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), cette suite a été repensée selon les usages de l'écosystème Rust. Les 42 protocoles de la version PHP sont tous couverts.

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

**Principe fondamental :** le noyau définit uniquement « ce qu'est un protocole » (6 traits), sans implémenter de protocole concret. Les protocoles sont des crates indépendantes, ajoutées comme dépendances selon les besoins, et s'intègrent au noyau en implémentant les traits unifiés du SDK.

## Démarrage rapide

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
    // 1. Construction du noyau
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

    // 2. Démarrage (vérifie l'enregistrement des protocoles)
    kernel.boot().expect("boot failed");

    // 3. Consultation des protocoles enregistrés
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Arrêt
    kernel.shutdown().expect("shutdown failed");
}
```

## Protocoles pris en charge (38)

### Ethernet industriel

| Protocole | Crate | Description |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Pile binaire, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Bus de terrain

| Protocole | Crate | Description |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, commandes 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 maître-esclave à scrutation, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | SCADA électrique, scrutation Classe 0, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS sous-station, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, trame SD2, passerelle CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP sur CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, échange de données |
| IO-Link | `industrial-protocols-iolink` | Série point à point, paramètres ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Version Ethernet industriel, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, registre à décalage en anneau |

### IoT/Messagerie

| Protocole | Crate | Description |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART sur TCP/UDP, port 5094 |

### Bus automobile

| Protocole | Crate | Description |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Bus de carrosserie, UART 19200 bps |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | Double canal 10 Mbps, déclenché par le temps |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Multimédia par fibre optique, 25/50/150 Mbps |

### Bâtiment/Éclairage

| Protocole | Crate | Description |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, puce Neuron, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, commande en diffusion |

### Ponts matériels

| Protocole | Crate | Description |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, datagrammes BRD/APRD |
| POWERLINK | `industrial-protocols-powerlink` | Trames SoC/PReq/PRes, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | Télégrammes MDT/AT, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | Jeton 5 Mbps, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Codage Manchester, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Anneau à fibre optique Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | Jeton pair à pair SA85/BM85 |
| ISA100 | `industrial-protocols-isa100` | Mesh sans fil, passerelle YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, passerelle Emerson 1420 |

### Bus système

| Protocole | Crate | Description |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Espace de configuration + MMIO, pont noyau libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Adressage géographique, Eurocard 3U/6U |

## Concepts clés / Référence API

L'architecture du noyau et toutes les API publiques (les 6 traits du SDK Protocol, les composants du Kernel, l'abstraction de transport Bridge, la chaîne d'erreurs Error, la couche asynchrone Async) sont décrites dans la [documentation de référence API](API.md).

## Structure du projet

```
industrial-protocols-rust/
├── Cargo.toml              # racine du workspace
├── kernel/                 # micro-noyau (10 modules)
│   ├── sdk/                # 6 traits de base + types
│   ├── registry.rs         # ProtocolRegistry
│   ├── connection.rs       # ConnectionManager
│   ├── config.rs           # ConfigRepository
│   ├── gateway.rs          # GatewayEngine
│   ├── circuit.rs          # CircuitBreaker
│   ├── event.rs            # EventBus
│   ├── retry.rs            # RetryConfig
│   ├── error.rs            # ProtocolError
│   └── bridge.rs           # trait Transport
├── bridge/
│   ├── socket/             # TCP/UDP
│   ├── serial/             # Série (feature-gated)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # Préréglages fournisseurs
├── protocols/              # 38 crates de protocoles
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (28 de plus)
├── async-layer/            # wrapper asynchrone tokio
└── examples/               # simple-modbus
```

## Envergure du projet

| Indicateur | Valeur |
|------|------|
| Total de crates | 45 (kernel + 5 bridge + 38 protocoles + async) |
| Suite de tests | 137 passing |
| Modules du noyau | 10 |
| Traits SDK | 6 |
| Stratégies de connexion | 3 (Simple/Persistent/Pooled) |
| Stratégies de nouvelle tentative | 4 (Fixed/Exponential/Linear/None) |
| Types d'erreur | 10 variantes + chaîne de contexte |
| Rust minimum | 1.75+ (édition 2024) |

## Décisions techniques

| Décision | Choix | Raison |
|------|------|------|
| Architecture | Micro-noyau + SDK de protocoles | Noyau minimal, protocoles découplés |
| Organisation des paquets | Cargo workspace + crates indépendants | Compilation et versionnage indépendants |
| Async | Noyau synchrone + tokio optionnel | Choix à coût zéro |
| Erreurs | thiserror + context() | Structuré + chaîne de contexte |
| Configuration | TOML + Builder | Piloté par fichiers + surcharge en code |
| Connexion | Abstraction trait + 3 stratégies | Adaptation flexible aux scénarios |
| Matériel | trait Transport + FFI | Interface unifiée logiciel/matériel |
| Trait | Types associés + ségrégation des interfaces | Sûreté de type à la compilation |

## Principes de conception

1. **YAGNI** — le noyau fournit uniquement l'infrastructure commune à tous les protocoles
2. **Ségrégation des interfaces** — 6 traits indépendants, implémentés selon les besoins
3. **Sûreté de type** — les types associés distinguent à la compilation les adresses/valeurs propres à chaque protocole
4. **Optionnel à coût zéro** — la couche async est indépendante, sans runtime imposé
5. **Erreurs transparentes** — la chaîne de contexte préserve le chemin de défaillance complet

## Soutien / Donations

Si ce projet vous est utile, vous pouvez soutenir son développement par un don (montant libre ; merci à chaque contributeur).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Virements internationaux (virement bancaire SWIFT)

| Élément | Information |
|------|------|
| Nom du bénéficiaire | WANG KEXUN |
| Numéro de compte du bénéficiaire | 881015918251 |
| Banque du bénéficiaire | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| Code bancaire | 387 |
| Adresse de la banque | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Banque correspondante pour virements transfrontaliers (banque intermédiaire)** : à noter, il s'agit des informations de la banque correspondante (intermédiaire) pour virements transfrontaliers, **et non des informations de la banque du bénéficiaire**. Veuillez demander à votre banque émettrice si ces informations de banque correspondante sont requises.

- **Virements en dollars de Hong Kong, yuans et dollars américains** (banque correspondante : Citibank) :
  - Nom de la banque : Citibank N.A. Hong Kong
  - SWIFT Code : CITIHKHXXXX
  - Code bancaire : 006
  - Nom de l'agence : Hong Kong Branch
  - Code de l'agence : 391
  - Adresse de la banque : Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Virements dans d'autres devises** (banque correspondante : BNY Mellon) :
  - Nom de la banque : THE BANK OF NEW YORK MELLON
  - SWIFT Code : IRVTUS3NXXX
  - Adresse de la banque : THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
