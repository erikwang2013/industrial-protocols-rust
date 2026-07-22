use industrial_protocols_kernel::*;
use industrial_protocols_kernel::sdk::*;

struct TestProtocol;
impl Protocol for TestProtocol {
    fn name(&self) -> &'static str { "test" }
    fn version(&self) -> &'static str { "1.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

#[test]
fn test_kernel_builder_boot_shutdown() {
    let config_toml = r#"
[devices.test-001]
protocol = "test"
variant = "tcp"
host = "127.0.0.1"
port = 9999
timeout_ms = 1000
"#;
    let mut kernel = Kernel::builder().config_from_str(config_toml).with_protocol(Box::new(TestProtocol)).build();
    assert!(kernel.boot().is_ok());
    assert!(kernel.shutdown().is_ok());
}

#[test]
fn test_registry_access() {
    let kernel = Kernel::builder().with_protocol(Box::new(TestProtocol)).build();
    assert!(kernel.protocol_registry().get("test").is_some());
}

#[test]
fn test_boot_fails_missing_protocol() {
    let config_toml = r#"
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "192.168.1.10"
port = 502
"#;
    let mut kernel = Kernel::builder().config_from_str(config_toml).build();
    assert!(kernel.boot().is_err());
}
