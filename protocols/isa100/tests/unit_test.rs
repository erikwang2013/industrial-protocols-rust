// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_isa100::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

struct MockConn {
    responses: VecDeque<Vec<u8>>,
    sent: Vec<Vec<u8>>,
    fail: bool,
}

impl MockConn {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockConn {
            responses: responses.into(),
            sent: Vec::new(),
            fail: false,
        }
    }
    fn failing() -> Self {
        MockConn {
            responses: VecDeque::new(),
            sent: Vec::new(),
            fail: true,
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "conn down",
            ));
        }
        self.sent.push(data.to_vec());
        self.responses
            .pop_front()
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected request"))
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        !self.fail
    }
}

/// Response with an f64 at bytes 13..21 (big-endian), per read()'s layout.
fn f64_resp(v: f64) -> Vec<u8> {
    let mut r = vec![0u8; 21];
    r[13..21].copy_from_slice(&v.to_be_bytes());
    r
}

// ---- frame builders ----

#[test]
fn build_read_object_exact_bytes() {
    // [Cmd 0x01][DeviceID 8B BE][ObjectID 2B BE][Attribute 2B BE]
    let f = build_read_object(0x0102030405060708, 0x0102, 0x0003);
    assert_eq!(
        f,
        vec![
            0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x01, 0x02, 0x00, 0x03
        ]
    );
    assert_eq!(f.len(), 13);
}

#[test]
fn build_read_object_zero_id() {
    let f = build_read_object(0, 0, 0);
    assert_eq!(f, vec![0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn build_read_object_max_values() {
    let f = build_read_object(u64::MAX, 0xFFFF, 0xFFFF);
    assert_eq!(f.len(), 13);
    assert_eq!(&f[1..9], &u64::MAX.to_be_bytes()[..]);
    assert_eq!(&f[9..11], &[0xFF, 0xFF]);
    assert_eq!(&f[11..13], &[0xFF, 0xFF]);
}

#[test]
fn build_read_process_data_exact_bytes() {
    // [Cmd 0x02][DeviceID 8B BE][Zero padding 8B]
    let f = build_read_process_data(0x0102030405060708);
    assert_eq!(
        f,
        vec![
            0x02, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0, 0, 0, 0, 0, 0, 0, 0
        ]
    );
    assert_eq!(f.len(), 17);
}

#[test]
fn build_read_process_data_zero_id() {
    let f = build_read_process_data(0);
    assert_eq!(
        f,
        vec![0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}

// ---- builder ----

#[test]
fn id_gateway_builders_set_fields() {
    assert_eq!(Isa100Protocol::new().device_id, 0x0102030405060708);
    assert_eq!(Isa100Protocol::default().device_id, 0x0102030405060708);
    assert_eq!(Isa100Protocol::new().gateway_host, "192.168.1.1");
    assert_eq!(Isa100Protocol::new().id(7).device_id, 7);
    assert_eq!(
        Isa100Protocol::new().gateway("gw.local").gateway_host,
        "gw.local"
    );
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = Isa100Protocol::new();
    assert_eq!(p.name(), "isa100");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::HardwareBridge);
    assert_eq!(p.supported_variants(), &["11a"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_be_f64() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![f64_resp(3.5)]);
    assert_eq!(p.read(&mut conn, &0x100).unwrap(), 3.5);
}

#[test]
fn read_ok_parses_negative_f64() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![f64_resp(-1.25)]);
    assert_eq!(p.read(&mut conn, &0x100).unwrap(), -1.25);
}

#[test]
fn read_short_response_returns_zero() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![vec![0; 20]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0.0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0.0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_sends_read_object_for_address() {
    let p = Isa100Protocol::new().id(9);
    let mut conn = MockConn::new(vec![vec![0; 21]]);
    p.read(&mut conn, &0x100).unwrap();
    assert_eq!(conn.sent, vec![build_read_object(9, 0x100, 0)]);
}

#[test]
fn read_batch_collects_values() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![f64_resp(1.0), f64_resp(2.0), f64_resp(3.0)]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 4, 8]).unwrap(),
        vec![1.0, 2.0, 3.0]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<f64>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![f64_resp(7.0); 3]);
    assert_eq!(
        p.read_range(&mut conn, &0x50, 3).unwrap(),
        vec![7.0, 7.0, 7.0]
    );
}

#[test]
fn read_range_zero_count() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), Vec::<f64>::new());
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::new(vec![vec![0]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("isa100"));
    assert_eq!(conn.sent, vec![build_read_process_data(p.device_id)]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = Isa100Protocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = Isa100Protocol::new().id(0x102030405060708);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "isa100:102030405060708");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
