// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_dnp3::{Dnp3Protocol, build_class0_poll, parse_class0_response};
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

fn response_prefix() -> Vec<u8> {
    // start(2) + len(1) + control(1) + dest(2) + src(2) + crc(2)
    vec![0x05, 0x64, 0x00, 0x44, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00]
}

fn binary_input_resp(count: u8) -> Vec<u8> {
    let mut r = response_prefix();
    r.extend_from_slice(&[0x81, 0x00, 0x00, 0x01, 0x00, 0x00, count]);
    r.push(0x00); // parser needs i+4 < len to enter the response branch
    r
}

fn analog_resp(values: &[f32]) -> Vec<u8> {
    let mut r = response_prefix();
    r.extend_from_slice(&[0x81, 0x00, 0x00, 0x30, 0x01, 0x00]);
    for v in values {
        r.extend_from_slice(&v.to_le_bytes());
    }
    r
}

// ── frame builder ──

#[test]
fn class0_poll_frame_exact_bytes() {
    let f = build_class0_poll(0x0102, 0x0304);
    assert_eq!(&f[..2], &[0x05, 0x64]);
    let data_len = 5 + 7; // control+dest+src + transport(1 + 6 app bytes)
    assert_eq!(f[2], data_len as u8);
    assert_eq!(f[3], 0x44); // unconfirmed user data
    assert_eq!(&f[4..6], &[0x04, 0x03]); // dest LE
    assert_eq!(&f[6..8], &[0x02, 0x01]); // src LE
    assert_eq!(f[10], 0xC0 | 0x06); // FIR+FIN + app len 6
    assert_eq!(f[11], 0x01); // Read function
    // transport <= 16 bytes so no body CRC is appended
    assert_eq!(&f[12..17], &[0x3C, 0x02, 0x06, 0x01, 0x00]); // class 0 body
    assert_eq!(f.len(), 17); // header(10) + transport(7)
}

// ── response parsing ──

#[test]
fn parse_binary_input_response() {
    let values = parse_class0_response(&binary_input_resp(4)).unwrap();
    assert_eq!(values, vec![4.0]);
}

#[test]
fn parse_analog_input_response() {
    let values = parse_class0_response(&analog_resp(&[1.5, -2.25])).unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0], 1.5);
    assert_eq!(values[1], -2.25);
}

#[test]
fn parse_response_errors() {
    assert!(parse_class0_response(&[0x05]).is_err()); // too short
    let mut bad_start = response_prefix();
    bad_start[0] = 0x00;
    assert!(parse_class0_response(&bad_start).is_err());
    // valid header but no response function code -> empty
    let mut empty = response_prefix();
    empty.extend_from_slice(&[0x00, 0x00, 0x00]);
    assert_eq!(parse_class0_response(&empty).unwrap(), Vec::<f64>::new());
}

// ── metadata ──

#[test]
fn dnp3_metadata() {
    let p = Dnp3Protocol::new(1, 100);
    assert_eq!(p.name(), "dnp3");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["tcp"]);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_returns_first_value() {
    let p = Dnp3Protocol::new(1, 100);
    let mut conn = MockConnection::new(vec![binary_input_resp(3)]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 3.0);
    assert_eq!(&conn.sent[0][..2], &[0x05, 0x64]);
}

#[test]
fn read_empty_response_and_connection_failure() {
    let p = Dnp3Protocol::new(1, 100);
    let mut conn = MockConnection::new(vec![response_prefix()]);
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
    let p = Dnp3Protocol::new(1, 100);
    let mut conn = MockConnection::new(vec![binary_input_resp(1), binary_input_resp(2)]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0, 1]).unwrap();
    assert_eq!(vals, vec![1.0, 2.0]);
    let mut conn2 = MockConnection::new(vec![binary_input_resp(9), binary_input_resp(9)]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &0, 2).unwrap();
    assert_eq!(vals2, vec![9.0, 9.0]);
    // count 0 sends nothing
    let mut conn3 = MockConnection::new(vec![]);
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn3, &0, 0).unwrap(),
        Vec::<f64>::new()
    );
}

// ── ProtocolWrite ──

#[test]
fn write_is_unsupported() {
    let p = Dnp3Protocol::new(1, 2);
    let mut conn = MockConnection::new(vec![]);
    let err = ProtocolWrite::write(&p, &mut conn, &0, &1.0).unwrap_err();
    assert!(matches!(err, ProtocolError::Unsupported { .. }));
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = Dnp3Protocol::new(1, 2);
    let mut conn = MockConnection::new(vec![binary_input_resp(1)]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("Class 0"));
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = Dnp3Protocol::new(0x10, 0x20);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "dnp3:src16");
    assert_eq!(d.custom_fields["source_addr"], "16");
    assert_eq!(d.custom_fields["dest_addr"], "32");
}
