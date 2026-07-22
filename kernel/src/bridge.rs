// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 桥接层 — Transport 抽象
//!
//! 定义 [`Transport`] trait，屏蔽底层 I/O 差异（TCP、串口、CAN、FFI），
//! 协议 crate 通过此 trait 与物理层通信，不感知具体传输方式。

use std::io;
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
