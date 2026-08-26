// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_cclinkie::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

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

// ---- frame builder ----

#[test]
fn build_cyclic_data_request_exact_bytes() {
    assert_eq!(
        build_cyclic_data_request(1),
        vec![0x03, 0x00, 0x00, 0x01, 0x10, 0x00, 0x01, 0x00]
    );
}

#[test]
fn build_cyclic_data_request_station_be() {
    // station encoded big-endian at bytes 2..4
    let f = build_cyclic_data_request(0x1234);
    assert_eq!(f[2], 0x12);
    assert_eq!(f[3], 0x34);
}

#[test]
fn build_cyclic_data_request_max_station() {
    let f = build_cyclic_data_request(0xFFFF);
    assert_eq!(f[2], 0xFF);
    assert_eq!(f[3], 0xFF);
    assert_eq!(f.len(), 8);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = CclinkIeProtocol::new(1);
    assert_eq!(p.name(), "cclinkie");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["field"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_be_word() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0; 8].into_iter().chain([0x12, 0x34]).collect()]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0x1234);
}

#[test]
fn read_short_response_returns_zero() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = CclinkIeProtocol::new(1);
    let mk = |hi: u8, lo: u8| {
        let mut v = vec![0; 8];
        v.extend_from_slice(&[hi, lo]);
        v
    };
    let mut conn = MockConn::new(vec![mk(0, 1), mk(0, 2), mk(0, 3)]);
    assert_eq!(p.read_batch(&mut conn, &[1, 2, 3]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = CclinkIeProtocol::new(1);
    let mut v = vec![0; 8];
    v.extend_from_slice(&[0x00, 0x2A]);
    let mut conn = MockConn::new(vec![v; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::new(vec![vec![0; 10]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("ok"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = CclinkIeProtocol::new(1);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = CclinkIeProtocol::new(9);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "cclinkie:9");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
