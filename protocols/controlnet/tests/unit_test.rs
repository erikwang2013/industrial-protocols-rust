// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_controlnet::*;
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
fn build_ucmm_read_exact_bytes() {
    // 16-byte preamble (0xAA) + start delimiter (0xAB) + source + service
    // + class/instance/attr (LE) + CRC placeholder (2x 0x00) = 27 bytes
    let f = build_ucmm_read(1, 0x0001, 0x0002, 0x0003);
    assert_eq!(f.len(), 27);
    assert!(f[..16].iter().all(|&b| b == 0xAA));
    assert_eq!(f[16], 0xAB);
    assert_eq!(f[17], 0x01);
    assert_eq!(f[18], 0x52);
    assert_eq!(&f[19..21], &[0x01, 0x00]);
    assert_eq!(&f[21..23], &[0x02, 0x00]);
    assert_eq!(&f[23..25], &[0x03, 0x00]);
    assert_eq!(&f[25..27], &[0x00, 0x00]);
}

#[test]
fn build_ucmm_read_fields_le() {
    let f = build_ucmm_read(0x7F, 0x1234, 0x5678, 0x9ABC);
    assert_eq!(f[17], 0x7F);
    assert_eq!(&f[19..21], &[0x34, 0x12]);
    assert_eq!(&f[21..23], &[0x78, 0x56]);
    assert_eq!(&f[23..25], &[0xBC, 0x9A]);
}

// ---- builder ----

#[test]
fn node_builder_sets_node() {
    assert_eq!(ControlnetProtocol::new().node, 1);
    assert_eq!(ControlnetProtocol::new().node(42).node, 42);
    assert_eq!(ControlnetProtocol::default().node, 1);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = ControlnetProtocol::new();
    assert_eq!(p.name(), "controlnet");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["1784-pcics"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_le_word() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 22].into_iter().chain([0x34, 0x12]).collect()]);
    assert_eq!(p.read(&mut conn, &0x01).unwrap(), 0x1234);
}

#[test]
fn read_short_response_returns_zero() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 23]]);
    assert_eq!(p.read(&mut conn, &0x01).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0x01).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0x01).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = ControlnetProtocol::new();
    let mk = |lo: u8| {
        let mut v = vec![0; 22];
        v.extend_from_slice(&[lo, 0x00]);
        v
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(p.read_batch(&mut conn, &[1, 2, 3]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = ControlnetProtocol::new();
    let mut v = vec![0; 22];
    v.extend_from_slice(&[0x2A, 0x00]);
    let mut conn = MockConn::new(vec![v; 3]);
    assert_eq!(p.read_range(&mut conn, &1, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &1, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 26]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("ucmm"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = ControlnetProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = ControlnetProtocol::new().node(4);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "cn:node4");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
