// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_kline::*;

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
fn build_fast_init_exact_bytes() {
    assert_eq!(build_fast_init(), vec![0x33]);
}

#[test]
fn build_read_dtc_exact_bytes() {
    // ReadDiagnosticTroubleCodes request
    assert_eq!(build_read_dtc(), vec![0x68, 0x6A, 0xF1, 0x01, 0x07, 0xC3]);
}

// ---- KWP2000 response parser ----

#[test]
fn parse_kwp2000_happy_path() {
    // [Format 0x48][Len 4][Filler][payload...] -> payload after 3-byte header
    let resp = vec![0x48, 0x04, 0xF1, 0x01, 0x02, 0x03];
    assert_eq!(parse_kwp2000_response(&resp).unwrap(), vec![0x01, 0x02]);
}

#[test]
fn parse_kwp2000_empty_input_errors() {
    assert!(parse_kwp2000_response(&[]).is_err());
    assert!(parse_kwp2000_response(&[0x48, 0x04]).is_err());
}

#[test]
fn parse_kwp2000_wrong_format_byte_errors() {
    let resp = vec![0x49, 0x04, 0xF1, 0x01, 0x02, 0x03];
    let err = parse_kwp2000_response(&resp).unwrap_err();
    assert!(err.contains("format"));
}

#[test]
fn parse_kwp2000_truncated_errors() {
    let resp = vec![0x48, 0x08, 0xF1, 0x01];
    let err = parse_kwp2000_response(&resp).unwrap_err();
    assert!(err.contains("truncated"));
}

#[test]
fn parse_kwp2000_short_length_does_not_panic() {
    // len < 2 would panic on data[3..len+1]; parser must return an error-free result
    assert!(parse_kwp2000_response(&[0x48, 0x00, 0xF1, 0x00]).is_ok());
    assert!(parse_kwp2000_response(&[0x48, 0x01, 0xF1, 0x00]).is_ok());
}

#[test]
fn parse_kwp2000_boundary_length() {
    // len == 2 -> empty payload
    let resp = vec![0x48, 0x02, 0xF1, 0x00];
    assert_eq!(parse_kwp2000_response(&resp).unwrap(), Vec::<u8>::new());
}

// ---- builder ----

#[test]
fn new_sets_addr() {
    assert_eq!(KlineProtocol::new(0x33).addr, 0x33);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = KlineProtocol::new(0x33);
    assert_eq!(p.name(), "kline");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::AutomotiveBus);
    assert_eq!(p.supported_variants(), &["iso9141", "iso14230"]);
}

// ---- ProtocolRead ----

#[test]
fn read_sends_init_then_dtc_and_parses() {
    let p = KlineProtocol::new(0x33);
    let resp = vec![0x48, 0x04, 0xF1, 0x01, 0x02, 0x03];
    let mut conn = MockConn::new(vec![vec![0x33], resp]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), vec![0x01, 0x02]);
    assert_eq!(conn.sent, vec![build_fast_init(), build_read_dtc()]);
}

#[test]
fn read_parse_error_on_short_response() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![vec![0x33], vec![]]);
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Parse { .. }));
}

#[test]
fn read_parse_error_on_bad_format_byte() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![vec![0x33], vec![0x49, 0x04, 0xF1, 0x01]]);
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Parse { .. }));
}

#[test]
fn read_connection_failure_propagates() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = KlineProtocol::new(0x33);
    let resp = vec![0x48, 0x04, 0xF1, 0x0A, 0x0B, 0x00];
    let mut conn = MockConn::new(vec![vec![0x33], resp.clone(), vec![0x33], resp.clone()]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 1]).unwrap(),
        vec![vec![0x0A, 0x0B], vec![0x0A, 0x0B]]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = KlineProtocol::new(0x33);
    let resp = vec![0x48, 0x04, 0xF1, 0x01, 0x02, 0x00];
    let mut conn = MockConn::new(vec![vec![0x33], resp.clone(), vec![0x33], resp.clone()]);
    assert_eq!(
        p.read_range(&mut conn, &0, 2).unwrap(),
        vec![vec![0x01, 0x02], vec![0x01, 0x02]]
    );
}

#[test]
fn read_range_zero_count() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![vec![0x33]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("kline"));
    assert_eq!(conn.sent, vec![build_fast_init()]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = KlineProtocol::new(0x33);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "kline:51");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
