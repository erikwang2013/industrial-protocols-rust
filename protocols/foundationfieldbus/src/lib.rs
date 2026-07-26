// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # FOUNDATIONFIELDBUS — Foundation Fieldbus H1/HSE 协议。前导码+定界符 DLL、需 NI USB-8486。

//! Foundation Fieldbus H1/HSE via NI USB-8486 gateway.
//! H1: 31.25 kbps over twisted pair. HSE: 100 Mbps Ethernet.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct FfProtocol {
    pub device_id: u32,
    pub variant: FfVariant,
}
#[derive(Debug, Clone, Copy)]
pub enum FfVariant {
    H1,
    Hse,
}
impl Default for FfProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl FfProtocol {
    pub fn new() -> Self {
        FfProtocol {
            device_id: 0x001234,
            variant: FfVariant::H1,
        }
    }
    pub fn with_id(mut self, id: u32) -> Self {
        self.device_id = id;
        self
    }
}

// FF H1: [Preamble 2B] [Delimiter 1B] [DLL 1B] [Data...] [CRC 2B]
// FF HSE: TCP/IP encapsulated

pub fn build_h1_read(device: u32, tag: &str) -> Vec<u8> {
    let mut f = vec![0xAA, 0xAA, 0x86]; // preamble + delimiter
    f.push(0x04); // DLL: data link layer
    f.extend_from_slice(&device.to_be_bytes());
    f.extend_from_slice(tag.as_bytes());
    f
}

pub fn build_hse_read_request(device: u32, index: u16) -> Vec<u8> {
    vec![
        0x01,
        0x00,
        (device >> 16) as u8,
        device as u8,
        (index >> 8) as u8,
        index as u8,
        0x00,
        0x04,
    ]
}

impl Protocol for FfProtocol {
    fn name(&self) -> &'static str {
        "foundationfieldbus"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::Fieldbus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["h1", "hse"]
    }
}

impl ProtocolRead for FfProtocol {
    type Address = u16;
    type Value = f32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_hse_read_request(self.device_id, *addr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        if resp.len() >= 10 {
            Ok(f32::from_be_bytes([resp[6], resp[7], resp[8], resp[9]]))
        } else {
            Ok(0.0)
        }
    }
    fn read_batch(&self, c: &mut dyn Connection, a: &[Self::Address]) -> Result<Vec<Self::Value>> {
        a.iter().map(|x| self.read(c, x)).collect()
    }
    fn read_range(
        &self,
        c: &mut dyn Connection,
        s: &Self::Address,
        n: u16,
    ) -> Result<Vec<Self::Value>> {
        (0..n).map(|_| self.read(c, s)).collect()
    }
}

impl ProtocolHealth for FfProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_hse_read_request(self.device_id, 0)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "FF HSE read ok".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: 0,
                last_seen: std::time::Instant::now(),
                details: format!("{e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for FfProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("ff:{}", self.device_id),
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
    fn test_h1() {
        let f = build_h1_read(0x1234, "TAG");
        assert_eq!(f[0], 0xAA);
        assert_eq!(f[2], 0x86);
    }
    #[test]
    fn test_meta() {
        assert_eq!(FfProtocol::new().name(), "foundationfieldbus");
    }
}
