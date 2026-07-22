pub mod frame;

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;
use std::sync::atomic::{AtomicU16, Ordering};

pub struct ModbusProtocol {
    variant: ModbusVariant,
    unit_id: u8,
    transaction_id: AtomicU16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusVariant { Tcp, Rtu, Ascii }

impl ModbusProtocol {
    pub fn tcp() -> Self {
        ModbusProtocol { variant: ModbusVariant::Tcp, unit_id: 1, transaction_id: AtomicU16::new(1) }
    }

    pub fn rtu(unit_id: u8) -> Self {
        ModbusProtocol { variant: ModbusVariant::Rtu, unit_id, transaction_id: AtomicU16::new(1) }
    }

    pub fn ascii(unit_id: u8) -> Self {
        ModbusProtocol { variant: ModbusVariant::Ascii, unit_id, transaction_id: AtomicU16::new(1) }
    }

    pub fn with_unit_id(mut self, id: u8) -> Self { self.unit_id = id; self }

    fn next_tid(&self) -> u16 { self.transaction_id.fetch_add(1, Ordering::SeqCst) }
}

impl Protocol for ModbusProtocol {
    fn name(&self) -> &'static str { "modbus" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["tcp", "rtu", "ascii"] }
}

impl ProtocolRead for ModbusProtocol {
    type Address = u16;
    type Value = u16;

    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        let frame = match self.variant {
            ModbusVariant::Tcp => frame::build_read_holding_registers_tcp(self.next_tid(), self.unit_id, *addr, 1),
            ModbusVariant::Rtu => frame::build_read_holding_registers_rtu(self.unit_id, *addr, 1),
            ModbusVariant::Ascii => frame::build_read_holding_registers_ascii(self.unit_id, *addr, 1),
        };
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError {
                device: "modbus-device".into(), code: exc as u16,
                message: format!("Modbus exception {exc:#04X}"),
            });
        }
        frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })?
            .into_iter().next()
            .ok_or_else(|| ProtocolError::Parse { protocol: "modbus".into(), reason: "empty response".into() })
    }

    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        if addrs.is_empty() { return Ok(vec![]); }
        let min = *addrs.iter().min().unwrap();
        let max = *addrs.iter().max().unwrap();
        let frame = match self.variant {
            ModbusVariant::Tcp => frame::build_read_holding_registers_tcp(self.next_tid(), self.unit_id, min, max - min + 1),
            ModbusVariant::Rtu => frame::build_read_holding_registers_rtu(self.unit_id, min, max - min + 1),
            ModbusVariant::Ascii => frame::build_read_holding_registers_ascii(self.unit_id, min, max - min + 1),
        };
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        let all = frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })?;
        Ok(addrs.iter().map(|a| all[(*a - min) as usize]).collect())
    }

    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        let frame = match self.variant {
            ModbusVariant::Tcp => frame::build_read_holding_registers_tcp(self.next_tid(), self.unit_id, *start, count),
            ModbusVariant::Rtu => frame::build_read_holding_registers_rtu(self.unit_id, *start, count),
            ModbusVariant::Ascii => frame::build_read_holding_registers_ascii(self.unit_id, *start, count),
        };
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        frame::parse_read_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "modbus".into(), reason: e })
    }
}

impl ProtocolWrite for ModbusProtocol {
    type Address = u16;
    type Value = u16;

    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()> {
        let frame = match self.variant {
            ModbusVariant::Tcp => frame::build_write_single_register_tcp(self.next_tid(), self.unit_id, *addr, *value),
            ModbusVariant::Rtu => frame::build_write_single_register_rtu(self.unit_id, *addr, *value),
            ModbusVariant::Ascii => frame::build_write_single_register_ascii(self.unit_id, *addr, *value),
        };
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError {
                device: "modbus-device".into(), code: exc as u16,
                message: format!("Modbus exception {exc:#04X}"),
            });
        }
        Ok(())
    }

    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        if items.is_empty() { return Ok(()); }
        let start = items[0].0;
        let values: Vec<u16> = items.iter().map(|(_, v)| *v).collect();
        let frame = match self.variant {
            ModbusVariant::Tcp => frame::build_write_multiple_registers_tcp(self.next_tid(), self.unit_id, start, &values),
            ModbusVariant::Rtu => frame::build_write_multiple_registers_rtu(self.unit_id, start, &values),
            ModbusVariant::Ascii => {
                // ASCII doesn't have FC 0x10 natively, fall back to multiple single writes
                for (addr, val) in items {
                    let f = frame::build_write_single_register_ascii(self.unit_id, *addr, *val);
                    conn.send(&f).map_err(ProtocolError::from)?;
                }
                return Ok(());
            }
        };
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        if let Some(exc) = frame::parse_exception(&resp) {
            return Err(ProtocolError::DeviceError {
                device: "modbus-device".into(), code: exc as u16,
                message: format!("Modbus exception {exc:#04X}"),
            });
        }
        Ok(())
    }
}

impl ProtocolHealth for ModbusProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        match self.read(conn, &0) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("modbus/{:?}: read holding register 0 ok", self.variant),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("modbus health check failed: {e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for ModbusProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("modbus-{:?}-{}", self.variant, self.unit_id),
            uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("variant".into(), format!("{:?}", self.variant)),
                ("unit_id".into(), self.unit_id.to_string()),
            ]),
        })
    }
}
