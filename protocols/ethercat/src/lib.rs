// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # ETHERCAT — EtherCAT 协议。BRD/APRD 数据报、EtherType 0x88A4、需 SOEM/TwinCAT。

//! EtherCAT (Ethernet for Control Automation Technology) protocol.
//! Uses standard Ethernet frames with EtherType 0x88A4.

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct EthercatProtocol {
    pub slave_addr: u16,
}
impl EthercatProtocol {
    pub fn new(slave_addr: u16) -> Self {
        EthercatProtocol { slave_addr }
    }
}

// EtherCAT frame: [Ethernet header] [EtherCAT header 2B] [Datagrams...]
// EtherType: 0x88A4
// Datagram: [Cmd 1B] [Idx 1B] [Addr 4B] [Len 2B] [IRQ 2B] [Data...] [WKC 2B]

pub fn build_brd(address: u32, data: &[u8]) -> Vec<u8> {
    let mut f = Vec::new();
    f.push(0x07); // Cmd: BRD (Broadcast Read)
    f.push(0x01); // Index
    f.extend_from_slice(&address.to_le_bytes());
    let len = data.len() as u16;
    f.extend_from_slice(&(if len == 0 { 1 } else { len }).to_le_bytes());
    f.extend_from_slice(&0u16.to_le_bytes()); // IRQ
    if !data.is_empty() {
        f.extend_from_slice(data);
    }
    f.extend_from_slice(&0u16.to_le_bytes()); // Working Counter
    f
}

pub fn build_read_reg(slave: u16, address: u32, len: u16) -> Vec<u8> {
    let mut f = Vec::new();
    let phys_addr = ((slave as u32) << 16) | (address & 0xFFFF);
    f.push(0x04); // Cmd: APRD (Auto Increment Physical Read)
    f.push(0x00);
    f.extend_from_slice(&phys_addr.to_le_bytes());
    f.extend_from_slice(&len.to_le_bytes());
    f.extend_from_slice(&0u16.to_le_bytes()); // IRQ
    f.extend_from_slice(&0u16.to_le_bytes()); // WKC
    f
}

impl Protocol for EthercatProtocol {
    fn name(&self) -> &'static str {
        "ethercat"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["soem"]
    }
}

impl ProtocolRead for EthercatProtocol {
    type Address = u32;
    type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_reg(self.slave_addr, *addr, 4);
        conn.send(&f).map_err(ProtocolError::from)
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

impl ProtocolHealth for EthercatProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_brd(0x0130, &[])) {
            // Read AL Status
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "ethercat BRD ok".into(),
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

impl ProtocolDiagnostics for EthercatProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("ethercat:{}", self.slave_addr),
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
    fn test_brd() {
        let f = build_brd(0x0130, &[]);
        assert_eq!(f[0], 0x07);
        assert_eq!(f.len(), 12);
    }
    #[test]
    fn test_meta() {
        assert_eq!(EthercatProtocol::new(0).name(), "ethercat");
    }
}
