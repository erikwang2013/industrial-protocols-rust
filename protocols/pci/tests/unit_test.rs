// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_pci::*;

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
fn build_config_read_exact_bytes() {
    // [Bus][Device][Function][Register][Width]
    let f = build_config_read(1, 2, 3, 0x10, 4);
    assert_eq!(f, vec![1, 2, 3, 0x10, 4]);
    assert_eq!(f.len(), 5);
}

#[test]
fn build_config_read_all_max() {
    let f = build_config_read(0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    assert_eq!(f, vec![0xFF; 5]);
}

#[test]
fn build_config_read_width_edge_values() {
    assert_eq!(build_config_read(0, 0, 0, 0, 0), vec![0, 0, 0, 0, 0]);
    assert_eq!(build_config_read(0, 0, 0, 0x3F, 8), vec![0, 0, 0, 0x3F, 8]);
}

#[test]
fn build_mmio_read_exact_bytes() {
    // [Bar][0x80][Offset LE 4][Len LE 4]
    let f = build_mmio_read(0, 0x1000, 4);
    assert_eq!(
        f,
        vec![0, 0x80, 0x00, 0x10, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.len(), 10);
}

#[test]
fn build_mmio_read_max_values() {
    let f = build_mmio_read(0xFF, u32::MAX, u32::MAX);
    assert_eq!(&f[2..6], &u32::MAX.to_le_bytes()[..]);
    assert_eq!(&f[6..10], &u32::MAX.to_le_bytes()[..]);
}

// ---- builder ----

#[test]
fn bdf_builder_sets_fields() {
    assert_eq!(PciProtocol::new().bus, 0);
    assert_eq!(PciProtocol::new().device, 0);
    assert_eq!(PciProtocol::new().function, 0);
    assert_eq!(PciProtocol::default().bus, 0);
    let p = PciProtocol::new().bdf(1, 2, 3);
    assert_eq!((p.bus, p.device, p.function), (1, 2, 3));
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = PciProtocol::new();
    assert_eq!(p.name(), "pci");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::SystemBus);
    assert_eq!(p.supported_variants(), &["pci", "pcie"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_le_u32() {
    // read() builds u32 LE from bytes [5][6][3][4]
    let p = PciProtocol::new().bdf(1, 2, 3);
    let mut resp = vec![0u8; 7];
    resp[3] = 0xAB;
    resp[4] = 0xCD;
    resp[5] = 0x12;
    resp[6] = 0x34;
    let mut conn = MockConn::new(vec![resp]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0xCDAB3412);
    assert_eq!(conn.sent, vec![build_config_read(1, 2, 3, 0, 4)]);
}

#[test]
fn read_short_response_returns_zero() {
    let p = PciProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 6]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = PciProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = PciProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = PciProtocol::new();
    let mk = |v: u32| {
        let b = v.to_le_bytes();
        let mut r = vec![0u8; 7];
        r[3] = b[2];
        r[4] = b[3];
        r[5] = b[0];
        r[6] = b[1];
        r
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(p.read_batch(&mut conn, &[0, 4, 8]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = PciProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<u32>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = PciProtocol::new();
    let mut r = vec![0u8; 7];
    r[3] = 0;
    r[4] = 0;
    r[5] = 42;
    r[6] = 0;
    let mut conn = MockConn::new(vec![r; 3]);
    assert_eq!(p.read_range(&mut conn, &0x10, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = PciProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), Vec::<u32>::new());
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = PciProtocol::new().bdf(1, 2, 3);
    let mut resp = vec![0u8; 7];
    resp[3] = 0x86;
    resp[4] = 0x80;
    resp[5] = 0x01;
    resp[6] = 0x00; // vendor 0x8086
    let mut conn = MockConn::new(vec![resp]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("vendor"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = PciProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = PciProtocol::new().bdf(1, 2, 3);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "pci:01:02.3");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
