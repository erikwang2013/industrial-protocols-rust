// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_ethernet_ip::{
    EthernetIpProtocol, EthernetIpValue, build_cip_read_tag, build_register_session,
    parse_cip_read_tag_response, parse_register_session_response,
};
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

fn reg_session_resp(session: u32) -> Vec<u8> {
    let mut r = vec![0u8; 28];
    r[0] = 0x65;
    r[1] = 0x00;
    r[4..8].copy_from_slice(&session.to_le_bytes());
    r
}

fn cip_resp(value: i16) -> Vec<u8> {
    let cip = [0x4C, 0x00, 0x02, value as u8, (value >> 8) as u8];
    let mut r = vec![0u8; 34];
    r.extend_from_slice(&(cip.len() as u16).to_le_bytes());
    r.extend_from_slice(&cip);
    r
}

// ── RegisterSession ──

#[test]
fn register_session_frame_exact_bytes() {
    let f = build_register_session();
    assert_eq!(f.len(), 28);
    assert_eq!(&f[..2], &[0x65, 0x00]); // cmd 0x0065 LE
    assert_eq!(&f[2..4], &[0x04, 0x00]); // data length 4
    assert_eq!(&f[24..26], &[0x01, 0x00]); // protocol version 1
    assert_eq!(&f[26..28], &[0x00, 0x00]); // options
}

#[test]
fn parse_register_session_response_ok_and_err() {
    assert_eq!(
        parse_register_session_response(&reg_session_resp(0x12345678)).unwrap(),
        0x12345678
    );
    assert!(parse_register_session_response(&[0u8; 10]).is_err()); // too short
    assert!(parse_register_session_response(&reg_session_resp(0)).is_err()); // session 0
}

// ── CIP Read Tag ──

#[test]
fn cip_read_tag_frame_pads_odd_tags() {
    let f = build_cip_read_tag(0x01020304, "TAG");
    // 24 header + 4 iface + 2 timeout + 2 count + 2 type + 2 len + cip
    let cip = 10; // service+pathsize+seg(4 for "TAG")+pad+elements(2)
    assert_eq!(f.len(), 24 + 4 + 2 + 2 + 2 + 2 + cip);
    assert_eq!(&f[..2], &[0x6F, 0x00]); // SendRRData
    assert_eq!(&f[4..8], &[0x04, 0x03, 0x02, 0x01]); // session LE
    assert_eq!(f[36], 0x4C); // Read Tag service
    assert_eq!(f[37], 0x02); // path size in words (1 + 3 + 1 pad = 5 -> 2)
    assert_eq!(f[38], 0x91); // symbolic segment
    assert_eq!(f[39], 0x03); // tag length
    assert_eq!(&f[40..43], b"TAG");
    assert_eq!(f[43], 0x00); // pad byte
    assert_eq!(&f[44..46], &[0x01, 0x00]); // 1 element
    // even-length tag: no pad, elements follow the tag directly
    let f2 = build_cip_read_tag(0, "T4G1");
    assert_eq!(f2.len(), 46);
    assert_eq!(f2[43], b'1'); // last tag char (no pad byte)
    assert_eq!(&f2[44..46], &[0x01, 0x00]); // elements
}

#[test]
fn parse_cip_read_tag_response_ok_and_err() {
    assert_eq!(
        parse_cip_read_tag_response(&cip_resp(42)).unwrap(),
        vec![42, 0]
    );
    let mut bad_status = cip_resp(1);
    bad_status[37] = 0x05; // cip status byte
    let err = parse_cip_read_tag_response(&bad_status).unwrap_err();
    assert!(err.contains("status 0x05"));
    assert!(parse_cip_read_tag_response(&[0u8; 30]).is_err()); // too short
    // item length overruns buffer
    let mut trunc = vec![0u8; 38];
    trunc[34] = 10;
    assert!(parse_cip_read_tag_response(&trunc).is_err());
}

// ── metadata ──

#[test]
fn ethernet_ip_metadata() {
    let p = EthernetIpProtocol::new(0);
    assert_eq!(p.name(), "ethernet-ip");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["tcp"]);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_registers_session_once_then_reads_tag() {
    let p = EthernetIpProtocol::new(0);
    let mut conn = MockConnection::new(vec![reg_session_resp(0x0100), cip_resp(42)]);
    let v = ProtocolRead::read(&p, &mut conn, &"Tag1".to_string()).unwrap();
    assert!(matches!(v, EthernetIpValue::Int(42)));
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(&conn.sent[0][..2], &[0x65, 0x00]); // RegisterSession
    assert_eq!(&conn.sent[1][..2], &[0x6F, 0x00]); // SendRRData
    // session reused on second read: no new RegisterSession
    let mut conn2 = MockConnection::new(vec![cip_resp(7)]);
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn2, &"Tag1".to_string()).unwrap(),
        EthernetIpValue::Int(7)
    ));
    assert_eq!(conn2.sent.len(), 1);
}

#[test]
fn read_single_byte_value_falls_back_to_bool() {
    let p = EthernetIpProtocol::new(0);
    // 34-byte ENIP prefix + item len 3 + cip [0x4C, 0x00, 0x02] (no value bytes)
    let mut resp = vec![0u8; 34];
    resp.extend_from_slice(&3u16.to_le_bytes());
    resp.extend_from_slice(&[0x4C, 0x00, 0x02]);
    let mut conn = MockConnection::new(vec![reg_session_resp(5), resp]);
    let v = ProtocolRead::read(&p, &mut conn, &"B".to_string()).unwrap();
    assert!(matches!(v, EthernetIpValue::Bool(false)));
}

#[test]
fn read_errors() {
    let p = EthernetIpProtocol::new(0);
    // register session fails -> Parse error from bad response
    let mut conn = MockConnection::new(vec![vec![0u8; 4]]);
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &"T".to_string()).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &"T".to_string()).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = EthernetIpProtocol::new(0);
    let mut conn = MockConnection::new(vec![reg_session_resp(1), cip_resp(1), cip_resp(2)]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals.len(), 2);
    let mut conn2 = MockConnection::new(vec![cip_resp(9)]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &"a".to_string(), 1).unwrap();
    assert_eq!(vals2.len(), 1);
}

// ── ProtocolWrite / Health / Diagnostics ──

#[test]
fn write_is_unsupported() {
    let p = EthernetIpProtocol::new(0);
    let mut conn = MockConnection::new(vec![]);
    let err = ProtocolWrite::write(
        &p,
        &mut conn,
        &"a".to_string(),
        &EthernetIpValue::Bool(true),
    )
    .unwrap_err();
    assert!(matches!(err, ProtocolError::Unsupported { .. }));
}

#[test]
fn health_healthy_and_unhealthy() {
    let p = EthernetIpProtocol::new(0);
    let mut conn = MockConnection::new(vec![reg_session_resp(9)]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("RegisterSession"));
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = EthernetIpProtocol::new(3);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "enip:slot3");
    assert_eq!(d.custom_fields["slot"], "3");
    assert_eq!(d.custom_fields["session"], "0");
}
