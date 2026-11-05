// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # MQTT — MQTT 3.1.1 协议。CONNECT/PUBLISH/SUBSCRIBE/PINGREQ、QoS 0/1、保留消息。

//! MQTT 3.1.1 protocol implementation.
//!
//! Implements MQTT client with CONNECT, PUBLISH, SUBSCRIBE, PINGREQ.

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct MqttProtocol {
    pub client_id: String,
    keep_alive: u16,
    clean_session: bool,
    username: Option<String>,
    password: Option<String>,
}

impl MqttProtocol {
    pub fn new(client_id: impl Into<String>) -> Self {
        MqttProtocol {
            client_id: client_id.into(),
            keep_alive: 60,
            clean_session: true,
            username: None,
            password: None,
        }
    }

    pub fn with_keep_alive(mut self, secs: u16) -> Self {
        self.keep_alive = secs;
        self
    }
    pub fn with_clean_session(mut self, clean: bool) -> Self {
        self.clean_session = clean;
        self
    }
    pub fn with_credentials(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.into());
        self.password = Some(pass.into());
        self
    }
}

// MQTT Control Packet types
const CONNECT: u8 = 0x10;
const CONNACK: u8 = 0x20;
const PUBLISH: u8 = 0x30;
const SUBSCRIBE: u8 = 0x82;
const PINGREQ: u8 = 0xC0;
const PINGRESP: u8 = 0xD0;

fn encode_remaining_length(len: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let mut remaining = len;
    loop {
        let mut byte = (remaining % 128) as u8;
        remaining /= 128;
        if remaining > 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if remaining == 0 {
            break;
        }
    }
    result
}

fn encode_utf8(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(2 + bytes.len());
    result.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    result.extend_from_slice(bytes);
    result
}

/// Build MQTT CONNECT packet.
pub fn build_connect(
    client_id: &str,
    keep_alive: u16,
    clean_session: bool,
    username: Option<&str>,
    password: Option<&str>,
) -> Vec<u8> {
    let mut payload = Vec::new();
    // Variable header: protocol name "MQTT" + level 4 + flags + keepalive
    payload.extend_from_slice(&encode_utf8("MQTT"));
    payload.push(4); // protocol level (3.1.1)
    let mut flags = 0u8;
    if clean_session {
        flags |= 0x02;
    } // Clean Session
    if username.is_some() {
        flags |= 0x80;
    }
    if password.is_some() {
        flags |= 0x40;
    }
    payload.push(flags);
    payload.extend_from_slice(&keep_alive.to_be_bytes());

    // Payload: Client ID
    payload.extend_from_slice(&encode_utf8(client_id));
    if let Some(u) = username {
        payload.extend_from_slice(&encode_utf8(u));
    }
    if let Some(p) = password {
        payload.extend_from_slice(&encode_utf8(p));
    }

    let remaining = encode_remaining_length(payload.len());
    let mut frame = Vec::with_capacity(1 + remaining.len() + payload.len());
    frame.push(CONNECT);
    frame.extend(&remaining);
    frame.extend(&payload);
    frame
}

/// Parse CONNACK. Returns (session_present, return_code).
pub fn parse_connack(data: &[u8]) -> std::result::Result<(bool, u8), String> {
    if data.len() < 4 {
        return Err("CONNACK too short".into());
    }
    if data[0] != CONNACK {
        return Err("not CONNACK".into());
    }
    let session_present = data[2] & 0x01 != 0;
    let return_code = data[3];
    Ok((session_present, return_code))
}

/// Build MQTT PUBLISH packet.
pub fn build_publish(
    topic: &str,
    payload: &[u8],
    qos: u8,
    retain: bool,
    packet_id: u16,
) -> Vec<u8> {
    let mut variable = encode_utf8(topic);
    if qos > 0 {
        variable.extend_from_slice(&packet_id.to_be_bytes());
    }
    let mut flags = PUBLISH;
    if retain {
        flags |= 0x01;
    }
    flags |= (qos & 0x03) << 1;

    let total_len = variable.len() + payload.len();
    let remaining = encode_remaining_length(total_len);
    let mut frame = Vec::with_capacity(1 + remaining.len() + total_len);
    frame.push(flags);
    frame.extend(&remaining);
    frame.extend(&variable);
    frame.extend_from_slice(payload);
    frame
}

/// Parse PUBLISH response, extracting topic and payload.
pub fn parse_publish(data: &[u8]) -> std::result::Result<(String, Vec<u8>), String> {
    if data.len() < 4 {
        return Err("PUBLISH too short".into());
    }
    let mut pos = 1;
    // skip remaining length (variable-length encoded)
    while pos < data.len() && data[pos] & 0x80 != 0 {
        pos += 1;
    }
    pos += 1;
    if pos + 2 > data.len() {
        return Err("PUBLISH header truncated".into());
    }
    let topic_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    if pos + topic_len > data.len() {
        return Err("PUBLISH topic truncated".into());
    }
    let topic = String::from_utf8_lossy(&data[pos..pos + topic_len]).to_string();
    pos += topic_len;
    let qos = (data[0] >> 1) & 0x03;
    if qos > 0 {
        if pos + 2 > data.len() {
            return Err("PUBLISH packet id truncated".into());
        }
        pos += 2;
    } // skip packet ID
    let payload = data[pos..].to_vec();
    Ok((topic, payload))
}

/// Build MQTT SUBSCRIBE packet.
pub fn build_subscribe(topics: &[(&str, u8)], packet_id: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&packet_id.to_be_bytes());
    for (topic, qos) in topics {
        payload.extend_from_slice(&encode_utf8(topic));
        payload.push(*qos & 0x03);
    }
    let remaining = encode_remaining_length(payload.len());
    let mut frame = Vec::with_capacity(1 + remaining.len() + payload.len());
    frame.push(SUBSCRIBE);
    frame.extend(&remaining);
    frame.extend(&payload);
    frame
}

/// Build PINGREQ.
pub fn build_pingreq() -> Vec<u8> {
    vec![PINGREQ, 0x00]
}

/// Check if response is PINGRESP.
pub fn is_pingresp(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == PINGRESP
}

impl Protocol for MqttProtocol {
    fn name(&self) -> &'static str {
        "mqtt"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IoTMessaging
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["3.1.1"]
    }
}

impl ProtocolRead for MqttProtocol {
    type Address = String; // topic name
    type Value = MqttValue;

    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        // Subscribe + wait for publish
        let sub = build_subscribe(&[(addr, 0)], 1);
        let _resp = conn.send(&sub).map_err(ProtocolError::from)?;
        // In practice, we'd wait for a PUBLISH message here
        // For now, return empty
        Ok(MqttValue {
            topic: addr.clone(),
            payload: vec![],
        })
    }

    fn read_batch(
        &self,
        conn: &mut dyn Connection,
        addrs: &[Self::Address],
    ) -> Result<Vec<Self::Value>> {
        addrs.iter().map(|a| self.read(conn, a)).collect()
    }

    fn read_range(
        &self,
        conn: &mut dyn Connection,
        start: &Self::Address,
        count: u16,
    ) -> Result<Vec<Self::Value>> {
        (0..count).map(|_| self.read(conn, start)).collect()
    }
}

impl ProtocolWrite for MqttProtocol {
    type Address = String;
    type Value = MqttValue;

    fn write(
        &self,
        conn: &mut dyn Connection,
        addr: &Self::Address,
        value: &Self::Value,
    ) -> Result<()> {
        let frame = build_publish(addr, &value.payload, 0, false, 1);
        conn.send(&frame).map_err(ProtocolError::from)?;
        Ok(())
    }

    fn write_batch(
        &self,
        conn: &mut dyn Connection,
        items: &[(Self::Address, Self::Value)],
    ) -> Result<()> {
        for (a, v) in items {
            self.write(conn, a, v)?;
        }
        Ok(())
    }
}

impl ProtocolHealth for MqttProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        let ping = build_pingreq();
        match conn.send(&ping) {
            Ok(resp) if is_pingresp(&resp) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "mqtt: PINGRESP ok".into(),
            }),
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Degraded,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "mqtt: unexpected PING response".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: 0,
                last_seen: std::time::Instant::now(),
                details: format!("mqtt: PING failed: {e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for MqttProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: self.client_id.clone(),
            uptime_secs: None,
            error_count: 0,
            last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("client_id".into(), self.client_id.clone()),
                ("keep_alive".into(), self.keep_alive.to_string()),
            ]),
        })
    }
}

#[derive(Debug, Clone)]
pub struct MqttValue {
    pub topic: String,
    pub payload: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_packet() {
        let frame = build_connect("test-client", 60, true, None, None);
        assert_eq!(frame[0], CONNECT);
        assert!(frame.len() > 10);
    }

    #[test]
    fn test_publish_packet() {
        let frame = build_publish("test/topic", b"hello", 0, false, 0);
        assert_eq!(frame[0] & 0xF0, 0x30);
    }

    #[test]
    fn test_subscribe_packet() {
        let frame = build_subscribe(&[("sensor/#", 0)], 1);
        assert_eq!(frame[0], SUBSCRIBE);
    }

    #[test]
    fn test_pingreq() {
        let frame = build_pingreq();
        assert_eq!(frame[0], PINGREQ);
        assert_eq!(frame[1], 0x00);
    }

    #[test]
    fn test_parse_publish() {
        // Build a simple publish payload manually
        let payload = b"hello";
        let topic = "test";
        let mut frame = vec![0x30];
        let var = encode_utf8(topic);
        let total = var.len() + payload.len();
        frame.extend(&encode_remaining_length(total));
        frame.extend(&var);
        frame.extend_from_slice(payload);

        let (t, p) = parse_publish(&frame).unwrap();
        assert_eq!(t, "test");
        assert_eq!(p, b"hello");
    }

    #[test]
    fn test_mqtt_metadata() {
        let p = MqttProtocol::new("client-1").with_keep_alive(30);
        assert_eq!(p.name(), "mqtt");
    }
}
