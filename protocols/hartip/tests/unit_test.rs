// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_hartip::*;
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
fn build_hartip_init_exact_bytes() {
    // Version 1, Init msg type, MsgID 1, status/seq zeroed
    assert_eq!(
        build_hartip_init(),
        vec![0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn build_hartip_read_pv_exact_bytes() {
    // 10-byte header + 5-byte HART command-1 body with checksum
    let f = build_hartip_read_pv();
    assert_eq!(
        f,
        vec![
            0x01, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x05, 0x82, 0x00, 0x01, 0x00,
            0x01
        ]
    );
    assert_eq!(f.len(), 15);
}

#[test]
fn build_hartip_read_pv_length_field() {
    let f = build_hartip_read_pv();
    // length (bytes 8..10, BE) must equal body length 5
    assert_eq!(((f[8] as usize) << 8) | f[9] as usize, 5);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = HartIpProtocol::new();
    assert_eq!(p.name(), "hartip");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IoTMessaging);
    assert_eq!(p.supported_variants(), &["tcp", "udp"]);
}

// ---- ProtocolRead ----

fn pv_response(value: f32) -> Vec<u8> {
    let mut v = vec![0u8; 16];
    v.extend_from_slice(&value.to_be_bytes());
    v
}

#[test]
fn read_ok_parses_be_f32() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![pv_response(1.5)]);
    assert_eq!(p.read(&mut conn, &"dev1".into()).unwrap(), 1.5);
}

#[test]
fn read_short_response_returns_zero() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 19]]);
    assert_eq!(p.read(&mut conn, &"dev1".into()).unwrap(), 0.0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &"dev1".into()).unwrap(), 0.0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &"dev1".into()).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_with_string_addresses() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![pv_response(1.0), pv_response(2.0), pv_response(3.0)]);
    let addrs = vec!["dev1".to_string(), "dev2".to_string(), "dev3".to_string()];
    assert_eq!(
        p.read_batch(&mut conn, &addrs).unwrap(),
        vec![1.0, 2.0, 3.0]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![pv_response(42.0); 3]);
    assert_eq!(
        p.read_range(&mut conn, &"dev1".into(), 3).unwrap(),
        vec![42.0; 3]
    );
}

#[test]
fn read_range_zero_count() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &"dev1".into(), 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 10]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("init"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = HartIpProtocol::new();
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "hartip-device");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
