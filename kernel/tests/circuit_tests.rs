use industrial_protocols_kernel::circuit::*;
use industrial_protocols_kernel::error::ProtocolError;
use std::time::Duration;

#[test]
fn test_starts_closed() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_opens_after_threshold() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1"); cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
}

#[test]
fn test_success_resets() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 3, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1"); cb.record_success("dev1");
    cb.record_failure("dev1"); cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Closed);
}

#[test]
fn test_call_trips_breaker() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 2, cooldown: Duration::from_millis(100) });
    let err = || Err::<i32, _>(ProtocolError::Connection("fail".into()));
    assert!(cb.call("dev1", err).is_err());
    assert!(cb.call("dev1", err).is_err());
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    assert!(cb.call("dev1", || Ok(42)).is_err()); // circuit open
}

#[test]
fn test_independent_devices() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 2, cooldown: Duration::from_secs(30) });
    cb.record_failure("dev1"); cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    assert_eq!(cb.state("dev2"), CircuitState::Closed);
}

#[test]
fn test_half_open_after_cooldown() {
    let cb = CircuitBreaker::new(CircuitConfig { threshold: 1, cooldown: Duration::from_millis(10) });
    cb.record_failure("dev1");
    assert_eq!(cb.state("dev1"), CircuitState::Open);
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(cb.state("dev1"), CircuitState::HalfOpen);
}
