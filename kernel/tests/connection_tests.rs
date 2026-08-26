// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::*;
use std::io;
use std::sync::{Arc, Mutex};

struct MockTransport {
    alive: bool,
    send_fn: Arc<Mutex<Vec<Vec<u8>>>>,
    responses: Vec<Vec<u8>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockTransport {
    fn new(responses: Vec<Vec<u8>>) -> Self {
        MockTransport {
            alive: true,
            send_fn: Arc::new(Mutex::new(Vec::new())),
            responses,
            call_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl Connection for MockTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.send_fn.lock().unwrap().push(data.to_vec());
        let idx = *self.call_count.lock().unwrap();
        *self.call_count.lock().unwrap() += 1;
        Ok(if idx < self.responses.len() {
            self.responses[idx].clone()
        } else {
            vec![]
        })
    }
    fn close(&mut self) -> io::Result<()> {
        self.alive = false;
        Ok(())
    }
    fn is_alive(&self) -> bool {
        self.alive
    }
}

#[test]
fn test_simple_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Simple);
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![1]])))),
        )
        .unwrap();
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_persistent_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![1]])))),
        )
        .unwrap();
    assert!(manager.is_connected("plc-001"));
}

#[test]
fn test_pooled_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Pooled {
        min: 1,
        max: 2,
        timeout_ms: 5000,
    });
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![42]])))),
        )
        .unwrap();
    assert!(manager.is_connected("plc-001"));
    manager.disconnect("plc-001").unwrap();
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_shutdown() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![])))),
        )
        .unwrap();
    manager
        .connect(
            "plc-002",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![])))),
        )
        .unwrap();
    manager.shutdown().unwrap();
    assert!(!manager.is_connected("plc-001"));
    assert!(!manager.is_connected("plc-002"));
}

#[test]
fn test_connect_factory_error_propagates() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    let err = manager.connect(
        "plc-001",
        Box::new(|| Err(io::Error::new(io::ErrorKind::ConnectionRefused, "nope"))),
    );
    assert!(err.is_err());
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_send_on_unconnected_device() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    let err = manager.send("plc-001", b"data").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}

#[test]
fn test_send_on_connected_device() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![9]])))),
        )
        .unwrap();
    let resp = manager.send("plc-001", b"ping").unwrap();
    assert_eq!(resp, vec![9]);
}

#[test]
fn test_disconnect_unknown_device_is_ok() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    assert!(manager.disconnect("ghost").is_ok());
}

#[test]
fn test_duplicate_connect_reuses_existing_connection() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    let calls = Arc::new(Mutex::new(0usize));
    let calls2 = calls.clone();
    manager
        .connect(
            "plc-001",
            Box::new(move || {
                *calls2.lock().unwrap() += 1;
                Ok(Box::new(MockTransport::new(vec![])))
            }),
        )
        .unwrap();
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![])))),
        )
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), 1);
}

#[test]
fn test_health_report() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    assert!(manager.health("plc-001").is_none());
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![])))),
        )
        .unwrap();
    let info = manager.health("plc-001").unwrap();
    assert!(info.connected);
}

#[test]
fn test_simple_strategy_send_fails_after_connect() {
    let manager = ConnectionManager::new(ConnectionStrategy::Simple);
    manager
        .connect(
            "plc-001",
            Box::new(|| Ok(Box::new(MockTransport::new(vec![])))),
        )
        .unwrap();
    assert!(!manager.is_connected("plc-001"));
    assert!(manager.send("plc-001", b"x").is_err());
}

#[test]
fn test_shutdown_empty_manager() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    assert!(manager.shutdown().is_ok());
}

#[test]
fn test_connection_strategy_clone_and_match() {
    let s = ConnectionStrategy::Pooled {
        min: 1,
        max: 2,
        timeout_ms: 100,
    };
    let s2 = s.clone();
    assert!(matches!(s2, ConnectionStrategy::Pooled { max: 2, .. }));
    assert!(matches!(
        ConnectionStrategy::Simple,
        ConnectionStrategy::Simple
    ));
    assert!(matches!(
        ConnectionStrategy::Persistent,
        ConnectionStrategy::Persistent
    ));
}
