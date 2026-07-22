// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # KLINE — K-Line OBD-II 协议。ISO 9141/14230、5-baud 快速初始化、KWP2000 DTC 读取。

//! K-Line OBD-II protocol (ISO 9141/14230).
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct KlineProtocol { pub addr: u8 }
impl KlineProtocol { pub fn new(addr: u8) -> Self { KlineProtocol { addr } } }

// ISO 9141-2: 5-baud init sequence (0x33 at 5 baud) followed by fast init at 10400 baud
// ISO 14230 (KWP2000): Fast init with 25ms low + 25ms high

pub fn build_fast_init() -> Vec<u8> { vec![0x33] } // 5-baud init byte

pub fn build_read_dtc() -> Vec<u8> {
    vec![0x68, 0x6A, 0xF1, 0x01, 0x07, 0xC3] // ReadDiagnosticTroubleCodes
}

pub fn parse_kwp2000_response(data: &[u8]) -> std::result::Result<Vec<u8>, String> {
    if data.len() < 4 { return Err("KWP2000 response too short".into()); }
    if data[0] != 0x48 { return Err("invalid format byte".into()); }
    let len = data[1] as usize;
    if data.len() < len + 2 { return Err("truncated".into()); }
    Ok(data[3..len+1].to_vec())
}

impl Protocol for KlineProtocol {
    fn name(&self) -> &'static str { "kline" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::AutomotiveBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["iso9141", "iso14230"] }
}

impl ProtocolRead for KlineProtocol {
    type Address = u8; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        conn.send(&build_fast_init()).map_err(ProtocolError::from)?;
        let f = build_read_dtc();
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        parse_kwp2000_response(&resp).map_err(|e| ProtocolError::Parse{protocol:"kline".into(),reason:e})
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for KlineProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_fast_init()) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"kline init ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for KlineProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("kline:{}",self.addr),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_init() { let f = build_fast_init(); assert_eq!(f[0],0x33); }
    #[test] fn test_meta() { assert_eq!(KlineProtocol::new(0x33).name(),"kline"); }
}
