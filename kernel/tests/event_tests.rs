// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::event::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_subscribe_and_publish() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Connection, move |event| {
        r.lock().unwrap().push(format!("{:?}", event));
    });
    bus.publish(ProtocolEvent::Connected {
        device_id: "plc-001".into(),
        protocol: "modbus".into(),
        timestamp: std::time::Instant::now(),
    });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(received.lock().unwrap()[0].contains("plc-001"));
}

#[test]
fn test_error_event() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Error, move |event| {
        r.lock().unwrap().push(format!("{:?}", event));
    });
    bus.publish(ProtocolEvent::Error {
        device_id: "plc-001".into(),
        error: "timeout".into(),
        severity: Severity::Critical,
    });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(received.lock().unwrap().len(), 1);
}

#[test]
fn test_event_topic_routing() {
    assert_eq!(
        ProtocolEvent::ReadCompleted {
            device_id: "d".into(),
            address: "a".into(),
            latency_ms: 5
        }
        .topic(),
        EventTopic::Connection
    );
    assert_eq!(
        ProtocolEvent::CircuitBreakerTriggered {
            device_id: "d".into(),
            failures: 5
        }
        .topic(),
        EventTopic::Error
    );
    assert_eq!(
        ProtocolEvent::HealthChanged {
            device_id: "d".into(),
            from: HealthState::Healthy,
            to: HealthState::Unhealthy
        }
        .topic(),
        EventTopic::Health
    );
}

#[test]
fn test_topic_for_remaining_variants() {
    assert_eq!(
        ProtocolEvent::Disconnected {
            device_id: "d".into(),
            reason: "bye".into()
        }
        .topic(),
        EventTopic::Connection
    );
    assert_eq!(
        ProtocolEvent::WriteCompleted {
            device_id: "d".into(),
            address: "a".into(),
            latency_ms: 1
        }
        .topic(),
        EventTopic::Connection
    );
    assert_eq!(
        ProtocolEvent::Error {
            device_id: "d".into(),
            error: "e".into(),
            severity: Severity::Error
        }
        .topic(),
        EventTopic::Error
    );
}

#[test]
fn test_topic_filtering_ignores_other_topics() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(0usize));
    let r = received.clone();
    bus.subscribe(EventTopic::Health, move |_| {
        *r.lock().unwrap() += 1;
    });
    bus.publish(ProtocolEvent::Connected {
        device_id: "d".into(),
        protocol: "p".into(),
        timestamp: std::time::Instant::now(),
    });
    assert_eq!(*received.lock().unwrap(), 0);
    bus.publish(ProtocolEvent::HealthChanged {
        device_id: "d".into(),
        from: HealthState::Healthy,
        to: HealthState::Degraded,
    });
    assert_eq!(*received.lock().unwrap(), 1);
}

#[test]
fn test_multiple_subscribers_all_notified() {
    let bus = EventBus::new(16);
    let a = Arc::new(Mutex::new(0usize));
    let b = Arc::new(Mutex::new(0usize));
    let (a1, b1) = (a.clone(), b.clone());
    bus.subscribe(EventTopic::Connection, move |_| {
        *a1.lock().unwrap() += 1;
    });
    bus.subscribe(EventTopic::Connection, move |_| {
        *b1.lock().unwrap() += 1;
    });
    bus.publish(ProtocolEvent::Connected {
        device_id: "d".into(),
        protocol: "p".into(),
        timestamp: std::time::Instant::now(),
    });
    assert_eq!(*a.lock().unwrap(), 1);
    assert_eq!(*b.lock().unwrap(), 1);
}

#[test]
fn test_publish_without_subscribers_is_noop() {
    let bus = EventBus::new(16);
    bus.publish(ProtocolEvent::Connected {
        device_id: "d".into(),
        protocol: "p".into(),
        timestamp: std::time::Instant::now(),
    });
}

#[test]
fn test_event_clone_and_debug() {
    let ev = ProtocolEvent::Connected {
        device_id: "d".into(),
        protocol: "p".into(),
        timestamp: std::time::Instant::now(),
    };
    let ev2 = ev.clone();
    assert!(format!("{ev2:?}").contains("Connected"));
}

#[test]
fn test_severity_and_health_state_variants() {
    let severities = [
        Severity::Info,
        Severity::Warning,
        Severity::Error,
        Severity::Critical,
    ];
    assert_eq!(severities.len(), 4);
    assert!(matches!(HealthState::Healthy, HealthState::Healthy));
    assert!(matches!(HealthState::Unknown, HealthState::Unknown));
}
