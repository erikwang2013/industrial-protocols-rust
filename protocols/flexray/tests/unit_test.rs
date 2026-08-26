// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_flexray::*;
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
fn build_static_frame_exact_bytes() {
    // Header 5B: status + frame ID BE + length + CRC placeholder, then payload
    let f = build_static_frame(7, 0, &[1, 2, 3]);
    assert_eq!(f, vec![0x00, 0x00, 0x07, 0x03, 0x00, 0x01, 0x02, 0x03]);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_static_frame_empty_payload() {
    let f = build_static_frame(1, 0, &[]);
    assert_eq!(f, vec![0x00, 0x00, 0x01, 0x00, 0x00]);
    assert_eq!(f.len(), 5);
}

#[test]
fn build_static_frame_max_payload() {
    let payload = vec![0xA5; 254];
    let f = build_static_frame(0xFFFF, 0xFF, &payload);
    assert_eq!(f.len(), 5 + 254);
    assert_eq!(f[3], 254); // length byte
    assert_eq!(&f[5..], &payload[..]);
}

#[test]
fn build_static_frame_frame_id_be() {
    let f = build_static_frame(0x1234, 0, &[]);
    assert_eq!(f[1], 0x12);
    assert_eq!(f[2], 0x34);
}

#[test]
fn build_wakeup_pattern_exact_bytes() {
    assert_eq!(build_wakeup_pattern(), vec![0xFF, 0x00, 0xFF, 0x00]);
}

// ---- builder ----

#[test]
fn slot_builder_sets_slot() {
    assert_eq!(FlexrayProtocol::new().slot, 1);
    assert_eq!(FlexrayProtocol::new().cycle, 0);
    assert_eq!(FlexrayProtocol::new().slot(9).slot, 9);
    assert_eq!(FlexrayProtocol::default().slot, 1);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = FlexrayProtocol::new();
    assert_eq!(p.name(), "flexray");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::AutomotiveBus);
    assert_eq!(p.supported_variants(), &["dual"]);
}

// ---- ProtocolRead ----

#[test]
fn read_returns_response_verbatim() {
    let p = FlexrayProtocol::new();
    let canned = vec![0x00, 0x00, 0x01, 0x02, 0x00, 0xDE, 0xAD];
    let mut conn = MockConn::new(vec![canned.clone()]);
    assert_eq!(p.read(&mut conn, &1).unwrap(), canned);
}

#[test]
fn read_empty_response_ok() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &1).unwrap(), vec![]);
}

#[test]
fn read_connection_failure_propagates() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_batch_collects_responses() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![vec![1], vec![2], vec![3]]);
    let v = p.read_batch(&mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(v, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn read_batch_empty_input() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<Vec<u8>>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![vec![0x2A]; 3]);
    let v = p.read_range(&mut conn, &1, 3).unwrap();
    assert_eq!(v, vec![vec![0x2A], vec![0x2A], vec![0x2A]]);
}

#[test]
fn read_range_zero_count() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        p.read_range(&mut conn, &1, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 4]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("wakeup"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = FlexrayProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = FlexrayProtocol::new().slot(3);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "flexray:3");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
