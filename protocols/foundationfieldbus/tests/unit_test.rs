// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_foundationfieldbus::{FfProtocol, build_h1_read, build_hse_read_request};
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use std::collections::VecDeque;
use std::io;

struct MockConnection {
    responses: VecDeque<Vec<u8>>,
    fail: bool,
    sent: Vec<Vec<u8>>,
}

impl MockConnection {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockConnection {
            responses: responses.into(),
            fail: false,
            sent: vec![],
        }
    }
    fn failing() -> Self {
        MockConnection {
            responses: VecDeque::new(),
            fail: true,
            sent: vec![],
        }
    }
}

impl Connection for MockConnection {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.sent.push(data.to_vec());
        if self.fail {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "mock failure",
            ));
        }
        Ok(self.responses.pop_front().unwrap_or_default())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

// ── frame builders ──

#[test]
fn h1_read_frame_exact_bytes() {
    let f = build_h1_read(0x001234, "TAG");
    assert_eq!(&f[..3], &[0xAA, 0xAA, 0x86]); // preamble + delimiter
    assert_eq!(f[3], 0x04); // DLL
    assert_eq!(&f[4..8], &[0x00, 0x00, 0x12, 0x34]); // device id BE
    assert_eq!(&f[8..], b"TAG");
    assert_eq!(f.len(), 11);
    // empty tag
    let f2 = build_h1_read(0, "");
    assert_eq!(f2.len(), 8);
}

#[test]
fn hse_read_request_exact_bytes() {
    let f = build_hse_read_request(0x00001234, 0x0102);
    assert_eq!(f.len(), 8);
    assert_eq!(f, vec![0x01, 0x00, 0x00, 0x34, 0x01, 0x02, 0x00, 0x04]);
    // max device id and index still fit
    let f2 = build_hse_read_request(0xFFFFFFFF, 0xFFFF);
    assert_eq!(f2, vec![0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x04]);
}

// ── metadata ──

#[test]
fn ff_metadata() {
    let p = FfProtocol::new().with_id(0x0000CAFE);
    assert_eq!(p.name(), "foundationfieldbus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["h1", "hse"]);
    assert_eq!(p.device_id, 0x0000CAFE);
    let _d: FfProtocol = Default::default();
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_parses_float_from_response() {
    let p = FfProtocol::new();
    let mut resp = vec![0u8; 6];
    resp.extend_from_slice(&2.5f32.to_be_bytes());
    let mut conn = MockConnection::new(vec![resp]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0x0102).unwrap(), 2.5);
    // sent an HSE read request for the protocol device id
    assert_eq!(conn.sent[0].len(), 8);
    assert_eq!(&conn.sent[0][2..4], &[0x00, 0x34]);
}

#[test]
fn read_short_response_returns_zero() {
    let p = FfProtocol::new();
    let mut conn = MockConnection::new(vec![vec![0; 9]]); // < 10 bytes
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0.0);
    let mut conn2 = MockConnection::new(vec![vec![]]);
    assert_eq!(ProtocolRead::read(&p, &mut conn2, &0).unwrap(), 0.0);
}

#[test]
fn read_connection_failure() {
    let p = FfProtocol::new();
    let mut conn = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &0).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = FfProtocol::new();
    let r = |v: f32| {
        let mut b = vec![0u8; 6];
        b.extend_from_slice(&v.to_be_bytes());
        b
    };
    let mut conn = MockConnection::new(vec![r(1.0), r(2.0)]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0, 1]).unwrap();
    assert_eq!(vals, vec![1.0, 2.0]);
    let mut conn2 = MockConnection::new(vec![r(4.0), r(4.0), r(4.0)]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &0, 3).unwrap();
    assert_eq!(vals2, vec![4.0, 4.0, 4.0]);
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = FfProtocol::new();
    let mut conn = MockConnection::new(vec![vec![0; 10]]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("HSE"));
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = FfProtocol::new().with_id(0x0012);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "ff:18");
}
