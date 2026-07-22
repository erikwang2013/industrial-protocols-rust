// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::sync::{Arc, Mutex};

pub type EventHandler = Box<dyn Fn(ProtocolEvent) + Send + 'static>;

pub struct EventBus {
    handlers: Arc<Mutex<Vec<(EventTopic, EventHandler)>>>,
}

impl EventBus {
    pub fn new(_capacity: usize) -> Self {
        EventBus { handlers: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn subscribe(&self, topic: EventTopic, handler: impl Fn(ProtocolEvent) + Send + 'static) {
        self.handlers.lock().unwrap().push((topic, Box::new(handler)));
    }

    pub fn publish(&self, event: ProtocolEvent) {
        let topic = event.topic();
        let handlers = self.handlers.lock().unwrap();
        for (t, handler) in handlers.iter() {
            if *t == topic { handler(event.clone()); }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTopic { Connection, Error, Health }

#[derive(Debug, Clone)]
pub enum HealthState { Healthy, Degraded, Unhealthy, Unknown }

#[derive(Debug, Clone)]
pub enum Severity { Info, Warning, Error, Critical }

#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    Connected { device_id: String, protocol: String, timestamp: std::time::Instant },
    Disconnected { device_id: String, reason: String },
    ReadCompleted { device_id: String, address: String, latency_ms: u64 },
    WriteCompleted { device_id: String, address: String, latency_ms: u64 },
    Error { device_id: String, error: String, severity: Severity },
    HealthChanged { device_id: String, from: HealthState, to: HealthState },
    CircuitBreakerTriggered { device_id: String, failures: u32 },
}

impl ProtocolEvent {
    pub fn topic(&self) -> EventTopic {
        match self {
            ProtocolEvent::Connected { .. } | ProtocolEvent::Disconnected { .. }
            | ProtocolEvent::ReadCompleted { .. } | ProtocolEvent::WriteCompleted { .. } => EventTopic::Connection,
            ProtocolEvent::Error { .. } | ProtocolEvent::CircuitBreakerTriggered { .. } => EventTopic::Error,
            ProtocolEvent::HealthChanged { .. } => EventTopic::Health,
        }
    }
}
