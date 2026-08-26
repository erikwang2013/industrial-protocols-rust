// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_devicenet::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

struct MockConn {
    responses: VecDeque<Vec<u8>>,
    fail: bool,
}

impl MockConn {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockConn {
            responses: responses.into(),
            fail: false,
        }
    }
    fn failing() -> Self {
        MockConn {
            responses: VecDeque::new(),
            fail: true,
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "conn down",
            ));
        }
        Ok(self.responses.pop_front().unwrap_or_default())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        !self.fail
    }
}

// ---- frame builders ----

#[test]
fn build_explicit_request_exact_bytes() {
    // CAN ID 0x600|mac as 2 bytes, then service/class/instance/attr
    let f = build_explicit_request(5, 0x0E, 0x20, 0x01, 0x01);
    assert_eq!(f, vec![0x06, 0x05, 0x0E, 0x20, 0x01, 0x01]);
    assert_eq!(f.len(), 6);
}

#[test]
fn build_explicit_request_mac_affects_can_id() {
    let f = build_explicit_request(0xFF, 0x0E, 0x20, 0x01, 0x01);
    assert_eq!(f[0], 0x06);
    assert_eq!(f[1], 0xFF);
}

#[test]
fn build_poll_request_exact_bytes() {
    // Group 1 CAN ID (0x04) + MAC in ID bits 2..8, plus MAC byte
    // 0x04 | (5 << 2) = 0x04 | 0x14 = 0x14
    assert_eq!(build_poll_request(5), vec![0x14, 0x05, 0x00, 0x00]);
    assert_eq!(build_poll_request(0), vec![0x04, 0x00, 0x00, 0x00]);
}

#[test]
fn build_poll_request_max_mac_id() {
    let f = build_poll_request(0x3F);
    assert_eq!(f[0], 0xFC); // 0x04 | (0x3F << 2)
    assert_eq!(f[1], 0x3F);
}

// ---- builder ----

#[test]
fn with_mac_builder_sets_mac() {
    assert_eq!(DevicenetProtocol::new().mac_id, 0);
    assert_eq!(DevicenetProtocol::new().with_mac(12).mac_id, 12);
    assert_eq!(DevicenetProtocol::default().mac_id, 0);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = DevicenetProtocol::new();
    assert_eq!(p.name(), "devicenet");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["scanner"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_le_word() {
    let p = DevicenetProtocol::new().with_mac(5);
    // address packs class into high byte, attribute into low byte
    let mut conn = MockConn::new(vec![vec![0; 4].into_iter().chain([0x34, 0x12]).collect()]);
    assert_eq!(p.read(&mut conn, &0x2001).unwrap(), 0x1234);
}

#[test]
fn read_short_response_returns_zero() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 5]]);
    assert_eq!(p.read(&mut conn, &0x2001).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0x2001).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0x2001).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = DevicenetProtocol::new();
    let mk = |lo: u8| {
        let mut v = vec![0; 4];
        v.extend_from_slice(&[lo, 0x00]);
        v
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(
        p.read_batch(&mut conn, &[0x2001, 0x2002, 0x2003]).unwrap(),
        vec![1, 2, 3]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = DevicenetProtocol::new();
    let mut v = vec![0; 4];
    v.extend_from_slice(&[0x2A, 0x00]);
    let mut conn = MockConn::new(vec![v; 3]);
    assert_eq!(
        p.read_range(&mut conn, &0x2001, 3).unwrap(),
        vec![42, 42, 42]
    );
}

#[test]
fn read_range_zero_count() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0x2001, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 4]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("poll"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = DevicenetProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = DevicenetProtocol::new().with_mac(8);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "dn:8");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
