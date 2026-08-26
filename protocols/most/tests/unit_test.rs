// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_most::*;

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
fn build_control_message_exact_bytes() {
    // [Preamble 0x01 0x00][Target BE][FktID BE][Len][Data]
    let f = build_control_message(0x0100, 0x0101, &[]);
    assert_eq!(f, vec![0x01, 0x00, 0x01, 0x00, 0x01, 0x01, 0x00]);
    assert_eq!(f.len(), 7);
}

#[test]
fn build_control_message_with_data() {
    let f = build_control_message(0x0100, 0x0101, &[0xAA, 0xBB]);
    assert_eq!(
        f,
        vec![0x01, 0x00, 0x01, 0x00, 0x01, 0x01, 0x02, 0xAA, 0xBB]
    );
    assert_eq!(f.len(), 9);
}

#[test]
fn build_control_message_max_target_and_len() {
    let data = vec![0x5A; 255];
    let f = build_control_message(0xFFFF, 0xFFFF, &data);
    assert_eq!(f[6], 0xFF);
    assert_eq!(f.len(), 7 + 255);
    assert_eq!(&f[7..], &data[..]);
}

#[test]
fn build_netblock_read_exact_bytes() {
    // [Preamble 0x01 0x02][Target BE][Offset BE 4][Len 0x0004]
    let f = build_netblock_read(0x0100, 0x0102);
    assert_eq!(
        f,
        vec![0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x04]
    );
    assert_eq!(f.len(), 10);
}

#[test]
fn build_netblock_read_zero_offset() {
    let f = build_netblock_read(0x0100, 0);
    assert_eq!(
        f,
        vec![0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04]
    );
}

#[test]
fn build_netblock_read_max_offset() {
    let f = build_netblock_read(0xFFFF, 0xFFFFFFFF);
    assert_eq!(&f[2..4], &[0xFF, 0xFF]);
    assert_eq!(&f[4..8], &u32::MAX.to_be_bytes()[..]);
}

// ---- builder ----

#[test]
fn addr_builder_sets_node_addr() {
    assert_eq!(MostProtocol::new().node_addr, 0x0100);
    assert_eq!(MostProtocol::default().node_addr, 0x0100);
    assert_eq!(MostProtocol::new().addr(0x0200).node_addr, 0x0200);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = MostProtocol::new();
    assert_eq!(p.name(), "most");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::AutomotiveBus);
    assert_eq!(p.supported_variants(), &["25", "50", "150"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_returns_raw_response() {
    let p = MostProtocol::new().addr(0x0200);
    let mut conn = MockConn::new(vec![vec![0xDE, 0xAD]]);
    assert_eq!(p.read(&mut conn, &0x0102).unwrap(), vec![0xDE, 0xAD]);
    assert_eq!(conn.sent, vec![build_netblock_read(0x0200, 0x0102)]);
}

#[test]
fn read_empty_response_ok() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), Vec::<u8>::new());
}

#[test]
fn read_connection_failure_propagates() {
    let p = MostProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![vec![1], vec![2], vec![3]]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 4, 8]).unwrap(),
        vec![vec![1], vec![2], vec![3]]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![vec![9]; 3]);
    assert_eq!(
        p.read_range(&mut conn, &0x100, 3).unwrap(),
        vec![vec![9]; 3]
    );
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn read_range_zero_count() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = MostProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("most"));
    assert_eq!(
        conn.sent,
        vec![build_control_message(p.node_addr, 0x0101, &[])]
    );
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = MostProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = MostProtocol::new().addr(0x0200);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "most:512");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
