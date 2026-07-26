// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # CCLINKIE — CC-Link IE Field 协议。工业以太网版本、Cyclic Data 循环数据交换。

//! CC-Link IE Field industrial Ethernet protocol.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct CclinkIeProtocol {
    pub station_no: u16,
}
impl CclinkIeProtocol {
    pub fn new(station_no: u16) -> Self {
        CclinkIeProtocol { station_no }
    }
}

pub fn build_cyclic_data_request(station: u16) -> Vec<u8> {
    vec![
        0x03,
        0x00,
        (station >> 8) as u8,
        station as u8,
        0x10,
        0x00,
        0x01,
        0x00,
    ]
}

impl Protocol for CclinkIeProtocol {
    fn name(&self) -> &'static str {
        "cclinkie"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["field"]
    }
}

impl ProtocolRead for CclinkIeProtocol {
    type Address = u16;
    type Value = u16;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_cyclic_data_request(self.station_no);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        if resp.len() >= 10 {
            Ok(u16::from_be_bytes([resp[8], resp[9]]))
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

impl ProtocolHealth for CclinkIeProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_cyclic_data_request(self.station_no)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "cclinkie ok".into(),
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

impl ProtocolDiagnostics for CclinkIeProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("cclinkie:{}", self.station_no),
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
    fn test_frame() {
        let f = build_cyclic_data_request(1);
        assert_eq!(f.len(), 8);
        assert_eq!(f[0], 0x03);
    }
    #[test]
    fn test_meta() {
        assert_eq!(CclinkIeProtocol::new(1).name(), "cclinkie");
    }
}
