// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_asinterface::*;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
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
fn asinterface_data_exchange_exact_bytes() {
    let f = build_data_exchange(1, 0);
    assert_eq!(f, vec![0x00, 0x01, 0x01, 0x00, 0x00]);
    assert_eq!(f.len(), 5);
}

#[test]
fn asinterface_data_exchange_masks_addr_and_data() {
    // address is masked to 5 bits, data to 4 bits
    let f = build_data_exchange(0x20, 0xFF);
    assert_eq!(f, vec![0x00, 0x01, 0x00, 0x0F, 0x00]);
    let f = build_data_exchange(0x1F, 0x0F);
    assert_eq!(f[2], 0x1F);
    assert_eq!(f[3], 0x0F);
}

#[test]
fn asinterface_read_id_exact_bytes() {
    assert_eq!(build_read_id(1), vec![0x00, 0x03, 0x01, 0x0A, 0x00]);
    assert_eq!(build_read_id(0)[2], 0x00);
    assert_eq!(build_read_id(0x1F)[2], 0x1F);
    assert_eq!(build_read_id(0x20)[2], 0x00); // masked to 5 bits
}

// ---------- protocol metadata & builders ----------

#[test]
fn asinterface_metadata() {
    let p = AsiProtocol::new();
    assert_eq!(p.name(), "asinterface");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["v3"]);
}

#[test]
fn asinterface_default_and_slave_builder() {
    let p = AsiProtocol::default();
    assert_eq!(p.slave_addr, 0);
    let p = AsiProtocol::new().slave(7);
    assert_eq!(p.slave_addr, 7);
}

// ---------- ProtocolRead ----------

#[test]
fn asinterface_read_returns_data_nibble() {
    let p = AsiProtocol::new().slave(1);
    let mut conn = MockConn::new(vec![0x00, 0x01, 0x01, 0x0B]);
    let v = ProtocolRead::read(&p, &mut conn, &0).unwrap();
    assert_eq!(v, 0x0B);
    assert_eq!(conn.sent, vec![build_data_exchange(1, 0)]);
}

#[test]
fn asinterface_read_short_response_returns_zero() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::new(vec![0x00, 0x01, 0x01]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0);
}

#[test]
fn asinterface_read_connection_failure() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn asinterface_read_batch_and_range() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::new(vec![0; 4]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(vals, vec![0; 3]);
    let mut conn = MockConn::new(vec![0; 4]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &1, 4).unwrap();
    assert_eq!(vals, vec![0; 4]);
    assert_eq!(conn.sent.len(), 4);
}

#[test]
fn asinterface_read_batch_empty_and_range_zero() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &1, 0).unwrap(),
        Vec::<u8>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn asinterface_read_batch_connection_failure() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &[1]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolWrite ----------

#[test]
fn asinterface_write_sends_data_exchange() {
    let p = AsiProtocol::new().slave(2);
    let mut conn = MockConn::new(vec![0x00]);
    ProtocolWrite::write(&p, &mut conn, &0, &0x0A).unwrap();
    assert_eq!(conn.sent, vec![build_data_exchange(2, 0x0A)]);
}

#[test]
fn asinterface_write_batch_multiple() {
    let p = AsiProtocol::new().slave(3);
    let mut conn = MockConn::new(vec![0x00]);
    ProtocolWrite::write_batch(&p, &mut conn, &[(1, 0x01), (2, 0x02)]).unwrap();
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(conn.sent[0], build_data_exchange(3, 0x01));
    assert_eq!(conn.sent[1], build_data_exchange(3, 0x02));
}

#[test]
fn asinterface_write_connection_failure() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolWrite::write(&p, &mut conn, &0, &0x01).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn asinterface_health_healthy() {
    let p = AsiProtocol::new().slave(4);
    let mut conn = MockConn::new(vec![0x00]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("id"));
    assert_eq!(conn.sent, vec![build_read_id(4)]);
}

#[test]
fn asinterface_health_unhealthy() {
    let p = AsiProtocol::new();
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn asinterface_diagnostics_fields() {
    let p = AsiProtocol::new().slave(6);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "asi:6");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
