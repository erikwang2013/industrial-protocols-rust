// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_profinet::{
    ProfinetProtocol, build_dcp_identify_request, build_record_data_read,
    parse_dcp_identify_response,
};
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

// ── frame builders ──

#[test]
fn dcp_identify_request_exact_bytes() {
    let f = build_dcp_identify_request(0x01020304);
    assert_eq!(&f[..2], &[0xFE, 0xFD]); // FrameID
    assert_eq!(&f[2..4], &[0x01, 0x00]); // Identify request
    assert_eq!(&f[4..6], &[0x00, 0x00]); // reserved
    assert_eq!(&f[6..10], &[0x01, 0x02, 0x03, 0x04]); // Xid BE
    assert_eq!(&f[10..12], &[0x00, 0x00]); // response delay
    // two option blocks: IP (02 00 00 00) + DeviceProperties (01 00 00 00)
    assert_eq!(
        &f[12..20],
        &[0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
    );
    assert_eq!(f.len(), 20);
    // xid 0 works
    assert_eq!(build_dcp_identify_request(0).len(), 20);
}

#[test]
fn record_data_read_exact_bytes() {
    let f = build_record_data_read(0x00000001, 0x0002, 0x0003, 0x0004);
    assert_eq!(&f[..2], &[0xFC, 0x01]);
    assert_eq!(&f[2..6], &[0x00, 0x00, 0x00, 0x01]); // api
    assert_eq!(&f[6..8], &[0x00, 0x02]); // slot
    assert_eq!(&f[8..10], &[0x00, 0x03]); // subslot
    assert_eq!(&f[10..12], &[0x00, 0x04]); // index
    assert_eq!(&f[12..14], &[0x00, 0x00]); // length
    assert_eq!(f.len(), 14);
}

// ── response parsing ──

fn dcp_response_with_ip_and_name() -> Vec<u8> {
    let mut r = vec![0u8; 10];
    r.extend_from_slice(&[0x02, 0x02, 0x00, 0x04, 192, 168, 1, 10]);
    r.extend_from_slice(&[0x02, 0x01, 0x00, 0x03, b'P', b'N', b'1']);
    r
}

#[test]
fn parse_dcp_identify_response_happy() {
    let info = parse_dcp_identify_response(&dcp_response_with_ip_and_name()).unwrap();
    assert_eq!(info.name, "PN1");
    assert_eq!(info.ip.as_deref(), Some("192.168.1.10"));
}

#[test]
fn parse_dcp_identify_response_errors_and_defaults() {
    assert!(parse_dcp_identify_response(&[0u8; 9]).is_err()); // too short
    // unknown blocks and truncated block lengths are tolerated
    let mut unknown = vec![0u8; 10];
    unknown.extend_from_slice(&[0x09, 0x09, 0x00, 0x02, 0x01, 0x02]);
    unknown.extend_from_slice(&[0x02, 0x02, 0x00, 0x04, 192]); // truncated ip block
    let info = parse_dcp_identify_response(&unknown).unwrap();
    assert!(info.name.is_empty());
    assert_eq!(info.ip, None);
}

// ── metadata ──

#[test]
fn profinet_metadata() {
    let p = ProfinetProtocol::new();
    assert_eq!(p.name(), "profinet");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["nrt"]);
    let _d: ProfinetProtocol = Default::default();
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_parses_address_and_returns_raw_response() {
    let p = ProfinetProtocol::new();
    let canned = vec![0xDE, 0xAD];
    let mut conn = MockConnection::new(vec![canned.clone()]);
    let v = ProtocolRead::read(&p, &mut conn, &"A:2:3:4".to_string()).unwrap();
    assert_eq!(v, canned);
    // frame encodes api=0xA (hex), slot=2, subslot=3, index=4
    assert_eq!(&conn.sent[0][2..6], &[0x00, 0x00, 0x00, 0x0A]);
    assert_eq!(&conn.sent[0][6..8], &[0x00, 0x02]);
    assert_eq!(&conn.sent[0][8..10], &[0x00, 0x03]);
    assert_eq!(&conn.sent[0][10..12], &[0x00, 0x04]);
}

#[test]
fn read_address_fallbacks_and_errors() {
    let p = ProfinetProtocol::new();
    let mut conn = MockConnection::new(vec![vec![]]);
    // api in hex, others decimal; garbage parts fall back to 0
    ProtocolRead::read(&p, &mut conn, &"garbage:also".to_string()).unwrap();
    assert_eq!(&conn.sent[0][2..6], &[0; 4]);
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &"0:0:0:0".to_string()).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = ProfinetProtocol::new();
    let mut conn = MockConnection::new(vec![vec![1], vec![2]]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals, vec![vec![1], vec![2]]);
    let mut conn2 = MockConnection::new(vec![vec![3], vec![3]]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &"a".to_string(), 2).unwrap();
    assert_eq!(vals2.len(), 2);
}

// ── ProtocolWrite / Discover / Health / Diagnostics ──

#[test]
fn write_is_unsupported() {
    let p = ProfinetProtocol::new();
    let mut conn = MockConnection::new(vec![]);
    let err = ProtocolWrite::write(&p, &mut conn, &"a".to_string(), &vec![1u8]).unwrap_err();
    assert!(matches!(err, ProtocolError::Unsupported { .. }));
}

#[test]
fn discover_returns_device_from_dcp_response() {
    let p = ProfinetProtocol::new();
    let mut t = MockTransport {
        resp: dcp_response_with_ip_and_name(),
        fail: false,
    };
    let devs = ProtocolDiscover::discover(&p, &mut t).unwrap();
    assert_eq!(devs.len(), 1);
    assert_eq!(devs[0].device_id, "PN1");
    assert_eq!(devs[0].address, "192.168.1.10");
    assert_eq!(devs[0].metadata["station_name"], "PN1");
}

#[test]
fn discover_transport_and_parse_failures() {
    let p = ProfinetProtocol::new();
    let mut t = MockTransport {
        resp: vec![],
        fail: true,
    };
    assert!(matches!(
        ProtocolDiscover::discover(&p, &mut t).unwrap_err(),
        ProtocolError::Connection(_)
    ));
    let mut t2 = MockTransport {
        resp: vec![0u8; 5],
        fail: false,
    };
    assert!(matches!(
        ProtocolDiscover::discover(&p, &mut t2).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
}

#[test]
fn health_always_healthy_without_io() {
    let p = ProfinetProtocol::new();
    // health never touches the connection
    let mut conn = MockConnection::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(conn.sent.is_empty());
}

#[test]
fn diagnostics_reports_fields() {
    let p = ProfinetProtocol::new();
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "pn-device");
}
