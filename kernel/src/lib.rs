// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # Industrial Protocols Kernel
//!
//! 工业通信协议集的微内核。内核只定义「协议是什么」（通过 6 个 SDK trait），
//! 不包含任何具体协议实现。
//!
//! ## 核心组件
//!
//! - [`ProtocolRegistry`] — 协议注册表，按名称或分类查找协议
//! - [`ConnectionManager`] — 设备连接管理器，支持 3 种连接策略
//! - [`ConfigRepository`] — TOML 配置加载与运行时覆盖
//! - [`EventBus`] — 事件发布/订阅总线
//! - [`CircuitBreaker`] — 熔断保护器
//! - [`RetryConfig`] — 重试配置（固定/指数/线性/无）
//! - [`GatewayEngine`] — 协议间数据转发引擎
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use industrial_protocols_kernel::*;
//! use industrial_protocols_kernel::sdk::*;
//!
//! let mut kernel = Kernel::builder()
//!     .config_from_str(r#"
//!         [devices.plc-001]
//!         protocol = "modbus"
//!         host = "192.168.1.10"
//!         port = 502
//!     "#)
//!     .with_protocol(Box::new(MyProtocol))
//!     .build();
//!
//! kernel.boot()?;
//! // ... 使用协议读写设备 ...
//! kernel.shutdown()?;
//! ```

pub mod bridge;
pub mod circuit;
pub mod config;
pub mod connection;
pub mod error;
pub mod event;
pub mod gateway;
pub mod registry;
pub mod retry;
pub mod sdk;

use config::ConfigRepository;
use connection::{ConnectionManager, ConnectionStrategy};
use event::EventBus;
use gateway::GatewayEngine;
use registry::ProtocolRegistry;
use sdk::Protocol;
use std::io;

/// 内核结构体，是工业协议应用的入口点。
///
/// 通过 [`KernelBuilder`] 构建，使用 Builder 模式配置协议、连接策略等参数，
/// 然后调用 [`boot`](Kernel::boot) 启动，使用完毕后调用 [`shutdown`](Kernel::shutdown) 关闭。
pub struct Kernel {
    registry: ProtocolRegistry,
    conn_manager: ConnectionManager,
    config: Option<ConfigRepository>,
    events: EventBus,
    gateway: GatewayEngine,
}

impl Kernel {
    /// 创建内核构建器。
    pub fn builder() -> KernelBuilder { KernelBuilder::new() }

    /// 启动内核，校验所有配置文件中的设备均有对应协议注册。
    pub fn boot(&mut self) -> io::Result<()> {
        if let Some(ref config) = self.config {
            for device_id in config.device_ids() {
                let dev = config.get_device(&device_id).unwrap();
                if self.registry.get(&dev.protocol).is_none() {
                    return Err(io::Error::new(io::ErrorKind::NotFound,
                        format!("protocol '{}' not registered (needed by device '{device_id}')", dev.protocol)));
                }
            }
        }
        Ok(())
    }

    /// 关闭内核，断开所有设备连接并停止网关引擎。
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.gateway.stop();
        self.conn_manager.shutdown()
    }

    /// 获取协议注册表的只读引用。
    pub fn protocol_registry(&self) -> &ProtocolRegistry { &self.registry }

    /// 获取连接管理器的只读引用。
    pub fn connection_manager(&self) -> &ConnectionManager { &self.conn_manager }

    /// 获取配置仓库的只读引用。
    ///
    /// # Panics
    /// 如果未通过 builder 加载配置（即 `config_from_str` 或 `config_from_file` 未被调用）。
    pub fn config(&self) -> &ConfigRepository { self.config.as_ref().expect("config not loaded") }

    /// 获取事件总线的只读引用。
    pub fn event_bus(&self) -> &EventBus { &self.events }

    /// 获取网关引擎的只读引用。
    pub fn gateway(&self) -> &GatewayEngine { &self.gateway }

    /// 获取网关引擎的可变引用。
    pub fn gateway_mut(&mut self) -> &mut GatewayEngine { &mut self.gateway }
}

/// 内核构建器，使用 Builder 模式配置内核参数。
///
/// # 示例
///
/// ```rust,no_run
/// let kernel = Kernel::builder()
///     .config_from_str(toml_str)
///     .with_protocol(Box::new(protocol))
///     .with_connection_strategy(ConnectionStrategy::Persistent)
///     .build();
/// ```
pub struct KernelBuilder {
    config: Option<ConfigRepository>,
    protocols: Vec<Box<dyn Protocol>>,
    connection_strategy: ConnectionStrategy,
    event_capacity: usize,
    gateway_interval_ms: u64,
}

impl KernelBuilder {
    fn new() -> Self {
        KernelBuilder {
            config: None, protocols: Vec::new(),
            connection_strategy: ConnectionStrategy::Persistent,
            event_capacity: 64,
            gateway_interval_ms: 100,
        }
    }

    /// 从 TOML 字符串加载设备配置。
    pub fn config_from_str(mut self, toml_str: &str) -> Self {
        self.config = Some(ConfigRepository::from_str(toml_str).expect("invalid TOML config"));
        self
    }

    /// 从 TOML 文件加载设备配置。
    pub fn config_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.config = Some(ConfigRepository::load(path).expect("failed to load config file"));
        self
    }

    /// 注册一个协议实例。
    pub fn with_protocol(mut self, protocol: Box<dyn Protocol>) -> Self { self.protocols.push(protocol); self }

    /// 设置设备连接策略。
    pub fn with_connection_strategy(mut self, strategy: ConnectionStrategy) -> Self { self.connection_strategy = strategy; self }

    /// 设置事件总线通道缓冲区大小。
    pub fn with_event_capacity(mut self, cap: usize) -> Self { self.event_capacity = cap; self }

    /// 设置网关引擎的轮询间隔（毫秒）。
    pub fn with_gateway_interval(mut self, ms: u64) -> Self { self.gateway_interval_ms = ms; self }

    /// 构建内核实例。
    pub fn build(self) -> Kernel {
        let mut registry = ProtocolRegistry::new();
        for p in self.protocols { registry.register(p); }
        Kernel {
            registry,
            conn_manager: ConnectionManager::new(self.connection_strategy),
            config: self.config,
            events: EventBus::new(self.event_capacity),
            gateway: GatewayEngine::new(self.gateway_interval_ms),
        }
    }
}
