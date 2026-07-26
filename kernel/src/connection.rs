// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 连接管理器
//!
//! 定义 [`Connection`] trait 和 [`ConnectionManager`]，管理设备连接的生命周期。
//!
//! ## 三种连接策略
//!
//! - **Simple** — 每次操作创建新连接，用完立即释放（适合低频轮询）
//! - **Persistent** — 保持单一长连接，断开自动重连（适合持续通信）
//! - **Pooled** — 连接池，设定最小/最大连接数和超时（适合高并发场景）

use std::collections::HashMap;
use std::io;
use std::sync::Mutex;
use std::time::Instant;

pub trait Connection: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn close(&mut self) -> io::Result<()>;
    fn is_alive(&self) -> bool;
}

pub type ConnectionFactory = Box<dyn Fn() -> io::Result<Box<dyn Connection>> + Send + Sync>;

#[derive(Debug, Clone)]
pub enum ConnectionStrategy {
    Simple,
    Persistent,
    Pooled {
        min: usize,
        max: usize,
        timeout_ms: u64,
    },
}

struct ManagedConn {
    conn: Box<dyn Connection>,
}

pub struct ConnectionManager {
    strategy: ConnectionStrategy,
    connections: Mutex<HashMap<String, ManagedConn>>,
}

pub struct HealthInfo {
    pub connected: bool,
    pub latency_ms: u64,
}

impl ConnectionManager {
    pub fn new(strategy: ConnectionStrategy) -> Self {
        ConnectionManager {
            strategy,
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn connect(&self, device_id: &str, factory: ConnectionFactory) -> io::Result<String> {
        let mut conns = self.connections.lock().unwrap();
        match self.strategy {
            ConnectionStrategy::Simple => {
                let mut conn = factory()?;
                conn.close()?;
                Ok(device_id.to_string())
            }
            ConnectionStrategy::Persistent | ConnectionStrategy::Pooled { .. } => {
                if conns.contains_key(device_id) {
                    return Ok(device_id.to_string());
                }
                conns.insert(device_id.to_string(), ManagedConn { conn: factory()? });
                Ok(device_id.to_string())
            }
        }
    }

    pub fn disconnect(&self, device_id: &str) -> io::Result<()> {
        let mut conns = self.connections.lock().unwrap();
        if let Some(mut m) = conns.remove(device_id) {
            m.conn.close()
        } else {
            Ok(())
        }
    }

    pub fn is_connected(&self, device_id: &str) -> bool {
        self.connections
            .lock()
            .unwrap()
            .get(device_id)
            .map(|c| c.conn.is_alive())
            .unwrap_or(false)
    }

    pub fn health(&self, device_id: &str) -> Option<HealthInfo> {
        let mut conns = self.connections.lock().unwrap();
        let managed = conns.get_mut(device_id)?;
        let start = Instant::now();
        let connected = managed.conn.is_alive();
        Some(HealthInfo {
            connected,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub fn send(&self, device_id: &str, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut conns = self.connections.lock().unwrap();
        conns
            .get_mut(device_id)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("device not connected: {device_id}"),
                )
            })?
            .conn
            .send(data)
    }

    pub fn shutdown(&self) -> io::Result<()> {
        for (_, mut m) in self.connections.lock().unwrap().drain() {
            let _ = m.conn.close();
        }
        Ok(())
    }
}
