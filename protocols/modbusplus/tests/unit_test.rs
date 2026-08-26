// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::collections::VecDeque;
use std::io;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_modbusplus::*;

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
fn build_read_request_exact_bytes() {
    // [Preamble 0xFF][SD 0x68][Dest][Src][LLC 0x03][Ref hi][Ref lo][Cnt hi][Cnt lo][CRC 0x00][CRC 0x00]
    let f = build_read_request(1, 2, 0x0102, 3);
    assert_eq!(
        f,
        vec![
            0xFF, 0x68, 0x02, 0x01, 0x03, 0x01, 0x02, 0x00, 0x03, 0x00, 0x00
        ]
    );
    assert_eq!(f.len(), 11);
}

#[test]
fn build_read_request_max_values() {
    let f = build_read_request(0x7F, 0x80, 0xFFFF, 0xFFFF);
    assert_eq!(
        f,
        vec![
            0xFF, 0x68, 0x80, 0x7F, 0x03, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00
        ]
    );
}

#[test]
fn build_read_request_dest_wraps_on_max_station() {
    // dest = station + 1; station 0xFF wraps to dest 0x00
    let f = build_read_request(0xFF, 0, 0, 1);
    assert_eq!(f[2], 0x00);
    assert_eq!(f[3], 0xFF);
}

#[test]
fn build_global_data_read_exact_bytes() {
    let f = build_global_data_read(1);
    assert_eq!(f, vec![0xFF, 0x68, 0xFF, 0x01, 0x80, 0x00, 0x00, 0x00]);
    assert_eq!(f.len(), 8);
}

#[test]
fn build_global_data_read_max_station() {
    let f = build_global_data_read(0xFF);
    assert_eq!(f, vec![0xFF, 0x68, 0xFF, 0xFF, 0x80, 0x00, 0x00, 0x00]);
}

// ---- builder ----

#[test]
fn station_builder_sets_station() {
    assert_eq!(ModbusplusProtocol::new().station, 1);
    assert_eq!(ModbusplusProtocol::default().station, 1);
    assert_eq!(ModbusplusProtocol::new().station(7).station, 7);
}

// ---- protocol metadata ----

#[test]
fn protocol_metadata_fields() {
    let p = ModbusplusProtocol::new();
    assert_eq!(p.name(), "modbusplus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::HardwareBridge);
    assert_eq!(p.supported_variants(), &["sa85", "bm85"]);
}

// ---- ProtocolRead ----

#[test]
fn read_ok_parses_be_u16() {
    let p = ModbusplusProtocol::new();
    let mut resp = vec![0u8; 10];
    resp[6] = 0x12;
    resp[7] = 0x34;
    let mut conn = MockConn::new(vec![resp]);
    assert_eq!(p.read(&mut conn, &0x100).unwrap(), 0x1234);
}

#[test]
fn read_short_response_returns_zero() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::new(vec![vec![0; 9]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_empty_response_returns_zero() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::new(vec![vec![]]);
    assert_eq!(p.read(&mut conn, &0).unwrap(), 0);
}

#[test]
fn read_connection_failure_propagates() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::failing();
    let err = p.read(&mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn read_sends_read_request_for_address() {
    let p = ModbusplusProtocol::new().station(3);
    let mut conn = MockConn::new(vec![vec![0; 10]]);
    p.read(&mut conn, &0x0102).unwrap();
    assert_eq!(conn.sent, vec![build_read_request(3, 4, 0x0102, 1)]);
}

#[test]
fn read_batch_collects_values() {
    let p = ModbusplusProtocol::new();
    let mk = |v: u16| {
        let mut r = vec![0u8; 10];
        r[6] = (v >> 8) as u8;
        r[7] = v as u8;
        r
    };
    let mut conn = MockConn::new(vec![mk(1), mk(2), mk(3)]);
    assert_eq!(p.read_batch(&mut conn, &[0, 4, 8]).unwrap(), vec![1, 2, 3]);
}

#[test]
fn read_batch_empty_input() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_batch(&mut conn, &[]).unwrap(), Vec::<u16>::new());
}

#[test]
fn read_range_repeats_start_address() {
    let p = ModbusplusProtocol::new();
    let mut r = vec![0u8; 10];
    r[6] = 0x00;
    r[7] = 42;
    let mut conn = MockConn::new(vec![r; 3]);
    assert_eq!(p.read_range(&mut conn, &0, 3).unwrap(), vec![42, 42, 42]);
}

#[test]
fn read_range_zero_count() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(p.read_range(&mut conn, &0, 0).unwrap(), Vec::<u16>::new());
}

// ---- ProtocolHealth ----

#[test]
fn health_healthy_on_success() {
    let p = ModbusplusProtocol::new().station(4);
    let mut conn = MockConn::new(vec![vec![]]);
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("modbus+"));
    assert_eq!(conn.sent, vec![build_global_data_read(4)]);
}

#[test]
fn health_unhealthy_on_connection_failure() {
    let p = ModbusplusProtocol::new();
    let mut conn = MockConn::failing();
    let h = p.health(&mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
}

// ---- ProtocolDiagnostics ----

#[test]
fn diagnostics_fields_present() {
    let p = ModbusplusProtocol::new().station(5);
    let mut conn = MockConn::new(vec![]);
    let d = p.diagnostics(&mut conn).unwrap();
    assert_eq!(d.device_id, "mb+:5");
    assert_eq!(d.error_count, 0);
    assert!(d.last_error.is_none());
    assert!(d.custom_fields.is_empty());
}
