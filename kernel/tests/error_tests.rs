use industrial_protocols_kernel::error::ProtocolError;
use std::time::Duration;

#[test]
fn test_connection_error_display() {
    let err = ProtocolError::Connection("192.168.1.10:502: connection refused".into());
    let msg = err.to_string();
    assert!(msg.contains("connection refused"));
}

#[test]
fn test_timeout_error_display() {
    let err = ProtocolError::Timeout { device: "plc-001".into(), timeout_ms: 3000 };
    let msg = err.to_string();
    assert!(msg.contains("plc-001"));
    assert!(msg.contains("3000"));
}

#[test]
fn test_context_chain_display() {
    let inner = ProtocolError::Timeout { device: "plc-001".into(), timeout_ms: 3000 };
    let outer = inner.context("failed to read holding register 40001 from PLC-001");
    let msg = outer.to_string();
    assert!(msg.contains("failed to read"));
    assert!(msg.contains("plc-001"));
}

#[test]
fn test_device_error_with_hex_code() {
    let err = ProtocolError::DeviceError { device: "plc-001".into(), code: 0x02, message: "illegal data address".into() };
    let msg = err.to_string();
    assert!(msg.contains("0x02"));
    assert!(msg.contains("illegal data address"));
}

#[test]
fn test_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = ProtocolError::from(io_err);
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn test_circuit_open_error() {
    let err = ProtocolError::CircuitOpen { device: "robot-003".into(), retry_after: Duration::from_secs(30) };
    let msg = err.to_string();
    assert!(msg.contains("robot-003"));
}
