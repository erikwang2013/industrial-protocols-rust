// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_lonworks::*;

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
fn build_network_var_fetch_exact_bytes() {
    // [Ver 0x01][PDU 0x12][Subnet][Node][0x00][NV hi][NV lo][0x61]
    let f = build_network_var_fetch(1, 2, 0x0102);
    assert_eq!(f, vec![0x01, 0x12, 0x01, 0x02, 0x00, 0x01, 0x02, 0x61]);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_network_var_fetch_zero_nv() {
    let f = build_network_var_fetch(0, 0, 0);
    assert_eq!(f, vec![0x01, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61]);
}

#[test]
fn build_network_var_fetch_max_nv() {
    let f = build_network_var_fetch(0xFF, 0xFF, 0xFFFF);
    assert_eq!(f, vec![0x01, 0x12, 0xFF, 0xFF, 0x00, 0xFF, 0xFF, 0x61]);
}

#[test]
fn build_service_pin_message_exact_bytes() {
    // [Ver][PDU 0x92][0x00][NodeID 8B BE]
    let f = build_service_pin_message(0x0102030405060708);
    assert_eq!(
        f,
        vec![
            0x01, 0x92, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
        ]
    );
    assert_eq!(f.len(), 11);
}

#[test]
fn build_service_pin_message_zero_id() {
    let f = build_service_pin_message(0);
    assert_eq!(f, vec![0x01, 0x92, 0x00, 0, 0, 0, 0, 0, 0, 0, 0]);
}

// ---- builder ----

#[test]
fn subnet_builder_sets_fields() {
    assert_eq!(LonworksProtocol::new().subnet, 1);
    assert_eq!(LonworksProtocol::new().node, 1);
    assert_eq!(LonworksProtocol::default().subnet, 1);
    assert_eq!(LonworksProtocol::new().subnet(7).subnet, 7);
    assert_eq!(LonworksProtocol::new().subnet(7).node, 1);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = LonworksProtocol::new();
    assert_eq!(p.name(), "lonworks");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::BuildingLighting);
    assert_eq!(p.supported_variants(), &["ftt10", "tp1250"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_float() {
    // read() builds f32 BE from bytes [8][9][7][6]; 1.0f32 = 0x3F800000
    let mut resp = vec![0u8; 10];
    resp[6] = 0x00;
    resp[7] = 0x00;
    resp[8] = 0x3F;
    resp[9] = 0x80;
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![resp]);
    assert_eq!(p.read(&mut conn, &0x100).unwrap(), 1.0);
}

#[test]
fn read_short_response_returns_zero() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0.0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0.0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_sends_network_var_fetch_for_address() {
    let p = LonworksProtocol::new().subnet(3);
    let mut conn = MockConn::new(vec![vec![0; 10]]);
    p.read(&mut conn, &0x0102).unwrap();
    assert_eq!(conn.sent, vec![build_network_var_fetch(3, 1, 0x0102)]);
}

#[test]
fn read_batch_collects_values() {
    let p = LonworksProtocol::new();
    let mk = |v: f32| {
        let mut r = vec![0u8; 10];
        let b = v.to_be_bytes();
        r[6] = b[2];
        r[7] = b[3];
        r[8] = b[0];
        r[9] = b[1];
        r
    };
    let mut conn = MockConn::new(vec![mk(1.5), mk(2.5), mk(3.5)]);
    assert_eq!(
        p.read_batch(&mut conn, &[0, 4, 8]).unwrap(),
        vec![1.5, 2.5, 3.5]
    );
}

#[test]
fn read_batch_empty_input() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<f32>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = LonworksProtocol::new();
    let mut r = vec![0u8; 10];
    r[8] = 0x40; // 2.0f32 = 0x40000000
    let mut conn = MockConn::new(vec![r; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![2.0, 2.0, 2.0]);
}

#[test]
fn read_range_zero_count() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), Vec::<f32>::new());
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("lonworks"));
    assert_eq!(conn.sent, vec![build_service_pin_message(0x010203040506)]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = LonworksProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = LonworksProtocol::new().subnet(2);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "lon:2/1");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
