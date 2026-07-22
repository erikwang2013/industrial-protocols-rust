//! EtherNet/IP protocol implementation.
//!
//! Implements ENIP (EtherNet/IP) session management and
//! CIP (Common Industrial Protocol) Read Tag service.

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct EthernetIpProtocol {
    pub slot: u8,
    session_handle: AtomicU32,
}

impl EthernetIpProtocol {
    pub fn new(slot: u8) -> Self {
        EthernetIpProtocol { slot, session_handle: AtomicU32::new(0) }
    }
}

// ENIP encapsulation header: [Cmd 2B] [Len 2B] [Session 4B] [Status 4B] [Context 8B] [Options 4B]
const ENIP_HEADER_SIZE: usize = 24;

fn build_enip_header(cmd: u16, session: u32, data_len: u16) -> Vec<u8> {
    let mut hdr = Vec::with_capacity(ENIP_HEADER_SIZE);
    hdr.extend_from_slice(&cmd.to_le_bytes());
    hdr.extend_from_slice(&data_len.to_le_bytes());
    hdr.extend_from_slice(&session.to_le_bytes());
    hdr.extend_from_slice(&0u32.to_le_bytes()); // status
    hdr.extend_from_slice(&0u64.to_le_bytes()); // context
    hdr.extend_from_slice(&0u32.to_le_bytes()); // options
    hdr
}

/// Build ENIP Register Session request.
pub fn build_register_session() -> Vec<u8> {
    let mut frame = build_enip_header(0x0065, 0, 4);
    frame.extend_from_slice(&1u16.to_le_bytes()); // protocol version
    frame.extend_from_slice(&0u16.to_le_bytes()); // options flags
    frame
}

/// Parse Register Session response, returning the session handle.
pub fn parse_register_session_response(data: &[u8]) -> std::result::Result<u32, String> {
    if data.len() < 28 { return Err("RegisterSession response too short".into()); }
    let session = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if session == 0 { return Err("RegisterSession failed: got session 0".into()); }
    Ok(session)
}

/// Build a CIP Read Tag request (connected, using UCMM).
/// Tag path: [Segments...]
pub fn build_cip_read_tag(session: u32, tag_name: &str) -> Vec<u8> {
    // CIP request: [Service 1B] [Path Size 1B] [Path...] [Elements 2B]
    let tag_bytes = tag_name.as_bytes();
    let cip_path_size = 1 + tag_bytes.len() + 1; // ANCI extended symbol + name + pad
    let mut cip = Vec::with_capacity(6 + cip_path_size);
    cip.push(0x4C); // CIP Read Tag service
    cip.push((cip_path_size / 2) as u8); // path size in words
    // Symbolic segment (ANCI extended)
    cip.push(0x91); // symbolic segment type
    cip.push(tag_bytes.len() as u8); // length
    cip.extend_from_slice(tag_bytes);
    if tag_bytes.len() % 2 == 1 { cip.push(0x00); } // pad to word align
    cip.extend_from_slice(&1u16.to_le_bytes()); // number of elements

    // ENIP SendRRData command
    let mut frame = build_enip_header(0x006F, session, (6 + cip.len()) as u16);
    frame.extend_from_slice(&0u32.to_le_bytes()); // interface handle
    frame.extend_from_slice(&0u16.to_le_bytes()); // timeout
    frame.extend_from_slice(&(cip.len() as u16).to_le_bytes()); // item count (2)
    frame.extend_from_slice(&0x00B2u16.to_le_bytes()); // unconnected data item
    frame.extend_from_slice(&(cip.len() as u16).to_le_bytes()); // item length
    frame.extend(&cip);
    frame
}

/// Parse CIP Read Tag response.
pub fn parse_cip_read_tag_response(data: &[u8]) -> std::result::Result<Vec<u8>, String> {
    // Skip ENIP header (24) + interface(4) + timeout(2) + item count(2) + type(2) + length(2) = 36
    if data.len() < 38 { return Err("CIP response too short".into()); }
    let item_len = u16::from_le_bytes([data[34], data[35]]) as usize;
    let cip_start = 36;
    let cip_end = cip_start + item_len;
    if data.len() < cip_end { return Err("CIP response truncated".into()); }
    let cip = &data[cip_start..cip_end];
    // CIP response: [Service 1B] [Status 1B] [Type 1B] [Value...]
    if cip.len() < 3 { return Err("CIP data too short".into()); }
    if cip[1] != 0x00 { return Err(format!("CIP error: status 0x{:02X}", cip[1])); }
    // Return the value bytes (skip service + status + data type)
    Ok(cip[3..].to_vec())
}

impl Protocol for EthernetIpProtocol {
    fn name(&self) -> &'static str { "ethernet-ip" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
}

impl ProtocolRead for EthernetIpProtocol {
    type Address = String;
    type Value = EthernetIpValue;

    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        // Register session first if needed
        if self.session_handle.load(Ordering::SeqCst) == 0 {
            let reg = build_register_session();
            let resp = conn.send(&reg).map_err(ProtocolError::from)?;
            let session = parse_register_session_response(&resp)
                .map_err(|e| ProtocolError::Parse { protocol: "ethernet-ip".into(), reason: e })?;
            self.session_handle.store(session, Ordering::SeqCst);
        }

        let session = self.session_handle.load(Ordering::SeqCst);
        let frame = build_cip_read_tag(session, addr);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        let data = parse_cip_read_tag_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "ethernet-ip".into(), reason: e })?;

        // Try to parse as INT
        if data.len() >= 2 {
            let val = i16::from_le_bytes([data[0], data[1]]);
            return Ok(EthernetIpValue::Int(val));
        }
        Ok(EthernetIpValue::Bool(false))
    }

    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        addrs.iter().map(|a| self.read(conn, a)).collect()
    }

    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        (0..count).map(|_| self.read(conn, start)).collect()
    }
}

impl ProtocolWrite for EthernetIpProtocol {
    type Address = String;
    type Value = EthernetIpValue;

    fn write(&self, _conn: &mut dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "ethernet-ip".into(), operation: "write — use CIP Write Tag".into() })
    }

    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        for (a, v) in items { self.write(conn, a, v)?; }
        Ok(())
    }
}

impl ProtocolHealth for EthernetIpProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        // Register session to check connectivity
        let reg = build_register_session();
        match conn.send(&reg) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "ethernet-ip: RegisterSession ok".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("EIP health check failed: {e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for EthernetIpProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("enip:slot{}", self.slot),
            uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("slot".into(), self.slot.to_string()),
                ("session".into(), self.session_handle.load(Ordering::SeqCst).to_string()),
            ]),
        })
    }
}

#[derive(Debug, Clone)]
pub enum EthernetIpValue { Int(i16), DInt(i32), Real(f32), Bool(bool), String(String) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_session_frame() {
        let frame = build_register_session();
        assert_eq!(frame.len(), 28); // ENIP header(24) + prot_ver(2) + flags(2)
        let cmd = u16::from_le_bytes([frame[0], frame[1]]);
        assert_eq!(cmd, 0x0065);
    }

    #[test]
    fn test_eip_metadata() {
        let p = EthernetIpProtocol::new(0);
        assert_eq!(p.name(), "ethernet-ip");
    }

    #[test]
    fn test_cip_read_tag_frame() {
        let frame = build_cip_read_tag(0x12345678, "MyTag");
        assert!(frame.len() > ENIP_HEADER_SIZE);
        assert_eq!(frame[0], 0x6F); // SendRRData command low byte
    }
}
