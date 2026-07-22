// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use std::io;
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
