// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_worldfip::*;
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
fn worldfip_read_object_exact_bytes() {
    let f = build_read_object(1, 0x0102);
    assert_eq!(f, vec![0x80, 0x04, 0x00, 0x01, 0x00, 0x01, 0x02]);
    assert_eq!(f.len(), 7);
}

#[test]
fn worldfip_read_object_edge_values() {
    let f = build_read_object(0xFF, 0xFFFF);
    assert_eq!(f[3], 0xFF);
    assert_eq!(&f[5..7], &[0xFF, 0xFF]);
    let f = build_read_object(0, 0);
    assert_eq!(f[3], 0x00);
    assert_eq!(&f[5..7], &[0x00, 0x00]);
}

#[test]
fn worldfip_write_object_exact_bytes() {
    let f = build_write_object(1, 0x0102, &[0xDE, 0xAD]);
    assert_eq!(
        f,
        vec![0x80, 0x05, 0x00, 0x01, 0x00, 0x01, 0x02, 0xDE, 0xAD]
    );
}

#[test]
fn worldfip_write_object_empty_data() {
    let f = build_write_object(1, 0x0102, &[]);
    assert_eq!(f, vec![0x80, 0x05, 0x00, 0x01, 0x00, 0x01, 0x02]);
    assert_eq!(f.len(), 7);
}

#[test]
fn worldfip_write_object_appends_data() {
    let data = vec![0x11; 100];
    let f = build_write_object(9, 0x0001, &data);
    assert_eq!(f.len(), 7 + 100);
    assert_eq!(&f[7..], &data[..]);
}

// ---------- protocol metadata & builders ----------

#[test]
fn worldfip_metadata() {
    let p = WorldfipProtocol::new();
    assert_eq!(p.name(), "worldfip");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["fip", "fipio"]);
}

#[test]
fn worldfip_default_and_station_builder() {
    let p = WorldfipProtocol::default();
    assert_eq!(p.station, 1);
    let p = WorldfipProtocol::new().station(4);
    assert_eq!(p.station, 4);
}

// ---------- ProtocolRead ----------

#[test]
fn worldfip_read_returns_response() {
    let p = WorldfipProtocol::new().station(2);
    let mut conn = MockConn::new(vec![0x00, 0xAB]);
    let v = ProtocolRead::read(&p, &mut conn, &0x0102).unwrap();
    assert_eq!(v, vec![0x00, 0xAB]);
    assert_eq!(conn.sent, vec![build_read_object(2, 0x0102)]);
}

#[test]
fn worldfip_read_empty_response() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read(&p, &mut conn, &1).unwrap(),
        Vec::<u8>::new()
    );
}

#[test]
fn worldfip_read_connection_failure() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn worldfip_read_batch_and_range() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::new(vec![0x00]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(vals, vec![vec![0x00]; 3]);
    let mut conn = MockConn::new(vec![0x00]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &1, 4).unwrap();
    assert_eq!(vals, vec![vec![0x00]; 4]);
    assert_eq!(conn.sent.len(), 4);
}

#[test]
fn worldfip_read_batch_empty_and_range_zero() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &1, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn worldfip_read_batch_connection_failure() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &[1]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn worldfip_health_healthy() {
    let p = WorldfipProtocol::new().station(3);
    let mut conn = MockConn::new(vec![0x00]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("read"));
    assert_eq!(conn.sent, vec![build_read_object(3, 0)]);
}

#[test]
fn worldfip_health_unhealthy() {
    let p = WorldfipProtocol::new();
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn worldfip_diagnostics_fields() {
    let p = WorldfipProtocol::new().station(8);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "wfip:8");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
