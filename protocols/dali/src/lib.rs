// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # DALI — DALI 数字照明协议。IEC 62386、ARC 电弧功率控制、广播命令。

//! DALI (Digital Addressable Lighting Interface) protocol.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct DaliProtocol {
    pub short_addr: u8,
}
impl DaliProtocol {
    pub fn new(short_addr: u8) -> Self {
        DaliProtocol { short_addr }
    }
}

// DALI frame: [Address 1B] [Command 1B] (Manchester encoded at 1200 bps)
// 16-bit forward frame: [Start bit] [Addr 8b] [Data 8b] [Stop bits]
// 8-bit backward frame: [Start bit] [Data 8b] [Stop bits]

pub fn build_forward_frame(addr: u8, cmd: u8) -> Vec<u8> {
    vec![addr, cmd]
}

pub fn build_arc_power(addr: u8, level: u8) -> Vec<u8> {
    build_forward_frame(addr << 1 | 0x01, level.min(254))
}

pub fn build_query_status(addr: u8) -> Vec<u8> {
    build_forward_frame(addr << 1 | 0x01, 0x90)
}

pub fn build_off() -> Vec<u8> {
    vec![0xFE, 0x00]
} // broadcast off
pub fn build_max() -> Vec<u8> {
    vec![0xFE, 0x05]
} // broadcast max

impl Protocol for DaliProtocol {
    fn name(&self) -> &'static str {
        "dali"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::BuildingLighting
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["iec62386"]
    }
}

impl ProtocolRead for DaliProtocol {
    type Address = u8;
    type Value = u8;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_query_status(self.short_addr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 2 { resp[1] } else { 0 })
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

impl ProtocolWrite for DaliProtocol {
    type Address = u8;
    type Value = u8;
    fn write(
        &self,
        conn: &mut dyn Connection,
        _addr: &Self::Address,
        value: &Self::Value,
    ) -> Result<()> {
        let f = build_arc_power(self.short_addr, *value);
        conn.send(&f).map_err(ProtocolError::from)?;
        Ok(())
    }
    fn write_batch(
        &self,
        c: &mut dyn Connection,
        i: &[(Self::Address, Self::Value)],
    ) -> Result<()> {
        for (a, v) in i {
            self.write(c, a, v)?;
        }
        Ok(())
    }
}

impl ProtocolHealth for DaliProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(v) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("dali status={v}"),
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

impl ProtocolDiagnostics for DaliProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("dali:{}", self.short_addr),
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
    fn test_off() {
        assert_eq!(build_off(), vec![0xFE, 0x00]);
    }
    #[test]
    fn test_max() {
        assert_eq!(build_max(), vec![0xFE, 0x05]);
    }
    #[test]
    fn test_meta() {
        assert_eq!(DaliProtocol::new(0).name(), "dali");
    }
}
