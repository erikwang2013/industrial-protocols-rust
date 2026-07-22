// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! FlexRay automotive high-speed bus protocol.
//! 10 Mbps dual-channel, time-triggered. Requires FlexRay controller.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct FlexrayProtocol { pub slot: u16, pub cycle: u8 }
impl FlexrayProtocol { pub fn new() -> Self { FlexrayProtocol { slot: 1, cycle: 0 } }
    pub fn slot(mut self, s: u16) -> Self { self.slot = s; self } }

// FlexRay frame: [Header 5B] [Payload 0-254B] [CRC 3B]
// Header: Status(1B) + FrameID(2B) + Length(1B) + HeaderCRC(1B)

pub fn build_static_frame(slot: u16, cycle: u8, payload: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(5 + payload.len());
    f.push(0x00); // status
    f.extend_from_slice(&slot.to_be_bytes()); // frame ID
    f.push(payload.len() as u8); // length
    f.push(0x00); // header CRC placeholder
    f.extend_from_slice(payload);
    f
}

pub fn build_wakeup_pattern() -> Vec<u8> { vec![0xFF, 0x00, 0xFF, 0x00] }

impl Protocol for FlexrayProtocol {
    fn name(&self) -> &'static str { "flexray" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::AutomotiveBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["dual"] }
}

impl ProtocolRead for FlexrayProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Ok(conn.send(&build_static_frame(self.slot, self.cycle, &[])).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for FlexrayProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_wakeup_pattern()) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"flexray wakeup ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for FlexrayProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("flexray:{}",self.slot),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_frame() { let f = build_static_frame(1,0,&[1,2]); assert_eq!(f.len(),7); }
    #[test] fn test_meta() { assert_eq!(FlexrayProtocol::new().name(),"flexray"); }
}
