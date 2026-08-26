// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_saej1850::*;
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

/// Independent CRC-8 (poly 0x1D, init 0xFF) to validate frame checksums.
fn crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0xFF;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x1D
            } else {
                crc << 1
            };
        }
    }
    crc
}

// ---------- frame builders ----------

#[test]
fn saej1850_request_exact_bytes() {
    let f = build_request(0x01, 0x0C);
    assert_eq!(&f[0..5], &[0x68, 0x6A, 0xF1, 0x01, 0x0C]);
    assert_eq!(f.len(), 6);
    assert_eq!(*f.last().unwrap(), crc8(&f[..5]));
}

#[test]
fn saej1850_request_crc_checksum() {
    // independent CRC-8 (poly 0x1D) of the 5-byte header, including DTC mode
    let f = build_request(0x03, 0x00);
    assert_eq!(f[5], crc8(&f[..5]));
}

#[test]
fn saej1850_read_dtc_and_vin() {
    let dtc = build_read_dtc();
    assert_eq!(&dtc[3..5], &[0x03, 0x00]);
    assert_eq!(*dtc.last().unwrap(), crc8(&dtc[..5]));
    let vin = build_read_vin();
    assert_eq!(&vin[3..5], &[0x09, 0x02]);
    assert_eq!(*vin.last().unwrap(), crc8(&vin[..5]));
}

#[test]
fn saej1850_request_edge_mode_pid() {
    let f = build_request(0x00, 0x00);
    assert_eq!(f[3], 0x00);
    let f = build_request(0xFF, 0xFF);
    assert_eq!(f[3], 0xFF);
    assert_eq!(f[4], 0xFF);
    assert_eq!(f.len(), 6);
}

// ---------- protocol metadata ----------

#[test]
fn saej1850_metadata() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    assert_eq!(p.name(), "saej1850");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::AutomotiveBus);
    assert_eq!(p.supported_variants(), &["vpw", "pwm"]);
}

// ---------- ProtocolRead ----------

#[test]
fn saej1850_read_sends_mode01_request() {
    let p = SaeJ1850Protocol::new(J1850Variant::Pwm);
    let mut conn = MockConn::new(vec![0x41, 0x0C, 0x00]);
    let v = ProtocolRead::read(&p, &mut conn, &0x0C).unwrap();
    assert_eq!(v, vec![0x41, 0x0C, 0x00]);
    assert_eq!(conn.sent, vec![build_request(0x01, 0x0C)]);
}

#[test]
fn saej1850_read_connection_failure() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &1).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn saej1850_read_batch_and_range() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    let mut conn = MockConn::new(vec![0x41]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0x00, 0x0C]).unwrap();
    assert_eq!(vals, vec![vec![0x41], vec![0x41]]);
    assert_eq!(conn.sent.len(), 2);
    let mut conn = MockConn::new(vec![0x41]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &0x0C, 3).unwrap();
    assert_eq!(vals, vec![vec![0x41]; 3]);
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn saej1850_read_batch_empty_and_range_zero() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

// ---------- ProtocolHealth ----------

#[test]
fn saej1850_health_healthy() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    let mut conn = MockConn::new(vec![0x41]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("mode01"));
    assert_eq!(conn.sent, vec![build_request(0x01, 0x00)]);
}

#[test]
fn saej1850_health_unhealthy() {
    let p = SaeJ1850Protocol::new(J1850Variant::Pwm);
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn saej1850_diagnostics_fields() {
    let p = SaeJ1850Protocol::new(J1850Variant::Vpw);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "j1850-device");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
