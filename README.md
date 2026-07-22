# Industrial Protocols Rust

Rust 工业通信协议集 —— 微内核 + 协议 SDK 架构，覆盖 38 种工业协议。

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

[English](README.en.md)

参考 PHP 版 [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) 功能集，按 Rust 生态习惯重新设计。PHP 版 42 个协议全部覆盖。

## 架构

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

**核心理念：** 内核只定义「协议是什么」（6 个 trait），不包含具体协议实现。协议作为独立 crate 按需依赖，通过实现统一 SDK trait 接入内核。

## 快速开始

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

## 支持协议（38 种）

### 工业以太网

| 协议 | Crate | 说明 |
|------|-------|------|
| Modbus | `industrial-protocols-modbus` | TCP/RTU/ASCII，FC 01-06/10/15，CRC-16/LRC |
| BACnet | `industrial-protocols-bacnet` | BACnet/IP，Who-Is/I-Am，ReadProperty (ASN.1) |
| EtherNet/IP | `industrial-protocols-ethernet-ip` | ENIP + CIP Read Tag，RegisterSession |
| OPC UA | `industrial-protocols-opcua` | Binary 协议栈，Hello/Ack，SecurityPolicy |
| Profinet | `industrial-protocols-profinet` | NRT，DCP Identify，Record Data Read |

### 现场总线

| 协议 | Crate | 说明 |
|------|-------|------|
| HART | `industrial-protocols-hart` | FSK 4-20mA，Commands 0/1/3 |
| CC-Link | `industrial-protocols-cclink` | RS-485 主从轮询，CRC-16/XMODEM |
| DNP3 | `industrial-protocols-dnp3` | 电力 SCADA，Class 0 轮询，CRC-16/DNP |
| IEC 61850 | `industrial-protocols-iec61850` | MMS 变电站，TPKT+COTP |
| PROFIBUS | `industrial-protocols-profibus` | DP/PA/FMS，SD2 帧，CP 5611 网关 |
| CANopen | `industrial-protocols-canopen` | CiA 301，NMT/SDO/PDO |
| DeviceNet | `industrial-protocols-devicenet` | CIP over CAN，Explicit + Poll |
| Foundation Fieldbus | `industrial-protocols-foundationfieldbus` | H1/HSE，NI USB-8486 |
| AS-Interface | `industrial-protocols-asinterface` | AS-i V3，Data Exchange |
| IO-Link | `industrial-protocols-iolink` | 点对点串行，ISDU 参数 |
| CC-Link IE Field | `industrial-protocols-cclinkie` | 工业以太网版，Cyclic Data |
| INTERBUS | `industrial-protocols-interbus` | Phoenix IBS，环网移位寄存器 |

### IoT/消息

| 协议 | Crate | 说明 |
|------|-------|------|
| MQTT | `industrial-protocols-mqtt` | 3.1.1，CONNECT/PUBLISH/SUBSCRIBE/PINGREQ |
| HART-IP | `industrial-protocols-hartip` | HART over TCP/UDP，端口 5094 |

### 汽车总线

| 协议 | Crate | 说明 |
|------|-------|------|
| LIN | `industrial-protocols-lin` | 车身总线，19200 bps UART |
| K-Line | `industrial-protocols-kline` | OBD-II，ISO 9141/14230，KWP2000 |
| FlexRay | `industrial-protocols-flexray` | 10 Mbps 双通道，时间触发 |
| SAE J1850 | `industrial-protocols-saej1850` | OBD-II VPW/PWM，CRC-8 |
| MOST | `industrial-protocols-most` | 光纤多媒体，25/50/150 Mbps |

### 楼宇/照明

| 协议 | Crate | 说明 |
|------|-------|------|
| LonWorks | `industrial-protocols-lonworks` | LonTalk，Neuron 芯片，NV Fetch |
| DALI | `industrial-protocols-dali` | IEC 62386，ARC Power，广播控制 |

### 硬件桥接

| 协议 | Crate | 说明 |
|------|-------|------|
| EtherCAT | `industrial-protocols-ethercat` | SOEM/TwinCAT，BRD/APRD Datagram |
| POWERLINK | `industrial-protocols-powerlink` | SoC/PReq/PRes 帧，EtherType 0x88AB |
| SERCOS III | `industrial-protocols-sercos` | MDT/AT Telegram，Service Channel |
| ControlNet | `industrial-protocols-controlnet` | 5 Mbps 令牌传递，1784-PCIC/S |
| WorldFIP | `industrial-protocols-worldfip` | 曼彻斯特编码，FIP/Fipio |
| Lightbus | `industrial-protocols-lightbus` | Beckhoff 光纤环网 |
| Modbus Plus | `industrial-protocols-modbusplus` | SA85/BM85 令牌对等 |
| ISA100 | `industrial-protocols-isa100` | 无线 Mesh，YFGW410 网关 |
| WirelessHART | `industrial-protocols-wirelesshart` | IEC 62591，Emerson 1420 网关 |

### 系统总线

| 协议 | Crate | 说明 |
|------|-------|------|
| PCI/PCIe | `industrial-protocols-pci` | 配置空间 + MMIO，libpci 内核桥接 |
| VME/VPX | `industrial-protocols-vme` | A16/A24/A32，AM Codes |
| CompactPCI | `industrial-protocols-cpci` | 地理寻址，3U/6U Eurocard |

## 核心概念

### 1. Protocol SDK — 6 个 Trait

每个协议通过实现以下 trait 接入内核：

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // 协议特定地址类型
    type Value: ValueLike;      // 协议特定值类型
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, ...) -> Result<Vec<Self::Value>>;
    fn read_range(&self, ...) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol { ... }
pub trait ProtocolSubscribe: Protocol { ... }
pub trait ProtocolDiscover { ... }
pub trait ProtocolHealth: Protocol { ... }
pub trait ProtocolDiagnostics: Protocol { ... }
```

**关联类型**让地址和值类型安全：Modbus 用 `u16`，OPC UA 用 `String`，BACnet 用 `f32`。**Trait 拆分**让协议按需实现——只读传感器不需要 `ProtocolWrite`。

### 2. Kernel — 微内核

内核不实现任何协议，只提供共享基础设施：

| 组件 | 职责 | 关键 API |
|------|------|----------|
| `ProtocolRegistry` | 协议注册/查找/分类 | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | 设备连接生命周期 | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML 配置 + 运行时覆盖 | `load()`, `get_device()`, `set_override()` |
| `EventBus` | 发布/订阅事件 | `subscribe()`, `publish()` |
| `CircuitBreaker` | 熔断 Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | 固定/指数/线性/无 | `run()`, `start()` |
| `GatewayEngine` | 协议间数据转发规则 | `add_rule()`, `start()/stop()` |

连接管理支持三种策略：
```rust
pub enum ConnectionStrategy {
    Simple,      // 每次新建，用完释放
    Persistent,  // 单连接复用，断开自动重连
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

### 3. Bridge Layer — 传输抽象

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | 实现 | 依赖 |
|--------|------|------|
| `bridge-socket` | TCP 连接 + UDP 收发 | std |
| `bridge-serial` | 真实串口，可选 `serial` feature | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | 厂商 C SDK 动态加载 | `libloading` |
| `bridge-vendor` | Siemens/AB/Schneider 预置 | — |

### 4. Error — 结构化错误链

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 非法地址"
    CircuitOpen { device, retry_after },  // 熔断器已断开
    Context { context, source },  // 附加上文，形成错误链
    // ...
}
```

错误链示例：
```
读取 PLC-001 保持寄存器 40001 失败
  → 超时: device=plc-001, timeout=3000ms
    → 连接失败: 192.168.1.10:502: Connection refused
```

### 5. Async Layer — 可选 tokio 包装

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// 异步事件流
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```

## 项目结构

```
industrial-protocols-rust/
├── Cargo.toml              # workspace root
├── kernel/                 # 微内核 (10 modules)
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
│   ├── serial/             # 串口 (feature-gated)
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

## 项目规模

| 指标 | 数值 |
|------|------|
| 总 crates | 45 (kernel + 5 bridge + 38 protocols + async) |
| 测试套件 | 137 passing |
| 内核模块 | 10 |
| SDK traits | 6 |
| 连接策略 | 3 (Simple/Persistent/Pooled) |
| 重试策略 | 4 (Fixed/Exponential/Linear/None) |
| 错误类型 | 10 变体 + 上下文链 |
| 最低 Rust | 1.75+ (2024 edition) |

## 技术决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 架构 | 微内核 + 协议 SDK | 内核极简，协议解耦 |
| 包组织 | Cargo workspace + 独立 crate | 独立编译、独立版本 |
| 异步 | 同步内核 + 可选 tokio | 零成本选择 |
| 错误 | thiserror + context() | 结构化 + 上下文链 |
| 配置 | TOML + Builder | 文件驱动 + 代码覆盖 |
| 连接 | trait 抽象 + 3 策略 | 灵活场景适配 |
| 硬件 | Transport trait + FFI | 软硬统一接口 |
| trait | 关联类型 + 接口隔离 | 编译期类型安全 |

## 设计原则

1. **YAGNI** — 内核只提供协议共需的基础设施
2. **接口隔离** — 6 个独立 trait，按需实现
3. **类型安全** — 关联类型编译期区分不同协议的地址/值
4. **零成本可选** — async 层独立，不引入强制运行时
5. **错误透明** — 上下文链保留完整故障路径

## License

MIT — Copyright (c) 2026 erik [erik@erik.xyz](mailto:erik@erik.xyz)
