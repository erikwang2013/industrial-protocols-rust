// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 网关引擎
//!
//! 规则驱动的协议间数据转发引擎。通过后台线程周期性轮询，
//! 将源设备的数据经变换后转发到目标设备。
//!
//! ## 转发规则
//!
//! 每条规则定义：
//! - **source** — 源设备 + 地址（从哪读数据）
//! - **target** — 目标设备 + 地址（写到哪去）
//! - **transform** — 数据变换（透传 / 缩放 / 偏移 / 缩放+偏移）
//!
//! ## 适配器
//!
//! 协议通过实现 [`GatewayAdapter`] trait 来支持网关转发。
//! 引擎负责调度和错误计数，适配器负责协议特定的读写逻辑。

use crate::error::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Adapter that protocols implement to support gateway forwarding.
pub trait GatewayAdapter: Send + Sync {
    /// Forward a value from source endpoint to target, applying the transform.
    fn forward(
        &self,
        source_device: &str,
        source_address: &str,
        target_device: &str,
        target_address: &str,
        transform: &Transform,
    ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum Transform {
    Passthrough,
    Scale(f64),
    Offset(f64),
    ScaleAndOffset { scale: f64, offset: f64 },
}

#[derive(Debug, Clone)]
pub struct GatewayEndpoint {
    pub device: String,
    pub address: String,
}

#[derive(Debug, Clone)]
pub struct GatewayRule {
    pub name: String,
    pub source: GatewayEndpoint,
    pub target: GatewayEndpoint,
    pub transform: Transform,
}

pub struct GatewayEngine {
    rules: Vec<GatewayRule>,
    interval_ms: u64,
    adapters: Vec<Box<dyn GatewayAdapter>>,
    running: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
    error_count: Arc<Mutex<u64>>,
}

impl GatewayEngine {
    pub fn new(interval_ms: u64) -> Self {
        GatewayEngine {
            rules: Vec::new(),
            interval_ms,
            adapters: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            handle: Mutex::new(None),
            error_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_rule(&mut self, rule: GatewayRule) {
        self.rules.push(rule);
    }

    pub fn add_adapter(&mut self, adapter: Box<dyn GatewayAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn error_count(&self) -> u64 {
        *self.error_count.lock().unwrap()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Start the gateway polling loop on a background thread.
    pub fn start(&mut self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return; // already running
        }

        let running = self.running.clone();
        let error_count = self.error_count.clone();
        let rules: Vec<_> = self.rules.clone();
        let adapters: Vec<Arc<dyn GatewayAdapter>> = std::mem::take(&mut self.adapters)
            .into_iter()
            .map(Arc::from)
            .collect();
        let interval = Duration::from_millis(self.interval_ms);

        let handle = thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                for rule in &rules {
                    for adapter in &adapters {
                        if let Err(_e) = adapter.forward(
                            &rule.source.device,
                            &rule.source.address,
                            &rule.target.device,
                            &rule.target.address,
                            &rule.transform,
                        ) {
                            *error_count.lock().unwrap() += 1;
                        }
                    }
                }
                thread::sleep(interval);
            }
        });

        *self.handle.lock().unwrap() = Some(handle);
    }

    /// Stop the gateway polling loop.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Drop for GatewayEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    struct TestAdapter {
        call_count: Arc<AtomicU32>,
    }

    impl TestAdapter {
        fn new() -> (Self, Arc<AtomicU32>) {
            let count = Arc::new(AtomicU32::new(0));
            (TestAdapter { call_count: count.clone() }, count)
        }
    }

    impl GatewayAdapter for TestAdapter {
        fn forward(
            &self,
            src_dev: &str,
            src_addr: &str,
            tgt_dev: &str,
            tgt_addr: &str,
            _transform: &Transform,
        ) -> Result<()> {
            // Verify endpoints are passed correctly
            assert_eq!(src_dev, "plc-001");
            assert_eq!(tgt_dev, "mqtt-broker");
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_add_rule() {
        let mut engine = GatewayEngine::new(1000);
        engine.add_rule(GatewayRule {
            name: "test".into(),
            source: GatewayEndpoint { device: "plc-001".into(), address: "40001".into() },
            target: GatewayEndpoint { device: "mqtt".into(), address: "topic/plc".into() },
            transform: Transform::Passthrough,
        });
        assert_eq!(engine.rule_count(), 1);
    }

    #[test]
    fn test_start_stop() {
        let mut engine = GatewayEngine::new(10);
        engine.add_rule(GatewayRule {
            name: "t".into(),
            source: GatewayEndpoint { device: "plc-001".into(), address: "40001".into() },
            target: GatewayEndpoint { device: "mqtt-broker".into(), address: "t".into() },
            transform: Transform::Passthrough,
        });

        let (adapter, count) = TestAdapter::new();
        engine.add_adapter(Box::new(adapter));
        engine.start();

        // Wait for a few poll cycles
        thread::sleep(Duration::from_millis(50));
        engine.stop();

        assert!(count.load(Ordering::SeqCst) > 0);
        assert!(!engine.is_running());
    }

    #[test]
    fn test_double_start_noop() {
        let mut engine = GatewayEngine::new(100);
        engine.add_rule(GatewayRule {
            name: "t".into(),
            source: GatewayEndpoint { device: "plc-001".into(), address: "40001".into() },
            target: GatewayEndpoint { device: "mqtt-broker".into(), address: "t".into() },
            transform: Transform::Passthrough,
        });

        let (adapter, _count) = TestAdapter::new();
        engine.add_adapter(Box::new(adapter));
        engine.start();
        engine.start(); // second start should be noop
        engine.stop();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_scale_transform() {
        let t = Transform::Scale(2.5);
        assert!(matches!(t, Transform::Scale(2.5)));
    }

    #[test]
    fn test_offset_transform() {
        let t = Transform::Offset(-10.0);
        assert!(matches!(t, Transform::Offset(-10.0)));
    }

    #[test]
    fn test_scale_and_offset_transform() {
        let t = Transform::ScaleAndOffset { scale: 1.8, offset: 32.0 };
        assert!(matches!(t, Transform::ScaleAndOffset { scale: 1.8, offset: 32.0 }));
    }
}
