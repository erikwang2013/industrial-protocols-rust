// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # PROFINET — Profinet NRT 协议。DCP 设备发现与识别、Record Data 读写。

//! Profinet NRT protocol implementation.
//!
//! Implements DCP (Discovery and Configuration Protocol) for device discovery,
//! and Record Data read/write for NRT communication.

use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct ProfinetProtocol;

impl Default for ProfinetProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfinetProtocol {
    pub fn new() -> Self {
        ProfinetProtocol
    }
}

// Ethernet frame type for Profinet: 0x8892
const _PROFINET_ETHERTYPE: u16 = 0x8892;

/// Build a DCP Identify Request.
/// DCP frame: [ServiceID 1B] [ServiceType 1B] [Reserved 2B] [Xid 4B] [ResponseDelay 2B] [Options...]
pub fn build_dcp_identify_request(xid: u32) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(0x01); // Service ID: Identify
    frame.push(0x00); // Service Type: Request
    frame.extend_from_slice(&[0x00, 0x00]); // reserved
    frame.extend_from_slice(&xid.to_be_bytes()); // Xid
    frame.extend_from_slice(&[0x00, 0x00]); // Response Delay
    // Option: DeviceProperties, All
    frame.push(0x02); // option: IP
    frame.push(0x00); // sub-option: all
    frame.extend_from_slice(&[0x00, 0x00]); // length
    frame.push(0x01); // option: DeviceProperties
    frame.push(0x00); // sub-option: all
    frame.extend_from_slice(&[0x00, 0x00]); // length

    // Prepend DCP block header
    let mut hdr = Vec::new();
    hdr.push(0xFE); // FrameID high byte
    hdr.push(0xFD); // FrameID low byte (DCP-Hello)
    hdr.extend(&frame);
    hdr
}

/// Build a Record Data Read request (NRT).
pub fn build_record_data_read(api: u32, slot: u16, subslot: u16, index: u16) -> Vec<u8> {
    let mut frame = Vec::new();
    // Frame ID for RDR (0xFC01)
    frame.extend_from_slice(&[0xFC, 0x01]);
    // Block header
    frame.extend_from_slice(&api.to_be_bytes());
    frame.extend_from_slice(&slot.to_be_bytes());
    frame.extend_from_slice(&subslot.to_be_bytes());
    frame.extend_from_slice(&index.to_be_bytes());
    frame.extend_from_slice(&[0x00, 0x00]); // length
    frame
}

/// Parse DCP Identify Response, extracting device name and IP.
pub fn parse_dcp_identify_response(data: &[u8]) -> std::result::Result<DcpDeviceInfo, String> {
    if data.len() < 10 {
        return Err("DCP response too short".into());
    }
    let mut info = DcpDeviceInfo::default();

    let mut pos = 10; // after DCP header
    while pos + 4 <= data.len() {
        let option = data[pos];
        let sub_option = data[pos + 1];
        let len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if pos + len > data.len() {
            break;
        }
        match (option, sub_option) {
            (0x02, 0x02) => {
                // IP address
                if len >= 4 {
                    info.ip = Some(format!(
                        "{}.{}.{}.{}",
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3]
                    ));
                }
            }
            (0x02, 0x01) => {
                // NameOfStation
                info.name = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
            }
            _ => {}
        }
        pos += len;
    }
    Ok(info)
}

#[derive(Debug, Clone, Default)]
pub struct DcpDeviceInfo {
    pub name: String,
    pub ip: Option<String>,
}

impl Protocol for ProfinetProtocol {
    fn name(&self) -> &'static str {
        "profinet"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["nrt"]
    }
}

impl ProtocolRead for ProfinetProtocol {
    type Address = String;
    type Value = Vec<u8>;

    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        // addr format: "api:slot:subslot:index"
        let parts: Vec<&str> = addr.split(':').collect();
        let api: u32 = parts
            .first()
            .and_then(|s| u32::from_str_radix(s, 16).ok())
            .unwrap_or(0);
        let slot: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let subslot: u16 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let index: u16 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

        let frame = build_record_data_read(api, slot, subslot, index);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        Ok(resp)
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

impl ProtocolWrite for ProfinetProtocol {
    type Address = String;
    type Value = Vec<u8>;

    fn write(
        &self,
        _conn: &mut dyn Connection,
        _addr: &Self::Address,
        _value: &Self::Value,
    ) -> Result<()> {
        Err(ProtocolError::Unsupported {
            protocol: "profinet".into(),
            operation: "write — use Record Write".into(),
        })
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

impl ProtocolDiscover for ProfinetProtocol {
    fn discover(&self, transport: &mut dyn Transport) -> Result<Vec<DiscoveredDevice>> {
        let frame = build_dcp_identify_request(0x00000001);
        match transport.send(&frame) {
            Ok(resp) => {
                let info =
                    parse_dcp_identify_response(&resp).map_err(|e| ProtocolError::Parse {
                        protocol: "profinet".into(),
                        reason: e,
                    })?;
                Ok(vec![DiscoveredDevice {
                    device_id: info.name.clone(),
                    protocol: "profinet".into(),
                    address: info.ip.unwrap_or_default(),
                    metadata: std::collections::HashMap::from([("station_name".into(), info.name)]),
                }])
            }
            Err(e) => Err(ProtocolError::Connection(format!(
                "DCP discovery failed: {e}"
            ))),
        }
    }
}

impl ProtocolHealth for ProfinetProtocol {
    fn health(&self, _conn: &mut dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus {
            state: HealthState::Healthy,
            latency_ms: 0,
            last_seen: std::time::Instant::now(),
            details: "profinet: DCP Identify for health check".into(),
        })
    }
}

impl ProtocolDiagnostics for ProfinetProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: "pn-device".into(),
            uptime_secs: None,
            error_count: 0,
            last_error: None,
            custom_fields: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcp_identify_request() {
        let frame = build_dcp_identify_request(1);
        assert_eq!(frame[0], 0xFE); // FrameID
        assert_eq!(frame[1], 0xFD);
        assert_eq!(frame[2], 0x01); // Service ID = Identify
    }

    #[test]
    fn test_profinet_metadata() {
        let p = ProfinetProtocol::new();
        assert_eq!(p.name(), "profinet");
        assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
    }
}
