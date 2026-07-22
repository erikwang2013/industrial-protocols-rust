//! Modbus Plus (MB+) via Schneider SA85/BM85 adapter.
//! Token-passing peer-to-peer at 1 Mbps over RS-485.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct ModbusplusProtocol { pub station: u8 }
impl ModbusplusProtocol { pub fn new() -> Self { ModbusplusProtocol { station: 1 } }
    pub fn station(mut self, s: u8) -> Self { self.station = s; self } }

// MB+ frame: [Preamble 1B] [SD 1B] [Dest 1B] [Src 1B] [LLC 1B] [Data...] [CRC 2B]

pub fn build_read_request(station: u8, dest: u8, ref_addr: u16, count: u16) -> Vec<u8> {
    let mut f = vec![0xFF, 0x68, dest, station, 0x03];
    f.extend_from_slice(&ref_addr.to_be_bytes());
    f.extend_from_slice(&count.to_be_bytes());
    f.push(0x00); f.push(0x00);
    f
}

pub fn build_global_data_read(station: u8) -> Vec<u8> {
    vec![0xFF, 0x68, 0xFF, station, 0x80, 0x00, 0x00, 0x00]
}

impl Protocol for ModbusplusProtocol {
    fn name(&self) -> &'static str { "modbusplus" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::HardwareBridge }
    fn supported_variants(&self) -> &'static [&'static str] { &["sa85","bm85"] }
}

impl ProtocolRead for ModbusplusProtocol {
    type Address = u16; type Value = u16;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_request(self.station, self.station + 1, *addr, 1);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 10 { u16::from_be_bytes([resp[6],resp[7]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for ModbusplusProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_global_data_read(self.station)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"modbus+ ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for ModbusplusProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("mb+:{}",self.station),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_request(1,2,0x100,1); assert_eq!(f[0],0xFF); }
    #[test] fn test_meta() { assert_eq!(ModbusplusProtocol::new().name(),"modbusplus"); }
}
