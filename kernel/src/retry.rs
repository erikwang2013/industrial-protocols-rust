// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 重试配置
//!
//! 支持四种重试策略：
//!
//! - **Fixed** — 固定延迟（如每次重试间隔 100ms）
//! - **Exponential** — 指数退避（每次延迟翻倍，有上限）
//! - **Linear** — 线性增长（每次延迟增加固定步长）
//! - **None** — 不重试
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use industrial_protocols_kernel::retry::*;
//!
//! let config = RetryConfig {
//!     max_attempts: 3,
//!     strategy: RetryStrategy::Exponential { base_ms: 100, max_ms: 5000 },
//! };
//! let result = config.run(|| {
//!     // 尝试操作，返回 Result
//!     Ok::<i32, &str>(42)
//! });
//! ```

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub strategy: RetryStrategy,
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Fixed { delay_ms: u64 },
    Exponential { base_ms: u64, max_ms: u64 },
    Linear { base_ms: u64, step_ms: u64 },
    None,
}

impl RetryConfig {
    pub fn start(&self) -> RetryState {
        RetryState { config: self.clone(), attempt: 0, current_delay: self.initial_delay() }
    }

    fn initial_delay(&self) -> Duration {
        match &self.strategy {
            RetryStrategy::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            RetryStrategy::Exponential { base_ms, .. } => Duration::from_millis(*base_ms),
            RetryStrategy::Linear { base_ms, .. } => Duration::from_millis(*base_ms),
            RetryStrategy::None => Duration::ZERO,
        }
    }

    pub fn run<F, T, E>(&self, mut f: F) -> std::result::Result<T, E>
    where F: FnMut() -> std::result::Result<T, E>
    {
        let mut state = self.start();
        loop {
            match f() {
                Ok(val) => return Ok(val),
                Err(e) => match state.next_delay() {
                    Some(delay) => std::thread::sleep(delay),
                    None => return Err(e),
                },
            }
        }
    }
}

pub struct RetryState {
    config: RetryConfig,
    attempt: u32,
    current_delay: Duration,
}

impl RetryState {
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempt >= self.config.max_attempts.saturating_sub(1) {
            return None;
        }
        let delay = self.current_delay;
        self.advance();
        Some(delay)
    }

    fn advance(&mut self) {
        self.attempt += 1;
        self.current_delay = match &self.config.strategy {
            RetryStrategy::Fixed { delay_ms } => Duration::from_millis(*delay_ms),
            RetryStrategy::Exponential { base_ms: _, max_ms } => {
                let next = self.current_delay.as_millis() as u64 * 2;
                Duration::from_millis(next.min(*max_ms))
            }
            RetryStrategy::Linear { base_ms: _, step_ms } => {
                self.current_delay + Duration::from_millis(*step_ms)
            }
            RetryStrategy::None => Duration::ZERO,
        };
    }
}

impl Iterator for RetryState {
    type Item = Duration;
    fn next(&mut self) -> Option<Duration> { self.next_delay() }
}
