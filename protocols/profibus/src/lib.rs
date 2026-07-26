// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # PROFIBUS — PROFIBUS DP/PA/FMS 协议。SD2 帧格式、FCS 累加和校验、需 CP 5611 网关。

//! PROFIBUS DP/PA/FMS via Siemens CP 5611 / Anybus gateway.
//! Typically accessed through a TCP gateway that converts to PROFIBUS frames.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct ProfibusProtocol { pub slave_addr: u8, pub gateway_host: String }
impl Default for ProfibusProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfibusProtocol { pub fn new() -> Self { ProfibusProtocol { slave_addr: 1, gateway_host: "192.168.1.100".into() } }
    pub fn with_slave(mut self, addr: u8) -> Self { self.slave_addr = addr; self }
    pub fn with_gateway(mut self, host: &str) -> Self { self.gateway_host = host.into(); self }
}

// PROFIBUS DP frame: [SD 1B] [DA/SA 1B] [FC 1B] [Data...] [FCS 1B] [ED 1B]
// SD1=0x10 (fixed length), SD2=0x68 (variable), SD3=0xA2 (short ack), SD4=0xDC (token)

pub fn build_dp_read(slave: u8, slot: u8, index: u8, len: u8) -> Vec<u8> {
    let mut f = vec![0x68, 0x05, 0x05]; // SD2, LE, LEr
    f.push(0x80 | slave); // DA (Destination Address)
    f.push(0x00); // SA (Source Address) = master
    f.push(0x5C); // FC: Data Exchange Read
    f.push(slot); f.push(index); f.push(len);
    let fcs: u8 = f.iter().fold(0u8, |a, &b| a.wrapping_add(b));
    f.push(fcs); f.push(0x16); // ED
    f
}

pub fn build_dp_write(slave: u8, slot: u8, index: u8, data: &[u8]) -> Vec<u8> {
    let len = data.len() as u8 + 3;
    let mut f = vec![0x68, len, len, 0x80 | slave, 0x00, 0x5D, slot, index, data.len() as u8];
    f.extend_from_slice(data);
    let fcs: u8 = f.iter().fold(0u8, |a, &b| a.wrapping_add(b));
    f.push(fcs); f.push(0x16);
    f
}

pub fn build_diag_request(slave: u8) -> Vec<u8> {
    vec![0x68, 0x03, 0x03, 0x80 | slave, 0x00, 0x5A, 0xA0, 0x16]
}

impl Protocol for ProfibusProtocol {
    fn name(&self) -> &'static str { "profibus" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["dp", "pa", "fms"] }
}

impl ProtocolRead for ProfibusProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let slot = (*addr >> 8) as u8; let index = (*addr & 0xFF) as u8;
        let f = build_dp_read(self.slave_addr, slot, index, 4);
        conn.send(&f).map_err(ProtocolError::from)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolWrite for ProfibusProtocol {
    type Address = u16; type Value = Vec<u8>;
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        let slot = (*addr >> 8) as u8; let index = (*addr & 0xFF) as u8;
        conn.send(&build_dp_write(self.slave_addr, slot, index, value)).map_err(ProtocolError::from)?; Ok(())
    }
    fn write_batch(&self,c:&mut dyn Connection,i:&[(Self::Address,Self::Value)])->Result<()>{for(a,v)in i{self.write(c,a,v)?;}Ok(())}
}

impl ProtocolHealth for ProfibusProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_diag_request(self.slave_addr)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"profibus diag ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for ProfibusProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("pb:slave{}",self.slave_addr),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_diag() { let f = build_diag_request(1); assert_eq!(f[0],0x68); assert_eq!(f[3],0x81); }
    #[test] fn test_meta() { assert_eq!(ProfibusProtocol::new().name(),"profibus"); }
}
