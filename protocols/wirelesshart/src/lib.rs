//! WirelessHART (IEC 62591) wireless industrial protocol.
//! 2.4 GHz DSSS mesh. Emerson 1410/1420 Smart Wireless Gateway.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct WirelessHartProtocol { pub device_tag: String }
impl WirelessHartProtocol { pub fn new() -> Self { WirelessHartProtocol { device_tag: "TT-101".into() } }
    pub fn tag(mut self, t: &str) -> Self { self.device_tag = t.into(); self } }

// WirelessHART over gateway TCP (Emerson 1420 XML/JSON API):
// Simplified binary encapsulation: [Token 4B] [Cmd 1B] [TagLen 1B] [Tag nB] [Data...]

pub fn build_read_pv(tag: &str) -> Vec<u8> {
    let mut f = vec![0x00,0x00,0x00,0x01, 0x01, tag.len() as u8];
    f.extend_from_slice(tag.as_bytes()); f
}

pub fn build_read_device_status(tag: &str) -> Vec<u8> {
    let mut f = vec![0x00,0x00,0x00,0x02, 0x03, tag.len() as u8];
    f.extend_from_slice(tag.as_bytes()); f
}

impl Protocol for WirelessHartProtocol {
    fn name(&self) -> &'static str { "wirelesshart" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::HardwareBridge }
    fn supported_variants(&self) -> &'static [&'static str] { &["iec62591"] }
}

impl ProtocolRead for WirelessHartProtocol {
    type Address = String; type Value = f32;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_pv(&self.device_tag);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 10 { f32::from_be_bytes([resp[6],resp[7],resp[8],resp[9]]) } else { 0.0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for WirelessHartProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_read_device_status(&self.device_tag)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"whart status ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for WirelessHartProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:self.device_tag.clone(),uptime_secs:None,error_count:0,last_error:None,custom_fields:std::collections::HashMap::from([("tag".into(),self.device_tag.clone())])})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_pv() { let f = build_read_pv("TT-101"); assert_eq!(f[4],1); assert_eq!(f[5],6); }
    #[test] fn test_meta() { assert_eq!(WirelessHartProtocol::new().name(),"wirelesshart"); }
}
