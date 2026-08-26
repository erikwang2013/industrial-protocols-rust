// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_dali::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

struct MockConn {
    responses: VecDeque<Vec<u8>>,
    fail: bool,
    last_sent: Vec<u8>,
    sent_frames: Vec<Vec<u8>>,
}

impl MockConn {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockConn {
            responses: responses.into(),
            fail: false,
            last_sent: Vec::new(),
            sent_frames: Vec::new(),
        }
    }
    fn failing() -> Self {
        MockConn {
            responses: VecDeque::new(),
            fail: true,
            last_sent: Vec::new(),
            sent_frames: Vec::new(),
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.last_sent = data.to_vec();
        self.sent_frames.push(data.to_vec());
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
fn build_forward_frame_exact_bytes() {
    assert_eq!(build_forward_frame(0x01, 0x90), vec![0x01, 0x90]);
    assert_eq!(build_forward_frame(0xFF, 0xFF), vec![0xFF, 0xFF]);
}

#[test]
fn build_arc_power_sets_addressing_bit() {
    // addr << 1 | 0x01 marks an addressed (non-broadcast) frame
    assert_eq!(build_arc_power(3, 200), vec![0x07, 200]);
    assert_eq!(build_arc_power(0, 128), vec![0x01, 128]);
}

#[test]
fn build_arc_power_clamps_level_to_254() {
    assert_eq!(build_arc_power(0, 255), vec![0x01, 254]);
    assert_eq!(build_arc_power(0, 254), vec![0x01, 254]);
}

#[test]
fn build_query_status_exact_bytes() {
    assert_eq!(build_query_status(3), vec![0x07, 0x90]);
    assert_eq!(build_query_status(0), vec![0x01, 0x90]);
}

#[test]
fn build_off_and_max_exact_bytes() {
    assert_eq!(build_off(), vec![0xFE, 0x00]);
    assert_eq!(build_max(), vec![0xFE, 0x05]);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = DaliProtocol::new(1);
    assert_eq!(p.name(), "dali");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::BuildingLighting);
    assert_eq!(p.supported_variants(), &["iec62386"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_returns_status_byte() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![0x00, 0x64]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0x64);
}

#[test]
fn read_short_response_returns_zero() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![0x00]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![0, 1], vec![0, 2], vec![0, 3]]);
    assert_eq!(p.read_batch(&mut conn, &[0, 1, 2]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![0, 42]; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), vec![]);
}

// ---- ProtocolWrite ----

#[test]
fn write_sends_arc_power_frame() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![]]);
    p.write(&mut conn, &0, &200).unwrap();
    assert_eq!(conn.last_sent, vec![0x07, 200]);
}

#[test]
fn write_batch_sends_multiple_frames() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![], vec![]]);
    p.write_batch(&mut conn, &[(0, 10), (0, 20)]).unwrap();
    assert_eq!(conn.sent_frames.len(), 2);
    assert_eq!(conn.sent_frames[0], vec![0x07, 10]);
    assert_eq!(conn.sent_frames[1], vec![0x07, 20]);
}

#[test]
fn write_connection_failure_propagates() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::failing();
    let err = p.write(&mut conn, &0, &10).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::new(vec![vec![0x00, 0x64]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("status=100"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = DaliProtocol::new(3);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = DaliProtocol::new(6);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "dali:6");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
