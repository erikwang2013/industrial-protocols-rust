// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_iec61850::*;
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
fn build_tpkt_exact_bytes() {
    // TPKT: version 3, reserved 0, length = 4 + payload
    let f = build_tpkt(&[1, 2, 3]);
    assert_eq!(f, vec![0x03, 0x00, 0x00, 0x07, 0x01, 0x02, 0x03]);
}

#[test]
fn build_tpkt_empty_payload() {
    assert_eq!(build_tpkt(&[]), vec![0x03, 0x00, 0x00, 0x04]);
}

#[test]
fn build_tpkt_large_payload_length_bytes() {
    // payload > 251 forces the upper length byte
    let f = build_tpkt(&vec![0x00; 300]);
    assert_eq!(f.len(), 304);
    assert_eq!(f[2], 0x01);
    assert_eq!(f[3], 0x30); // 304 = 0x0130
}

#[test]
fn build_tpkt_max_short_length() {
    // 251-byte payload keeps the length in the low byte (255 = 0x00FF)
    let f = build_tpkt(&vec![0x00; 251]);
    assert_eq!(f[2], 0x00);
    assert_eq!(f[3], 0xFF);
}

#[test]
fn build_connect_request_structure() {
    let f = build_connect_request();
    assert_eq!(f[0], 0x03);
    assert_eq!(f[1], 0x00);
    // payload = 7-byte COTP + 14-byte MMS = 21, TPKT length = 25
    assert_eq!(((f[2] as usize) << 8) | f[3] as usize, 25);
    assert_eq!(f.len(), 25);
    assert_eq!(f[4], 0x18); // COTP connect request
    assert_eq!(f[11], 0xA8); // MMS initiate request
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = Iec61850Protocol::new();
    assert_eq!(p.name(), "iec61850");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["mms"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_returns_zero_after_connect() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![vec![0x03, 0x00, 0x00, 0x04]]);
    assert_eq!(p.read(&mut conn, &"ied/LLN0.ST.Val".into()).unwrap(), 0.0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &"ied/LLN0.ST.Val".into()).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_with_string_addresses() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![vec![0; 4]; 3]);
    let addrs = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(
        p.read_batch(&mut conn, &addrs).unwrap(),
        vec![0.0, 0.0, 0.0]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![vec![0; 4]; 3]);
    assert_eq!(
        p.read_range(&mut conn, &"ied/LLN0.ST.Val".into(), 3)
            .unwrap(),
        vec![0.0; 3]
    );
}

#[test]
fn read_range_zero_count() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &"ied/LLN0.ST.Val".into(), 0)
            .unwrap(),
        vec![]
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![vec![0x03, 0x00, 0x00, 0x04]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("COTP"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = Iec61850Protocol::new();
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "ied-001");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
