// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 串口传输
//!
//! 通过 `serialport` crate 实现真实串口通信。
//! 启用 `serial` feature 后可用（需要系统安装 libudev）。
//! 未启用时使用存根实现，返回明确的错误提示。

use industrial_protocols_kernel::bridge::Transport;
use std::io;

#[derive(Debug, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub stop_bits: u8,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum Parity { None, Even, Odd }

/// Returns a list of available serial port names on the system.
/// Requires the `serial` feature.
#[cfg(feature = "serial")]
pub fn available_ports() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

/// Always returns empty list when `serial` feature is not enabled.
#[cfg(not(feature = "serial"))]
pub fn available_ports() -> Vec<String> {
    Vec::new()
}

/// Real serial transport — requires the `serial` feature.
#[cfg(feature = "serial")]
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

#[cfg(feature = "serial")]
impl SerialTransport {
    pub fn open(config: SerialConfig) -> io::Result<Self> {
        use serialport::{DataBits, Parity as SpParity, StopBits};
        use std::time::Duration;

        let parity = match config.parity {
            Parity::None => SpParity::None,
            Parity::Even => SpParity::Even,
            Parity::Odd => SpParity::Odd,
        };
        let data_bits = match config.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        };
        let stop_bits = match config.stop_bits {
            1 => StopBits::One,
            2 => StopBits::Two,
            _ => StopBits::One,
        };

        let port = serialport::new(&config.port, config.baud_rate)
            .parity(parity)
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .timeout(Duration::from_millis(config.timeout_ms))
            .open()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(SerialTransport { port })
    }
}

#[cfg(feature = "serial")]
impl Transport for SerialTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        use std::io::{Read, Write};
        self.port.write_all(data)?;
        self.port.flush()?;
        let mut buf = vec![0u8; 4096];
        match self.port.read(&mut buf) {
            Ok(n) => { buf.truncate(n); Ok(buf) }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => Ok(vec![]),
            Err(e) => Err(e),
        }
    }
    fn is_open(&self) -> bool { true }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}

/// Stub transport when `serial` feature is not enabled.
#[cfg(not(feature = "serial"))]
pub struct SerialTransport { config: SerialConfig }

#[cfg(not(feature = "serial"))]
impl SerialTransport {
    pub fn open(config: SerialConfig) -> io::Result<Self> { Ok(SerialTransport { config }) }
}

#[cfg(not(feature = "serial"))]
impl Transport for SerialTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Err(io::Error::new(io::ErrorKind::Unsupported,
            format!("Serial transport not available (enable 'serial' feature). Port: {}", self.config.port)))
    }
    fn is_open(&self) -> bool { false }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_ports_does_not_panic() {
        let _ports = available_ports();
    }

    #[test]
    fn test_parity_types() {
        assert!(matches!(Parity::None, Parity::None));
        assert!(matches!(Parity::Even, Parity::Even));
        assert!(matches!(Parity::Odd, Parity::Odd));
    }

    #[test]
    fn test_config_fields() {
        let config = SerialConfig {
            port: "/dev/ttyS0".into(), baud_rate: 9600,
            data_bits: 8, parity: Parity::None, stop_bits: 1, timeout_ms: 500,
        };
        assert_eq!(config.baud_rate, 9600);
        assert_eq!(config.data_bits, 8);
    }
}
