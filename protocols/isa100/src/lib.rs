// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! ISA100.11a wireless industrial protocol.
//! 2.4 GHz DSSS, IPv6-based mesh. Yokogawa YFGW410 / Honeywell OneWireless gateway.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct Isa100Protocol { pub device_id: u64, pub gateway_host: String }
impl Isa100Protocol { pub fn new() -> Self { Isa100Protocol { device_id: 0x0102030405060708, gateway_host: "192.168.1.1".into() } }
    pub fn id(mut self, id: u64) -> Self { self.device_id = id; self }
    pub fn gateway(mut self, h: &str) -> Self { self.gateway_host = h.into(); self } }

// ISA100.11a application layer over gateway TCP:
// [Cmd 1B] [DeviceID 8B] [ObjectID 2B] [Attribute 2B] [Data...]

pub fn build_read_object(dev_id: u64, obj: u16, attr: u16) -> Vec<u8> {
    let mut f = vec![0x01]; f.extend_from_slice(&dev_id.to_be_bytes());
    f.extend_from_slice(&obj.to_be_bytes()); f.extend_from_slice(&attr.to_be_bytes()); f
}

pub fn build_read_process_data(dev_id: u64) -> Vec<u8> {
    let mut f = vec![0x02]; f.extend_from_slice(&dev_id.to_be_bytes()); f.extend_from_slice(&[0;8]); f
}

impl Protocol for Isa100Protocol {
    fn name(&self) -> &'static str { "isa100" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::HardwareBridge }
    fn supported_variants(&self) -> &'static [&'static str] { &["11a"] }
}

impl ProtocolRead for Isa100Protocol {
    type Address = u16; type Value = f64;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_read_object(self.device_id, *addr, 0);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 21 { f64::from_be_bytes([resp[13],resp[14],resp[15],resp[16],resp[17],resp[18],resp[19],resp[20]]) } else { 0.0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for Isa100Protocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_read_process_data(self.device_id)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"isa100 pd ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for Isa100Protocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("isa100:{:x}",self.device_id),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_read() { let f = build_read_object(0x0102,0x100,0); assert_eq!(f[0],1); assert_eq!(f.len(),13); }
    #[test] fn test_meta() { assert_eq!(Isa100Protocol::new().name(),"isa100"); }
}
