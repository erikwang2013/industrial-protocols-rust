// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 事件总线
//!
//! 发布/订阅模式的事件系统。内核内部和协议实现可通过事件总线通信，
//! 无需直接耦合。
//!
//! ## 事件主题
//!
//! - **Connection** — 设备连接/断开、读写完成
//! - **Error** — 协议错误、熔断触发
//! - **Health** — 设备健康状态变更

use std::sync::{Arc, Mutex};

pub type EventHandler = Arc<dyn Fn(ProtocolEvent) + Send + Sync + 'static>;

pub struct EventBus {
    handlers: Arc<Mutex<Vec<(EventTopic, EventHandler)>>>,
}

impl EventBus {
    pub fn new(_capacity: usize) -> Self {
        EventBus {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(
        &self,
        topic: EventTopic,
        handler: impl Fn(ProtocolEvent) + Send + Sync + 'static,
    ) {
        self.handlers
            .lock()
            .unwrap()
            .push((topic, Arc::new(handler)));
    }

    pub fn publish(&self, event: ProtocolEvent) {
        let topic = event.topic();
        // Clone the matching handlers before invoking them so a handler that
        // subscribes or publishes again cannot deadlock on this mutex.
        let handlers: Vec<EventHandler> = {
            let handlers = self.handlers.lock().unwrap();
            handlers
                .iter()
                .filter(|(t, _)| *t == topic)
                .map(|(_, h)| Arc::clone(h))
                .collect()
        };
        for handler in handlers {
            handler(event.clone());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTopic {
    Connection,
    Error,
    Health,
}

#[derive(Debug, Clone)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ProtocolEvent {
    Connected {
        device_id: String,
        protocol: String,
        timestamp: std::time::Instant,
    },
    Disconnected {
        device_id: String,
        reason: String,
    },
    ReadCompleted {
        device_id: String,
        address: String,
        latency_ms: u64,
    },
    WriteCompleted {
        device_id: String,
        address: String,
        latency_ms: u64,
    },
    Error {
        device_id: String,
        error: String,
        severity: Severity,
    },
    HealthChanged {
        device_id: String,
        from: HealthState,
        to: HealthState,
    },
    CircuitBreakerTriggered {
        device_id: String,
        failures: u32,
    },
}

impl ProtocolEvent {
    pub fn topic(&self) -> EventTopic {
        match self {
            ProtocolEvent::Connected { .. }
            | ProtocolEvent::Disconnected { .. }
            | ProtocolEvent::ReadCompleted { .. }
            | ProtocolEvent::WriteCompleted { .. } => EventTopic::Connection,
            ProtocolEvent::Error { .. } | ProtocolEvent::CircuitBreakerTriggered { .. } => {
                EventTopic::Error
            }
            ProtocolEvent::HealthChanged { .. } => EventTopic::Health,
        }
    }
}
