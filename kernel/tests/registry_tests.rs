use industrial_protocols_kernel::registry::*;
use industrial_protocols_kernel::sdk::*;

struct ModbusProtocol;
impl Protocol for ModbusProtocol {
    fn name(&self) -> &'static str { "modbus" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

struct OpcuaProtocol;
impl Protocol for OpcuaProtocol {
    fn name(&self) -> &'static str { "opcua" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["binary"] }
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
    assert_eq!(reg.list_by_category(ProtocolCategory::IndustrialEthernet).len(), 1);
    assert_eq!(reg.list_by_category(ProtocolCategory::Fieldbus).len(), 0);
}

#[test]
fn test_protocol_names() {
    let mut reg = ProtocolRegistry::new();
    reg.register(Box::new(ModbusProtocol));
    assert!(reg.protocol_names().contains(&"modbus".to_string()));
}
