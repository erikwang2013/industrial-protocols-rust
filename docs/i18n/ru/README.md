# Industrial Protocols Rust

Набор промышленных протоколов связи на Rust — архитектура «микроядро + Protocol SDK», покрывающая 38 промышленных протоколов.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## Многоязычность / Languages

| Язык | Документация |
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

За основу взят функционал PHP-версии [industrial-protocols](https://github.com/erikwang2013/industrial-protocols), переработанный в соответствии с экосистемой Rust. Все 42 протокола PHP-версии покрыты.

## Архитектура

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

**Основная идея:** ядро определяет только «что такое протокол» (6 trait-ов) и не содержит реализаций конкретных протоколов. Протоколы подключаются как отдельные crate-ы по мере необходимости, интегрируясь в ядро через единый SDK-интерфейс.

## Быстрый старт

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
    // 1. Собираем ядро
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

    // 2. Запуск (проверка регистрации протокола)
    kernel.boot().expect("boot failed");

    // 3. Просмотр зарегистрированных протоколов
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. Остановка
    kernel.shutdown().expect("shutdown failed");
}
```

## Поддерживаемые протоколы (38)

### Промышленный Ethernet

| Протокол | Crate | Описание |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII, FC 01-06/10/15, CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP, Who-Is/I-Am, ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag, RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Бинарный стек, Hello/Ack, SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT, DCP Identify, Record Data Read |

### Полевая шина

| Протокол | Crate | Описание |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA, Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 ведущий/ведомый с опросом, CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | Энергетическая SCADA, опрос Class 0, CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS на подстанциях, TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS, кадры SD2, шлюз CP 5611 |
| CANopen | `industrial-protocols-canopen` | CiA 301, NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP поверх CAN, Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE, NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3, Data Exchange |
| IO-Link | `industrial-protocols-iolink` | Последовательная точка-точка, параметры ISDU |
| CC-Link IE Field | `industrial-protocols-cclinkie` | Версия на промышленном Ethernet, Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS, кольцо со сдвиговыми регистрами |

### IoT/Сообщения

| Протокол | Crate | Описание |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1, CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART по TCP/UDP, порт 5094 |

### Автомобильные шины

| Протокол | Crate | Описание |
|------|-------|------|
| LIN | `industrial-protocols-lin` | Кузовная шина, 19200 бод UART |
| K-Line | `industrial-protocols-kline` | OBD-II, ISO 9141/14230, KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Мбит/с, два канала, время-триггерная передача |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM, CRC-8 |
| MOST | `industrial-protocols-most` | Оптоволоконное мультимедиа, 25/50/150 Мбит/с |

### Здания/Освещение

| Протокол | Crate | Описание |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk, чип Neuron, NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386, ARC Power, широковещательное управление |

### Аппаратный мост

| Протокол | Crate | Описание |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT, BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | Кадры SoC/PReq/PRes, EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram, Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Мбит/с, передача маркера, 1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | Манчестерское кодирование, FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Оптоволоконное кольцо Beckhoff |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85, одноранговый доступ по маркеру |
| ISA100 | `industrial-protocols-isa100` | Беспроводная Mesh-сеть, шлюз YFGW410 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591, шлюз Emerson 1420 |

### Системные шины

| Протокол | Crate | Описание |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | Конфигурационное пространство + MMIO, мост к ядру через libpci |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32, AM Codes |
| CompactPCI | `industrial-protocols-cpci` | Географическая адресация, Eurocard 3U/6U |

## Основные концепции / API 参考

Архитектура ядра и все публичные API (6 Trait-ов Protocol SDK, компоненты Kernel, транспортная абстракция Bridge, цепочка ошибок Error, асинхронный слой Async) описаны в [документации по API](API.md).

## Структура проекта

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # микроядро (10 модулей)
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
│   ├── serial/             # последовательный порт (за feature-флагом)
│   ├── can/                # SocketCAN
│   ├── ffi/                # libloading
│   └── vendor/             # пресеты вендоров
├── protocols/              # 38 crate-ов протоколов
│   ├── modbus/  opcua/  bacnet/  ethernet-ip/  profinet/
│   ├── mqtt/  dnp3/  hart/  cclink/  iec61850/
│   ├── canopen/  profibus/  devicenet/  ... (ещё 28)
├── async-layer/            # асинхронная обёртка tokio
└── examples/               # simple-modbus
```

## Масштаб проекта

| Показатель | Значение |
|------|------|
| Всего crate-ов | 45 (kernel + 5 bridge + 38 protocols + async) |
| Тестовый набор | 137 passed |
| Модулей ядра | 10 |
| SDK traits | 6 |
| Стратегий подключения | 3 (Simple/Persistent/Pooled) |
| Стратегий повтора | 4 (Fixed/Exponential/Linear/None) |
| Типов ошибок | 10 вариантов + цепочка контекста |
| Минимальная версия Rust | 1.75+ (edition 2024) |

## Технические решения

| Решение | Выбор | Обоснование |
|------|------|------|
| Архитектура | Микроядро + Protocol SDK | Минималистичное ядро, развязанные протоколы |
| Организация пакетов | Cargo workspace + отдельные crate-ы | Раздельная компиляция, раздельные версии |
| Асинхронность | Синхронное ядро + опциональный tokio | Бесплатный выбор (zero-cost) |
| Ошибки | thiserror + context() | Структурированность + цепочка контекста |
| Конфигурация | TOML + Builder | Файловый драйвер + переопределение кодом |
| Подключение | Абстракция trait + 3 стратегии | Адаптация к различным сценариям |
| Аппаратура | Transport trait + FFI | Единый интерфейс «софт/железо» |
| Trait | Ассоциированные типы + изоляция интерфейсов | Типобезопасность на этапе компиляции |

## Принципы проектирования

1. **YAGNI** — ядро предоставляет только инфраструктуру, общую для всех протоколов
2. **Изоляция интерфейсов** — 6 независимых trait-ов, реализуются по мере необходимости
3. **Типобезопасность** — ассоциированные типы различают адреса/значения разных протоколов на этапе компиляции
4. **Опциональность с нулевой стоимостью** — async-слой независим, не навязывает обязательную среду выполнения
5. **Прозрачность ошибок** — цепочка контекста сохраняет полный путь отказа

## Пожертвования / Donations

Если этот проект оказался вам полезен, поддержите его донатом (сумма любая — благодарим каждого поддержавшего).

### WeChat Pay / Alipay

![WeChat Pay](weixinpay.png) ![Alipay](alipay.png)

### Международный перевод (банковский SWIFT-перевод)

| Пункт | Информация |
|------|------|
| Имя получателя | WANG KEXUN |
| Номер счёта получателя | 881015918251 |
| Банк получателя | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| Банковский код | 387 |
| Адрес банка | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **Банк-посредник (correspondent bank) для трансграничных переводов**: обратите внимание — это информация о банке-посреднике (correspondent bank) для трансграничных переводов, **а не о банке получателя**. Уточните в банке-отправителе, требуется ли указывать данные банка-посредника.

- **Переводы в гонконгских долларах, юанях и долларах США** (банк-посредник Citibank):
  - Название банка: Citibank N.A. Hong Kong
  - SWIFT Code: CITIHKHXXXX
  - Банковский код: 006
  - Название отделения: Hong Kong Branch
  - Код отделения: 391
  - Адрес банка: Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **Переводы в других валютах** (банк-посредник BNY Mellon):
  - Название банка: THE BANK OF NEW YORK MELLON
  - SWIFT Code: IRVTUS3NXXX
  - Адрес банка: THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
