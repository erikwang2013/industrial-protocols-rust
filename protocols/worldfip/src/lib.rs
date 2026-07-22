//! WorldFIP/Fipio fieldbus protocol.
//! 31.25 kbps / 1 Mbps / 2.5 Mbps over twisted pair or fiber.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct WorldfipProtocol { pub station: u8 }
impl WorldfipProtocol { pub fn new() -> Self { WorldfipProtocol { station: 1 } }
    pub fn station(mut self, s: u8) -> Self { self.station = s; self } }

// WorldFIP frame: [FCS 1B] [Control 1B] [Address 3B] [Data...] [FCS 2B]
// Uses Manchester encoding at physical layer

pub fn build_read_object(station: u8, object_id: u16) -> Vec<u8> {
    let mut f = vec![0x80, 0x04, 0x00, station, 0x00];
    f.extend_from_slice(&object_id.to_be_bytes());
    f
}

pub fn build_write_object(station: u8, object_id: u16, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0x80, 0x05, 0x00, station, 0x00];
    f.extend_from_slice(&object_id.to_be_bytes());
    f.extend_from_slice(data);
    f
}

impl Protocol for WorldfipProtocol {
    fn name(&self) -> &'static str { "worldfip" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["fip","fipio"] }
}

impl ProtocolRead for WorldfipProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        Ok(conn.send(&build_read_object(self.station, *addr)).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for WorldfipProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_read_object(self.station, 0)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"worldfip read ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for WorldfipProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("wfip:{}",self.station),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_object(1,0x100); assert_eq!(f.len(),7); }
    #[test] fn test_meta() { assert_eq!(WorldfipProtocol::new().name(),"worldfip"); }
}
