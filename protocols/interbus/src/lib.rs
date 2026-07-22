// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! INTERBUS protocol via Phoenix Contact IBS controller.
//! Ring topology, 500 kbps/2 Mbps, master-slave with shift register.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct InterbusProtocol { pub slave_id: u8 }
impl InterbusProtocol { pub fn new() -> Self { InterbusProtocol { slave_id: 1 } }
    pub fn slave(mut self, id: u8) -> Self { self.slave_id = id; self } }

// INTERBUS frame: [Loopback 1B] [Data...] [CRC 2B]
// IBS Master generates a 13-bit ID cycle followed by data cycle

pub fn build_read_request(slave: u8, reg: u16) -> Vec<u8> {
    vec![0x00, 0x01, slave, (reg>>8) as u8, reg as u8, 0x00, 0x00]
}

pub fn build_write_request(slave: u8, reg: u16, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0x00, 0x02, slave, (reg>>8) as u8, reg as u8];
    f.push(data.len() as u8); f.extend_from_slice(data);
    f
}

impl Protocol for InterbusProtocol {
    fn name(&self) -> &'static str { "interbus" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["ibs"] }
}

impl ProtocolRead for InterbusProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        Ok(conn.send(&build_read_request(self.slave_id, *addr)).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for InterbusProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"interbus read ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for InterbusProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("ibs:slave{}",self.slave_id),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_request(1,0x100); assert_eq!(f.len(),7); }
    #[test] fn test_meta() { assert_eq!(InterbusProtocol::new().name(),"interbus"); }
}
