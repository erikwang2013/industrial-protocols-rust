// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_iolink::*;
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
fn build_read_pdin_exact_bytes() {
    // [Channel][Service 0x48][Index 0x0000][Subindex 0x00]
    let f = build_read_pdin(3);
    assert_eq!(f, vec![3, 0x48, 0x00, 0x00, 0x00]);
    assert_eq!(f.len(), 5);
}

#[test]
fn build_read_pdin_port_max() {
    let f = build_read_pdin(0xFF);
    assert_eq!(f[0], 0xFF);
    assert_eq!(f[1], 0x48);
}

#[test]
fn build_read_param_exact_bytes() {
    // [Channel][Service 0x43][Index hi][Index lo][Subindex]
    let f = build_read_param(3, 0x0102, 7);
    assert_eq!(f, vec![3, 0x43, 0x01, 0x02, 7]);
    assert_eq!(f.len(), 5);
}

#[test]
fn build_read_param_index_zero_and_max() {
    assert_eq!(build_read_param(1, 0, 0), vec![1, 0x43, 0x00, 0x00, 0x00]);
    assert_eq!(
        build_read_param(1, 0xFFFF, 0xFF),
        vec![1, 0x43, 0xFF, 0xFF, 0xFF]
    );
}

#[test]
fn build_write_param_exact_bytes() {
    let f = build_write_param(3, 0x0102, 7, &[1, 2, 3]);
    assert_eq!(f, vec![3, 0x43, 0x01, 0x02, 7, 1, 2, 3]);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_write_param_empty_data() {
    let f = build_write_param(3, 0x0102, 7, &[]);
    assert_eq!(f, vec![3, 0x43, 0x01, 0x02, 7]);
    assert_eq!(f.len(), 5);
}

// ---- builder ----

#[test]
fn port_builder_sets_port() {
    assert_eq!(IolinkProtocol::new().port, 1);
    assert_eq!(IolinkProtocol::default().port, 1);
    assert_eq!(IolinkProtocol::new().master_host, "192.168.1.1");
    assert_eq!(IolinkProtocol::new().port(7).port, 7);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = IolinkProtocol::new();
    assert_eq!(p.name(), "iolink");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["com1", "com2", "com3"]);
}

// ---- ProtocolRead ----

#[test]
fn read_sends_read_param_and_returns_response() {
    let p = IolinkProtocol::new().port(3);
    let mut conn = MockConn::new(vec![vec![0x11, 0x22]]);
    assert_eq!(p.read(&mut conn, &0x0102).unwrap(), vec![0x11, 0x22]);
    assert_eq!(conn.sent, vec![build_read_param(3, 0x0102, 0)]);
}

#[test]
fn read_empty_response_ok() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &1).unwrap(), Vec::<u8>::new());
}

#[test]
fn read_connection_failure_propagates() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::new(vec![vec![1], vec![2], vec![3]]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 4, 8]).unwrap(),
        vec![vec![1], vec![2], vec![3]]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = IolinkProtocol::new().port(2);
    let mut conn = MockConn::new(vec![vec![5]; 3]);
    assert_eq!(
        p.read_range(&mut conn, &0x100, 3).unwrap(),
        vec![vec![5]; 3]
    );
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn read_range_zero_count() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolWrite ----

#[test]
fn write_sends_write_param() {
    let p = IolinkProtocol::new().port(3);
    let mut conn = MockConn::new(vec![vec![]]);
    p.write(&mut conn, &0x0102, &vec![0xAA, 0xBB]).unwrap();
    assert_eq!(
        conn.sent,
        vec![build_write_param(3, 0x0102, 0, &[0xAA, 0xBB])]
    );
}

#[test]
fn write_connection_failure_propagates() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.write(&mut conn, &1, &vec![1]).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn write_batch_multiple_writes_in_order() {
    let p = IolinkProtocol::new().port(1);
    let mut conn = MockConn::new(vec![vec![]; 2]);
    p.write_batch(&mut conn, &[(0x100, vec![1]), (0x200, vec![2])])
        .unwrap();
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(conn.sent[0], build_write_param(1, 0x100, 0, &[1]));
    assert_eq!(conn.sent[1], build_write_param(1, 0x200, 0, &[2]));
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = IolinkProtocol::new().port(4);
    let mut conn = MockConn::new(vec![vec![0]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("PDin"));
    assert_eq!(conn.sent, vec![build_read_pdin(4)]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = IolinkProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = IolinkProtocol::new().port(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "iolink:port5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
