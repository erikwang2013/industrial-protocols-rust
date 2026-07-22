// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 厂商预设配置
//!
//! 提供常见工业设备的连接预设参数：
//!
//! - Siemens S7-1200
//! - Allen-Bradley CompactLogix
//! - Schneider Electric Modicon M340

use std::collections::HashMap;

pub fn siemens_s7_1200() -> HashMap<String, toml::Value> {
    let mut m = HashMap::new();
    m.insert("vendor".into(), toml::Value::String("Siemens".into()));
    m.insert("model".into(), toml::Value::String("S7-1200".into()));
    m.insert("rack".into(), toml::Value::Integer(0));
    m.insert("slot".into(), toml::Value::Integer(1));
    m
}

pub fn allen_bradley_compactlogix() -> HashMap<String, toml::Value> {
    let mut m = HashMap::new();
    m.insert("vendor".into(), toml::Value::String("Allen-Bradley".into()));
    m.insert("model".into(), toml::Value::String("CompactLogix".into()));
    m.insert("slot".into(), toml::Value::Integer(0));
    m
}

pub fn schneider_m340() -> HashMap<String, toml::Value> {
    let mut m = HashMap::new();
    m.insert("vendor".into(), toml::Value::String("Schneider Electric".into()));
    m.insert("model".into(), toml::Value::String("Modicon M340".into()));
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_siemens_preset() { assert_eq!(siemens_s7_1200().get("vendor").unwrap().as_str(), Some("Siemens")); }
    #[test]
    fn test_ab_preset() { assert_eq!(allen_bradley_compactlogix().get("vendor").unwrap().as_str(), Some("Allen-Bradley")); }
}
