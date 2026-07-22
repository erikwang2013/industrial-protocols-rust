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

pub struct Kernel {
    registry: ProtocolRegistry,
    conn_manager: ConnectionManager,
    config: Option<ConfigRepository>,
    events: EventBus,
    gateway: GatewayEngine,
}

impl Kernel {
    pub fn builder() -> KernelBuilder { KernelBuilder::new() }

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

    pub fn shutdown(&mut self) -> io::Result<()> {
        self.gateway.stop();
        self.conn_manager.shutdown()
    }

    pub fn protocol_registry(&self) -> &ProtocolRegistry { &self.registry }
    pub fn connection_manager(&self) -> &ConnectionManager { &self.conn_manager }
    pub fn config(&self) -> &ConfigRepository { self.config.as_ref().expect("config not loaded") }
    pub fn event_bus(&self) -> &EventBus { &self.events }
    pub fn gateway(&self) -> &GatewayEngine { &self.gateway }
    pub fn gateway_mut(&mut self) -> &mut GatewayEngine { &mut self.gateway }
}

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

    pub fn config_from_str(mut self, toml_str: &str) -> Self {
        self.config = Some(ConfigRepository::from_str(toml_str).expect("invalid TOML config"));
        self
    }

    pub fn config_from_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.config = Some(ConfigRepository::load(path).expect("failed to load config file"));
        self
    }

    pub fn with_protocol(mut self, protocol: Box<dyn Protocol>) -> Self { self.protocols.push(protocol); self }

    pub fn with_connection_strategy(mut self, strategy: ConnectionStrategy) -> Self { self.connection_strategy = strategy; self }

    pub fn with_event_capacity(mut self, cap: usize) -> Self { self.event_capacity = cap; self }

    pub fn with_gateway_interval(mut self, ms: u64) -> Self { self.gateway_interval_ms = ms; self }

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
