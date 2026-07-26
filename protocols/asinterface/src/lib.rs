// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # ASINTERFACE — AS-Interface V3 协议。Data Exchange 数据交换、Read ID Code、需 B+W 网关。

//! AS-Interface (Actuator Sensor Interface) via Bihl+Wiedemann gateway.
//! Master-slave polling at 167 kbps, up to 62 slaves.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct AsiProtocol { pub slave_addr: u8 }
impl Default for AsiProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl AsiProtocol { pub fn new() -> Self { AsiProtocol { slave_addr: 0 } }
    pub fn slave(mut self, addr: u8) -> Self { self.slave_addr = addr; self }
}

// AS-i frame: [Master Request 14b] → [Slave Response 7b]
// Request: [ST 1b][Control 1b] [Addr 5b] [Info 5b] [Parity 1b][EB 1b]

pub fn build_data_exchange(addr: u8, data: u8) -> Vec<u8> {
    let st = 0x00; // start bit
    let control = 0x01; // data exchange
    vec![st, control, addr & 0x1F, data & 0x0F, 0x00]
}

pub fn build_read_id(addr: u8) -> Vec<u8> {
    vec![0x00, 0x03, addr & 0x1F, 0x0A, 0x00] // Read ID Code
}

impl Protocol for AsiProtocol {
    fn name(&self) -> &'static str { "asinterface" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["v3"] }
}

impl ProtocolRead for AsiProtocol {
    type Address = u8; type Value = u8;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_data_exchange(self.slave_addr, 0);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 4 { resp[3] } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolWrite for AsiProtocol {
    type Address = u8; type Value = u8;
    fn write(&self, conn: &mut dyn Connection, _addr: &Self::Address, value: &Self::Value) -> Result<()> {
        conn.send(&build_data_exchange(self.slave_addr, *value)).map_err(ProtocolError::from)?; Ok(())
    }
    fn write_batch(&self,c:&mut dyn Connection,i:&[(Self::Address,Self::Value)])->Result<()>{for(a,v)in i{self.write(c,a,v)?;}Ok(())}
}

impl ProtocolHealth for AsiProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_read_id(self.slave_addr)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"asi read id ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for AsiProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("asi:{}",self.slave_addr),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_exchange() { let f = build_data_exchange(1,0); assert_eq!(f.len(),5); }
    #[test] fn test_meta() { assert_eq!(AsiProtocol::new().name(),"asinterface"); }
}
