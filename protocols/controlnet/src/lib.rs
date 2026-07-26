// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # CONTROLNET — ControlNet 协议。5 Mbps 令牌传递、UCMM 非连接消息、需 1784-PCIC/S。

//! ControlNet protocol via Allen-Bradley 1784-PCIC/S card.
//! 5 Mbps, scheduled token-passing for real-time I/O.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct ControlnetProtocol {
    pub node: u8,
}
impl Default for ControlnetProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlnetProtocol {
    pub fn new() -> Self {
        ControlnetProtocol { node: 1 }
    }
    pub fn node(mut self, n: u8) -> Self {
        self.node = n;
        self
    }
}

// ControlNet MAC frame: [Preamble 16B] [StartDelim 1B] [Source 1B] [Lpackets...] [CRC 2B] [EndDelim 1B]

pub fn build_ucmm_read(node: u8, class: u16, instance: u16, attr: u16) -> Vec<u8> {
    let mut f = vec![0xAA; 16];
    f.push(0xAB); // preamble + start
    f.push(node);
    f.push(0x52); // UCMM Unconnected Send
    f.extend_from_slice(&class.to_le_bytes());
    f.extend_from_slice(&instance.to_le_bytes());
    f.extend_from_slice(&attr.to_le_bytes());
    f.push(0x00);
    f.push(0x00); // CRC placeholder
    f
}

impl Protocol for ControlnetProtocol {
    fn name(&self) -> &'static str {
        "controlnet"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::Fieldbus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["1784-pcics"]
    }
}

impl ProtocolRead for ControlnetProtocol {
    type Address = u16;
    type Value = u16;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let class = *addr;
        let instance = 1u16;
        let attr = 1u16;
        let f = build_ucmm_read(self.node, class, instance, attr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 24 {
            u16::from_le_bytes([resp[22], resp[23]])
        } else {
            0
        })
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

impl ProtocolHealth for ControlnetProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_ucmm_read(self.node, 0x01, 1, 1)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "controlnet ucmm ok".into(),
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

impl ProtocolDiagnostics for ControlnetProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("cn:node{}", self.node),
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
    fn test_ucmm() {
        let f = build_ucmm_read(1, 0x01, 1, 1);
        assert_eq!(f[0], 0xAA);
        assert_eq!(f[16], 0xAB);
    }
    #[test]
    fn test_meta() {
        assert_eq!(ControlnetProtocol::new().name(), "controlnet");
    }
}
