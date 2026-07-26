// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # HART — HART 协议。4-20mA FSK 调制解调、通用命令 0/1/3、XOR 校验。

//! HART protocol - FSK modulated over 4-20mA.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct HartProtocol {
    pub poll_addr: u8,
}

impl HartProtocol {
    pub fn new(poll_addr: u8) -> Self {
        HartProtocol { poll_addr }
    }
}

pub fn build_command(addr: u8, cmd: u8, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0xFF; 5];
    f.push(0x82);
    f.push(addr);
    f.push(cmd);
    f.push(data.len() as u8);
    f.extend_from_slice(data);
    let checksum: u8 = f[5..].iter().fold(0u8, |a, &b| a ^ b);
    f.push(checksum);
    f
}

pub fn build_read_unique_id(addr: u8) -> Vec<u8> {
    build_command(addr, 0, &[])
}
pub fn build_read_pv(addr: u8) -> Vec<u8> {
    build_command(addr, 1, &[])
}
pub fn parse_float(data: &[u8], offset: usize) -> f32 {
    if offset + 4 <= data.len() {
        f32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    } else {
        0.0
    }
}

impl Protocol for HartProtocol {
    fn name(&self) -> &'static str {
        "hart"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::Fieldbus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["fsk"]
    }
}

impl ProtocolRead for HartProtocol {
    type Address = u8;
    type Value = f32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let frame = build_command(self.poll_addr, *addr, &[]);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if resp.len() < 8 {
            return Err(ProtocolError::Parse {
                protocol: "hart".into(),
                reason: "response too short".into(),
            });
        }
        Ok(parse_float(&resp, 6))
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

impl ProtocolHealth for HartProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "hart cmd 0 ok".into(),
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

impl ProtocolDiagnostics for HartProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("hart:{}", self.poll_addr),
            uptime_secs: None,
            error_count: 0,
            last_error: None,
            custom_fields: std::collections::HashMap::from([(
                "poll_addr".into(),
                self.poll_addr.to_string(),
            )]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_frame() {
        let f = build_read_unique_id(0);
        assert_eq!(f[5], 0x82);
        assert_eq!(f[7], 0);
    }
    #[test]
    fn test_meta() {
        assert_eq!(HartProtocol::new(0).name(), "hart");
    }
}
