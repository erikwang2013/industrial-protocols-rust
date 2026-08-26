// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_interbus::*;
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

// ---- frame builders ----

#[test]
fn build_read_request_exact_bytes() {
    // [Loopback 0x00][Cmd 0x01][Slave][Reg hi][Reg lo][CRC 0x00][CRC 0x00]
    let f = build_read_request(1, 0x0102);
    assert_eq!(f, vec![0x00, 0x01, 0x01, 0x01, 0x02, 0x00, 0x00]);
    assert_eq!(f.len(), 7);
}

#[test]
fn build_read_request_register_zero() {
    let f = build_read_request(0xFF, 0);
    assert_eq!(f, vec![0x00, 0x01, 0xFF, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn build_read_request_register_max() {
    let f = build_read_request(0x7F, 0xFFFF);
    assert_eq!(f, vec![0x00, 0x01, 0x7F, 0xFF, 0xFF, 0x00, 0x00]);
}

#[test]
fn build_write_request_exact_bytes() {
    // [Loopback][Cmd 0x02][Slave][Reg hi][Reg lo][Len][Data...]
    let f = build_write_request(2, 0x0100, &[0xAA, 0xBB]);
    assert_eq!(f, vec![0x00, 0x02, 0x02, 0x01, 0x00, 0x02, 0xAA, 0xBB]);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_write_request_empty_data() {
    let f = build_write_request(2, 0x0100, &[]);
    assert_eq!(f, vec![0x00, 0x02, 0x02, 0x01, 0x00, 0x00]);
    assert_eq!(f.len(), 6);
}

#[test]
fn build_write_request_max_length_data() {
    let data = vec![0x5A; 255];
    let f = build_write_request(2, 0x0100, &data);
    assert_eq!(f[5], 0xFF);
    assert_eq!(f.len(), 6 + 255);
    assert_eq!(&f[6..], &data[..]);
}

// ---- builder ----

#[test]
fn slave_builder_sets_slave_id() {
    assert_eq!(InterbusProtocol::new().slave_id, 1);
    assert_eq!(InterbusProtocol::default().slave_id, 1);
    assert_eq!(InterbusProtocol::new().slave(7).slave_id, 7);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = InterbusProtocol::new();
    assert_eq!(p.name(), "interbus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["ibs"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_returns_raw_response() {
    let p = InterbusProtocol::new().slave(3);
    let mut conn = MockConn::new(vec![vec![0xDE, 0xAD, 0xBE, 0xEF]]);
    assert_eq!(
        p.read(&mut conn, &0x100).unwrap(),
        vec![0xDE, 0xAD, 0xBE, 0xEF]
    );
    assert_eq!(conn.sent, vec![build_read_request(3, 0x100)]);
}

#[test]
fn read_empty_response_ok() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), Vec::<u8>::new());
}

#[test]
fn read_connection_failure_propagates() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![1], vec![2], vec![3]]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 4, 8]).unwrap(),
        vec![vec![1], vec![2], vec![3]]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_batch_propagates_connection_error() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![1]]);
    let err = p.read_batch(&mut conn, &[0, 1]).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_range_repeats_start_address() {
    let p = InterbusProtocol::new().slave(4);
    let mut conn = MockConn::new(vec![vec![9]; 3]);
    assert_eq!(
        p.read_range(&mut conn, &0x200, 3).unwrap(),
        vec![vec![9]; 3]
    );
    assert_eq!(conn.sent.len(), 3);
    assert!(conn.sent.iter().all(|f| *f == build_read_request(4, 0x200)));
}

#[test]
fn read_range_zero_count() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![0]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("interbus"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = InterbusProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = InterbusProtocol::new().slave(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "ibs:slave5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
