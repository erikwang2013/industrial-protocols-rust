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
    assert_eq!(dev.connection_strategy, ConnectionStrategyConfig::Persistent);
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
