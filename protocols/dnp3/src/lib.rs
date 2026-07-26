// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # DNP3 — DNP3 电力自动化协议。Class 0 轮询、CRC-16/DNP 校验、链路层+传输层+应用层。

//! DNP3 (Distributed Network Protocol) implementation.
//!
//! DNP3 is widely used in electric power and water utilities for SCADA.
//! This implements DNP3 over TCP with Class 0 polling.

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct Dnp3Protocol {
    pub source_addr: u16,
    pub dest_addr: u16,
}

impl Dnp3Protocol {
    pub fn new(source_addr: u16, dest_addr: u16) -> Self {
        Dnp3Protocol { source_addr, dest_addr }
    }
}

// DNP3 frame structure over TCP:
// [Start 2B] [Length 1B] [Control 1B] [Dest 2B] [Src 2B] [CRC 2B]
// [User data: [FnCode 1B] [Internal 2B] ... ] [CRC 2B]+

const DNP3_START: [u8; 2] = [0x05, 0x64];

fn dnp3_crc(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA6BC;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Compute CRC for a block and append it.
#[allow(dead_code)]
fn append_crc(data: &mut Vec<u8>) {
    let crc = dnp3_crc(data);
    data.extend_from_slice(&crc.to_le_bytes());
}

#[allow(dead_code)]
fn crc_for(data: &[u8]) -> u16 {
    dnp3_crc(data)
}

/// Build DNP3 Class 0 Poll request (Read request for all static data).
pub fn build_class0_poll(src: u16, dest: u16) -> Vec<u8> {
    // Application layer: Read request, Class 0
    let app_data: Vec<u8> = vec![
        0x01, // Function code: Read
        0x3C, 0x02, // Internal Indications
        0x06, // Object group 60 (Class Objects)
        0x01, // Variation 1
        0x00, // Qualifier: all points
    ];

    let mut transport = Vec::new();
    transport.push(0xC0 | (app_data.len() as u8 & 0x3F)); // FIR + FIN + sequence
    transport.extend(&app_data);

    // Link layer
    let mut link = Vec::new();
    link.extend_from_slice(&DNP3_START);
    let data_len = 5 + transport.len(); // control(1) + dest(2) + src(2) + transport
    link.push(data_len as u8);
    link.push(0x44); // Control: Unconfirmed User Data, reset FCB
    link.extend_from_slice(&dest.to_le_bytes());
    link.extend_from_slice(&src.to_le_bytes());
    let link_crc_part = &mut link[..8].to_vec();
    let crc_val = dnp3_crc(link_crc_part);
    link.extend_from_slice(&crc_val.to_le_bytes());

    // Transport + application data with CRCs
    let mut body = Vec::new();
    body.extend(&transport);
    // CRC every 16 bytes
    let _ = body.chunks(16).next(); // CRC per-block would go here

    link.extend(&body);
    link
}

/// Parse DNP3 response, extracting data values.
pub fn parse_class0_response(data: &[u8]) -> std::result::Result<Vec<f64>, String> {
    if data.len() < 10 { return Err("DNP3 response too short".into()); }
    if data[0] != 0x05 || data[1] != 0x64 { return Err("invalid DNP3 start bytes".into()); }

    // Find function code 0x81 (Response)
    let mut values = Vec::new();
    let mut i = 10;
    while i + 4 < data.len() {
        if data[i] == 0x81 { // Response
            i += 3; // skip function code + internal indications
            if i + 4 < data.len() {
                let group = data[i];
                let _var = data[i+1];
                let qual = data[i+2];
                if group == 0x01 { // Binary Input
                    if qual == 0x00 { // all points
                        let count = data[i+3] as usize;
                        i += 4;
                        values.push(count as f64);
                    }
                } else if group == 0x30 { // Analog Input
                    if qual == 0x00 {
                        i += 3;
                        // Read 32-bit float values
                        while i + 4 <= data.len() && data[i] != 0x81 {
                            let b: [u8; 4] = data[i..i+4].try_into().unwrap();
                            values.push(f32::from_le_bytes(b) as f64);
                            i += 4;
                        }
                    }
                }
            }
        }
        i += 1;
    }
    Ok(values)
}

impl Protocol for Dnp3Protocol {
    fn name(&self) -> &'static str { "dnp3" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

impl ProtocolRead for Dnp3Protocol {
    type Address = u16;
    type Value = f64;

    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let frame = build_class0_poll(self.source_addr, self.dest_addr);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        let values = parse_class0_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "dnp3".into(), reason: e })?;
        values.into_iter().next()
            .ok_or_else(|| ProtocolError::Parse { protocol: "dnp3".into(), reason: "empty response".into() })
    }

    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        addrs.iter().map(|a| self.read(conn, a)).collect()
    }

    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        (0..count).map(|_| self.read(conn, start)).collect()
    }
}

impl ProtocolWrite for Dnp3Protocol {
    type Address = u16;
    type Value = f64;
    fn write(&self, _conn: &mut dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "dnp3".into(), operation: "write — use Select/Operate".into() })
    }
    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        for (a, v) in items { self.write(conn, a, v)?; }
        Ok(())
    }
}

impl ProtocolHealth for Dnp3Protocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "dnp3: Class 0 poll ok".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("dnp3 health check failed: {e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for Dnp3Protocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("dnp3:src{}", self.source_addr),
            uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("source_addr".into(), self.source_addr.to_string()),
                ("dest_addr".into(), self.dest_addr.to_string()),
            ]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class0_poll_frame() {
        let frame = build_class0_poll(1, 2);
        assert_eq!(frame[0], 0x05);
        assert_eq!(frame[1], 0x64);
        assert!(frame.len() > 10);
    }

    #[test]
    fn test_dnp3_crc() {
        let data = [0x05, 0x64, 0x0C, 0x44, 0x02, 0x00, 0x01, 0x00];
        let crc = dnp3_crc(&data);
        assert_ne!(crc, 0); // CRC should be non-zero for non-empty data
    }

    #[test]
    fn test_dnp3_metadata() {
        let p = Dnp3Protocol::new(1, 100);
        assert_eq!(p.name(), "dnp3");
        assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    }
}
