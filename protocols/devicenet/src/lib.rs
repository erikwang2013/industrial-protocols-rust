// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! DeviceNet protocol via Anybus DeviceNet Scanner gateway.
//! DeviceNet uses CAN at 125/250/500 kbps with CIP application layer.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct DevicenetProtocol { pub mac_id: u8 }
impl DevicenetProtocol { pub fn new() -> Self { DevicenetProtocol { mac_id: 0 } }
    pub fn with_mac(mut self, id: u8) -> Self { self.mac_id = id; self }
}

// DeviceNet frame (11-bit CAN ID): [Group 2b][MsgID 4b][MAC Src/Dst]
// Explicit messaging: Group 3 (0x6) + MsgID + MAC
// I/O messaging: Group 1 (0x4) for Poll

pub fn build_explicit_request(mac_id: u8, service: u8, class: u8, instance: u8, attr: u8) -> Vec<u8> {
    let can_id: u16 = 0x600 | (mac_id as u16); // Group 3, src=0
    let mut f = vec![(can_id>>8) as u8, can_id as u8];
    f.push(service); // Get_Attribute_Single = 0x0E
    f.push(class); f.push(instance); f.push(attr);
    f
}

pub fn build_poll_request(mac_id: u8) -> Vec<u8> {
    vec![0x04 | ((mac_id & 0x3F) << 2), mac_id, 0x00, 0x00]
}

impl Protocol for DevicenetProtocol {
    fn name(&self) -> &'static str { "devicenet" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["scanner"] }
}

impl ProtocolRead for DevicenetProtocol {
    type Address = u16; type Value = u16;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let class = (*addr >> 8) as u8; let attr = *addr as u8;
        let f = build_explicit_request(self.mac_id, 0x0E, class, 1, attr);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 6 { u16::from_le_bytes([resp[4],resp[5]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for DevicenetProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_poll_request(self.mac_id)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"devicenet poll ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for DevicenetProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("dn:{}",self.mac_id),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_poll() { let f = build_poll_request(5); assert_eq!(f[0],0x04|20); }
    #[test] fn test_req() { let f = build_explicit_request(5,0x0E,1,1,1); assert!(f.len()>=6); }
    #[test] fn test_meta() { assert_eq!(DevicenetProtocol::new().name(),"devicenet"); }
}
