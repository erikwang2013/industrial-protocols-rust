// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! Beckhoff Lightbus fiber optic protocol.
//! 2.5 Mbps over fiber, ring topology. Requires Beckhoff Lightbus interface.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct LightbusProtocol { pub slave_id: u16 }
impl LightbusProtocol { pub fn new() -> Self { LightbusProtocol { slave_id: 1 } }
    pub fn slave(mut self, id: u16) -> Self { self.slave_id = id; self } }

// Lightbus frame over fiber: [Start 1B] [Addr 2B] [Cmd 1B] [Len 1B] [Data...] [CRC 2B]

pub fn build_read_register(slave: u16, reg: u16, len: u8) -> Vec<u8> {
    let mut f = vec![0x02, (slave>>8) as u8, slave as u8, 0x03, len];
    f.extend_from_slice(&reg.to_le_bytes());
    f.push(0x00); f.push(0x00); // CRC
    f
}

pub fn build_write_register(slave: u16, reg: u16, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0x02, (slave>>8) as u8, slave as u8, 0x04, data.len() as u8];
    f.extend_from_slice(&reg.to_le_bytes());
    f.extend_from_slice(data);
    f.push(0x00); f.push(0x00);
    f
}

impl Protocol for LightbusProtocol {
    fn name(&self) -> &'static str { "lightbus" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::HardwareBridge }
    fn supported_variants(&self) -> &'static [&'static str] { &["ring"] }
}

impl ProtocolRead for LightbusProtocol {
    type Address = u16; type Value = u16;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_register(self.slave_id, *addr, 2);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 9 { u16::from_le_bytes([resp[6],resp[7]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolWrite for LightbusProtocol {
    type Address = u16; type Value = u16;
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        conn.send(&build_write_register(self.slave_id, *addr, &value.to_le_bytes())).map_err(ProtocolError::from)?; Ok(())
    }
    fn write_batch(&self,c:&mut dyn Connection,i:&[(Self::Address,Self::Value)])->Result<()>{for(a,v)in i{self.write(c,a,v)?;}Ok(())}
}

impl ProtocolHealth for LightbusProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"lightbus ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for LightbusProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("lb:{}",self.slave_id),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_register(1,0x100,2); assert_eq!(f[0],0x02); }
    #[test] fn test_meta() { assert_eq!(LightbusProtocol::new().name(),"lightbus"); }
}
