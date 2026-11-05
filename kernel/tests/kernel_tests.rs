// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::ConnectionStrategy;
use industrial_protocols_kernel::event::{EventTopic, ProtocolEvent, Severity};
use industrial_protocols_kernel::gateway::{GatewayEndpoint, GatewayRule, Transform};
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_kernel::*;

struct TestProtocol;
impl Protocol for TestProtocol {
    fn name(&self) -> &'static str {
        "test"
    }
    fn version(&self) -> &'static str {
        "1.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["tcp"]
    }
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
    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(TestProtocol))
        .build();
    assert!(kernel.boot().is_ok());
    assert!(kernel.shutdown().is_ok());
}

#[test]
fn test_registry_access() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(TestProtocol))
        .build();
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

#[test]
fn test_boot_error_names_unregistered_protocol() {
    let config_toml = r#"
[devices.missing]
protocol = "ghost"
"#;
    let mut kernel = Kernel::builder().config_from_str(config_toml).build();
    let err = kernel.boot().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    assert!(err.to_string().contains("ghost"));
}

#[test]
fn test_boot_without_config_ok() {
    let mut kernel = Kernel::builder().build();
    assert!(kernel.boot().is_ok());
    assert!(kernel.shutdown().is_ok());
}

#[test]
#[should_panic(expected = "config not loaded")]
fn test_config_accessor_panics_without_config() {
    let kernel = Kernel::builder().build();
    let _ = kernel.config();
}

#[test]
fn test_config_from_str_invalid_toml_surfaces_on_boot() {
    let mut kernel = Kernel::builder().config_from_str("not [valid").build();
    let err = kernel.boot().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    assert!(err.to_string().contains("invalid TOML config"));
}

#[test]
fn test_gateway_and_connection_accessors() {
    let mut kernel = Kernel::builder()
        .with_protocol(Box::new(TestProtocol))
        .build();
    assert!(!kernel.connection_manager().is_connected("x"));
    assert_eq!(kernel.gateway().rule_count(), 0);
    kernel.gateway_mut().add_rule(GatewayRule {
        name: "r".into(),
        source: GatewayEndpoint {
            device: "a".into(),
            address: "1".into(),
        },
        target: GatewayEndpoint {
            device: "b".into(),
            address: "2".into(),
        },
        transform: Transform::Passthrough,
    });
    assert_eq!(kernel.gateway().rule_count(), 1);
}

#[test]
fn test_event_bus_accessor() {
    let kernel = Kernel::builder().build();
    let received = std::sync::Arc::new(std::sync::Mutex::new(0usize));
    let r = received.clone();
    kernel.event_bus().subscribe(EventTopic::Error, move |_| {
        *r.lock().unwrap() += 1;
    });
    kernel.event_bus().publish(ProtocolEvent::Error {
        device_id: "d".into(),
        error: "e".into(),
        severity: Severity::Warning,
    });
    assert_eq!(*received.lock().unwrap(), 1);
}

#[test]
fn test_builder_options() {
    let mut kernel = Kernel::builder()
        .with_connection_strategy(ConnectionStrategy::Simple)
        .with_event_capacity(4)
        .with_gateway_interval(10)
        .build();
    assert!(kernel.boot().is_ok());
    assert!(kernel.shutdown().is_ok());
}

#[test]
fn test_config_from_file() {
    let mut kernel = Kernel::builder()
        .config_from_file("tests/fixtures/sample.toml")
        .build();
    // Fixture references modbus/bacnet which are not registered here.
    assert!(kernel.boot().is_err());
}

#[test]
fn test_kernel_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Kernel>();
}
