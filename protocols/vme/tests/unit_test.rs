// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_vme::*;
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
fn vme_read_cycle_exact_bytes_per_space() {
    // A16 read: AM 0x29
    assert_eq!(
        build_read_cycle(VmeSpace::A16, 0x0100, 2),
        vec![0x29, 0x02, 0x00, 0x00, 0x01, 0x00]
    );
    // A24 read: AM 0x39
    assert_eq!(
        build_read_cycle(VmeSpace::A24, 0x010000, 4),
        vec![0x39, 0x04, 0x00, 0x01, 0x00, 0x00]
    );
    // A32 read: AM 0x09
    assert_eq!(
        build_read_cycle(VmeSpace::A32, 0x10000000, 1),
        vec![0x09, 0x01, 0x10, 0x00, 0x00, 0x00]
    );
}

#[test]
fn vme_read_cycle_zero_addr_and_width() {
    let f = build_read_cycle(VmeSpace::A16, 0, 0);
    assert_eq!(f, vec![0x29, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(f.len(), 6);
}

#[test]
fn vme_write_cycle_exact_bytes_per_space() {
    // A16 write: AM 0x2A
    assert_eq!(
        build_write_cycle(VmeSpace::A16, 0x0100, &[0xAA]),
        vec![0x2A, 0x01, 0x00, 0x00, 0x01, 0x00, 0xAA]
    );
    // A24 write: AM 0x3A
    assert_eq!(
        build_write_cycle(VmeSpace::A24, 0x010000, &[0x01, 0x02]),
        vec![0x3A, 0x02, 0x00, 0x01, 0x00, 0x00, 0x01, 0x02]
    );
    // A32 write: AM 0x0A
    assert_eq!(
        build_write_cycle(VmeSpace::A32, 0x10000000, &[0xFF]),
        vec![0x0A, 0x01, 0x10, 0x00, 0x00, 0x00, 0xFF]
    );
}

#[test]
fn vme_write_cycle_empty_data() {
    let f = build_write_cycle(VmeSpace::A16, 0, &[]);
    assert_eq!(f, vec![0x2A, 0x00, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(f.len(), 6);
}

#[test]
fn vme_write_cycle_length_byte_is_data_len() {
    let data = vec![0x11; 200];
    let f = build_write_cycle(VmeSpace::A32, 0, &data);
    assert_eq!(f[1], 200);
    assert_eq!(f.len(), 6 + 200);
    assert_eq!(&f[6..], &data[..]);
}

// ---------- protocol metadata & builders ----------

#[test]
fn vme_metadata() {
    let p = VmeProtocol::new();
    assert_eq!(p.name(), "vme");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::SystemBus);
    assert_eq!(p.supported_variants(), &["vme64", "vpx"]);
}

#[test]
fn vme_default_and_slot_builder() {
    let p = VmeProtocol::default();
    assert_eq!(p.slot, 1);
    assert!(matches!(p.address_space, VmeSpace::A16));
    let p = VmeProtocol::new().slot(3);
    assert_eq!(p.slot, 3);
}

// ---------- ProtocolRead ----------

#[test]
fn vme_read_parses_u32_response() {
    let p = VmeProtocol::new().slot(1);
    let mut conn = MockConn::new(vec![0x00, 0x00, 0x12, 0x34, 0x56, 0x78]);
    let v = ProtocolRead::read(&p, &mut conn, &0x0100).unwrap();
    assert_eq!(v, 0x12345678);
    // frame: A16 read of 0x0100, width 4
    assert_eq!(conn.sent, vec![build_read_cycle(VmeSpace::A16, 0x0100, 4)]);
}

#[test]
fn vme_read_short_response_returns_zero() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![0x00, 0x01]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0);
}

#[test]
fn vme_read_connection_failure() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn vme_read_batch_and_range() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![0; 6]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0x100, 0x200]).unwrap();
    assert_eq!(vals, vec![0; 2]);
    let mut conn = MockConn::new(vec![0; 6]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &0x100, 3).unwrap();
    assert_eq!(vals, vec![0; 3]);
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn vme_read_batch_empty_and_range_zero() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<u32>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &0, 0).unwrap(),
        Vec::<u32>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn vme_read_batch_connection_failure() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &[0]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolWrite ----------

#[test]
fn vme_write_sends_write_cycle() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![]);
    ProtocolWrite::write(&p, &mut conn, &0x0100u32, &0xCAFEBABEu32).unwrap();
    assert_eq!(
        conn.sent,
        vec![build_write_cycle(
            VmeSpace::A16,
            0x0100,
            &0xCAFEBABEu32.to_be_bytes()
        )]
    );
}

#[test]
fn vme_write_batch_multiple() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![]);
    ProtocolWrite::write_batch(&p, &mut conn, &[(0x100, 1), (0x200, 2)]).unwrap();
    assert_eq!(conn.sent.len(), 2);
}

#[test]
fn vme_write_connection_failure() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolWrite::write(&p, &mut conn, &0, &0).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn vme_health_healthy() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::new(vec![0; 6]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("vme addr0"));
}

#[test]
fn vme_health_unhealthy() {
    let p = VmeProtocol::new();
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn vme_diagnostics_fields() {
    let p = VmeProtocol::new().slot(5);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "vme:slot5");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
