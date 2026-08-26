// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_wirelesshart::*;
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
fn wirelesshart_read_pv_exact_bytes() {
    let f = build_read_pv("TT-101");
    assert_eq!(
        f,
        vec![
            0x00, 0x00, 0x00, 0x01, 0x01, 0x06, b'T', b'T', b'-', b'1', b'0', b'1'
        ]
    );
}

#[test]
fn wirelesshart_read_device_status_exact_bytes() {
    let f = build_read_device_status("AB");
    assert_eq!(f, vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x02, b'A', b'B']);
}

#[test]
fn wirelesshart_builders_empty_tag() {
    let f = build_read_pv("");
    assert_eq!(f, vec![0x00, 0x00, 0x00, 0x01, 0x01, 0x00]);
    let f = build_read_device_status("");
    assert_eq!(f, vec![0x00, 0x00, 0x00, 0x02, 0x03, 0x00]);
}

#[test]
fn wirelesshart_builders_tag_length_byte() {
    let tag = "T".repeat(44);
    let f = build_read_pv(&tag);
    assert_eq!(f[5], 44);
    assert_eq!(f.len(), 6 + 44);
    // tags longer than 255 wrap the length byte (len as u8 truncation)
    let tag = "T".repeat(300);
    let f = build_read_pv(&tag);
    assert_eq!(f[5], (300 % 256) as u8);
    assert_eq!(f.len(), 6 + 300);
}

// ---------- protocol metadata & builders ----------

#[test]
fn wirelesshart_metadata() {
    let p = WirelessHartProtocol::new();
    assert_eq!(p.name(), "wirelesshart");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::HardwareBridge);
    assert_eq!(p.supported_variants(), &["iec62591"]);
}

#[test]
fn wirelesshart_default_and_tag_builder() {
    let p = WirelessHartProtocol::default();
    assert_eq!(p.device_tag, "TT-101");
    let p = WirelessHartProtocol::new().tag("PT-200");
    assert_eq!(p.device_tag, "PT-200");
}

// ---------- ProtocolRead ----------

#[test]
fn wirelesshart_read_parses_f32_response() {
    let p = WirelessHartProtocol::new().tag("TT-101");
    let mut conn = MockConn::new(
        vec![0; 6]
            .into_iter()
            .chain(25.5f32.to_be_bytes())
            .collect(),
    );
    let v = ProtocolRead::read(&p, &mut conn, &"unused".into()).unwrap();
    assert_eq!(v, 25.5);
    assert_eq!(conn.sent, vec![build_read_pv("TT-101")]);
}

#[test]
fn wirelesshart_read_short_response_returns_zero() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::new(vec![0x00; 9]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &"a".into()).unwrap(), 0.0);
    let mut conn = MockConn::new(vec![]);
    assert_eq!(ProtocolRead::read(&p, &mut conn, &"a".into()).unwrap(), 0.0);
}

#[test]
fn wirelesshart_read_connection_failure() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &"a".into()).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn wirelesshart_read_batch_and_range() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::new(vec![0; 10]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals, vec![0.0; 2]);
    let mut conn = MockConn::new(vec![0; 10]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &"a".into(), 3).unwrap();
    assert_eq!(vals, vec![0.0; 3]);
    assert_eq!(conn.sent.len(), 3);
}

#[test]
fn wirelesshart_read_batch_empty_and_range_zero() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<f32>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &"a".into(), 0).unwrap(),
        Vec::<f32>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn wirelesshart_read_batch_connection_failure() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &["a".into()]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn wirelesshart_health_healthy() {
    let p = WirelessHartProtocol::new().tag("TT-101");
    let mut conn = MockConn::new(vec![0x00; 8]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("status"));
    assert_eq!(conn.sent, vec![build_read_device_status("TT-101")]);
}

#[test]
fn wirelesshart_health_unhealthy() {
    let p = WirelessHartProtocol::new();
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn wirelesshart_diagnostics_fields() {
    let p = WirelessHartProtocol::new().tag("PT-200");
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "PT-200");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert_eq!(d.custom_fields.get("tag"), Some(&"PT-200".to_string()));
}
