// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::retry::*;
use std::time::Duration;

#[test]
fn test_fixed_retry() {
    let config = RetryConfig {
        max_attempts: 3,
        strategy: RetryStrategy::Fixed { delay_ms: 100 },
    };
    let mut state = config.start();
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), Some(Duration::from_millis(100)));
    assert_eq!(state.next_delay(), None);
}

#[test]
fn test_exponential_backoff() {
    let config = RetryConfig {
        max_attempts: 4,
        strategy: RetryStrategy::Exponential {
            base_ms: 100,
            max_ms: 2000,
        },
    };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays.len(), 3);
    assert!(delays[0] >= Duration::from_millis(100));
    assert!(delays[1] >= delays[0]);
    assert!(delays[2] <= Duration::from_millis(2000));
}

#[test]
fn test_linear_backoff() {
    let config = RetryConfig {
        max_attempts: 4,
        strategy: RetryStrategy::Linear {
            base_ms: 100,
            step_ms: 50,
        },
    };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(delays[0], Duration::from_millis(100));
    assert_eq!(delays[1], Duration::from_millis(150));
    assert_eq!(delays[2], Duration::from_millis(200));
}

#[test]
fn test_no_retry() {
    let config = RetryConfig {
        max_attempts: 1,
        strategy: RetryStrategy::None,
    };
    assert!(config.start().next_delay().is_none());
}

#[test]
fn test_run_succeeds() {
    let config = RetryConfig {
        max_attempts: 3,
        strategy: RetryStrategy::Fixed { delay_ms: 1 },
    };
    let result = config.run(|| Ok::<i32, &str>(42));
    assert_eq!(result, Ok(42));
}

#[test]
fn test_run_retries_then_fails() {
    let config = RetryConfig {
        max_attempts: 3,
        strategy: RetryStrategy::Fixed { delay_ms: 1 },
    };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        Err::<i32, &str>("fail")
    });
    assert!(result.is_err());
    assert_eq!(calls, 3);
}

#[test]
fn test_exponential_caps_at_max_ms() {
    let config = RetryConfig {
        max_attempts: 5,
        strategy: RetryStrategy::Exponential {
            base_ms: 100,
            max_ms: 250,
        },
    };
    let delays: Vec<_> = config.start().collect();
    assert_eq!(
        delays,
        vec![
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(250),
            Duration::from_millis(250),
        ]
    );
}

#[test]
fn test_max_attempts_one_yields_no_delays() {
    let config = RetryConfig {
        max_attempts: 1,
        strategy: RetryStrategy::Fixed { delay_ms: 10 },
    };
    assert_eq!(config.start().next_delay(), None);
}

#[test]
fn test_run_succeeds_on_retry() {
    let config = RetryConfig {
        max_attempts: 3,
        strategy: RetryStrategy::Fixed { delay_ms: 1 },
    };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        if calls < 2 {
            Err::<i32, &str>("fail")
        } else {
            Ok(42)
        }
    });
    assert_eq!(result, Ok(42));
    assert_eq!(calls, 2);
}

#[test]
fn test_run_with_zero_attempts_still_calls_once() {
    let config = RetryConfig {
        max_attempts: 0,
        strategy: RetryStrategy::Fixed { delay_ms: 1 },
    };
    let mut calls = 0;
    let result = config.run(|| {
        calls += 1;
        Err::<i32, &str>("fail")
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
}

#[test]
fn test_retry_state_iterator_yields_max_attempts_minus_one() {
    let config = RetryConfig {
        max_attempts: 4,
        strategy: RetryStrategy::Linear {
            base_ms: 10,
            step_ms: 10,
        },
    };
    assert_eq!(config.start().count(), 3);
}
