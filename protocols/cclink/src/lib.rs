// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

//! CC-Link RS-485 protocol.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct CclinkProtocol { pub station_no: u8 }
impl CclinkProtocol { pub fn new(station_no: u8) -> Self { CclinkProtocol { station_no } } }

fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data { crc ^= (byte as u16) << 8; for _ in 0..8 { crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 }; } }
    crc
}

pub fn build_poll(station: u8, cmd: u8, data: &[u8]) -> Vec<u8> {
    let mut f = vec![0x10, 0x02, station, cmd]; f.extend_from_slice(data); f.push(0x10); f.push(0x03);
    let crc = crc16_xmodem(&f); f.extend_from_slice(&crc.to_le_bytes()); f
}

pub fn build_rx_read(station: u8, start: u16, count: u16) -> Vec<u8> {
    let mut d = Vec::new(); d.extend_from_slice(&start.to_be_bytes()); d.extend_from_slice(&count.to_be_bytes()); build_poll(station, 0x01, &d)
}

impl Protocol for CclinkProtocol {
    fn name(&self) -> &'static str { "cclink" } fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::Fieldbus }
    fn supported_variants(&self) -> &'static [&'static str] { &["rs485"] }
}

impl ProtocolRead for CclinkProtocol {
    type Address = u16; type Value = u16;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let f = build_rx_read(self.station_no, *addr, 1);
        let resp = conn.send(&f).map_err(ProtocolError::from)?;
        if resp.len() < 8 { return Err(ProtocolError::Parse{protocol:"cclink".into(),reason:"too short".into()}); }
        Ok(u16::from_be_bytes([resp[4], resp[5]]))
    }
    fn read_batch(&self,c:&mut dyn Connection,a:&[Self::Address])->Result<Vec<Self::Value>>{a.iter().map(|x|self.read(c,x)).collect()}
    fn read_range(&self,c:&mut dyn Connection,s:&Self::Address,n:u16)->Result<Vec<Self::Value>>{(0..n).map(|_|self.read(c,s)).collect()}
}

impl ProtocolHealth for CclinkProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus{state:HealthState::Healthy,latency_ms:start.elapsed().as_millis()as u64,last_seen:std::time::Instant::now(),details:"cclink ok".into()}),
            Err(e) => Ok(HealthStatus{state:HealthState::Unhealthy,latency_ms:0,last_seen:std::time::Instant::now(),details:format!("{e}")}),
        }
    }
}

impl ProtocolDiagnostics for CclinkProtocol {
    fn diagnostics(&self,_:&mut dyn Connection)->Result<DiagnosticsReport>{Ok(DiagnosticsReport{device_id:format!("cclink:{}",self.station_no),uptime_secs:None,error_count:0,last_error:None,custom_fields:Default::default()})}
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_frame() { let f = build_poll(1,0x01,&[1,2]); assert_eq!(f[0],0x10); assert_eq!(f[2],1); }
    #[test] fn test_meta() { assert_eq!(CclinkProtocol::new(1).name(),"cclink"); }
}
