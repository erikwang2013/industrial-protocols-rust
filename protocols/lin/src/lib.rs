// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # LIN — LIN 车身总线协议。19200 bps UART、同步+ID 帧头、增强型校验和。

//! LIN (Local Interconnect Network) body bus protocol - 19200 bps UART.
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct LinProtocol {
    pub id: u8,
}
impl LinProtocol {
    pub fn new(id: u8) -> Self {
        LinProtocol { id }
    }
}

// LIN frame: [Sync 0x55] [ID 1B] [Data nB] [Checksum 1B]
// ID byte: bits 0-5=PID, bits 6-7=parity (P0=bit0^bit1^bit2^bit4, P1=!(bit1^bit3^bit4^bit5))

pub fn build_header(id: u8) -> Vec<u8> {
    let p0 = (id & 1) ^ ((id >> 1) & 1) ^ ((id >> 2) & 1) ^ ((id >> 4) & 1);
    // logical NOT of a 0/1 bit; bitwise `!` would never be zero and always set 0x80
    let p1 = 1 - (((id >> 1) & 1) ^ ((id >> 3) & 1) ^ ((id >> 4) & 1) ^ ((id >> 5) & 1));
    let pid = (id & 0x3F) | (if p0 != 0 { 0x40 } else { 0 }) | (if p1 != 0 { 0x80 } else { 0 });
    vec![0x55, pid]
}

pub fn build_frame(id: u8, data: &[u8]) -> Vec<u8> {
    let mut f = build_header(id);
    f.extend_from_slice(data);
    let checksum: u16 = data.iter().fold(0u16, |c, &b| c.wrapping_add(b as u16));
    let checksum = if id >= 0x3C {
        // Enhanced checksum includes PID
        (checksum + id as u16) as u8
    } else {
        checksum as u8
    };
    f.push((!checksum).wrapping_add(1)); // 2's complement
    f
}

impl Protocol for LinProtocol {
    fn name(&self) -> &'static str {
        "lin"
    }
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::AutomotiveBus
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["uart"]
    }
}

impl ProtocolRead for LinProtocol {
    type Address = u8;
    type Value = Vec<u8>;
    fn read(&self, conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        let f = build_frame(self.id, &[]);
        conn.send(&f).map_err(ProtocolError::from)
    }
    fn read_batch(&self, c: &mut dyn Connection, a: &[Self::Address]) -> Result<Vec<Self::Value>> {
        a.iter().map(|x| self.read(c, x)).collect()
    }
    fn read_range(
        &self,
        c: &mut dyn Connection,
        s: &Self::Address,
        n: u16,
    ) -> Result<Vec<Self::Value>> {
        (0..n).map(|_| self.read(c, s)).collect()
    }
}

impl ProtocolHealth for LinProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match conn.send(&build_header(self.id)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "lin header ok".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: 0,
                last_seen: std::time::Instant::now(),
                details: format!("{e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for LinProtocol {
    fn diagnostics(&self, _: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("lin:{}", self.id),
            uptime_secs: None,
            error_count: 0,
            last_error: None,
            custom_fields: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_header() {
        let h = build_header(0x10);
        assert_eq!(h[0], 0x55);
    }
    #[test]
    fn test_frame() {
        let f = build_frame(0x10, &[1, 2, 3]);
        assert_eq!(f[0], 0x55);
    }
    #[test]
    fn test_meta() {
        assert_eq!(LinProtocol::new(0x10).name(), "lin");
    }
}
