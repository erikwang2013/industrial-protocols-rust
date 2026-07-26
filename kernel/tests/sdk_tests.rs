// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::Result;
use industrial_protocols_kernel::sdk::*;
use std::io;

struct TestProtocol;
impl Protocol for TestProtocol {
    fn name(&self) -> &'static str {
        "test"
    }
    fn version(&self) -> &'static str {
        "0.1.0"
    }
    fn category(&self) -> ProtocolCategory {
        ProtocolCategory::IndustrialEthernet
    }
    fn supported_variants(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

impl ProtocolRead for TestProtocol {
    type Address = u16;
    type Value = u16;
    fn read(&self, _conn: &mut dyn Connection, _addr: &Self::Address) -> Result<Self::Value> {
        Ok(42)
    }
    fn read_batch(
        &self,
        _conn: &mut dyn Connection,
        _addrs: &[Self::Address],
    ) -> Result<Vec<Self::Value>> {
        Ok(vec![1, 2, 3])
    }
    fn read_range(
        &self,
        _conn: &mut dyn Connection,
        _start: &Self::Address,
        count: u16,
    ) -> Result<Vec<Self::Value>> {
        Ok(vec![0; count as usize])
    }
}

impl ProtocolHealth for TestProtocol {
    fn health(&self, _conn: &mut dyn Connection) -> Result<HealthStatus> {
        Ok(HealthStatus {
            state: HealthState::Healthy,
            latency_ms: 1,
            last_seen: std::time::Instant::now(),
            details: "ok".into(),
        })
    }
}

struct TestConnection;
impl Connection for TestConnection {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(vec![])
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

#[test]
fn test_protocol_as_trait_object() {
    let p: Box<dyn Protocol> = Box::new(TestProtocol);
    assert_eq!(p.name(), "test");
    assert_eq!(p.category(), ProtocolCategory::IndustrialEthernet);
}

#[test]
fn test_protocol_read_as_trait_object() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let val = ProtocolRead::read(&p, &mut conn, &1).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_protocol_read_batch() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[1, 2, 3]).unwrap();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn test_protocol_health() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let status = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(status.state, HealthState::Healthy);
}
