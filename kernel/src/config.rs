// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 配置管理
//!
//! 通过 TOML 文件加载设备配置，支持运行时覆盖。
//!
//! ## 配置格式
//!
//! ```toml
//! [devices.plc-001]
//! protocol = "modbus"
//! variant = "tcp"
//! host = "192.168.1.10"
//! port = 502
//! unit_id = 1
//! timeout_ms = 3000
//! retry = { max = 3, strategy = "exponential", base_ms = 100, max_ms = 5000 }
//! connection_strategy = "persistent"
//! ```
//!
//! 协议特定字段（如 BACnet 的 `device_instance`）通过 `#[serde(flatten)]` 捕获到 `extras` 中。

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFile {
    #[serde(default)] pub devices: HashMap<String, DeviceConfigRaw>,
    #[serde(default)] pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceConfigRaw {
    pub protocol: String,
    #[serde(default)] pub variant: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub serial_port: Option<String>,
    pub baud_rate: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub unit_id: Option<u16>,
    pub retry: Option<RetryConfigRaw>,
    pub connection_strategy: Option<ConnectionStrategyConfig>,
    pub health_check: Option<bool>,
    #[serde(flatten)] pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfigRaw {
    pub max: u32,
    pub strategy: String,
    #[serde(default)] pub base_ms: u64,
    #[serde(default)] pub max_ms: u64,
    #[serde(default)] pub step_ms: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStrategyConfig { Simple, Persistent, Pooled { min: usize, max: usize, timeout_ms: u64 } }

#[derive(Debug, Clone, Deserialize)]
#[derive(Default)]
pub struct DefaultsConfig {
    #[serde(default = "default_timeout")] pub timeout_ms: u64,
    #[serde(default = "default_retry_max")] pub retry_max: u32,
    #[serde(default = "default_retry_strategy")] pub retry_strategy: String,
    #[serde(default = "default_health_interval")] pub health_check_interval_s: u64,
    #[serde(default = "default_cb_threshold")] pub circuit_breaker_threshold: u32,
    #[serde(default = "default_cb_cooldown")] pub circuit_breaker_cooldown_s: u64,
}

fn default_timeout() -> u64 { 5000 }
fn default_retry_max() -> u32 { 3 }
fn default_retry_strategy() -> String { "exponential".into() }
fn default_health_interval() -> u64 { 30 }
fn default_cb_threshold() -> u32 { 5 }
fn default_cb_cooldown() -> u64 { 30 }

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub protocol: String, pub variant: String,
    pub host: Option<String>, pub port: Option<u16>,
    pub serial_port: Option<String>, pub baud_rate: Option<u32>,
    pub timeout_ms: u64, pub unit_id: u16,
    pub retry: RetryConfigResolved,
    pub connection_strategy: ConnectionStrategyConfig,
    pub health_check: bool,
    pub extras: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct RetryConfigResolved { pub max: u32, pub strategy: String, pub base_ms: u64, pub max_ms: u64, pub step_ms: u64 }

pub struct ConfigRepository {
    raw: ConfigFile,
    overrides: HashMap<String, HashMap<String, toml::Value>>,
}

impl ConfigRepository {
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let raw: ConfigFile = toml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ConfigRepository { raw, overrides: HashMap::new() })
    }

    pub fn parse(toml_str: &str) -> std::io::Result<Self> {
        let raw: ConfigFile = toml::from_str(toml_str).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(ConfigRepository { raw, overrides: HashMap::new() })
    }

    pub fn get_device(&self, id: &str) -> Option<DeviceConfig> {
        let raw = self.raw.devices.get(id)?;
        let overrides = self.overrides.get(id);
        let timeout_ms = overrides.and_then(|o| o.get("timeout_ms")).and_then(|v| v.as_integer()).map(|v| v as u64)
            .unwrap_or(raw.timeout_ms.unwrap_or(self.raw.defaults.timeout_ms));
        let retry = raw.retry.clone().unwrap_or(RetryConfigRaw { max: self.raw.defaults.retry_max, strategy: self.raw.defaults.retry_strategy.clone(), base_ms: 100, max_ms: 5000, step_ms: 50 });
        Some(DeviceConfig {
            protocol: raw.protocol.clone(), variant: raw.variant.clone(),
            host: raw.host.clone(), port: raw.port,
            serial_port: raw.serial_port.clone(), baud_rate: raw.baud_rate, timeout_ms,
            unit_id: overrides.and_then(|o| o.get("unit_id")).and_then(|v| v.as_integer()).map(|v| v as u16).unwrap_or(raw.unit_id.unwrap_or(1)),
            retry: RetryConfigResolved { max: retry.max, strategy: retry.strategy, base_ms: retry.base_ms, max_ms: retry.max_ms, step_ms: retry.step_ms },
            connection_strategy: raw.connection_strategy.clone().unwrap_or(ConnectionStrategyConfig::Persistent),
            health_check: raw.health_check.unwrap_or(true),
            extras: raw.extras.clone(),
        })
    }

    pub fn device_ids(&self) -> Vec<String> { self.raw.devices.keys().cloned().collect() }
    pub fn defaults(&self) -> &DefaultsConfig { &self.raw.defaults }

    pub fn set_override(&mut self, device_id: &str, key: &str, value: toml::Value) {
        self.overrides.entry(device_id.to_string()).or_default().insert(key.to_string(), value);
    }

    pub fn clear_overrides(&mut self, device_id: &str) { self.overrides.remove(device_id); }
}
