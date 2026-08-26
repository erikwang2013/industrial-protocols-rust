// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::circuit::*;
use industrial_protocols_kernel::error::ProtocolError;
use std::time::Duration;

#[test]
fn test_starts_closed() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 3,
        cooldown: Duration::from_secs(30),
    });
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_opens_after_threshold() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 3,
        cooldown: Duration::from_secs(30),
    });
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_success_resets() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 3,
        cooldown: Duration::from_secs(30),
    });
    cb.record_failure("dev1");
    cb.record_success("dev1");
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_call_trips_breaker() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 2,
        cooldown: Duration::from_millis(100),
    });
    let err = || Err::<i32, _>(ProtocolError::Connection("fail".into()));
    assert!(cb.call("dev1", err).is_err());
    assert!(cb.call("dev1", err).is_err());
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    assert!(cb.call("dev1", || Ok(42)).is_err()); // circuit open
}

#[test]
fn test_independent_devices() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 2,
        cooldown: Duration::from_secs(30),
    });
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    assert_eq!(cb.state("dev2"), CircuitState::Closed);
}

#[test]
fn test_half_open_after_cooldown() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 1,
        cooldown: Duration::from_millis(10),
    });
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(cb.state("dev1"), CircuitState::HalfOpen);
}

#[test]
fn test_call_success_resets_failures() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 2,
        cooldown: Duration::from_secs(30),
    });
    cb.record_failure("dev1");
    let val = cb.call("dev1", || Ok::<i32, ProtocolError>(7));
    assert_eq!(val.unwrap(), 7);
    // Success cleared the failure count, so two more failures reopen.
    cb.record_failure("dev1");
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_call_open_reports_retry_after() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 1,
        cooldown: Duration::from_secs(60),
    });
    cb.record_failure("dev1");
    let err = cb.call("dev1", || Ok::<i32, ProtocolError>(1)).unwrap_err();
    match err {
        ProtocolError::CircuitOpen {
            device,
            retry_after,
        } => {
            assert_eq!(device, "dev1");
            assert!(retry_after <= Duration::from_secs(60));
        }
        other => panic!("expected CircuitOpen, got {other:?}"),
    }
}

#[test]
fn test_half_open_success_transitions_to_closed() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 1,
        cooldown: Duration::from_millis(10),
    });
    cb.record_failure("dev1");
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(cb.state("dev1"), CircuitState::HalfOpen);
    let val = cb.call("dev1", || Ok::<i32, ProtocolError>(1));
    assert_eq!(val.unwrap(), 1);
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_half_open_failure_reopens() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 1,
        cooldown: Duration::from_millis(10),
    });
    cb.record_failure("dev1");
    std::thread::sleep(Duration::from_millis(20));
    let err = cb.call("dev1", || {
        Err::<i32, _>(ProtocolError::Connection("boom".into()))
    });
    assert!(err.is_err());
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_record_success_on_unknown_device_is_noop() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 3,
        cooldown: Duration::from_secs(30),
    });
    cb.record_success("ghost"); // must not panic
    assert_eq!(cb.state("ghost"), CircuitState::Closed);
}

#[test]
fn test_threshold_zero_opens_on_first_failure() {
    let cb = CircuitBreaker::new(CircuitConfig {
        threshold: 0,
        cooldown: Duration::from_secs(30),
    });
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_circuit_config_and_state_derives() {
    let cfg = CircuitConfig {
        threshold: 3,
        cooldown: Duration::from_secs(5),
    };
    let clone = cfg.clone();
    assert_eq!(clone.threshold, 3);
    assert_eq!(CircuitState::Closed, CircuitState::Closed);
    assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
}
