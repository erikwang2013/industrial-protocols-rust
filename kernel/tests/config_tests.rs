// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::config::*;

#[test]
fn test_load_and_get_device() {
    let repo = ConfigRepository::load("tests/fixtures/sample.toml").unwrap();
    let dev = repo.get_device("plc-001").unwrap();
    assert_eq!(dev.protocol, "modbus");
    assert_eq!(dev.variant, "tcp");
    assert_eq!(dev.host.as_deref(), Some("192.168.1.10"));
    assert_eq!(dev.port, Some(502));
    assert_eq!(dev.timeout_ms, 3000);
    assert_eq!(dev.retry.max, 3);
    assert_eq!(
        dev.connection_strategy,
        ConnectionStrategyConfig::Persistent
    );
}

#[test]
fn test_device_not_found() {
    let repo = ConfigRepository::load("tests/fixtures/sample.toml").unwrap();
    assert!(repo.get_device("nonexistent").is_none());
}

#[test]
fn test_list_devices() {
    let repo = ConfigRepository::load("tests/fixtures/sample.toml").unwrap();
    let ids = repo.device_ids();
    assert!(ids.contains(&"plc-001".to_string()));
    assert!(ids.contains(&"chiller-002".to_string()));
}

#[test]
fn test_defaults() {
    let repo = ConfigRepository::load("tests/fixtures/sample.toml").unwrap();
    assert_eq!(repo.defaults().timeout_ms, 5000);
    assert_eq!(repo.defaults().circuit_breaker_threshold, 5);
}

#[test]
fn test_set_override() {
    let mut repo = ConfigRepository::load("tests/fixtures/sample.toml").unwrap();
    repo.set_override("plc-001", "timeout_ms", toml::Value::Integer(9999));
    assert_eq!(repo.get_device("plc-001").unwrap().timeout_ms, 9999);
}

#[test]
fn test_parse_empty_config() {
    let repo = ConfigRepository::parse("").unwrap();
    assert!(repo.device_ids().is_empty());
    assert!(repo.get_device("anything").is_none());
}

#[test]
fn test_parse_invalid_toml() {
    assert!(ConfigRepository::parse("this is not [valid").is_err());
}

#[test]
fn test_parse_missing_protocol_field_fails() {
    let toml_str = "[devices.x]\nhost = \"1.2.3.4\"\n";
    assert!(ConfigRepository::parse(toml_str).is_err());
}

#[test]
fn test_defaults_applied_for_missing_fields() {
    let repo = ConfigRepository::parse("[devices.plain]\nprotocol = \"modbus\"\n").unwrap();
    let dev = repo.get_device("plain").unwrap();
    assert_eq!(dev.timeout_ms, 5000); // defaults.timeout_ms
    assert_eq!(dev.unit_id, 1);
    assert!(dev.health_check);
    assert_eq!(
        dev.connection_strategy,
        ConnectionStrategyConfig::Persistent
    );
    assert_eq!(dev.retry.max, 3);
    assert_eq!(dev.retry.strategy, "exponential");
    assert_eq!(dev.retry.base_ms, 100);
    assert_eq!(dev.retry.max_ms, 5000);
    assert_eq!(dev.retry.step_ms, 50);
    assert!(dev.extras.is_empty());
}

#[test]
fn test_serial_device_fields() {
    let repo = ConfigRepository::parse(
        "[devices.serial]\nprotocol = \"modbus\"\nvariant = \"rtu\"\nserial_port = \"/dev/ttyUSB0\"\nbaud_rate = 9600\n",
    )
    .unwrap();
    let dev = repo.get_device("serial").unwrap();
    assert_eq!(dev.variant, "rtu");
    assert_eq!(dev.serial_port.as_deref(), Some("/dev/ttyUSB0"));
    assert_eq!(dev.baud_rate, Some(9600));
}

#[test]
fn test_extras_capture_protocol_specific_fields() {
    let repo = ConfigRepository::parse(
        "[devices.bacnet]\nprotocol = \"bacnet\"\ndevice_instance = 1001\n",
    )
    .unwrap();
    let dev = repo.get_device("bacnet").unwrap();
    let extra = dev.extras.get("device_instance").unwrap();
    assert_eq!(extra.as_integer(), Some(1001));
}

#[test]
fn test_pooled_strategy_parsing() {
    let repo = ConfigRepository::parse(
        "[devices.pool]\nprotocol = \"modbus\"\nconnection_strategy = { pooled = { min = 1, max = 4, timeout_ms = 1000 } }\n",
    )
    .unwrap();
    let dev = repo.get_device("pool").unwrap();
    assert_eq!(
        dev.connection_strategy,
        ConnectionStrategyConfig::Pooled {
            min: 1,
            max: 4,
            timeout_ms: 1000
        }
    );
}

#[test]
fn test_override_unit_id_and_clear() {
    let mut repo =
        ConfigRepository::parse("[devices.plc]\nprotocol = \"modbus\"\nunit_id = 1\n").unwrap();
    repo.set_override("plc", "unit_id", toml::Value::Integer(9));
    assert_eq!(repo.get_device("plc").unwrap().unit_id, 9);
    repo.clear_overrides("plc");
    assert_eq!(repo.get_device("plc").unwrap().unit_id, 1);
}

#[test]
fn test_override_on_unknown_device_keeps_device_missing() {
    let mut repo = ConfigRepository::parse("[devices.plc]\nprotocol = \"modbus\"\n").unwrap();
    repo.set_override("ghost", "timeout_ms", toml::Value::Integer(1));
    assert!(repo.get_device("ghost").is_none());
}
