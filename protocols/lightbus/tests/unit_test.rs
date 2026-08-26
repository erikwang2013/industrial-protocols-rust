// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_lightbus::*;

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
fn build_read_register_exact_bytes() {
    // [Start 0x02][Addr hi][Addr lo][Cmd 0x03][Len][Reg LE][CRC 0x00][CRC 0x00]
    let f = build_read_register(1, 0x0100, 2);
    assert_eq!(
        f,
        vec![0x02, 0x00, 0x01, 0x03, 0x02, 0x00, 0x01, 0x00, 0x00]
    );
    assert_eq!(f.len(), 9);
}

#[test]
fn build_read_register_max_address() {
    let f = build_read_register(0xFFFF, 0xFFFF, 0xFF);
    assert_eq!(
        f,
        vec![0x02, 0xFF, 0xFF, 0x03, 0xFF, 0xFF, 0xFF, 0x00, 0x00]
    );
}

#[test]
fn build_read_register_len_zero() {
    let f = build_read_register(1, 0, 0);
    assert_eq!(f[4], 0x00);
    assert_eq!(f.len(), 9);
}

#[test]
fn build_write_register_exact_bytes() {
    // [Start][Addr hi][Addr lo][Cmd 0x04][Len][Reg LE][Data][CRC 0x00][CRC 0x00]
    let f = build_write_register(1, 0x0100, &[0xAA]);
    assert_eq!(
        f,
        vec![0x02, 0x00, 0x01, 0x04, 0x01, 0x00, 0x01, 0xAA, 0x00, 0x00]
    );
    assert_eq!(f.len(), 10);
}

#[test]
fn build_write_register_empty_data() {
    let f = build_write_register(1, 0, &[]);
    assert_eq!(
        f,
        vec![0x02, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.len(), 9);
}

#[test]
fn build_write_register_max_length_data() {
    let data = vec![0x11; 255];
    let f = build_write_register(1, 0, &data);
    assert_eq!(f[4], 0xFF);
    assert_eq!(f.len(), 9 + 255);
    assert_eq!(&f[7..7 + 255], &data[..]);
}

// ---- builder ----

#[test]
fn slave_builder_sets_slave_id() {
    assert_eq!(LightbusProtocol::new().slave_id, 1);
    assert_eq!(LightbusProtocol::default().slave_id, 1);
    assert_eq!(LightbusProtocol::new().slave(9).slave_id, 9);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = LightbusProtocol::new();
    assert_eq!(p.name(), "lightbus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::HardwareBridge);
    assert_eq!(p.supported_variants(), &["ring"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_le_u16() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![
        vec![0; 6].into_iter().chain([0x34, 0x12, 0x00]).collect(),
    ]);
    assert_eq!(p.read(&mut conn, &0x100).unwrap(), 0x1234);
}

#[test]
fn read_short_response_returns_zero() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 8]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_sends_read_register_for_address() {
    let p = LightbusProtocol::new().slave(2);
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    p.read(&mut conn, &0x0100).unwrap();
    assert_eq!(conn.sent, vec![build_read_register(2, 0x0100, 2)]);
}

#[test]
fn read_batch_collects_values() {
    let p = LightbusProtocol::new();
    let mk = |v: u16| {
        let mut r = vec![0u8; 9];
        r[6..8].copy_from_slice(&v.to_le_bytes());
        r
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(p.read_batch(&mut conn, &[0, 4, 8]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<u16>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = LightbusProtocol::new();
    let mut v = vec![0u8; 9];
    v[6..8].copy_from_slice(&42u16.to_le_bytes());
    let mut conn = MockConn::new(vec![v; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), Vec::<u16>::new());
}

// ---- ProtocolWrite ----

#[test]
fn write_sends_write_register() {
    let p = LightbusProtocol::new().slave(3);
    let mut conn = MockConn::new(vec![vec![]]);
    p.write(&mut conn, &0x0100, &0xABCD).unwrap();
    assert_eq!(
        conn.sent,
        vec![build_write_register(3, 0x0100, &0xABCDu16.to_le_bytes())]
    );
}

#[test]
fn write_connection_failure_propagates() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.write(&mut conn, &1, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn write_batch_multiple_writes_in_order() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![]; 2]);
    p.write_batch(&mut conn, &[(0x100, 1), (0x200, 2)]).unwrap();
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(
        conn.sent[0],
        build_write_register(1, 0x100, &1u16.to_le_bytes())
    );
    assert_eq!(
        conn.sent[1],
        build_write_register(1, 0x200, &2u16.to_le_bytes())
    );
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("lightbus"));
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = LightbusProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = LightbusProtocol::new().slave(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "lb:5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
