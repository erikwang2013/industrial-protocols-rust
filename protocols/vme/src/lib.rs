// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # VME — VME/VPX 工控背板协议。A16/A24/A32 地址空间、AM 地址修饰码。

//! VME/VPX industrial backplane bus protocol.
//! VME64 32/64-bit, 3U/6U form factor. Requires VME bridge module.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct VmeProtocol { pub slot: u8, pub address_space: VmeSpace }
#[derive(Debug,Clone,Copy)] pub enum VmeSpace { A16, A24, A32 }
impl VmeProtocol { pub fn new() -> Self { VmeProtocol { slot: 1, address_space: VmeSpace::A16 } }
    pub fn slot(mut self, s: u8) -> Self { self.slot = s; self } }

// VME bus cycle: [AM 6b] [Address 16/24/32b] [Data 8/16/32/64b]
// Accessed via VME bridge (e.g., Tundra Universe II, PCI-VME adapter)

pub fn build_read_cycle(space: VmeSpace, addr: u32, width: u8) -> Vec<u8> {
    let am = match space { VmeSpace::A16 => 0x29, VmeSpace::A24 => 0x39, VmeSpace::A32 => 0x09 };
    let mut f = vec![am, width];
    f.extend_from_slice(&addr.to_be_bytes()); f
}

pub fn build_write_cycle(space: VmeSpace, addr: u32, data: &[u8]) -> Vec<u8> {
    let am = match space { VmeSpace::A16 => 0x2A, VmeSpace::A24 => 0x3A, VmeSpace::A32 => 0x0A };
    let mut f = vec![am, data.len() as u8];
    f.extend_from_slice(&addr.to_be_bytes()); f.extend_from_slice(data); f
}

impl Protocol for VmeProtocol {
    fn name(&self) -> &'static str { "vme" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::SystemBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["vme64","vpx"] }
}

impl ProtocolRead for VmeProtocol {
    type Address = u32; type Value = u32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_cycle(self.address_space, *addr, 4);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 6 { u32::from_be_bytes([resp[2],resp[3],resp[4],resp[5]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolWrite for VmeProtocol {
    type Address = u32; type Value = u32;
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        conn.send(&build_write_cycle(self.address_space, *addr, &value.to_be_bytes())).map_err(ProtocolError::from)?; Ok(())
    }
    fn write_batch(&self,c:&mut dyn Connection,i:&[(Self::Address,Self::Value)])->Result<()>{for(a,v)in i{self.write(c,a,v)?;}Ok(())}
}

impl ProtocolHealth for VmeProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(v) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:format!("vme addr0=0x{v:08x}")}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for VmeProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("vme:slot{}",self.slot),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_cycle(VmeSpace::A16,0x100,2); assert_eq!(f[0],0x29); }
    #[test] fn test_meta() { assert_eq!(VmeProtocol::new().name(),"vme"); }
}
