// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_opcua::{
    Authentication, OPCUAValue, OpcuaProtocol, SecurityPolicy, build_hello, build_read_request,
    parse_acknowledge,
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

fn ack_response(version: u32) -> Vec<u8> {
    let mut r = b"ACK".to_vec();
    r.push(b'F');
    r.extend_from_slice(&0u32.to_le_bytes()); // size
    r.extend_from_slice(&version.to_le_bytes()); // protocol version @ [8..12]
    r.extend_from_slice(&65536u32.to_le_bytes());
    r.extend_from_slice(&65536u32.to_le_bytes());
    r.extend_from_slice(&0u32.to_le_bytes()); // max message size
    r
}

// ── Hello ──

#[test]
fn hello_message_exact_bytes() {
    let url = "opc.tcp://localhost:4840";
    let f = build_hello(url);
    assert_eq!(&f[0..3], b"HEL");
    assert_eq!(f[3], b'F');
    let total = 28 + url.len();
    assert_eq!(f.len(), total);
    assert_eq!(u32::from_le_bytes([f[4], f[5], f[6], f[7]]), total as u32);
    assert_eq!(u32::from_le_bytes([f[8], f[9], f[10], f[11]]), 0); // protocol version
    assert_eq!(
        u32::from_le_bytes([f[24], f[25], f[26], f[27]]),
        url.len() as u32
    );
    assert_eq!(&f[28..], url.as_bytes());
    // empty url is allowed and yields minimal frame
    let f2 = build_hello("");
    assert_eq!(f2.len(), 28);
}

// ── Acknowledge ──

#[test]
fn acknowledge_happy_and_errors() {
    assert_eq!(parse_acknowledge(&ack_response(42)).unwrap(), 42);
    assert!(parse_acknowledge(&[0; 10]).is_err()); // too short
    let mut wrong = ack_response(0);
    wrong[0] = b'X';
    assert!(parse_acknowledge(&wrong).is_err()); // not ACK
}

// ── Read request ──

#[test]
fn read_request_structure() {
    let f = build_read_request(&["ns=0;i=1", "ns=0;i=2"], 7);
    assert_eq!(&f[0..3], b"MSG");
    assert_eq!(f[3], b'F');
    // MessageSize must equal the full emitted length
    assert_eq!(u32::from_le_bytes([f[4], f[5], f[6], f[7]]), f.len() as u32);
    assert_eq!(
        u32::from_le_bytes([f[8], f[9], f[10], f[11]]),
        f.len() as u32 - 8 - 4
    );
    // body: encoding mask, namespace, request handle, then 2 bytes per node id
    assert_eq!(f[12], 0x01);
    assert_eq!(f[13], 0x00);
    assert_eq!(u32::from_le_bytes([f[14], f[15], f[16], f[17]]), 7);
    assert_eq!(f.len(), 8 + 4 + (2 + 4 + 2 * 2));
    // single node id -> one 2-byte entry
    let f1 = build_read_request(&["ns=0;i=5"], 1);
    assert_eq!(f1.len(), 8 + 4 + (2 + 4 + 2));
}

// ── metadata ──

#[test]
fn opcua_metadata() {
    let p = OpcuaProtocol::new("opc.tcp://host:4840")
        .with_security(SecurityPolicy::Basic256Sha256)
        .with_auth(Authentication::Username {
            user: "u".into(),
            password: "p".into(),
        });
    assert_eq!(p.name(), "opcua");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    assert_eq!(p.supported_variants(), &["binary"]);
}

// ── ProtocolRead over mock connection ──

#[test]
fn read_hello_then_read_request() {
    let p = OpcuaProtocol::new("opc.tcp://localhost:4840");
    let mut conn = MockConnection::new(vec![ack_response(0), vec![]]);
    let v = ProtocolRead::read(&p, &mut conn, &"ns=0;i=1".to_string()).unwrap();
    assert!(matches!(v, OPCUAValue::Boolean(true)));
    // two sends: Hello then Read request
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(&conn.sent[0][0..3], b"HEL");
    assert_eq!(&conn.sent[1][0..3], b"MSG");
}

#[test]
fn read_rejects_bad_ack_and_connection_failure() {
    let p = OpcuaProtocol::new("opc.tcp://x");
    let mut conn = MockConnection::new(vec![vec![0; 8]]); // not ACK
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &"ns=0;i=1".to_string()).unwrap_err(),
        ProtocolError::Parse { .. }
    ));
    let mut failing = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut failing, &"ns=0;i=1".to_string()).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn read_batch_and_range() {
    let p = OpcuaProtocol::new("opc.tcp://x");
    // each read = hello (needs ACK response) + read request (response ignored)
    let mut conn = MockConnection::new(vec![ack_response(0), vec![], ack_response(0), vec![]]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals.len(), 2);
    assert!(matches!(vals[0], OPCUAValue::Boolean(true)));
    let mut conn2 = MockConnection::new(vec![ack_response(0), vec![], ack_response(0), vec![]]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &"a".to_string(), 2).unwrap();
    assert_eq!(vals2.len(), 2);
}

// ── ProtocolWrite ──

#[test]
fn write_is_unsupported() {
    let p = OpcuaProtocol::new("opc.tcp://x");
    let mut conn = MockConnection::new(vec![]);
    let err =
        ProtocolWrite::write(&p, &mut conn, &"a".to_string(), &OPCUAValue::Byte(1)).unwrap_err();
    assert!(matches!(err, ProtocolError::Unsupported { .. }));
    // write_batch propagates the first unsupported error
    let err2 = ProtocolWrite::write_batch(&p, &mut conn, &[("a".to_string(), OPCUAValue::Null)])
        .unwrap_err();
    assert!(matches!(err2, ProtocolError::Unsupported { .. }));
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_and_unhealthy() {
    let p = OpcuaProtocol::new("opc.tcp://x");
    let mut conn = MockConnection::new(vec![ack_response(0)]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("Hello"));
    let mut failing = MockConnection::failing();
    let h2 = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h2.state, HealthState::Unhealthy);
}

#[test]
fn diagnostics_reports_fields() {
    let p = OpcuaProtocol::new("opc.tcp://ep").with_security(SecurityPolicy::Basic128Rsa15);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "opc.tcp://ep");
    assert_eq!(d.custom_fields["security"], "Basic128Rsa15");
    assert_eq!(d.custom_fields["endpoint"], "opc.tcp://ep");
}
