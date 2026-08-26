// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_cpci::*;
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

// ---- frame builders ----

#[test]
fn build_cpci_config_read_exact_bytes() {
    // [Slot][Bus][PCI Bus][Device][Function][Register]
    let f = build_cpci_config_read(3, 1, 2, 4, 0x10);
    assert_eq!(f, vec![3, 1, 1, 2, 4, 0x10]);
    assert_eq!(f.len(), 6);
}

#[test]
fn build_cpci_config_read_all_ff() {
    let f = build_cpci_config_read(0xFF, 0xFF, 0xFF, 0xFF, 0xFF);
    assert_eq!(f, vec![0xFF; 6]);
}

#[test]
fn build_cpci_slot_identify_exact_bytes() {
    // Geographic addressing: slot + 0xFF marker + zeros
    assert_eq!(
        build_cpci_slot_identify(3),
        vec![3, 0xFF, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        build_cpci_slot_identify(0),
        vec![0, 0xFF, 0x00, 0x00, 0x00, 0x00]
    );
}

// ---- builder ----

#[test]
fn slot_builder_sets_slot_and_bus() {
    assert_eq!(CpciProtocol::new().slot, 1);
    assert_eq!(CpciProtocol::new().bus, 0);
    let p = CpciProtocol::new().slot(7);
    assert_eq!(p.slot, 7);
    assert_eq!(CpciProtocol::default().slot, 1);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = CpciProtocol::new();
    assert_eq!(p.name(), "cpci");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::SystemBus);
    assert_eq!(p.supported_variants(), &["3u", "6u"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_le_u32() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![
        vec![0; 6]
            .into_iter()
            .chain([0x78, 0x56, 0x34, 0x12])
            .collect(),
    ]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0x12345678);
}

#[test]
fn read_short_response_returns_zero() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_values() {
    let p = CpciProtocol::new();
    let mk = |v: u32| {
        let mut r = vec![0; 6];
        r.extend_from_slice(&v.to_le_bytes());
        r
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(p.read_batch(&mut conn, &[0, 4, 8]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), vec![]);
}

#[test]
fn read_range_repeats_start_address() {
    let p = CpciProtocol::new();
    let mut v = vec![0; 6];
    v.extend_from_slice(&42u32.to_le_bytes());
    let mut conn = MockConn::new(vec![v; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), vec![]);
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 6]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("identify"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = CpciProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = CpciProtocol::new().slot(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "cpci:slot5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
