// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_canopen::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;

/// Inline mock connection: serves canned responses in order; `fail` mode
/// exercises the io::Error failure path.
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
fn build_nmt_command_exact_bytes() {
    assert_eq!(build_nmt_command(0x01, 2), vec![0x00, 0x01, 0x02]);
    assert_eq!(build_nmt_command(0x80, 0x7F), vec![0x00, 0x80, 0x7F]);
}

#[test]
fn build_sdo_read_exact_bytes() {
    // cob_id = 0x600 + node; index little-endian; subindex; 4 zero bytes
    let f = build_sdo_read(2, 0x1000, 0x00);
    assert_eq!(
        f,
        vec![0x02, 0x40, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.len(), 9);
}

#[test]
fn build_sdo_read_max_node_id() {
    let f = build_sdo_read(0xFF, 0xFFFF, 0xFF);
    assert_eq!(f[0], 0xFF); // 0x600 + 0xFF = 0x6FF, low byte 0xFF
    assert_eq!(f[2], 0xFF);
    assert_eq!(f[3], 0xFF);
    assert_eq!(f[4], 0xFF);
}

#[test]
fn build_sdo_read_index_endianness() {
    let f = build_sdo_read(1, 0x1234, 0x56);
    assert_eq!(f[2], 0x34);
    assert_eq!(f[3], 0x12);
    assert_eq!(f[4], 0x56);
}

// ---- parse_sdo_response ----

fn sdo_resp(cmd: u8, data: &[u8; 4]) -> Vec<u8> {
    let mut v = vec![0x00, cmd, 0x00, 0x00];
    v.extend_from_slice(data);
    v
}

#[test]
fn parse_sdo_response_u32() {
    let r = parse_sdo_response(&sdo_resp(0x43, &[0x78, 0x56, 0x34, 0x12])).unwrap();
    assert_eq!(r, 0x12345678);
}

#[test]
fn parse_sdo_response_u16() {
    let r = parse_sdo_response(&sdo_resp(0x4B, &[0x34, 0x12, 0x00, 0x00])).unwrap();
    assert_eq!(r, 0x1234);
}

#[test]
fn parse_sdo_response_u8() {
    let r = parse_sdo_response(&sdo_resp(0x4F, &[0x2A, 0x00, 0x00, 0x00])).unwrap();
    assert_eq!(r, 0x2A);
}

#[test]
fn parse_sdo_response_too_short() {
    assert!(parse_sdo_response(&[0x00, 0x43, 0x00]).is_err());
    assert!(parse_sdo_response(&[]).is_err());
}

#[test]
fn parse_sdo_response_abort() {
    let r = parse_sdo_response(&sdo_resp(0x80, &[0x01, 0x02, 0x03, 0x04]));
    assert!(r.is_err());
    assert!(r.unwrap_err().contains("SDO abort: 01020304"));
}

#[test]
fn parse_sdo_response_unexpected_cmd() {
    assert!(parse_sdo_response(&sdo_resp(0x60, &[0; 4])).is_err());
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = CanopenProtocol::new(3);
    assert_eq!(p.name(), "canopen");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["cia301"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_u32_value() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![sdo_resp(0x43, &[0x00, 0x00, 0x80, 0x3F])]);
    let v = p.read(&mut conn, &0x1000).unwrap();
    assert_eq!(v, 0x3F800000);
}

#[test]
fn read_rejects_short_response() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![vec![0x00, 0x43]]);
    let err = p.read(&mut conn, &0x1000).unwrap_err();
    assert!(matches!(err, ProtocolError::Parse { .. }));
}

#[test]
fn read_connection_failure_propagates() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0x1000).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![
        sdo_resp(0x43, &[0x01, 0x00, 0x00, 0x00]),
        sdo_resp(0x43, &[0x02, 0x00, 0x00, 0x00]),
        sdo_resp(0x43, &[0x03, 0x00, 0x00, 0x00]),
    ]);
    let v = p.read_batch(&mut conn, &[0x1000, 0x1001, 0x1002]).unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![sdo_resp(0x43, &[0x2A, 0, 0, 0]); 4]);
    let v = p.read_range(&mut conn, &0x2000, 4).unwrap();
    assert_eq!(v, vec![42, 42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0x2000, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::new(vec![vec![0x00, 0x00, 0x00]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("NMT"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = CanopenProtocol::new(2);
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("conn down"));
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = CanopenProtocol::new(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "canopen:5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
