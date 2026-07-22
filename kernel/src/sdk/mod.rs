// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 协议 SDK — 6 个核心 Trait
//!
//! 所有工业协议通过实现这些 trait 接入微内核。
//!
//! ## Trait 列表
//!
//! | Trait | 用途 | 必须实现 |
//! |-------|------|---------|
//! | [`Protocol`] | 协议元数据（名称、版本、分类、变体） | 所有协议 |
//! | [`ProtocolRead`] | 读取数据（单点、批量、范围） | 支持读取的协议 |
//! | [`ProtocolWrite`] | 写入数据（单点、批量） | 支持写入的协议 |
//! | [`ProtocolSubscribe`] | 订阅数据变更通知 | 支持推送的协议 |
//! | [`ProtocolDiscover`] | 网络扫描发现设备 | 支持发现的协议 |
//! | [`ProtocolHealth`] | 健康检查 | 建议实现 |
//! | [`ProtocolDiagnostics`] | 诊断信息获取 | 建议实现 |
//!
//! ## 关联类型
//!
//! 每个协议的地址和值类型通过关联类型定义，编译期类型安全：
//!
//! - Modbus: `Address = u16`, `Value = u16`（寄存器地址和 16 位值）
//! - OPC UA: `Address = String`, `Value = OPCUAValue`（NodeId 字符串和变体类型）
//! - BACnet: `Address = String`, `Value = f32`（对象标识符和浮点值）

pub mod types;

use crate::error::Result;
use crate::connection::Connection;
use crate::bridge::Transport;
pub use types::*;

/// 协议基础 trait，所有协议必须实现。
///
/// 提供协议的元数据信息：名称、版本、分类、支持的变体。
pub trait Protocol: Send + Sync {
    /// 协议名称，如 `"modbus"`、`"opcua"`。用作注册表的键。
    fn name(&self) -> &'static str;
    /// 协议版本号。
    fn version(&self) -> &'static str;
    /// 协议所属分类。
    fn category(&self) -> ProtocolCategory;
    /// 支持的变体列表，如 Modbus 支持 `["tcp", "rtu", "ascii"]`。
    fn supported_variants(&self) -> &'static [&'static str];
}

/// 读操作 trait。
///
/// 协议按需实现——只读传感器不需要实现 [`ProtocolWrite`]。
pub trait ProtocolRead: Protocol {
    /// 协议特有的地址类型。
    type Address: AddressLike;
    /// 协议特有的值类型。
    type Value: ValueLike;
    /// 从指定地址读取单个值。
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    /// 批量读取多个地址的值。
    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>>;
    /// 从起始地址连续读取指定数量的值。
    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>>;
}

/// 写操作 trait。
pub trait ProtocolWrite: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    /// 向指定地址写入单个值。
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()>;
    /// 批量写入多个地址的值。
    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()>;
}

/// 订阅操作 trait，用于支持数据推送的协议（如 MQTT、OPC UA）。
pub trait ProtocolSubscribe: Protocol {
    /// 订阅标识符类型。
    type SubscriptionId: Clone + Send + Sync;
    type Address: AddressLike;
    /// 订阅指定地址的数据变更。
    fn subscribe(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Self::SubscriptionId>;
    /// 取消订阅。
    fn unsubscribe(&self, conn: &mut dyn Connection, sub_id: &Self::SubscriptionId) -> Result<()>;
    /// 轮询获取订阅更新。
    fn poll(&self, conn: &mut dyn Connection) -> Result<Vec<SubscriptionUpdate<Self::SubscriptionId>>>;
}

/// 设备发现 trait，不要求实现 [`Protocol`]（因为发现可能发生在协议实例化之前）。
pub trait ProtocolDiscover {
    /// 通过网络发现设备。
    fn discover(&self, transport: &mut dyn Transport) -> Result<Vec<DiscoveredDevice>>;
}

/// 健康检查 trait，用于检测设备连接状态和延迟。
pub trait ProtocolHealth: Protocol {
    /// 执行健康检查，返回设备状态和延迟信息。
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus>;
}

/// 诊断信息 trait，用于获取设备运行统计和错误计数。
pub trait ProtocolDiagnostics: Protocol {
    /// 获取设备诊断报告。
    fn diagnostics(&self, conn: &mut dyn Connection) -> Result<DiagnosticsReport>;
}
