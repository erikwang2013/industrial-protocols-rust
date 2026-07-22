// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 协议注册表
//!
//! 管理所有已注册协议的索引。支持按名称查找和按分类筛选。
//! 每个协议实例通过 [`register`](ProtocolRegistry::register) 注册后，
//! 即可通过 [`get`](ProtocolRegistry::get) 按名称检索。

use crate::sdk::{Protocol, ProtocolCategory};
use std::collections::HashMap;

pub struct ProtocolRegistry { protocols: HashMap<String, Box<dyn Protocol>> }

impl ProtocolRegistry {
    pub fn new() -> Self { ProtocolRegistry { protocols: HashMap::new() } }

    pub fn register(&mut self, protocol: Box<dyn Protocol>) {
        self.protocols.insert(protocol.name().to_string(), protocol);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Protocol> {
        self.protocols.get(name).map(|p| p.as_ref())
    }

    pub fn list_all(&self) -> Vec<&dyn Protocol> {
        self.protocols.values().map(|p| p.as_ref()).collect()
    }

    pub fn list_by_category(&self, category: ProtocolCategory) -> Vec<&dyn Protocol> {
        self.protocols.values().filter(|p| p.category() == category).map(|p| p.as_ref()).collect()
    }

    pub fn protocol_names(&self) -> Vec<String> { self.protocols.keys().cloned().collect() }
}

impl Default for ProtocolRegistry { fn default() -> Self { Self::new() } }
