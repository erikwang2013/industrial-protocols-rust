//! MOST (Media Oriented Systems Transport) fiber optic protocol.
//! 25/50/150 Mbps over POF. Requires MOST network interface controller.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct MostProtocol { pub node_addr: u16 }
impl MostProtocol { pub fn new() -> Self { MostProtocol { node_addr: 0x0100 } }
    pub fn addr(mut self, a: u16) -> Self { self.node_addr = a; self } }

// MOST25 frame: synchronous at 44.1 kHz, 64 bytes per frame
// MOST150 frame: isochronous + packet data

pub fn build_control_message(target: u16, fkt_id: u16, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0x01, 0x00]; // preamble
    f.extend_from_slice(&target.to_be_bytes());
    f.extend_from_slice(&fkt_id.to_be_bytes());
    f.push(data.len() as u8);
    f.extend_from_slice(data);
    f
}

pub fn build_netblock_read(target: u16, offset: u32) -> Vec<u8> {
    let mut f = vec![0x01, 0x02];
    f.extend_from_slice(&target.to_be_bytes());
    f.extend_from_slice(&offset.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x04]); // length
    f
}

impl Protocol for MostProtocol {
    fn name(&self) -> &'static str { "most" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::AutomotiveBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["25","50","150"] }
}

impl ProtocolRead for MostProtocol {
    type Address = u32; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        Ok(conn.send(&build_netblock_read(self.node_addr, *addr)).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for MostProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_control_message(self.node_addr, 0x0101, &[])) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"most control ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for MostProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("most:{}",self.node_addr),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_ctrl() { let f = build_control_message(0x100,0x0101,&[]); assert_eq!(f[0],1); }
    #[test] fn test_meta() { assert_eq!(MostProtocol::new().name(),"most"); }
}
