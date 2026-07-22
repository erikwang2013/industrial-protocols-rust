// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! PCI/PCIe system bus protocol bridge.
//! Requires kernel driver or library bridge (e.g., pciutils/libpci).
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct PciProtocol { pub bus: u8, pub device: u8, pub function: u8 }
impl PciProtocol { pub fn new() -> Self { PciProtocol { bus: 0, device: 0, function: 0 } }
    pub fn bdf(mut self, b: u8, d: u8, f: u8) -> Self { self.bus = b; self.device = d; self.function = f; self } }

// PCI configuration space access via /sys/bus/pci or libpci:
// [Bus 1B] [Device 1B] [Function 1B] [Register 1B] [Width 1B] [Data...]

pub fn build_config_read(bus: u8, dev: u8, func: u8, reg: u8, width: u8) -> Vec<u8> {
    vec![bus, dev, func, reg, width]
}

pub fn build_mmio_read(bar: u8, offset: u32, len: u32) -> Vec<u8> {
    let mut f = vec![bar, 0x80]; f.extend_from_slice(&offset.to_le_bytes()); f.extend_from_slice(&len.to_le_bytes()); f
}

impl Protocol for PciProtocol {
    fn name(&self) -> &'static str { "pci" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::SystemBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["pci","pcie"] }
}

impl ProtocolRead for PciProtocol {
    type Address = u8; type Value = u32;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_config_read(self.bus, self.device, self.function, *addr, 4);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        Ok(if resp.len() >= 7 { u32::from_le_bytes([resp[5],resp[6],resp[3],resp[4]]) } else { 0 })
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for PciProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) { // Read Vendor/Device ID
            Ok(v) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:format!("pci vendor=0x{:08x}",v)}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for PciProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("pci:{:02x}:{:02x}.{}",self.bus,self.device,self.function),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_config() { let f = build_config_read(0,0,0,0,4); assert_eq!(f.len(),5); }
    #[test] fn test_mmio() { let f = build_mmio_read(0,0x1000,4); assert_eq!(f[0],0); assert_eq!(f[1],0x80); }
    #[test] fn test_meta() { assert_eq!(PciProtocol::new().name(),"pci"); }
}
