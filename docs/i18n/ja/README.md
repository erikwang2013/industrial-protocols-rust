# Industrial Protocols Rust

Rust 産業用通信プロトコル集 —— マイクロカーネル + プロトコル SDK アーキテクチャで、38 種類の産業用プロトコルをカバーします。

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

## 多言語 / Languages

| 言語 | ドキュメント |
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

PHP 版 [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) の機能セットを参考に、Rust エコシステムの慣習に合わせて再設計しました。PHP 版の 42 プロトコルをすべてカバーしています。

## アーキテクチャ

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

**核心理念：** カーネルは「プロトコルとは何か」（6 つの trait）のみを定義し、具体的なプロトコル実装は含みません。プロトコルは独立した crate として必要に応じて依存し、統一された SDK trait を実装することでカーネルに組み込まれます。

## クイックスタート

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
    // 1. カーネルを構築
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

    // 2. 起動（プロトコル登録を検証）
    kernel.boot().expect("boot failed");

    // 3. 登録済みプロトコルを確認
    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("{} v{} / {:?} / variants: {:?}",
            proto.name(), proto.version(),
            proto.category(), proto.supported_variants()
        );
    }

    // 4. 終了
    kernel.shutdown().expect("shutdown failed");
}
```

## 対応プロトコル（38 種）

### 産業用イーサネット

| プロトコル | Crate | 説明 |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII、FC 01-06/10/15、CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP、Who-Is/I-Am、ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag、RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary プロトコルスタック、Hello/Ack、SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT、DCP Identify、Record Data Read |

### フィールドバス

| プロトコル | Crate | 説明 |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA、Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 マスター/スレーブポーリング、CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | 電力 SCADA、Class 0 ポーリング、CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS 変電所、TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS、SD2 フレーム、CP 5611 ゲートウェイ |
| CANopen | `industrial-protocols-canopen` | CiA 301、NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN、Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE、NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3、Data Exchange |
| IO-Link | `industrial-protocols-iolink` | ポイントツーポイントシリアル、ISDU パラメータ |
| CC-Link IE Field | `industrial-protocols-cclinkie` | 産業用イーサネット版、Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS、リング型シフトレジスタ |

### IoT/メッセージング

| プロトコル | Crate | 説明 |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1、CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART over TCP/UDP、ポート 5094 |

### 自動車バス

| プロトコル | Crate | 説明 |
|------|-------|------|
| LIN | `industrial-protocols-lin` | ボディバス、19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II、ISO 9141/14230、KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps デュアルチャネル、時間トリガ |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM、CRC-8 |
| MOST | `industrial-protocols-most` | 光ファイバーマルチメディア、25/50/150 Mbps |

### ビル/照明

| プロトコル | Crate | 説明 |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk、Neuron チップ、NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386、ARC Power、ブロードキャスト制御 |

### ハードウェアブリッジ

| プロトコル | Crate | 説明 |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT、BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes フレーム、EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram、Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps トークンパッシング、1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | マンチェスター符号化、FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff 光ファイバーリングネットワーク |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 トークンピアツーピア |
| ISA100 | `industrial-protocols-isa100` | ワイヤレス Mesh、YFGW410 ゲートウェイ |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591、Emerson 1420 ゲートウェイ |

### システムバス

| プロトコル | Crate | 説明 |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | コンフィグレーション空間 + MMIO、libpci カーネルブリッジ |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32、AM Codes |
| CompactPCI | `industrial-protocols-cpci` | 地理的アドレッシング、3U/6U Eurocard |

## 核心理念 / API リファレンス

カーネルアーキテクチャと全公開 API（Protocol SDK の 6 つの Trait、Kernel コンポーネント、Bridge トランスポート抽象、Error エラーチェーン、Async 非同期レイヤー）については [API リファレンスドキュメント](API.md) を参照してください。

## プロジェクト構成

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # マイクロカーネル (10 modules)
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
│   ├── serial/             # シリアルポート (feature-gated)
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

## プロジェクト規模

| 指標 | 数値 |
|------|------|
| 総 crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| テストスイート | 137 passing |
| カーネルモジュール | 10 |
| SDK traits | 6 |
| 接続戦略 | 3 (Simple/Persistent/Pooled) |
| リトライ戦略 | 4 (Fixed/Exponential/Linear/None) |
| エラー型 | 10 バリアント + コンテキストチェーン |
| 最低 Rust | 1.75+ (2024 edition) |

## 技術的な決定

| 決定 | 選択 | 理由 |
|------|------|------|
| アーキテクチャ | マイクロカーネル + プロトコル SDK | カーネルは最小限、プロトコルは疎結合 |
| パッケージ構成 | Cargo workspace + 独立 crate | 独立コンパイル、独立バージョニング |
| 非同期 | 同期カーネル + オプションの tokio | ゼロコストの選択肢 |
| エラー | thiserror + context() | 構造化 + コンテキストチェーン |
| 設定 | TOML + Builder | ファイル駆動 + コード上書き |
| 接続 | trait 抽象 + 3 戦略 | 柔軟なシナリオ対応 |
| ハードウェア | Transport trait + FFI | ソフト/ハード統一インターフェース |
| trait | 関連型 + インターフェース分離 | コンパイル時の型安全性 |

## 設計原則

1. **YAGNI** — カーネルはプロトコル共通のインフラのみを提供
2. **インターフェース分離** — 6 つの独立した trait、必要に応じて実装
3. **型安全性** — 関連型でプロトコルごとのアドレス/値をコンパイル時に区別
4. **ゼロコストのオプション性** — async レイヤーは独立しており、強制ランタイムなし
5. **エラーの透明性** — コンテキストチェーンで完全な障害経路を保持

## 寄付による支援 / Donations

このプロジェクトが役に立ったなら、寄付でのご支援をお願いします（金額は任意です。すべての支援者に感謝します）。

### 微信支付 / 支付宝

![微信支付](weixinpay.png) ![支付宝](alipay.png)

### 海外送金（銀行 SWIFT 振込）

| 項目 | 情報 |
|------|------|
| 受取人名義 | WANG KEXUN |
| 受取口座番号 | 881015918251 |
| 受取銀行 | ZA Bank Limited |
| SWIFT Code | AABLHKHHXXX |
| 銀行番号 | 387 |
| 銀行住所 | Core F, Cyberport 3, 100 Cyberport Road, Hong Kong |

> **クロスボーダー送金の代理銀行（中継銀行）**：これはクロスボーダー送金の代理銀行（中継銀行）情報であり、**受取銀行の情報ではありません**。送金銀行に、クロスボーダー送金の代理銀行情報が必要かどうかをご確認ください。

- **香港ドル・人民元・米ドルの送金**（代理銀行 Citibank）：
  - 銀行名：Citibank N.A. Hong Kong
  - SWIFT Code：CITIHKHXXXX
  - 銀行番号：006
  - 支店名：Hong Kong Branch
  - 支店番号：391
  - 銀行住所：Citibank Tower, Citibank Plaza, 3 Garden Road, Central, Hong Kong
- **その他の通貨の送金**（代理銀行 BNY Mellon）：
  - 銀行名：THE BANK OF NEW YORK MELLON
  - SWIFT Code：IRVTUS3NXXX
  - 銀行住所：THE BANK OF NEW YORK MELLON, 240 GREENWICH STREET, NEW YORK, United States

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
