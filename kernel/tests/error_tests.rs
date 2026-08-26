// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

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
    let err = ProtocolError::Timeout {
        device: "plc-001".into(),
        timeout_ms: 3000,
    };
    let msg = err.to_string();
    assert!(msg.contains("plc-001"));
    assert!(msg.contains("3000"));
}

#[test]
fn test_context_chain_display() {
    let inner = ProtocolError::Timeout {
        device: "plc-001".into(),
        timeout_ms: 3000,
    };
    let outer = inner.context("failed to read holding register 40001 from PLC-001");
    let msg = outer.to_string();
    assert!(msg.contains("failed to read"));
    assert!(msg.contains("plc-001"));
}

#[test]
fn test_device_error_with_hex_code() {
    let err = ProtocolError::DeviceError {
        device: "plc-001".into(),
        code: 0x02,
        message: "illegal data address".into(),
    };
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
    let err = ProtocolError::CircuitOpen {
        device: "robot-003".into(),
        retry_after: Duration::from_secs(30),
    };
    let msg = err.to_string();
    assert!(msg.contains("robot-003"));
}

#[test]
fn test_parse_error_display() {
    let err = ProtocolError::Parse {
        protocol: "modbus".into(),
        reason: "crc mismatch".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("modbus"));
    assert!(msg.contains("crc mismatch"));
}

#[test]
fn test_device_not_found_display() {
    let err = ProtocolError::DeviceNotFound("plc-404".into());
    assert!(err.to_string().contains("plc-404"));
}

#[test]
fn test_unsupported_display() {
    let err = ProtocolError::Unsupported {
        protocol: "opcua".into(),
        operation: "write".into(),
    };
    let msg = err.to_string();
    assert!(msg.contains("opcua"));
    assert!(msg.contains("write"));
}

#[test]
fn test_config_error_display() {
    let err = ProtocolError::Config("bad toml".into());
    assert!(err.to_string().contains("bad toml"));
}

#[test]
fn test_from_string_and_str() {
    let a = ProtocolError::from(String::from("vendor exploded"));
    let b = ProtocolError::from("vendor exploded too");
    assert!(matches!(a, ProtocolError::ProtocolSpecific(_)));
    assert!(matches!(b, ProtocolError::ProtocolSpecific(_)));
    assert!(a.to_string().contains("vendor exploded"));
    assert!(b.to_string().contains("vendor exploded too"));
}

#[test]
fn test_context_chain_source() {
    let inner = ProtocolError::Timeout {
        device: "d".into(),
        timeout_ms: 5,
    };
    let outer = inner.context("step 1");
    let err: &dyn std::error::Error = &outer;
    let source = err.source().expect("context chain must expose a source");
    // The source chain is exposed and preserves the inner message.
    assert!(source.to_string().contains("device=d"));
}

#[test]
fn test_result_alias() {
    fn f() -> industrial_protocols_kernel::error::Result<u32> {
        Ok(1)
    }
    assert_eq!(f().unwrap(), 1);
}
