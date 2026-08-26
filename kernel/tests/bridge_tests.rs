// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::bridge::Transport;
use std::io;

struct EchoTransport {
    open: bool,
}

impl EchoTransport {
    fn new() -> Self {
        EchoTransport { open: true }
    }
}

impl Transport for EchoTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        Ok(data.to_vec())
    }
    fn is_open(&self) -> bool {
        self.open
    }
    fn close(&mut self) -> io::Result<()> {
        self.open = false;
        Ok(())
    }
}

#[test]
fn test_transport_trait_object_roundtrip() {
    let mut t: Box<dyn Transport> = Box::new(EchoTransport::new());
    assert!(t.is_open());
    let resp = t.send(b"abc").unwrap();
    assert_eq!(resp, b"abc");
    t.close().unwrap();
    assert!(!t.is_open());
}

#[test]
fn test_transport_send_error_propagates() {
    struct ErrTransport;
    impl Transport for ErrTransport {
        fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broke"))
        }
        fn is_open(&self) -> bool {
            true
        }
        fn close(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut t: Box<dyn Transport> = Box::new(ErrTransport);
    let err = t.send(b"x").unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn test_transport_send_accepts_empty_data() {
    let mut t: Box<dyn Transport> = Box::new(EchoTransport::new());
    let resp = t.send(b"").unwrap();
    assert!(resp.is_empty());
}
