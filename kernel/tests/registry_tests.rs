// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::registry::*;
use industrial_protocols_kernel::sdk::*;

struct ModbusProtocol;
impl Protocol for ModbusProtocol {
    fn name(&self) -> &'static str {
        "modbus"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

struct OpcuaProtocol;
impl Protocol for OpcuaProtocol {
    fn name(&self) -> &'static str {
        "opcua"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["binary"]
    }
}

#[test]
fn test_register_and_get() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    assert!(reg.get("modbus").is_some());
    assert!(reg.get("opcua").is_some());
    assert!(reg.get("bacnet").is_none());
}

#[test]
fn test_list_by_category() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    assert_eq!(
        reg.list_by_category(ProtocolCategory::IndustrialEthernet)
            .len(),
        1
    );
    assert_eq!(reg.list_by_category(ProtocolCategory::Fieldbus).len(), 0);
}

#[test]
fn test_protocol_names() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    assert!(reg.protocol_names().contains(&"modbus".to_string()));
}

struct ProfibusProtocol;
impl Protocol for ProfibusProtocol {
    fn name(&self) -> &'static str {
        "profibus"
    }
    fn version(&self) -> &'static str {
        "1.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::Fieldbus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["dp"]
    }
}

#[test]
fn test_list_all() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    assert_eq!(reg.list_all().len(), 2);
}

#[test]
fn test_register_overwrites_same_name() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(ModbusProtocol));
    assert_eq!(reg.list_all().len(), 1);
    assert_eq!(reg.protocol_names().len(), 1);
}

#[test]
fn test_list_by_category_multiple_protocols() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    reg.register(Box::new(OpcuaProtocol));
    reg.register(Box::new(ProfibusProtocol));
    assert_eq!(
        reg.list_by_category(ProtocolCategory::IndustrialEthernet)
            .len(),
        2
    );
    assert_eq!(reg.list_by_category(ProtocolCategory::Fieldbus).len(), 1);
    assert_eq!(
        reg.list_by_category(ProtocolCategory::IoTMessaging).len(),
        0
    );
}

#[test]
fn test_empty_registry() {
    let reg = ProtocolRegistry::new();
    assert!(reg.get("anything").is_none());
    assert!(reg.list_all().is_empty());
    assert!(reg.protocol_names().is_empty());
}

#[test]
fn test_default_registry() {
    let reg = ProtocolRegistry::default();
    assert!(reg.list_all().is_empty());
}
