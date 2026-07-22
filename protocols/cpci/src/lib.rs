// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! CompactPCI rack-mount PCI protocol.
//! 3U/6U Eurocard, PCI bus over backplane. Requires CompactPCI interface.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct CpciProtocol { pub slot: u8, pub bus: u8 }
impl CpciProtocol { pub fn new() -> Self { CpciProtocol { slot: 1, bus: 0 } }
    pub fn slot(mut self, s: u8) -> Self { self.slot = s; self } }

// CompactPCI uses same config space as PCI plus geographic addressing:
// [Slot 1B] [Bus 1B] [PCI Bus 1B] [Device 1B] [Function 1B] [Register 1B]

pub fn build_cpci_config_read(slot: u8, bus: u8, dev: u8, func: u8, reg: u8) -> Vec<u8> {
    vec![slot, bus, bus, dev, func, reg]
}

pub fn build_cpci_slot_identify(slot: u8) -> Vec<u8> {
    vec![slot, 0xFF, 0x00, 0x00, 0x00, 0x00] // Geographic addressing
}

impl Protocol for CpciProtocol {
    fn name(&self) -> &'static str { "cpci" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::SystemBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["3u","6u"] }
}

impl ProtocolRead for CpciProtocol {
    type Address = u8; type Value = u32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_cpci_config_read(self.slot, self.bus, 0, 0, *addr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 10 { u32::from_le_bytes([resp[6],resp[7],resp[8],resp[9]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for CpciProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_cpci_slot_identify(self.slot)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"cpci identify ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for CpciProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("cpci:slot{}",self.slot),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_cfg() { let f = build_cpci_config_read(1,0,0,0,0); assert_eq!(f.len(),6); assert_eq!(f[0],1); }
    #[test] fn test_meta() { assert_eq!(CpciProtocol::new().name(),"cpci"); }
}
