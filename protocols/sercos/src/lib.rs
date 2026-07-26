// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # SERCOS — SERCOS III 协议。MDT/AT 报文、Service Channel 服务通道 IDN 读写。

//! SERCOS III real-time Ethernet protocol.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct SercosProtocol {
    pub slave_id: u16,
}
impl SercosProtocol {
    pub fn new(slave_id: u16) -> Self {
        SercosProtocol { slave_id }
    }
}

// SERCOS III: Master Data Telegram (MDT) + Acknowledge Telegram (AT)
// Frame: [Telegram header] [Service channel] [Cyclic data]

pub fn build_mdt_phase_up(phase: u8) -> Vec<u8> {
    vec![0x80, 0x01, phase, 0x00, 0x00, 0x00, 0x00, 0x00]
}

pub fn build_service_channel_read(slave: u16, idn: u16, element: u8) -> Vec<u8> {
    vec![
        0x00,
        (slave >> 8) as u8,
        slave as u8,
        (idn >> 8) as u8,
        idn as u8,
        element,
        0x80,
        0x00,
    ]
}

impl Protocol for SercosProtocol {
    fn name(&self) -> &'static str {
        "sercos"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["iii"]
    }
}

impl ProtocolRead for SercosProtocol {
    type Address = u16;
    type Value = u32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_service_channel_read(self.slave_id, *addr, 0);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        if resp.len() >= 8 {
            Ok(u32::from_be_bytes([resp[4], resp[5], resp[6], resp[7]]))
        } else {
            Ok(0)
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

impl ProtocolHealth for SercosProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_mdt_phase_up(4)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "sercos phase ok".into(),
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

impl ProtocolDiagnostics for SercosProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("sercos:{}", self.slave_id),
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
    fn test_phase() {
        let f = build_mdt_phase_up(4);
        assert_eq!(f[0], 0x80);
        assert_eq!(f[2], 4);
    }
    #[test]
    fn test_meta() {
        assert_eq!(SercosProtocol::new(1).name(), "sercos");
    }
}
