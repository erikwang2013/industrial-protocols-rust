// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_mqtt::{
    MqttProtocol, MqttValue, build_connect, build_pingreq, build_publish, build_subscribe,
    is_pingresp, parse_connack, parse_publish,
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

// ── CONNECT ──

#[test]
fn connect_packet_exact_bytes() {
    let f = build_connect("cid", 60, true, None, None);
    // fixed header: type 0x10, remaining length
    assert_eq!(f[0], 0x10);
    let rem = f[1] as usize;
    assert_eq!(f.len(), 2 + rem);
    assert_eq!(rem, 15);
    // variable header: utf8 "MQTT", level 4, flags 0x02 (clean), keepalive
    assert_eq!(&f[2..4], &[0x00, 0x04]);
    assert_eq!(&f[4..8], b"MQTT");
    assert_eq!(f[8], 4);
    assert_eq!(f[9], 0x02);
    assert_eq!(&f[10..12], &[0x00, 60]);
    // payload: client id length + bytes
    assert_eq!(&f[12..14], &[0x00, 0x03]);
    assert_eq!(&f[14..17], b"cid");
}

#[test]
fn connect_packet_flags_and_credentials() {
    let f = build_connect("c", 0, false, Some("u"), Some("p"));
    assert_eq!(f[9], 0x80 | 0x40); // username + password bits, no clean session
    // payload: client id "c", then username "u", then password "p"
    assert_eq!(&f[12..14], &[0x00, 0x01]);
    assert_eq!(f[14], b'c');
    assert_eq!(&f[15..17], &[0x00, 0x01]);
    assert_eq!(f[17], b'u');
    assert_eq!(&f[18..20], &[0x00, 0x01]);
    assert_eq!(f[20], b'p');
}

#[test]
fn parse_connack_happy_and_errors() {
    assert_eq!(
        parse_connack(&[0x20, 0x02, 0x00, 0x00]).unwrap(),
        (false, 0)
    );
    assert_eq!(parse_connack(&[0x20, 0x02, 0x01, 0x05]).unwrap(), (true, 5));
    assert!(parse_connack(&[0x20, 0x02, 0x00]).is_err()); // too short
    assert!(parse_connack(&[0x30, 0x02, 0x00, 0x00]).is_err()); // not CONNACK
}

// ── PUBLISH ──

#[test]
fn publish_packet_qos_and_retain_flags() {
    let f = build_publish("t", b"x", 1, true, 7);
    assert_eq!(f[0], 0x30 | 0x02 | 0x01); // QoS1 + retain
    assert_eq!(f[1], 0x06); // remaining: topic(3) + pid(2) + payload(1)
    assert_eq!(&f[2..4], &[0x00, 0x01]); // topic len
    assert_eq!(f[4], b't');
    assert_eq!(&f[5..7], &[0x00, 0x07]); // packet id
    let f2 = build_publish("t", b"x", 2, false, 7);
    assert_eq!(f2[0], 0x30 | 0x04);
    // qos clamped to 2 bits
    let f3 = build_publish("t", b"x", 9, false, 0);
    assert_eq!(f3[0], 0x30 | (9 & 0x03) << 1);
}

#[test]
fn publish_packet_large_payload_uses_multi_byte_length() {
    let f = build_publish("topic", &[0u8; 200], 0, false, 0);
    assert!(f[1] & 0x80 != 0); // continuation bit -> 2-byte remaining length
    assert_eq!(f.len(), 2 + 2 + 6 + 200);
}

#[test]
fn parse_publish_happy_paths() {
    let f = build_publish("sensor/t", b"hello", 0, false, 0);
    let (t, p) = parse_publish(&f).unwrap();
    assert_eq!(t, "sensor/t");
    assert_eq!(p, b"hello");
    // QoS1 with packet id skipped
    let f1 = build_publish("a", b"z", 1, false, 0x0102);
    let (t1, p1) = parse_publish(&f1).unwrap();
    assert_eq!(t1, "a");
    assert_eq!(p1, b"z");
    // empty payload
    let f2 = build_publish("t", b"", 0, false, 0);
    assert_eq!(parse_publish(&f2).unwrap().1, Vec::<u8>::new());
}

#[test]
fn parse_publish_rejects_malformed_input_without_panicking() {
    assert!(parse_publish(&[0x30]).is_err()); // too short
    // remaining length never terminates -> header truncated
    assert!(parse_publish(&[0x30, 0x80, 0x80, 0x80]).is_err());
    // topic length overruns buffer
    assert!(parse_publish(&[0x30, 0x02, 0x00, 0x05, b'h', b'e']).is_err());
    // qos1 frame missing packet id
    assert!(parse_publish(&[0x32, 0x05, 0x00, 0x02, b'a', b'b']).is_err());
}

// ── SUBSCRIBE / PINGREQ ──

#[test]
fn subscribe_packet_multiple_topics() {
    let f = build_subscribe(&[("a/b", 0), ("c", 1)], 0x0102);
    assert_eq!(f[0], 0x82);
    assert_eq!(&f[2..4], &[0x01, 0x02]); // packet id
    assert_eq!(&f[4..6], &[0x00, 0x03]);
    assert_eq!(&f[6..9], b"a/b");
    assert_eq!(f[9], 0);
    // qos clamped
    let f2 = build_subscribe(&[("t", 5)], 1);
    assert_eq!(f2[f2.len() - 1], 5 & 0x03);
}

#[test]
fn pingreq_and_pingresp() {
    assert_eq!(build_pingreq(), vec![0xC0, 0x00]);
    assert!(is_pingresp(&[0xD0, 0x00]));
    assert!(is_pingresp(&[0xD0, 0x00, 0x00])); // extra bytes tolerated
    assert!(!is_pingresp(&[0xD0])); // too short
    assert!(!is_pingresp(&[0xC0, 0x00]));
    assert!(!is_pingresp(&[]));
}

// ── metadata ──

#[test]
fn mqtt_metadata_and_builders() {
    let p = MqttProtocol::new("c1")
        .with_keep_alive(30)
        .with_clean_session(false)
        .with_credentials("u", "p");
    assert_eq!(p.name(), "mqtt");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::IoTMessaging);
    assert_eq!(p.supported_variants(), &["3.1.1"]);
}

// ── ProtocolRead / ProtocolWrite over mock connection ──

#[test]
fn read_subscribes_and_returns_empty_payload() {
    let p = MqttProtocol::new("c");
    let mut conn = MockConnection::new(vec![vec![]]);
    let v = ProtocolRead::read(&p, &mut conn, &"sensor/1".to_string()).unwrap();
    assert_eq!(v.topic, "sensor/1");
    assert!(v.payload.is_empty());
    assert_eq!(conn.sent[0][0], 0x82); // SUBSCRIBE sent
}

#[test]
fn read_batch_and_range() {
    let p = MqttProtocol::new("c");
    let mut conn = MockConnection::new(vec![vec![], vec![]]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &["a".into(), "b".into()]).unwrap();
    assert_eq!(vals.len(), 2);
    assert_eq!(vals[0].topic, "a");
    let mut conn2 = MockConnection::new(vec![vec![], vec![], vec![]]);
    let vals2 = ProtocolRead::read_range(&p, &mut conn2, &"t".to_string(), 3).unwrap();
    assert_eq!(vals2.len(), 3);
    assert_eq!(conn2.sent.len(), 3);
}

#[test]
fn read_connection_failure() {
    let p = MqttProtocol::new("c");
    let mut conn = MockConnection::failing();
    assert!(matches!(
        ProtocolRead::read(&p, &mut conn, &"t".to_string()).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

#[test]
fn write_publishes_and_reports_errors() {
    let p = MqttProtocol::new("c");
    let v = MqttValue {
        topic: "t".into(),
        payload: b"data".to_vec(),
    };
    let mut conn = MockConnection::new(vec![vec![]]);
    ProtocolWrite::write(&p, &mut conn, &"t".to_string(), &v).unwrap();
    assert_eq!(conn.sent[0][0] & 0xF0, 0x30);
    let items = vec![("a".to_string(), v.clone()), ("b".to_string(), v.clone())];
    let mut conn2 = MockConnection::new(vec![vec![], vec![]]);
    ProtocolWrite::write_batch(&p, &mut conn2, &items).unwrap();
    assert_eq!(conn2.sent.len(), 2);
    let mut failing = MockConnection::failing();
    assert!(ProtocolWrite::write(&p, &mut failing, &"t".to_string(), &v).is_err());
}

// ── ProtocolHealth / ProtocolDiagnostics ──

#[test]
fn health_healthy_degraded_unhealthy() {
    let p = MqttProtocol::new("c");
    let mut conn = MockConnection::new(vec![vec![0xD0, 0x00]]);
    assert_eq!(
        ProtocolHealth::health(&p, &mut conn).unwrap().state,
        HealthState::Healthy
    );
    let mut conn2 = MockConnection::new(vec![vec![0x30, 0x00]]);
    assert_eq!(
        ProtocolHealth::health(&p, &mut conn2).unwrap().state,
        HealthState::Degraded
    );
    let mut failing = MockConnection::failing();
    let h = ProtocolHealth::health(&p, &mut failing).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("PING failed"));
}

#[test]
fn diagnostics_reports_fields() {
    let p = MqttProtocol::new("client-9").with_keep_alive(45);
    let mut conn = MockConnection::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "client-9");
    assert_eq!(d.custom_fields["keep_alive"], "45");
}
