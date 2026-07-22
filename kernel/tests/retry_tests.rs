// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::retry::*;
use std::time::Duration;

#[test]
fn test_fixed_retry() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 100 } };
    let mut state = config.start();
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), None);
}

#[test]
fn test_exponential_backoff() {
    let config = RetryConfig { max_attempts: 4, strategy: RetryStrategy::Exponential { base_ms: 100, max_ms: 2000 } };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays.len(), 3);
    assert!(delays[0] >= Duration::from_millis(100));
    assert!(delays[1] >= delays[0]);
    assert!(delays[2] <= Duration::from_millis(2000));
}

#[test]
fn test_linear_backoff() {
    let config = RetryConfig { max_attempts: 4, strategy: RetryStrategy::Linear { base_ms: 100, step_ms: 50 } };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays[0], Duration::from_millis(100));
    assert_eq!(delays[1], Duration::from_millis(150));
    assert_eq!(delays[2], Duration::from_millis(200));
}

#[test]
fn test_no_retry() {
    let config = RetryConfig { max_attempts: 1, strategy: RetryStrategy::None };
    assert!(config.start().next_delay().is_none());
}

#[test]
fn test_run_succeeds() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 1 } };
    let result = config.run(|| Ok::<i32, &str>(42));
    assert_eq!(result, Ok(42));
}

#[test]
fn test_run_retries_then_fails() {
    let config = RetryConfig { max_attempts: 3, strategy: RetryStrategy::Fixed { delay_ms: 1 } };
    let mut calls = 0;
    let result = config.run(|| { calls += 1; Err::<i32, &str>("fail") });
    assert!(result.is_err());
    assert_eq!(calls, 3);
}
