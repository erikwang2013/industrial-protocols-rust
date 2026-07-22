use industrial_protocols_kernel::event::*;
use std::sync::{Arc, Mutex};

#[test]
fn test_subscribe_and_publish() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Connection, move |event| { r.lock().unwrap().push(format!("{:?}", event)); });
    bus.publish(ProtocolEvent::Connected { device_id: "plc-001".into(), protocol: "modbus".into(), timestamp: std::time::Instant::now() });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert!(received.lock().unwrap()[0].contains("plc-001"));
}

#[test]
fn test_error_event() {
    let bus = EventBus::new(16);
    let received = Arc::new(Mutex::new(Vec::new()));
    let r = received.clone();
    bus.subscribe(EventTopic::Error, move |event| { r.lock().unwrap().push(format!("{:?}", event)); });
    bus.publish(ProtocolEvent::Error { device_id: "plc-001".into(), error: "timeout".into(), severity: Severity::Critical });
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(received.lock().unwrap().len(), 1);
}

#[test]
fn test_event_topic_routing() {
    assert_eq!(ProtocolEvent::ReadCompleted { device_id: "d".into(), address: "a".into(), latency_ms: 5 }.topic(), EventTopic::Connection);
    assert_eq!(ProtocolEvent::CircuitBreakerTriggered { device_id: "d".into(), failures: 5 }.topic(), EventTopic::Error);
    assert_eq!(ProtocolEvent::HealthChanged { device_id: "d".into(), from: HealthState::Healthy, to: HealthState::Unhealthy }.topic(), EventTopic::Health);
}
