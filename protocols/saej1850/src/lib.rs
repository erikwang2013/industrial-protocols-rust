// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # SAEJ1850 — SAE J1850 OBD-II 协议。VPW/PWM 两种变体、CRC-8 校验、Mode 01/03/09。

//! SAE J1850 OBD-II protocol (VPW 10.4kbps / PWM 41.6kbps).
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct SaeJ1850Protocol { pub variant: J1850Variant }
#[derive(Debug,Clone,Copy)] pub enum J1850Variant { Vpw, Pwm }

impl SaeJ1850Protocol { pub fn new(v: J1850Variant) -> Self { SaeJ1850Protocol { variant: v } } }

// J1850 frame: [Priority 3b] [Type 1b] [Target 1B] [Source 1B] [Data...] [CRC 1B]
// VPW uses variable pulse width, PWM uses pulse width modulation
// CRC-8: polynomial 0x1D

fn crc8(data: &[u8]) -> u8 {
    let mut crc: u8 = 0xFF;
    for &byte in data {
        crc ^= byte;
        for _ in 0..8 { crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x1D } else { crc << 1 }; }
    }
    crc
}

pub fn build_request(mode: u8, pid: u8) -> Vec<u8> {
    let header = 0x68; // priority=3, physical addressing
    let mut f = vec![header, 0x6A, 0xF1, mode, pid];
    let crc = crc8(&f);
    f.push(crc);
    f
}

pub fn build_read_dtc() -> Vec<u8> { build_request(0x03, 0x00) }
pub fn build_read_vin() -> Vec<u8> { build_request(0x09, 0x02) }

impl Protocol for SaeJ1850Protocol {
    fn name(&self) -> &'static str { "saej1850" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::AutomotiveBus }
    fn supported_variants(&self) -> &'static [&'static str] { &["vpw", "pwm"] }
}

impl ProtocolRead for SaeJ1850Protocol {
    type Address = u8; type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_request(0x01, *addr);
        Ok(conn.send(&f).map_err(ProtocolError::from)?)
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for SaeJ1850Protocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_request(0x01, 0x00)) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"j1850 mode01 ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for SaeJ1850Protocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:"j1850-device".into(),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_frame() { let f = build_read_dtc(); assert_eq!(f.len(),6); assert_eq!(f[0],0x68); }
    #[test] fn test_meta() { assert_eq!(SaeJ1850Protocol::new(J1850Variant::Vpw).name(),"saej1850"); }
}
