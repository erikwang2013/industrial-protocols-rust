// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # LONWORKS — LonWorks 楼宇自动化协议。LonTalk 协议栈、Neuron 芯片、网络变量读取。

//! LonWorks building automation protocol via Neuron chip / interface card.
//! Uses ISO/IEC 14908 (LonTalk) protocol stack.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct LonworksProtocol { pub subnet: u8, pub node: u8 }
impl Default for LonworksProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl LonworksProtocol { pub fn new() -> Self { LonworksProtocol { subnet: 1, node: 1 } }
    pub fn subnet(mut self, s: u8) -> Self { self.subnet = s; self } }

// LonTalk frame: [Version 1B] [PDU 1B] [Addr 3B] [Domain 1B] [NPDU...]

pub fn build_network_var_fetch(subnet: u8, node: u8, nv_index: u16) -> Vec<u8> {
    vec![0x01, 0x12, subnet, node, 0x00, (nv_index>>8) as u8, nv_index as u8, 0x61]
}

pub fn build_service_pin_message(node_id: u64) -> Vec<u8> {
    let mut f = vec![0x01, 0x92, 0x00];
    f.extend_from_slice(&node_id.to_be_bytes());
    f
}

impl Protocol for LonworksProtocol {
    fn name(&self) -> &'static str { "lonworks" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::BuildingLighting }
    fn supported_variants(&self) -> &'static [&'static str] { &["ftt10","tp1250"] }
}

impl ProtocolRead for LonworksProtocol {
    type Address = u16; type Value = f32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_network_var_fetch(self.subnet, self.node, *addr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 10 { f32::from_be_bytes([resp[8],resp[9],resp[7],resp[6]]) } else { 0.0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for LonworksProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_service_pin_message(0x010203040506)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"lonworks pin ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for LonworksProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("lon:{}/{}",self.subnet,self.node),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_nv() { let f = build_network_var_fetch(1,1,0); assert_eq!(f.len(),8); }
    #[test] fn test_meta() { assert_eq!(LonworksProtocol::new().name(),"lonworks"); }
}
