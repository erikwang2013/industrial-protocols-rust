// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # CANOPEN — CANopen CiA 301 协议。NMT 网络管理、SDO 服务数据对象、11-bit COB-ID。

//! CANopen protocol over CAN bus (CiA 301).
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct CanopenProtocol {
    pub node_id: u8,
}
impl CanopenProtocol {
    pub fn new(node_id: u8) -> Self {
        CanopenProtocol { node_id }
    }
}

// CANopen frame: COB-ID 11/29-bit + data up to 8 bytes
// NMT: COB-ID 0x000, SDO: 0x600+node, PDO: 0x180+node

pub fn build_nmt_command(cmd: u8, node_id: u8) -> Vec<u8> {
    vec![0x00, cmd, node_id]
}

pub fn build_sdo_read(node_id: u8, index: u16, subindex: u8) -> Vec<u8> {
    let cob_id: u16 = 0x600 + node_id as u16;
    vec![
        cob_id as u8,
        0x40,
        index as u8,
        (index >> 8) as u8,
        subindex,
        0x00,
        0x00,
        0x00,
        0x00,
    ]
}

pub fn parse_sdo_response(data: &[u8]) -> std::result::Result<u32, String> {
    if data.len() < 8 {
        return Err("SDO response too short".into());
    }
    let cmd = data[1];
    if cmd == 0x80 {
        return Err(format!(
            "SDO abort: {:02X}{:02X}{:02X}{:02X}",
            data[4], data[5], data[6], data[7]
        ));
    }
    match cmd {
        0x43 => Ok(u32::from_le_bytes([data[4], data[5], data[6], data[7]])),
        0x4B => Ok(u16::from_le_bytes([data[4], data[5]]) as u32),
        0x4F => Ok(data[4] as u32),
        _ => Err(format!("unexpected SDO response: 0x{cmd:02X}")),
    }
}

impl Protocol for CanopenProtocol {
    fn name(&self) -> &'static str {
        "canopen"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::Fieldbus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["cia301"]
    }
}

impl ProtocolRead for CanopenProtocol {
    type Address = u16;
    type Value = u32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_sdo_read(self.node_id, *addr, 0);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        parse_sdo_response(&resp).map_err(|e| ProtocolError::Parse {
            protocol: "canopen".into(),
            reason: e,
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

impl ProtocolHealth for CanopenProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_nmt_command(0x01, self.node_id)) {
            // NMT Start
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "canopen NMT ok".into(),
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

impl ProtocolDiagnostics for CanopenProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("canopen:{}", self.node_id),
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
    fn test_nmt() {
        let f = build_nmt_command(1, 2);
        assert_eq!(f[0], 0);
        assert_eq!(f[1], 1);
        assert_eq!(f[2], 2);
    }
    #[test]
    fn test_sdo_read() {
        let f = build_sdo_read(2, 0x1000, 0);
        assert_eq!(f[0], 0x02);
        assert_eq!(f[1], 0x40);
    }
    #[test]
    fn test_meta() {
        assert_eq!(CanopenProtocol::new(2).name(), "canopen");
    }
}
