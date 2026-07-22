// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # SDK 类型定义
//!
//! 定义协议 SDK 中使用的公共类型：
//! - 标记 trait（[`AddressLike`]、[`ValueLike`]）
//! - 枚举类型（[`ProtocolCategory`]、[`HealthState`]）
//! - 数据结构（[`HealthStatus`]、[`DiagnosticsReport`]、[`DiscoveredDevice`]）

use std::fmt::Debug;

pub trait AddressLike: Clone + Debug + Send + Sync {}
impl<T: Clone + Debug + Send + Sync> AddressLike for T {}

pub trait ValueLike: Clone + Debug + Send + Sync {}
impl<T: Clone + Debug + Send + Sync> ValueLike for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ProtocolCategory {
    IndustrialEthernet,
    Fieldbus,
    IoTMessaging,
    AutomotiveBus,
    BuildingLighting,
    HardwareBridge,
    SystemBus,
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub device_id: String,
    pub protocol: String,
    pub address: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub state: HealthState,
    pub latency_ms: u64,
    pub last_seen: std::time::Instant,
    pub details: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsReport {
    pub device_id: String,
    pub uptime_secs: Option<u64>,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub custom_fields: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionUpdate<Id = String> {
    pub subscription_id: Id,
    pub address: String,
    pub value: String,
    pub timestamp: std::time::Instant,
}
