// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::gateway::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

struct CountingAdapter {
    calls: Arc<AtomicU32>,
}

impl GatewayAdapter for CountingAdapter {
    fn forward(
        &self,
        _source_device: &str,
        _source_address: &str,
        _target_device: &str,
        _target_address: &str,
        _transform: &Transform,
    ) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct FailingAdapter;

impl GatewayAdapter for FailingAdapter {
    fn forward(
        &self,
        _source_device: &str,
        _source_address: &str,
        _target_device: &str,
        _target_address: &str,
        _transform: &Transform,
    ) -> Result<()> {
        Err(ProtocolError::Connection("boom".into()))
    }
}

fn rule(name: &str) -> GatewayRule {
    GatewayRule {
        name: name.to_string(),
        source: GatewayEndpoint {
            device: "src".into(),
            address: "1".into(),
        },
        target: GatewayEndpoint {
            device: "dst".into(),
            address: "2".into(),
        },
        transform: Transform::Passthrough,
    }
}

#[test]
fn test_engine_starts_inactive() {
    let engine = GatewayEngine::new(100);
    assert_eq!(engine.rule_count(), 0);
    assert_eq!(engine.error_count(), 0);
    assert!(!engine.is_running());
}

#[test]
fn test_add_multiple_rules() {
    let mut engine = GatewayEngine::new(100);
    engine.add_rule(rule("r1"));
    engine.add_rule(rule("r2"));
    assert_eq!(engine.rule_count(), 2);
}

#[test]
fn test_adapter_success_path_records_no_errors() {
    let mut engine = GatewayEngine::new(10);
    engine.add_rule(rule("r1"));
    let (adapter, calls) = {
        let calls = Arc::new(AtomicU32::new(0));
        (
            CountingAdapter {
                calls: calls.clone(),
            },
            calls,
        )
    };
    engine.add_adapter(Box::new(adapter));
    engine.start();
    std::thread::sleep(Duration::from_millis(50));
    engine.stop();
    assert!(calls.load(Ordering::SeqCst) > 0);
    assert_eq!(engine.error_count(), 0);
    assert!(!engine.is_running());
}

#[test]
fn test_adapter_failure_increments_error_count() {
    let mut engine = GatewayEngine::new(10);
    engine.add_rule(rule("r1"));
    engine.add_adapter(Box::new(FailingAdapter));
    engine.start();
    std::thread::sleep(Duration::from_millis(50));
    engine.stop();
    assert!(engine.error_count() > 0);
}

#[test]
fn test_multiple_adapters_all_invoked() {
    let mut engine = GatewayEngine::new(10);
    engine.add_rule(rule("r1"));
    let c1 = Arc::new(AtomicU32::new(0));
    let c2 = Arc::new(AtomicU32::new(0));
    engine.add_adapter(Box::new(CountingAdapter { calls: c1.clone() }));
    engine.add_adapter(Box::new(CountingAdapter { calls: c2.clone() }));
    engine.start();
    std::thread::sleep(Duration::from_millis(50));
    engine.stop();
    assert!(c1.load(Ordering::SeqCst) > 0);
    assert!(c2.load(Ordering::SeqCst) > 0);
}

#[test]
fn test_stop_without_start_is_noop() {
    let mut engine = GatewayEngine::new(100);
    engine.stop();
    assert!(!engine.is_running());
}

#[test]
fn test_stop_twice_is_noop() {
    let mut engine = GatewayEngine::new(100);
    engine.add_rule(rule("r1"));
    let calls = Arc::new(AtomicU32::new(0));
    engine.add_adapter(Box::new(CountingAdapter {
        calls: calls.clone(),
    }));
    engine.start();
    engine.stop();
    engine.stop();
    assert!(!engine.is_running());
}

#[test]
fn test_transform_variants() {
    assert!(matches!(Transform::Passthrough, Transform::Passthrough));
    assert!(matches!(Transform::Scale(2.5), Transform::Scale(2.5)));
    assert!(matches!(Transform::Offset(-10.0), Transform::Offset(-10.0)));
    assert!(matches!(
        Transform::ScaleAndOffset {
            scale: 1.8,
            offset: 32.0
        },
        Transform::ScaleAndOffset {
            scale: 1.8,
            offset: 32.0
        }
    ));
}

#[test]
fn test_rule_and_endpoint_fields() {
    let rule = rule("forward-1");
    assert_eq!(rule.name, "forward-1");
    assert_eq!(rule.source.device, "src");
    assert_eq!(rule.source.address, "1");
    assert_eq!(rule.target.device, "dst");
    assert_eq!(rule.target.address, "2");
}
