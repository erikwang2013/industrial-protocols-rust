// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_lin::*;

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
fn build_header_exact_pid() {
    // id 0x10: P0=1 (bit6), P1=0 (bit7 clear) -> PID 0x50
    assert_eq!(build_header(0x10), vec![0x55, 0x50]);
}

#[test]
fn build_header_both_parity_bits_set() {
    // id 0x01: P0 = 1^0^0^0 = 1, P1 = ~(0^0^0^0) = 1 -> PID 0xC1
    assert_eq!(build_header(0x01), vec![0x55, 0xC1]);
}

#[test]
fn build_header_id_max_six_bits() {
    // id 0x3F: P0 = 0, P1 = ~(1^1^1^1) = 1 -> PID 0xBF
    assert_eq!(build_header(0x3F), vec![0x55, 0xBF]);
}

#[test]
fn build_header_masks_upper_bits() {
    // id 0xFC masks to 0x3C with both parity bits clear -> PID 0x3C
    assert_eq!(build_header(0xFC), vec![0x55, 0x3C]);
}

#[test]
fn build_frame_checksum_twos_complement() {
    // data sum 6, id 0x10 < 0x3C (classic): checksum = !6 + 1 = 0xFA
    assert_eq!(
        build_frame(0x10, &[1, 2, 3]),
        vec![0x55, 0x50, 1, 2, 3, 0xFA]
    );
}

#[test]
fn build_frame_enhanced_checksum_includes_id() {
    // id 0x3C >= 0x3C (enhanced): checksum = 1 + 0x3C, negated = 0xC3
    assert_eq!(
        build_frame(0x3C, &[0x00, 0x01]),
        vec![0x55, 0x3C, 0x00, 0x01, 0xC3]
    );
}

#[test]
fn build_frame_empty_data() {
    // empty data: checksum = 0 -> !0 + 1 = 0x00
    assert_eq!(build_frame(0x10, &[]), vec![0x55, 0x50, 0x00]);
}

#[test]
fn build_frame_checksum_wraps() {
    // sum 0xFF + 0xFF = 0x1FE -> 0xFE, negated = 0x02
    assert_eq!(
        build_frame(0x10, &[0xFF, 0xFF]),
        vec![0x55, 0x50, 0xFF, 0xFF, 0x02]
    );
}

// ---- builder ----

#[test]
fn new_sets_id() {
    assert_eq!(LinProtocol::new(0x10).id, 0x10);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = LinProtocol::new(0x10);
    assert_eq!(p.name(), "lin");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::AutomotiveBus);
    assert_eq!(p.supported_variants(), &["uart"]);
}

// ---- ProtocolRead ----

#[test]
fn read_sends_empty_frame_and_returns_response() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![vec![1, 2, 3]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), vec![1, 2, 3]);
    assert_eq!(conn.sent, vec![build_frame(0x10, &[])]);
}

#[test]
fn read_empty_response_ok() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), Vec::<u8>::new());
}

#[test]
fn read_connection_failure_propagates() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![vec![1], vec![2], vec![3]]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 1, 2]).unwrap(),
        vec![vec![1], vec![2], vec![3]]
    );
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn read_batch_empty_input() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![vec![7]; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![vec![7]; 3]);
}

#[test]
fn read_range_zero_count() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![vec![]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("lin"));
    assert_eq!(conn.sent, vec![build_header(0x10)]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = LinProtocol::new(0x10);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "lin:16");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
