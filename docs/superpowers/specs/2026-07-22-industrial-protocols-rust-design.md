# Industrial Protocols Rust — Architecture Design

## 概述

参照 PHP 版 [industrial-protocols](https://github.com/erikwang2013/industrial-protocols) 的功能集，用 Rust 实现工业通信协议集库。采用 **微内核 + 协议 SDK** 的分层架构，Cargo workspace 组织独立子 crate，纯同步内核 + 可选异步层。

## 架构

```
┌─────────────────────────────────────────────┐
│              User Application               │
├─────────────────────────────────────────────┤
│         Async Layer (optional feature)       │
│       tokio::spawn / Stream / Channel        │
├─────────────────────────────────────────────┤
│               Micro-Kernel                   │
│  ┌──────────┐ ┌───────────┐ ┌────────────┐  │
│  │Protocol  │ │Connection │ │Config      │  │
│  │Registry  │ │Manager    │ │Repository  │  │
│  ├──────────┤ ├───────────┤ ├────────────┤  │
│  │Gateway   │ │Circuit    │ │Event       │  │
│  │Engine    │ │Breaker    │ │Bus         │  │
│  ├──────────┤ ├───────────┤ ├────────────┤  │
│  │Retry     │ │Error(Ctx) │ │Metrics     │  │
│  └──────────┘ └───────────┘ └────────────┘  │
├─────────────────────────────────────────────┤
│           Protocol SDK (6 Traits)            │
│  Read  Write  Subscribe  Discover  Health    │
│               Diagnostics                    │
├─────────────────────────────────────────────┤
│  Protocol Crates  │  Bridge Layer            │
│  modbus  opcua     │  socket  serial  can     │
│  bacnet  eip       │  ffi  vendor            │
│  profinet         │                          │
└─────────────────────────────────────────────┘
```

## Workspace 结构

```
industrial-protocols-rust/
├── Cargo.toml                   # [workspace] 定义
├── kernel/                      # industrial-protocols-kernel
│   └── src/
│       ├── lib.rs
│       ├── registry/            # ProtocolRegistry
│       ├── connection/          # ConnectionManager (3 strategies)
│       ├── config/              # ConfigRepository (serde + TOML)
│       ├── gateway/             # GatewayEngine
│       ├── circuit/             # CircuitBreaker
│       ├── event/               # EventBus
│       ├── retry/               # RetryPolicy (4 种)
│       ├── error.rs             # ProtocolError
│       └── sdk/                 # 6 个核心 trait
├── protocols/
│   ├── modbus/                  # Phase 1
│   ├── opcua/                   # Phase 1
│   ├── bacnet/                  # Phase 1
│   ├── ethernet-ip/             # Phase 1
│   └── profinet/                # Phase 1
├── bridge/
│   ├── socket/                  # Phase 1
│   ├── serial/                  # Phase 1
│   ├── can/                     # 骨架
│   ├── ffi/                     # 骨架
│   └── vendor/                  # 骨架
├── async-layer/                 # 可选 tokio 异步包装
└── examples/
```

命名规则：crate 名 `industrial-protocols-<name>`，模块名 `industrial_protocols_<name>`。

## 第一阶段范围

5 个纯软协议 + kernel + bridge 基础：

| 模块 | 范围 |
|------|------|
| kernel | 全部核心组件 |
| bridge-socket | TCP/UDP |
| bridge-serial | 串口 (Modbus RTU) |
| modbus | TCP/RTU/ASCII, FC 01/02/03/04/06/10 |
| opcua | Binary 协议栈, Session + Read/Write/Browse |
| bacnet | BACnet/IP, Who-Is/I-Am, ReadProperty |
| ethernet-ip | ENIP 会话 + CIP Read Tag |
| profinet | NRT, DCP 设备发现, Record Data 读写 |

## 协议 SDK — 6 个核心 Trait

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn read(&self, conn: &dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>>;
    fn read_range(&self, conn: &dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn write(&self, conn: &dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()>;
    fn write_batch(&self, conn: &dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()>;
}

pub trait ProtocolSubscribe: Protocol {
    type SubscriptionId;
    fn subscribe(&self, conn: &dyn Connection, addrs: &[Self::Address]) -> Result<Self::SubscriptionId>;
    fn unsubscribe(&self, conn: &dyn Connection, sub_id: &Self::SubscriptionId) -> Result<()>;
    fn poll(&self, conn: &dyn Connection) -> Result<Vec<SubscriptionUpdate>>;
}

pub trait ProtocolDiscover {
    fn discover(&self, transport: &dyn Transport) -> Result<Vec<DiscoveredDevice>>;
}

pub trait ProtocolHealth: Protocol {
    fn health(&self, conn: &dyn Connection) -> Result<HealthStatus>;
}

pub trait ProtocolDiagnostics: Protocol {
    fn diagnostics(&self, conn: &dyn Connection) -> Result<DiagnosticsReport>;
}
```

关联类型 `Address`/`Value` 让每个协议自定义地址和值类型。Trait 拆分让协议按需实现。

## Kernel 核心组件

| 组件 | 职责 |
|------|------|
| `ProtocolRegistry` | 协议注册、查找、按分类列举 |
| `ConnectionManager` | 管理设备连接生命周期，Simple / Persistent / Pooled 三种策略 |
| `ConfigRepository` | TOML 加载 + 运行时覆盖 + 设备配置查询 |
| `GatewayEngine` | 协议间数据转发（Modbus → MQTT 等），规则驱动 |
| `CircuitBreaker` | 熔断保护：closed → open → half_open |
| `EventBus` | 发布/订阅：连接事件、错误、告警 |
| `RetryConfig` | 4 种策略：Fixed / Exponential / Linear / None |

## 错误设计

`thiserror` 结构化错误 + `context()` 链式追加上下文：

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),
    Timeout { device: String, timeout_ms: u64 },
    Parse { protocol: String, reason: String },
    DeviceError { device: String, code: u16, message: String },
    DeviceNotFound(String),
    Unsupported { protocol: String, operation: String },
    CircuitOpen { device: String, retry_after: Duration },
    Config(String),
    ProtocolSpecific(Box<dyn StdError + Send + Sync>),
    Context { context: String, source: Box<ProtocolError> },
}
```

各协议 crate 通过 `impl From<ModbusError> for ProtocolError` 接入。

## 配置格式

TOML 文件驱动 + 代码 Builder 模式：

```toml
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "192.168.1.10"
port = 502
unit_id = 1
timeout_ms = 3000
retry = { max = 3, strategy = "exponential", base_ms = 100, max_ms = 5000 }
connection_strategy = "persistent"
health_check = true
```

## Bridge 层

`Transport` trait 抽象底层 I/O：

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> Result<Vec<u8>, TransportError>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> Result<(), TransportError>;
}
```

传输类型：`Tcp`, `Udp`, `Serial`, `Can`, `Ffi`, `Custom`。

FFI 桥接通过 `SafeVendorTransport` 封装所有 unsafe 调用，对外暴露安全 trait。

## 技术决策汇总

| 决策 | 选择 |
|------|------|
| 包组织 | Cargo workspace + 独立子 crate |
| 异步策略 | 纯同步内核 + 可选 tokio 异步层 |
| 硬件抽象 | 完整预留 Bridge 层（socket/serial/can/ffi/vendor） |
| 配置方式 | TOML 文件 + 代码 Builder |
| 错误处理 | thiserror 结构化类型 + context() 上下文链 |
| 连接管理 | Trait 抽象，内置 Simple/Persistent/Pooled |
| 架构 | 微内核 + 协议 SDK 分层架构 |

## 非功能需求

- **测试**：每个协议 crate 需要集成测试（mock/真实硬件两类）
- **文档**：每个 crate 需要 rustdoc；kernel 需要完整 API 文档
- **最低 Rust 版本**：1.75+（2024 edition）
- **外部依赖原则**：kernel 最小化外部依赖；协议 crate 可按需引入协议特定依赖
- **License**：MIT
