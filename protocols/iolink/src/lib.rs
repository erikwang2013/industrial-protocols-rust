// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # IOLINK — IO-Link 协议。点对点串行通信、PDin/PDout 过程数据、ISDU 参数读写。

//! IO-Link protocol via ifm/Balluff IO-Link Master gateway.
//! Point-to-point serial communication at 4.8/38.4/230.4 kbps.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct IolinkProtocol { pub port: u8, pub master_host: String }
impl Default for IolinkProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl IolinkProtocol { pub fn new() -> Self { IolinkProtocol { port: 1, master_host: "192.168.1.1".into() } }
    pub fn port(mut self, p: u8) -> Self { self.port = p; self }
}

// IO-Link frame via master TCP gateway:
// [Channel 1B] [Service 1B] [Index 2B] [Subindex 1B] [Data nB]

pub fn build_read_pdin(port: u8) -> Vec<u8> { vec![port, 0x48, 0x00, 0x00, 0x00] }

pub fn build_read_param(port: u8, index: u16, subindex: u8) -> Vec<u8> {
    vec![port, 0x43, (index>>8) as u8, index as u8, subindex]
}

pub fn build_write_param(port: u8, index: u16, subindex: u8, data: &[u8]) -> Vec<u8> {
    let mut f = vec![port, 0x43, (index>>8) as u8, index as u8, subindex];
    f.extend_from_slice(data); f
}

impl Protocol for IolinkProtocol {
    fn name(&self) -> &'static str { "iolink" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["com1","com2","com3"] }
}

impl ProtocolRead for IolinkProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let index = *addr; let subidx = 0u8;
        let f = build_read_param(self.port, index, subidx);
        conn.send(&f).map_err(ProtocolError::from)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolWrite for IolinkProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        let f = build_write_param(self.port, *addr, 0, value);
        conn.send(&f).map_err(ProtocolError::from)?; Ok(())
    }
    fn write_batch(&self,c:&mut dyn Connection,i:&[(Self::Address,Self::Value)])->Result<()>{for(a,v)in i{self.write(c,a,v)?;}Ok(())}
}

impl ProtocolHealth for IolinkProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_read_pdin(self.port)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"iolink PDin ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for IolinkProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("iolink:port{}",self.port),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_pdin() { let f = build_read_pdin(1); assert_eq!(f[0],1); assert_eq!(f[1],0x48); }
    #[test] fn test_meta() { assert_eq!(IolinkProtocol::new().name(),"iolink"); }
}
