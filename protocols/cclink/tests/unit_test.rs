// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_cclink::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

/// Independent CRC-16/XMODEM (poly 0x1021, init 0) used to cross-check
/// the frame checksum bytes.
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

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
fn build_poll_frame_exact_bytes() {
    let f = build_poll(1, 0x01, &[0x11, 0x22]);
    // [STX][ADDR][CMD][data...][ETX][CRC LE]
    let body = [0x10, 0x02, 0x01, 0x01, 0x11, 0x22, 0x10, 0x03];
    let crc = crc16_xmodem(&body);
    let expected: Vec<u8> = body
        .iter()
        .chain(crc.to_le_bytes().iter())
        .copied()
        .collect();
    assert_eq!(f, expected);
    assert_eq!(f.len(), 10);
}

#[test]
fn build_poll_crc_matches_check_value() {
    // Validate the test's own crc against the standard check vector
    // so the frame assertion is not circular.
    assert_eq!(crc16_xmodem(b"123456789"), 0x31C3);
}

#[test]
fn build_poll_empty_data() {
    let f = build_poll(3, 0x02, &[]);
    let body = [0x10, 0x02, 0x03, 0x02, 0x10, 0x03];
    let crc = crc16_xmodem(&body);
    let expected: Vec<u8> = body
        .iter()
        .chain(crc.to_le_bytes().iter())
        .copied()
        .collect();
    assert_eq!(f, expected);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_rx_read_frame_layout() {
    // start/count are big-endian inside the data section
    let f = build_rx_read(1, 0x1234, 0x0005);
    assert_eq!(f[0], 0x10);
    assert_eq!(f[1], 0x02);
    assert_eq!(f[2], 0x01);
    assert_eq!(f[3], 0x01); // read command
    assert_eq!(&f[4..6], &[0x12, 0x34]);
    assert_eq!(&f[6..8], &[0x00, 0x05]);
    assert_eq!(f[8], 0x10);
    assert_eq!(f[9], 0x03);
    assert_eq!(f.len(), 12);
}

#[test]
fn build_rx_read_max_values() {
    let f = build_rx_read(0xFF, 0xFFFF, 0xFFFF);
    assert_eq!(&f[4..8], &[0xFF, 0xFF, 0xFF, 0xFF]);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = CclinkProtocol::new(1);
    assert_eq!(p.name(), "cclink");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["rs485"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_be_word() {
    let p = CclinkProtocol::new(1);
    // response[4..6] interpreted big-endian
    let mut conn = MockConn::new(vec![vec![0, 0, 0, 0, 0x34, 0x12, 0, 0]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0x3412);
}

#[test]
fn read_rejects_short_response() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0, 0, 0]]);
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Parse { .. }));
}

#[test]
fn read_connection_failure_propagates() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![
        vec![0, 0, 0, 0, 0x00, 0x01, 0, 0],
        vec![0, 0, 0, 0, 0x00, 0x02, 0, 0],
        vec![0, 0, 0, 0, 0x00, 0x03, 0, 0],
    ]);
    assert_eq!(p.read_batch(&mut conn, &[1, 2, 3]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0, 0, 0, 0, 0x00, 0x2A, 0, 0]; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0, 0, 0, 0, 0, 0, 0, 0]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("ok"));
}

#[test]
fn health_unhealthy_on_failure() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

#[test]
fn health_unhealthy_on_short_response() {
    let p = CclinkProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![1, 2]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = CclinkProtocol::new(7);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "cclink:7");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
