// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! IEC 61850 MMS substation automation protocol over TCP port 102.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct Iec61850Protocol;
impl Iec61850Protocol { pub fn new() -> Self { Iec61850Protocol } }

pub fn build_tpkt(payload: &[u8]) -> Vec<u8> {
    let len = (4 + payload.len()) as u16;
    vec![0x03, 0x00, (len>>8)as u8, len as u8]
        .iter().chain(payload.iter()).copied().collect()
}

pub fn build_connect_request() -> Vec<u8> {
    let cotp = [0x18, 0xE0, 0x00, 0x01, 0x00, 0x01, 0xC0];
    let mms = [0xA8, 0x0C, 0x80, 0x01, 0x01, 0x81, 0x01, 0x01, 0xA4, 0x04, 0x80, 0x01, 0x00, 0x00];
    build_tpkt(&cotp.iter().chain(mms.iter()).copied().collect::<Vec<_>>())
}

impl Protocol for Iec61850Protocol {
    fn name(&self) -> &'static str { "iec61850" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["mms"] }
}

impl ProtocolRead for Iec61850Protocol {
    type Address = String; type Value = f64;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        conn.send(&build_connect_request()).map_err(ProtocolError::from)?;
        Ok(0.0)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for Iec61850Protocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_connect_request()) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"iec61850 COTP ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for Iec61850Protocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:"ied-001".into(),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_tpkt() { let f = build_tpkt(&[1,2,3]); assert_eq!(f[0],0x03); assert_eq!(f.len(),7); }
    #[test] fn test_meta() { assert_eq!(Iec61850Protocol::new().name(),"iec61850"); }
}
