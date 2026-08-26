// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_bacnet::{
    BacnetProtocol, build_i_am, build_read_property, build_who_is, build_who_is_any,
    parse_read_property_response,
};
use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
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

struct MockTransport {
    resp: Vec<u8>,
    fail: bool,
}

impl Transport for MockTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "transport down",
            ))
        } else {
            Ok(self.resp.clone())
        }
    }
    fn is_open(&self) -> bool {
        !self.fail
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn float_resp(value: f32) -> Vec<u8> {
    let mut r = vec![0x81, 0x0A, 0x00, 0x0C, 0x01, 0x04]; // BVLC + NPDU
    r.extend_from_slice(&[0x44, 0x04]);
    r.extend_from_slice(&value.to_be_bytes());
    r
}

fn uint_resp(value: u32) -> Vec<u8> {
    let mut r = vec![0x81, 0x0A, 0x00, 0x0C, 0x01, 0x04];
    r.extend_from_slice(&[0x22, 0x04]);
    r.extend_from_slice(&value.to_be_bytes());
    r
}

// ── frame builders ──

#[test]
fn who_is_frame_exact_bytes() {
    let f = build_who_is(0x00010203, 0x00020405);
    assert_eq!(&f[..4], &[0x81, 0x0A, 0x00, 0x14]);
    assert_eq!(f.len(), 20); // 4 BVLC + 2 NPDU + 14 APDU
    assert_eq!(&f[6..10], &[0x10, 0x08, 0x1C, 0x04]);
    assert_eq!(&f[10..14], &[0x00, 0x01, 0x02, 0x03]); // low instance
    assert_eq!(&f[16..20], &[0x00, 0x02, 0x04, 0x05]); // high instance
}

#[test]
fn who_is_any_covers_full_range() {
    let f = build_who_is_any();
    assert_eq!(f, build_who_is(0, 4194303));
}

#[test]
fn i_am_frame_structure() {
    let f = build_i_am(1001);
    assert_eq!(&f[..4], &[0x81, 0x0A, 0x00, 0x12]);
    assert_eq!(f.len(), 18); // 4 BVLC + 2 NPDU + 12 APDU
    assert_eq!(f[6], 0x10); // unconfirmed
    assert_eq!(f[7], 0x00); // I-Am service
    assert_eq!(&f[14..18], &[0x00, 0x00, 0x03, 0xE9]); // instance 1001
}

#[test]
fn read_property_frame_structure() {
    let f = build_read_property(0x2A, 1001, 0, 1001, 85);
    assert_eq!(&f[..4], &[0x81, 0x0A, 0x00, 0x16]);
    assert_eq!(f.len(), 22); // 4 BVLC + 2 NPDU + 16 APDU
    assert_eq!(f[8], 0x2A); // invoke id
    assert_eq!(f[9], 0x0C); // ReadProperty service
    assert_eq!(f[f.len() - 1], 85); // property id
}

// ── response parsing ──

#[test]
fn parse_read_property_real_and_unsigned() {
    assert_eq!(parse_read_property_response(&float_resp(1.5)).unwrap(), 1.5);
    assert_eq!(parse_read_property_response(&uint_resp(42)).unwrap(), 42.0);
}

#[test]
fn parse_read_property_errors() {
    assert!(parse_read_property_response(&[0x81, 0x0A, 0x00, 0x04]).is_err()); // too short
    let no_tag = vec![
        0x81, 0x0A, 0x00, 0x0C, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert!(parse_read_property_response(&no_tag).is_err());
    // truncated value after tag
    let trunc = vec![0x81, 0x0A, 0x00, 0x0A, 0x01, 0x04, 0x44, 0x04, 0x00, 0x00];
    assert!(parse_read_property_response(&trunc).is_err());
}

// ── metadata ──

#[test]
fn bacnet_metadata() {
    let p = BacnetProtocol::new(1001).with_vendor(42);
    assert_eq!(p.name(), "bacnet");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["ip", "ethernet"]);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_parses_address_and_float_value() {
    let p = BacnetProtocol::new(1001);
    let mut conn = MockConnection::new(vec![float_resp(3.25)]);
    let v = ProtocolRead::read(&p, &mut conn, &"0:1001:85".to_string()).unwrap();
    assert_eq!(v, 3.25);
    assert_eq!(conn.sent[0][9], 0x0C); // ReadProperty
    // unsigned path
    let mut conn2 = MockConnection::new(vec![uint_resp(7)]);
    assert_eq!(
        ProtocolRead::read(&p, &mut conn2, &"0:1001:85".to_string()).unwrap(),
        7.0
    );
}

#[test]
fn read_address_fallbacks() {
    let p = BacnetProtocol::new(999);
    let mut conn = MockConnection::new(vec![uint_resp(1)]);
    // bare "8" -> instance falls back to device_instance, prop defaults to 85
    ProtocolRead::read(&p, &mut conn, &"8".to_string()).unwrap();
    let mut conn2 = MockConnection::new(vec![uint_resp(1)]);
    // non-numeric parts fall back too
    ProtocolRead::read(&p, &mut conn2, &"x:y:z".to_string()).unwrap();
}

#[test]
fn read_errors() {
    let p = BacnetProtocol::new(1);
    let mut conn = MockConnection::new(vec![vec![0x81, 0x0A, 0x00, 0x04]]);
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &"0:1:85".to_string()).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &"0:1:85".to_string()).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = BacnetProtocol::new(1);
    let mut conn = MockConnection::new(vec![uint_resp(1), uint_resp(2)]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals, vec![1.0, 2.0]);
    let mut conn2 = MockConnection::new(vec![uint_resp(5), uint_resp(5), uint_resp(5)]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &"a".to_string(), 3).unwrap();
    assert_eq!(vals2, vec![5.0, 5.0, 5.0]);
}

// ── ProtocolWrite ──

#[test]
fn write_is_unsupported() {
    let p = BacnetProtocol::new(1);
    let mut conn = MockConnection::new(vec![]);
    let err = ProtocolWrite::write(&p, &mut conn, &"a".to_string(), &1.0).unwrap_err();
    assert!(matches!(err, ProtocolError::Unsupported { .. }));
}

// ── ProtocolDiscover ──

#[test]
fn discover_returns_device() {
    let p = BacnetProtocol::new(1234).with_vendor(9);
    let mut t = MockTransport {
        resp: vec![0x81],
        fail: false,
    };
    let devs = ProtocolDiscover::discover(&p, &mut t).unwrap();
    assert_eq!(devs.len(), 1);
    assert_eq!(devs[0].device_id, "bacnet:1234");
    assert_eq!(devs[0].protocol, "bacnet");
    assert_eq!(devs[0].metadata["vendor_id"], "9");
}

#[test]
fn discover_transport_failure() {
    let p = BacnetProtocol::new(1);
    let mut t = MockTransport {
        resp: vec![],
        fail: true,
    };
    let err = ProtocolDiscover::discover(&p, &mut t).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = BacnetProtocol::new(1);
    let mut conn = MockConnection::new(vec![uint_resp(1)]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = BacnetProtocol::new(7).with_vendor(3);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "bacnet:7");
    assert_eq!(d.custom_fields["device_instance"], "7");
    assert_eq!(d.custom_fields["vendor_id"], "3");
}
