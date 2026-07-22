// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! POWERLINK (Ethernet POWERLINK) real-time Ethernet protocol.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct PowerlinkProtocol { pub node_id: u8 }
impl PowerlinkProtocol { pub fn new(node_id: u8) -> Self { PowerlinkProtocol { node_id } } }

// POWERLINK frame: EtherType 0x88AB
// SoC (Start of Cycle), PReq (Poll Request), PRes (Poll Response), SoA (Start of Async)

pub fn build_ident_request(node_id: u8) -> Vec<u8> {
    vec![0x01, 0x00, 0x00, 0xAB, 0x88, node_id, 0x40, 0x00, 0x00, 0x00]
}

pub fn build_poll_request(node_id: u8) -> Vec<u8> {
    vec![0x01, node_id, 0x80, 0x00, 0x00, 0x00]
}

impl Protocol for PowerlinkProtocol {
    fn name(&self) -> &'static str { "powerlink" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["epi"] }
}

impl ProtocolRead for PowerlinkProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_poll_request(self.node_id);
        Ok(conn.send(&f).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for PowerlinkProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_ident_request(self.node_id)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"powerlink ident ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for PowerlinkProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("powerlink:{}",self.node_id),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_ident() { let f = build_ident_request(1); assert_eq!(f.len(),10); assert_eq!(f[0],1); }
    #[test] fn test_meta() { assert_eq!(PowerlinkProtocol::new(1).name(),"powerlink"); }
}
