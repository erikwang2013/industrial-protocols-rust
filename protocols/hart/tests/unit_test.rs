// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_hart::{
    HartProtocol, build_command, build_read_pv, build_read_unique_id, parse_float,
};
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

// ── build_command ──

#[test]
fn command_frame_exact_bytes_empty_data() {
    let f = build_command(0x2A, 0x03, &[]);
    assert_eq!(f.len(), 10);
    assert_eq!(&f[..5], &[0xFF; 5]);
    assert_eq!(f[5], 0x82); // start delimiter
    assert_eq!(f[6], 0x2A); // poll addr
    assert_eq!(f[7], 0x03); // command
    assert_eq!(f[8], 0x00); // byte count
    assert_eq!(f[9], f[5] ^ f[6] ^ f[7] ^ f[8]); // checksum = xor of f[5..9]
}

#[test]
fn command_frame_with_data_and_max_length() {
    let f = build_command(0x00, 0x05, &[0x10, 0x20, 0x30]);
    assert_eq!(f.len(), 13);
    assert_eq!(f[8], 0x03); // byte count
    assert_eq!(&f[9..12], &[0x10, 0x20, 0x30]);
    let xor_sum: u8 = f[5..12].iter().fold(0, |a, &b| a ^ b);
    assert_eq!(f[12], xor_sum);
    // data up to 255 bytes keeps the length byte from overflowing
    let big = build_command(0, 0, &[0xAB; 255]);
    assert_eq!(big.len(), 265);
    assert_eq!(big[8], 255);
}

#[test]
fn read_unique_id_and_read_pv_delegate() {
    assert_eq!(build_read_unique_id(0), build_command(0, 0, &[]));
    assert_eq!(build_read_pv(0), build_command(0, 1, &[]));
    assert_eq!(build_read_unique_id(1)[7], 0);
    assert_eq!(build_read_pv(1)[7], 1);
}

// ── parse_float ──

#[test]
fn parse_float_happy_and_boundaries() {
    assert_eq!(parse_float(&[0x3F, 0x80, 0x00, 0x00], 0), 1.0);
    assert_eq!(parse_float(&[0x00, 0x3F, 0x80, 0x00, 0x00], 1), 1.0); // offset 1
    // exact boundary: offset + 4 == len parses
    assert_eq!(parse_float(&[0x00, 0x00, 0x00, 0x00], 0), 0.0);
}

#[test]
fn parse_float_out_of_range_returns_zero() {
    assert_eq!(parse_float(&[], 0), 0.0);
    assert_eq!(parse_float(&[0x3F, 0x80, 0x00], 0), 0.0); // short
    assert_eq!(parse_float(&[0x3F, 0x80, 0x00, 0x00], 1), 0.0); // offset 1 -> 3 bytes left
    assert_eq!(parse_float(&[0x3F, 0x80, 0x00, 0x00], 4), 0.0); // offset == len
    assert_eq!(parse_float(&[0x3F, 0x80, 0x00, 0x00], 99), 0.0); // far out of range
}

// ── metadata ──

#[test]
fn hart_metadata() {
    let p = HartProtocol::new(0x2A);
    assert_eq!(p.name(), "hart");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["fsk"]);
    assert_eq!(p.poll_addr, 0x2A);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_parses_float_from_response() {
    let p = HartProtocol::new(0);
    let mut resp = vec![0u8; 6];
    resp.extend_from_slice(&1.5f32.to_be_bytes());
    let mut conn = MockConnection::new(vec![resp]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 1.5);
    // sent command 0 (read unique id) uses the protocol poll addr
    assert_eq!(conn.sent[0][6], 0);
    assert_eq!(conn.sent[0][7], 0);
}

#[test]
fn read_short_response_and_connection_failure() {
    let p = HartProtocol::new(0);
    let mut conn = MockConnection::new(vec![vec![0; 5]]); // < 8 bytes
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &0).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &0).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = HartProtocol::new(0);
    let r = |v: f32| {
        let mut b = vec![0u8; 6];
        b.extend_from_slice(&v.to_be_bytes());
        b
    };
    let mut conn = MockConnection::new(vec![r(1.0), r(2.0)]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0, 1]).unwrap();
    assert_eq!(vals, vec![1.0, 2.0]);
    let mut conn2 = MockConnection::new(vec![r(3.0), r(3.0), r(3.0)]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &0, 3).unwrap();
    assert_eq!(vals2, vec![3.0, 3.0, 3.0]);
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = HartProtocol::new(0);
    let mut resp = vec![0u8; 10];
    resp[6..10].copy_from_slice(&1.0f32.to_be_bytes());
    let mut conn = MockConnection::new(vec![resp]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("cmd 0 ok"));
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = HartProtocol::new(7);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "hart:7");
    assert_eq!(d.custom_fields["poll_addr"], "7");
}
