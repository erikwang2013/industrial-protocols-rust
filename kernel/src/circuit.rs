use crate::error::{ProtocolError, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CircuitConfig { pub threshold: u32, pub cooldown: Duration }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState { Closed, Open, HalfOpen }

struct DeviceCircuit { failures: u32, state: CircuitState, opened_at: Option<Instant> }

pub struct CircuitBreaker {
    config: CircuitConfig,
    devices: Mutex<HashMap<String, DeviceCircuit>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitConfig) -> Self {
        CircuitBreaker { config, devices: Mutex::new(HashMap::new()) }
    }

    pub fn state(&self, device_id: &str) -> CircuitState {
        let mut devices = self.devices.lock().unwrap();
        let entry = devices.entry(device_id.to_string()).or_insert(DeviceCircuit { failures: 0, state: CircuitState::Closed, opened_at: None });
        self.transition_if_ready(entry);
        entry.state
    }

    pub fn record_success(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        if let Some(entry) = devices.get_mut(device_id) {
            entry.failures = 0; entry.state = CircuitState::Closed; entry.opened_at = None;
        }
    }

    pub fn record_failure(&self, device_id: &str) {
        let mut devices = self.devices.lock().unwrap();
        let entry = devices.entry(device_id.to_string()).or_insert(DeviceCircuit { failures: 0, state: CircuitState::Closed, opened_at: None });
        entry.failures += 1;
        if entry.failures >= self.config.threshold {
            entry.state = CircuitState::Open; entry.opened_at = Some(Instant::now());
        }
    }

    pub fn call<F, T>(&self, device_id: &str, f: F) -> Result<T>
    where F: FnOnce() -> Result<T>
    {
        match self.state(device_id) {
            CircuitState::Open => {
                let devices = self.devices.lock().unwrap();
                let retry_after = devices.get(device_id).and_then(|e| e.opened_at)
                    .map(|t| self.config.cooldown.saturating_sub(t.elapsed())).unwrap_or(self.config.cooldown);
                Err(ProtocolError::CircuitOpen { device: device_id.into(), retry_after })
            }
            CircuitState::Closed | CircuitState::HalfOpen => match f() {
                Ok(val) => { self.record_success(device_id); Ok(val) }
                Err(e) => { self.record_failure(device_id); Err(e) }
            },
        }
    }

    fn transition_if_ready(&self, entry: &mut DeviceCircuit) {
        if entry.state == CircuitState::Open {
            if let Some(t) = entry.opened_at {
                if t.elapsed() >= self.config.cooldown { entry.state = CircuitState::HalfOpen; }
            }
        }
    }
}
