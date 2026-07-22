// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # HARTIP — HART-IP 协议。HART over TCP/UDP、端口 5094、Init/ReadPV 服务。

//! HART-IP protocol - HART over TCP/UDP port 5094.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct HartIpProtocol;
impl HartIpProtocol { pub fn new() -> Self { HartIpProtocol } }

// HART-IP header: [Version 1B] [MsgType 1B] [MsgID 2B] [Status 2B] [SeqNo 2B] [Length 2B]
pub fn build_hartip_init() -> Vec<u8> {
    vec![0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

pub fn build_hartip_read_pv() -> Vec<u8> {
    let header = [0x01, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x05];
    let body = [0x82, 0x00, 0x01, 0x00, 0x01]; // HART command 1 + checksum
    header.iter().chain(body.iter()).copied().collect()
}

impl Protocol for HartIpProtocol {
    fn name(&self) -> &'static str { "hartip" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IoTMessaging }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp", "udp"] }
}

impl ProtocolRead for HartIpProtocol {
    type Address = String; type Value = f32;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_hartip_read_pv();
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        if resp.len() >= 20 { Ok(f32::from_be_bytes([resp[16],resp[17],resp[18],resp[19]])) } else { Ok(0.0) }
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for HartIpProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_hartip_init()) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"hartip init ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for HartIpProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:"hartip-device".into(),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_init() { let f = build_hartip_init(); assert_eq!(f.len(),10); assert_eq!(f[0],1); }
    #[test] fn test_meta() { assert_eq!(HartIpProtocol::new().name(),"hartip"); }
}
