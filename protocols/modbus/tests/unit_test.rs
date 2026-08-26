// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_modbus::ModbusProtocol;
use industrial_protocols_modbus::frame::*;
use std::collections::VecDeque;
use std::io;

struct MockConnection {
    responses: VecDeque<Vec<u8>>,
    fail: bool,
    sent: Vec<Vec<u8>>,
}

impl MockConnection {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockConnection {
            responses: responses.into(),
            fail: false,
            sent: vec![],
        }
    }
    fn failing() -> Self {
        MockConnection {
            responses: VecDeque::new(),
            fail: true,
            sent: vec![],
        }
    }
}

impl Connection for MockConnection {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.sent.push(data.to_vec());
        if self.fail {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "mock failure",
            ));
        }
        Ok(self.responses.pop_front().unwrap_or_default())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

fn rtu_read_resp(addr: u8, fc: u8, values: &[u16]) -> Vec<u8> {
    let mut r = vec![addr, fc, (values.len() * 2) as u8];
    for v in values {
        r.extend_from_slice(&v.to_be_bytes());
    }
    let crc = crc16(&r);
    r.extend_from_slice(&crc.to_le_bytes());
    r
}

fn tcp_read_resp(uid: u8, values: &[u16]) -> Vec<u8> {
    let mut r = vec![
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x05,
        uid,
        0x03,
        (values.len() * 2) as u8,
    ];
    for v in values {
        r.extend_from_slice(&v.to_be_bytes());
    }
    r
}

fn exception_resp(fc: u8, code: u8) -> Vec<u8> {
    vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x01, fc | 0x80, code]
}

// ── checksums ──

#[test]
fn crc16_known_vector_and_empty() {
    assert_eq!(crc16(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x01]), 0x0A84);
    assert_eq!(crc16(&[]), 0xFFFF);
    assert_eq!(CRC16_TABLE.len(), 256);
}

#[test]
fn lrc_known_vector_and_empty() {
    assert_eq!(lrc(&[0x01, 0x03, 0x00, 0x0A, 0x00, 0x01]), 0xF1);
    assert_eq!(lrc(&[]), 0x00);
    assert_eq!(lrc(&[0x01, 0x02, 0xFD]), 0x00); // wraps to 0
}

// ── TCP builders: exact bytes ──

#[test]
fn tcp_read_holding_registers_exact_frame() {
    let f = build_read_holding_registers_tcp(0x1234, 7, 0x0102, 0x000A);
    assert_eq!(f.len(), 12);
    assert_eq!(
        f,
        vec![
            0x12, 0x34, 0x00, 0x00, 0x00, 0x06, 0x07, 0x03, 0x01, 0x02, 0x00, 0x0A
        ]
    );
    // max qty / max addr don't overflow
    let f2 = build_read_holding_registers_tcp(1, 1, 0xFFFF, 0xFFFF);
    assert_eq!(f2.len(), 12);
}

#[test]
fn tcp_write_single_register_exact_frame() {
    let f = build_write_single_register_tcp(1, 2, 0x0003, 0x00C8);
    assert_eq!(
        f,
        vec![
            0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x02, 0x06, 0x00, 0x03, 0x00, 0xC8
        ]
    );
}

#[test]
fn tcp_write_multiple_registers_exact_frame() {
    let f = build_write_multiple_registers_tcp(1, 1, 0, &[0x0001, 0x0002]);
    assert_eq!(f.len(), 17);
    assert_eq!(
        &f[..9],
        &[0x00, 0x01, 0x00, 0x00, 0x00, 0x09, 0x01, 0x10, 0x00]
    );
    assert_eq!(f[12], 0x04); // byte count = 2 values * 2
    assert_eq!(&f[13..], &[0x00, 0x01, 0x00, 0x02]);
    // empty values list still builds a well-formed header
    let f2 = build_write_multiple_registers_tcp(1, 1, 0, &[]);
    assert_eq!(f2[12], 0x00);
}

#[test]
fn tcp_coil_and_input_builders() {
    let coils = build_read_coils_tcp(1, 1, 0, 8);
    assert_eq!(&coils[7..9], &[0x01, 0x00]);
    let disc = build_read_discrete_inputs_tcp(1, 1, 0, 8);
    assert_eq!(&disc[7..9], &[0x02, 0x00]);
    let inp = build_read_input_registers_tcp(1, 1, 0, 8);
    assert_eq!(&inp[7..9], &[0x04, 0x00]);
    let single_coil = build_write_single_coil_tcp(1, 1, 100, true);
    assert_eq!(&single_coil[7..], &[0x05, 0x00, 0x64, 0xFF, 0x00]);
    let single_coil_off = build_write_single_coil_tcp(1, 1, 100, false);
    assert_eq!(&single_coil_off[10..], &[0x00, 0x00]);
    let multi_coils = build_write_multiple_coils_tcp(1, 1, 0, &[true; 9]);
    assert_eq!(multi_coils.len(), 15);
    assert_eq!(multi_coils[12], 2); // ceil(9/8)
    assert_eq!(multi_coils[13], 0xFF);
    assert_eq!(multi_coils[14], 0x01);
}

// ── RTU builders: CRC appended ──

#[test]
fn rtu_builders_append_valid_crc() {
    let f = build_read_holding_registers_rtu(1, 0, 10);
    assert_eq!(f.len(), 8);
    assert_eq!(crc16(&f[..6]), u16::from_le_bytes([f[6], f[7]]));
    let f = build_write_single_register_rtu(1, 5, 42);
    assert_eq!(f.len(), 8);
    assert_eq!(crc16(&f[..6]), u16::from_le_bytes([f[6], f[7]]));
    let f = build_write_multiple_registers_rtu(1, 0, &[1, 2, 3]);
    assert_eq!(f.len(), 15);
    assert_eq!(crc16(&f[..13]), u16::from_le_bytes([f[13], f[14]]));
    let f = build_write_single_coil_rtu(1, 9, true);
    assert_eq!(f.len(), 8);
    assert_eq!(&f[4..6], &[0xFF, 0x00]);
}

// ── ASCII builders ──

#[test]
fn ascii_builders_format_and_lrc() {
    let f = build_read_holding_registers_ascii(1, 0, 10);
    let s = String::from_utf8(f.clone()).unwrap();
    assert!(s.starts_with(':'));
    assert!(s.ends_with("\r\n"));
    assert_eq!(f.len(), 17); // ':' + 12 hex + 2 lrc + CR LF
    let lrc_byte = lrc(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]);
    assert!(s.contains(&format!("{lrc_byte:02X}")));
    let f2 = build_write_single_register_ascii(1, 0, 0xFFFF);
    assert_eq!(f2.len(), 17);
}

// ── backward-compat TCP builders ──

#[test]
fn backward_compat_builders_delegate_to_tcp() {
    assert_eq!(
        build_read_holding_registers(1, 0, 1),
        build_read_holding_registers_tcp(1, 1, 0, 1)
    );
    assert_eq!(
        build_write_single_register(1, 0, 1),
        build_write_single_register_tcp(1, 1, 0, 1)
    );
    assert_eq!(
        build_write_multiple_registers(1, 0, &[1]),
        build_write_multiple_registers_tcp(1, 1, 0, &[1])
    );
    assert_eq!(build_read_coils(1, 0, 1), build_read_coils_tcp(1, 1, 0, 1));
    assert_eq!(
        build_read_discrete_inputs(1, 0, 1),
        build_read_discrete_inputs_tcp(1, 1, 0, 1)
    );
    assert_eq!(
        build_read_input_registers(1, 0, 1),
        build_read_input_registers_tcp(1, 1, 0, 1)
    );
}

// ── response parsing ──

#[test]
fn parse_read_response_tcp_happy_and_errors() {
    let ok = tcp_read_resp(1, &[0x1234, 0x5678]);
    assert_eq!(parse_read_response_tcp(&ok).unwrap(), vec![0x1234, 0x5678]);
    // too short
    assert!(parse_read_response_tcp(&[0, 1, 0]).is_err());
    // truncated: byte count promises more than delivered
    let mut trunc = ok[..10].to_vec();
    trunc[8] = 0x04;
    assert!(parse_read_response_tcp(&trunc).is_err());
    // zero byte count -> empty values
    let zero = vec![0, 1, 0, 0, 0, 5, 1, 3, 0];
    assert_eq!(parse_read_response_tcp(&zero).unwrap(), Vec::<u16>::new());
}

#[test]
fn parse_read_response_rtu_happy_and_errors() {
    let ok = rtu_read_resp(1, 3, &[42, 66]);
    assert_eq!(parse_read_response_rtu(&ok).unwrap(), vec![42, 66]);
    assert!(parse_read_response_rtu(&[1, 3]).is_err()); // too short
    let mut bad = ok.clone();
    bad[3] ^= 0xFF; // corrupt a value -> CRC mismatch
    assert!(parse_read_response_rtu(&bad).is_err());
}

#[test]
fn parse_read_response_auto_detect() {
    assert_eq!(
        parse_read_response(&tcp_read_resp(1, &[7])).unwrap(),
        vec![7]
    );
    assert_eq!(
        parse_read_response(&rtu_read_resp(1, 3, &[7])).unwrap(),
        vec![7]
    );
    assert!(parse_read_response(&[]).is_err());
}

#[test]
fn parse_exception_detection() {
    assert_eq!(parse_exception_tcp(&exception_resp(3, 2)), Some(2));
    assert_eq!(parse_exception_tcp(&tcp_read_resp(1, &[1])), None);
    assert_eq!(parse_exception_tcp(&[]), None);
    assert_eq!(parse_exception_rtu(&[1, 0x83, 2]), Some(2));
    assert_eq!(parse_exception_rtu(&[1, 3, 2]), None);
    assert_eq!(parse_exception(&exception_resp(3, 4)), Some(4));
    assert_eq!(parse_exception(&[]), None);
}

// ── protocol metadata ──

#[test]
fn modbus_metadata() {
    let p = ModbusProtocol::tcp();
    assert_eq!(p.name(), "modbus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["tcp", "rtu", "ascii"]);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_tcp_happy_path() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![tcp_read_resp(1, &[0x012A])]);
    let v = ProtocolRead::read(&p, &mut conn, &0).unwrap();
    assert_eq!(v, 0x012A);
    // sent frame is a 12-byte TCP read request with TID 1
    assert_eq!(conn.sent[0].len(), 12);
    assert_eq!(conn.sent[0][7], 0x03);
    assert_eq!(&conn.sent[0][..2], &[0x00, 0x01]);
}

#[test]
fn read_rtu_and_ascii_variants() {
    let p = ModbusProtocol::rtu(3);
    let mut conn = MockConnection::new(vec![rtu_read_resp(3, 3, &[0x0100])]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &0).unwrap(), 0x0100);
    assert_eq!(conn.sent[0].len(), 8); // RTU frame

    let p2 = ModbusProtocol::ascii(3);
    let mut conn2 = MockConnection::new(vec![rtu_read_resp(3, 3, &[0x0100])]);
    assert_eq!(ProtocolRead::read(&p2, &mut conn2, &0).unwrap(), 0x0100);
    assert_eq!(conn2.sent[0][0], b':');
}

#[test]
fn read_transaction_ids_increment() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![tcp_read_resp(1, &[1]), tcp_read_resp(1, &[2])]);
    ProtocolRead::read(&p, &mut conn, &0).unwrap();
    ProtocolRead::read(&p, &mut conn, &0).unwrap();
    assert_eq!(&conn.sent[0][..2], &[0x00, 0x01]);
    assert_eq!(&conn.sent[1][..2], &[0x00, 0x02]);
}

#[test]
fn read_device_exception_error() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![exception_resp(3, 0x02)]);
    let err = ProtocolRead::read(&p, &mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::DeviceError { code: 2, .. }));
}

#[test]
fn read_connection_failure_and_short_response() {
    let p = ModbusProtocol::tcp();
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &0).unwrap_err(),
        ProtocolError::Connection(_)
    ));
    let mut short = MockConnection::new(vec![vec![0x01, 0x02]]);
    assert!(matches!(
        ProtocolRead::read(&p, &mut short, &0).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
    let mut empty = MockConnection::new(vec![vec![]]);
    assert!(ProtocolRead::read(&p, &mut empty, &0).is_err());
}

#[test]
fn read_batch_contiguous_and_sparse() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![tcp_read_resp(1, &[10, 11, 12])]);
    let v = ProtocolRead::read_batch(&p, &mut conn, &[10, 11, 12]).unwrap();
    assert_eq!(v, vec![10, 11, 12]);
    // sparse: one range request covering min..=max, values indexed back
    let mut conn2 = MockConnection::new(vec![tcp_read_resp(1, &[1, 2, 3])]);
    let v2 = ProtocolRead::read_batch(&p, &mut conn2, &[10, 12]).unwrap();
    assert_eq!(v2, vec![1, 3]);
    // empty batch short-circuits without sending
    let mut conn3 = MockConnection::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn3, &[]).unwrap(),
        Vec::<u16>::new()
    );
    assert!(conn3.sent.is_empty());
}

#[test]
fn read_range_zero_and_multi() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![tcp_read_resp(1, &[7, 8, 9])]);
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &0, 3).unwrap(),
        vec![7, 8, 9]
    );
    // count 0 still sends a request; a zero-value response parses to empty
    let mut conn2 = MockConnection::new(vec![vec![
        0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x01, 0x03, 0x00,
    ]]);
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn2, &0, 0).unwrap(),
        Vec::<u16>::new()
    );
    assert_eq!(conn2.sent.len(), 1);
}

// ── ProtocolWrite ──

#[test]
fn write_single_register_all_variants() {
    let mut conn = MockConnection::new(vec![vec![], vec![], vec![]]);
    ProtocolWrite::write(&ModbusProtocol::tcp(), &mut conn, &0, &0x00C8).unwrap();
    assert_eq!(conn.sent[0][7], 0x06);
    assert_eq!(&conn.sent[0][10..], &[0x00, 0xC8]);
    ProtocolWrite::write(&ModbusProtocol::rtu(1), &mut conn, &0, &0x00C8).unwrap();
    assert_eq!(conn.sent[1][1], 0x06);
    ProtocolWrite::write(&ModbusProtocol::ascii(1), &mut conn, &0, &0x00C8).unwrap();
    assert_eq!(conn.sent[2][0], b':');
}

#[test]
fn write_reports_exception_and_connection_failure() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![exception_resp(6, 1)]);
    let err = ProtocolWrite::write(&p, &mut conn, &0, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::DeviceError { code: 1, .. }));
    let mut failing = MockConnection::failing();
    assert!(ProtocolWrite::write(&p, &mut failing, &0, &1).is_err());
}

#[test]
fn write_batch_multiple_registers() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![vec![]]);
    ProtocolWrite::write_batch(&p, &mut conn, &[(0, 1), (1, 2)]).unwrap();
    assert_eq!(conn.sent[0][7], 0x10);
    // ASCII falls back to one single-write per item
    let p2 = ModbusProtocol::ascii(1);
    let mut conn2 = MockConnection::new(vec![vec![], vec![]]);
    ProtocolWrite::write_batch(&p2, &mut conn2, &[(0, 1), (1, 2)]).unwrap();
    assert_eq!(conn2.sent.len(), 2);
    // ASCII frames are hex text; both must contain the "06" function code
    let s0 = String::from_utf8(conn2.sent[0].clone()).unwrap();
    let s1 = String::from_utf8(conn2.sent[1].clone()).unwrap();
    assert!(s0.starts_with(':') && s0.contains("06") && s1.contains("06"));
    // empty batch sends nothing
    let mut conn3 = MockConnection::new(vec![]);
    ProtocolWrite::write_batch(&p, &mut conn3, &[]).unwrap();
    assert!(conn3.sent.is_empty());
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = ModbusProtocol::tcp();
    let mut conn = MockConnection::new(vec![tcp_read_resp(1, &[1])]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
    assert!(h2.details.contains("failed"));
}

#[test]
fn diagnostics_reports_fields() {
    let p = ModbusProtocol::rtu(7);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "modbus-Rtu-7");
    assert_eq!(d.custom_fields["unit_id"], "7");
    assert_eq!(d.custom_fields["variant"], "Rtu");
}
