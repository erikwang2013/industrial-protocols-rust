// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_powerlink::*;
use std::io;

/// Mock connection with a canned response and a failure mode.
struct MockConn {
    response: Vec<u8>,
    fail: bool,
    sent: Vec<Vec<u8>>,
}

impl MockConn {
    fn new(response: Vec<u8>) -> Self {
        MockConn {
            response,
            fail: false,
            sent: Vec::new(),
        }
    }
    fn failing() -> Self {
        MockConn {
            response: Vec::new(),
            fail: true,
            sent: Vec::new(),
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "mock failure"));
        }
        self.sent.push(data.to_vec());
        Ok(self.response.clone())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

// ---------- frame builders ----------

#[test]
fn powerlink_ident_request_exact_bytes() {
    let f = build_ident_request(1);
    assert_eq!(
        f,
        vec![0x01, 0x00, 0x00, 0xAB, 0x88, 0x01, 0x40, 0x00, 0x00, 0x00]
    );
}

#[test]
fn powerlink_ident_request_edge_node_ids() {
    assert_eq!(build_ident_request(0)[5], 0x00);
    assert_eq!(build_ident_request(0xFF)[5], 0xFF);
}

#[test]
fn powerlink_poll_request_exact_bytes() {
    assert_eq!(
        build_poll_request(7),
        vec![0x01, 0x07, 0x80, 0x00, 0x00, 0x00]
    );
    assert_eq!(build_poll_request(0)[1], 0x00);
    assert_eq!(build_poll_request(0xFF)[1], 0xFF);
}

#[test]
fn powerlink_poll_request_length() {
    assert_eq!(build_poll_request(3).len(), 6);
    assert_eq!(build_ident_request(3).len(), 10);
}

// ---------- protocol metadata ----------

#[test]
fn powerlink_metadata() {
    let p = PowerlinkProtocol::new(1);
    assert_eq!(p.name(), "powerlink");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["epi"]);
    assert_eq!(p.node_id, 1);
}

// ---------- ProtocolRead ----------

#[test]
fn powerlink_read_returns_response() {
    let p = PowerlinkProtocol::new(5);
    let mut conn = MockConn::new(vec![0xDE, 0xAD]);
    let v = ProtocolRead::read(&p, &mut conn, &1).unwrap();
    assert_eq!(v, vec![0xDE, 0xAD]);
    // sent frame must be the poll request for node 5
    assert_eq!(conn.sent, vec![build_poll_request(5)]);
}

#[test]
fn powerlink_read_connection_failure() {
    let p = PowerlinkProtocol::new(1);
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn powerlink_read_empty_response() {
    let p = PowerlinkProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read(&p, &mut conn, &2).unwrap(),
        Vec::<u8>::new()
    );
}

#[test]
fn powerlink_read_batch_collects() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::new(vec![0x01]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(vals, vec![vec![0x01], vec![0x01], vec![0x01]]);
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn powerlink_read_batch_empty() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::new(vec![0x01]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn powerlink_read_batch_connection_failure() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read_batch(&p, &mut conn, &[1]).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn powerlink_read_range_count_zero() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::new(vec![0x01]);
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &1, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn powerlink_read_range_multiple() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::new(vec![0xAA, 0xBB]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &9, 4).unwrap();
    assert_eq!(vals, vec![vec![0xAA, 0xBB]; 4]);
    assert_eq!(conn.sent.len(), 4);
}

#[test]
fn powerlink_read_range_connection_failure() {
    let p = PowerlinkProtocol::new(3);
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read_range(&p, &mut conn, &1, 2).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

// ---------- ProtocolHealth ----------

#[test]
fn powerlink_health_healthy() {
    let p = PowerlinkProtocol::new(1);
    let mut conn = MockConn::new(vec![0x00]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("ident"));
    assert_eq!(conn.sent, vec![build_ident_request(1)]);
}

#[test]
fn powerlink_health_unhealthy() {
    let p = PowerlinkProtocol::new(1);
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn powerlink_diagnostics_fields() {
    let p = PowerlinkProtocol::new(42);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "powerlink:42");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
