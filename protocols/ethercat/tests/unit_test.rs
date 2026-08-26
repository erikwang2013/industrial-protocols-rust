// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_ethercat::{EthercatProtocol, build_brd, build_read_reg};
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
fn brd_empty_data_uses_length_one() {
    let f = build_brd(0x0130, &[]);
    assert_eq!(f.len(), 12);
    assert_eq!(f[0], 0x07); // BRD command
    assert_eq!(f[1], 0x01); // index
    assert_eq!(&f[2..6], &[0x30, 0x01, 0x00, 0x00]); // address LE
    assert_eq!(&f[6..8], &[0x01, 0x00]); // len forced to 1
    assert_eq!(&f[8..10], &[0x00, 0x00]); // IRQ
    assert_eq!(&f[10..12], &[0x00, 0x00]); // WKC
}

#[test]
fn brd_with_data_includes_payload() {
    let f = build_brd(0xDEAD, &[1, 2, 3]);
    assert_eq!(f.len(), 15);
    assert_eq!(&f[6..8], &[0x03, 0x00]); // len 3
    assert_eq!(&f[10..13], &[1, 2, 3]);
}

#[test]
fn read_reg_encodes_physical_address() {
    let f = build_read_reg(1, 0x1234, 4);
    assert_eq!(f.len(), 12);
    assert_eq!(f[0], 0x04); // APRD command
    assert_eq!(f[1], 0x00);
    // phys addr = slave<<16 | addr = 0x00011234 LE
    assert_eq!(&f[2..6], &[0x34, 0x12, 0x01, 0x00]);
    assert_eq!(&f[6..8], &[0x04, 0x00]); // len
    // address masked to 16 bits
    let f2 = build_read_reg(2, 0xFFFFFFFF, 2);
    assert_eq!(&f2[2..6], &[0xFF, 0xFF, 0x02, 0x00]);
    // zero length allowed
    let f3 = build_read_reg(0, 0, 0);
    assert_eq!(&f3[6..8], &[0x00, 0x00]);
}

// ── metadata ──

#[test]
fn ethercat_metadata() {
    let p = EthercatProtocol::new(7);
    assert_eq!(p.name(), "ethercat");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["soem"]);
    assert_eq!(p.slave_addr, 7);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_returns_raw_response() {
    let p = EthercatProtocol::new(1);
    let canned = vec![0x11, 0x22, 0x33, 0x44];
    let mut conn = MockConnection::new(vec![canned.clone()]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0x1234).unwrap(), canned);
    // frame built with read_reg: cmd 0x04, len 4
    assert_eq!(conn.sent[0][0], 0x04);
    assert_eq!(&conn.sent[0][6..8], &[0x04, 0x00]);
}

#[test]
fn read_connection_failure() {
    let p = EthercatProtocol::new(1);
    let mut conn = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &0).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = EthercatProtocol::new(1);
    let mut conn = MockConnection::new(vec![vec![1], vec![2]]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0, 1]).unwrap();
    assert_eq!(vals, vec![vec![1], vec![2]]);
    let mut conn2 = MockConnection::new(vec![vec![3], vec![3]]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &0, 2).unwrap();
    assert_eq!(vals2, vec![vec![3], vec![3]]);
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = EthercatProtocol::new(1);
    let mut conn = MockConnection::new(vec![vec![0; 12]]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    // health sends a BRD read of AL Status
    assert_eq!(conn.sent[0][0], 0x07);
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = EthercatProtocol::new(0x0012);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "ethercat:18");
}
