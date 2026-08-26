// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_sercos::*;
use std::io;

/// Mock connection with a canned response and a failure mode.
struct MockConn {
    response: Vec<u8>,
    fail: bool,
    sent: Vec<Vec<u8>>,
}

impl MockConn {
    fn new(response: Vec<u8>) -> Self {
        MockConn {
            response,
            fail: false,
            sent: Vec::new(),
        }
    }
    fn failing() -> Self {
        MockConn {
            response: Vec::new(),
            fail: true,
            sent: Vec::new(),
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "mock failure"));
        }
        self.sent.push(data.to_vec());
        Ok(self.response.clone())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

// ---------- frame builders ----------

#[test]
fn sercos_mdt_phase_up_exact_bytes() {
    let f = build_mdt_phase_up(4);
    assert_eq!(f, vec![0x80, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(f.len(), 8);
}

#[test]
fn sercos_mdt_phase_up_edge_phases() {
    assert_eq!(build_mdt_phase_up(0)[2], 0x00);
    assert_eq!(build_mdt_phase_up(0xFF)[2], 0xFF);
}

#[test]
fn sercos_service_channel_read_exact_bytes() {
    let f = build_service_channel_read(0x0102, 0x1234, 3);
    assert_eq!(f, vec![0x00, 0x01, 0x02, 0x12, 0x34, 0x03, 0x80, 0x00]);
}

#[test]
fn sercos_service_channel_read_big_endian() {
    // slave and IDN are big-endian in the frame
    let f = build_service_channel_read(0x0100, 0x0001, 0);
    assert_eq!(&f[1..3], &[0x01, 0x00]);
    assert_eq!(&f[3..5], &[0x00, 0x01]);
    // max values
    let f = build_service_channel_read(0xFFFF, 0xFFFF, 0xFF);
    assert_eq!(&f[1..3], &[0xFF, 0xFF]);
    assert_eq!(f[5], 0xFF);
    assert_eq!(f.len(), 8);
}

// ---------- protocol metadata ----------

#[test]
fn sercos_metadata() {
    let p = SercosProtocol::new(1);
    assert_eq!(p.name(), "sercos");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["iii"]);
    assert_eq!(p.slave_id, 1);
}

// ---------- ProtocolRead ----------

#[test]
fn sercos_read_parses_u32_response() {
    let p = SercosProtocol::new(0x0102);
    let mut conn = MockConn::new(vec![0x00, 0x01, 0x02, 0x00, 0x00, 0x00, 0x01, 0x2C]);
    let v = ProtocolRead::read(&p, &mut conn, &0x1234).unwrap();
    assert_eq!(v, 0x12C);
    // sent frame targets slave 0x0102, idn 0x1234
    assert_eq!(
        conn.sent,
        vec![build_service_channel_read(0x0102, 0x1234, 0)]
    );
}

#[test]
fn sercos_read_short_response_returns_zero() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::new(vec![0x00, 0x01, 0x02]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &1).unwrap(), 0);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &1).unwrap(), 0);
}

#[test]
fn sercos_read_connection_failure() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn sercos_read_batch_and_range() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::new(vec![0; 8]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(vals, vec![0; 3]);
    let mut conn = MockConn::new(vec![0; 8]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &1, 4).unwrap();
    assert_eq!(vals, vec![0; 4]);
    assert_eq!(conn.sent.len(), 4);
}

#[test]
fn sercos_read_batch_empty_and_range_zero() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<u32>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &1, 0).unwrap(),
        Vec::<u32>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn sercos_read_batch_connection_failure() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &[1]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn sercos_health_healthy() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::new(vec![0x00; 8]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("phase"));
    assert_eq!(conn.sent, vec![build_mdt_phase_up(4)]);
}

#[test]
fn sercos_health_unhealthy() {
    let p = SercosProtocol::new(1);
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn sercos_diagnostics_fields() {
    let p = SercosProtocol::new(0x0100);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "sercos:256");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
