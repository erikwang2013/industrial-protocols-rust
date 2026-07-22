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
        MockTransport { alive: true, send_fn: Arc::new(Mutex::new(Vec::new())), responses, call_count: Arc::new(Mutex::new(0)) }
    }
}

impl Connection for MockTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.send_fn.lock().unwrap().push(data.to_vec());
        let idx = *self.call_count.lock().unwrap();
        *self.call_count.lock().unwrap() += 1;
        Ok(if idx < self.responses.len() { self.responses[idx].clone() } else { vec![] })
    }
    fn close(&mut self) -> io::Result<()> { self.alive = false; Ok(()) }
    fn is_alive(&self) -> bool { self.alive }
}

#[test]
fn test_simple_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Simple);
    manager.connect("plc-001", Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![1]]))))).unwrap();
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_persistent_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager.connect("plc-001", Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![1]]))))).unwrap();
    assert!(manager.is_connected("plc-001"));
}

#[test]
fn test_pooled_strategy() {
    let manager = ConnectionManager::new(ConnectionStrategy::Pooled { min: 1, max: 2, timeout_ms: 5000 });
    manager.connect("plc-001", Box::new(|| Ok(Box::new(MockTransport::new(vec![vec![42]]))))).unwrap();
    assert!(manager.is_connected("plc-001"));
    manager.disconnect("plc-001").unwrap();
    assert!(!manager.is_connected("plc-001"));
}

#[test]
fn test_shutdown() {
    let manager = ConnectionManager::new(ConnectionStrategy::Persistent);
    manager.connect("plc-001", Box::new(|| Ok(Box::new(MockTransport::new(vec![]))))).unwrap();
    manager.connect("plc-002", Box::new(|| Ok(Box::new(MockTransport::new(vec![]))))).unwrap();
    manager.shutdown().unwrap();
    assert!(!manager.is_connected("plc-001"));
    assert!(!manager.is_connected("plc-002"));
}
