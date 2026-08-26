// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::bridge::Transport;
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

impl ProtocolWrite for TestProtocol {
    type Address = u16;
    type Value = u16;
    fn write(
        &self,
        _conn: &mut dyn Connection,
        _addr: &Self::Address,
        _value: &Self::Value,
    ) -> Result<()> {
        Ok(())
    }
    fn write_batch(
        &self,
        _conn: &mut dyn Connection,
        _items: &[(Self::Address, Self::Value)],
    ) -> Result<()> {
        Ok(())
    }
}

impl ProtocolSubscribe for TestProtocol {
    type SubscriptionId = u32;
    type Address = u16;
    fn subscribe(
        &self,
        _conn: &mut dyn Connection,
        _addrs: &[Self::Address],
    ) -> Result<Self::SubscriptionId> {
        Ok(7)
    }
    fn unsubscribe(
        &self,
        _conn: &mut dyn Connection,
        _sub_id: &Self::SubscriptionId,
    ) -> Result<()> {
        Ok(())
    }
    fn poll(
        &self,
        _conn: &mut dyn Connection,
    ) -> Result<Vec<SubscriptionUpdate<Self::SubscriptionId>>> {
        Ok(vec![SubscriptionUpdate {
            subscription_id: 7,
            address: "40001".into(),
            value: "42".into(),
            timestamp: std::time::Instant::now(),
        }])
    }
}

impl ProtocolDiagnostics for TestProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: "plc-001".into(),
            uptime_secs: Some(3600),
            error_count: 3,
            last_error: Some("timeout".into()),
            custom_fields: Default::default(),
        })
    }
}

struct TestTransport;
impl Transport for TestTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(vec![])
    }
    fn is_open(&self) -> bool {
        true
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl ProtocolDiscover for TestProtocol {
    fn discover(&self, _transport: &mut dyn Transport) -> Result<Vec<DiscoveredDevice>> {
        Ok(vec![DiscoveredDevice {
            device_id: "found-1".into(),
            protocol: "test".into(),
            address: "1.2.3.4".into(),
            metadata: Default::default(),
        }])
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
    assert_eq!(status.latency_ms, 1);
    assert_eq!(status.details, "ok");
}

#[test]
fn test_protocol_read_range() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let vals = ProtocolRead::read_range(&p, &mut conn, &0, 4).unwrap();
    assert_eq!(vals, vec![0, 0, 0, 0]);
}

#[test]
fn test_protocol_write() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    ProtocolWrite::write(&p, &mut conn, &1, &2).unwrap();
    ProtocolWrite::write_batch(&p, &mut conn, &[(1, 2), (3, 4)]).unwrap();
}

#[test]
fn test_protocol_subscribe_roundtrip() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let sub = ProtocolSubscribe::subscribe(&p, &mut conn, &[1, 2]).unwrap();
    assert_eq!(sub, 7);
    ProtocolSubscribe::unsubscribe(&p, &mut conn, &sub).unwrap();
    let updates = ProtocolSubscribe::poll(&p, &mut conn).unwrap();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].subscription_id, 7);
    assert_eq!(updates[0].address, "40001");
    assert_eq!(updates[0].value, "42");
}

#[test]
fn test_protocol_discover() {
    let p = TestProtocol;
    let mut transport = TestTransport;
    let devices = ProtocolDiscover::discover(&p, &mut transport).unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].device_id, "found-1");
    assert_eq!(devices[0].protocol, "test");
    assert_eq!(devices[0].address, "1.2.3.4");
}

#[test]
fn test_protocol_diagnostics() {
    let p = TestProtocol;
    let mut conn = TestConnection;
    let report = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(report.device_id, "plc-001");
    assert_eq!(report.uptime_secs, Some(3600));
    assert_eq!(report.error_count, 3);
    assert_eq!(report.last_error.as_deref(), Some("timeout"));
}

#[test]
fn test_marker_traits_cover_common_types() {
    fn assert_address<T: AddressLike>() {}
    fn assert_value<T: ValueLike>() {}
    assert_address::<u16>();
    assert_address::<String>();
    assert_value::<u16>();
    assert_value::<f32>();
    assert_value::<String>();
}

#[test]
fn test_protocol_category_variants() {
    let cats = [
        ProtocolCategory::IndustrialEthernet,
        ProtocolCategory::Fieldbus,
        ProtocolCategory::IoTMessaging,
        ProtocolCategory::AutomotiveBus,
        ProtocolCategory::BuildingLighting,
        ProtocolCategory::HardwareBridge,
        ProtocolCategory::SystemBus,
    ];
    assert_eq!(cats.len(), 7);
    assert_ne!(cats[0], cats[1]);
}

#[test]
fn test_subscription_update_with_custom_id_type() {
    let update: SubscriptionUpdate<u32> = SubscriptionUpdate {
        subscription_id: 1,
        address: "a".into(),
        value: "v".into(),
        timestamp: std::time::Instant::now(),
    };
    assert_eq!(update.subscription_id, 1);
    let cloned = update.clone();
    assert_eq!(cloned.value, "v");
}
