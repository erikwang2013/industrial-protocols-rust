// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # TCP/UDP Socket 传输
//!
//! 提供 [`TcpTransport`] 和 [`UdpTransport`] 两种网络传输实现，
//! 实现 [`Transport`] trait。支持连接超时和读写超时配置。

use industrial_protocols_kernel::bridge::Transport;
use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub struct TcpTransport {
    stream: TcpStream,
    closed: bool,
}

impl TcpTransport {
    pub fn connect(addr: impl ToSocketAddrs, timeout_ms: u64) -> io::Result<Self> {
        let timeout = Duration::from_millis(timeout_ms);
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address resolved"))?;
        let stream = TcpStream::connect_timeout(&addr, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(TcpTransport {
            stream,
            closed: false,
        })
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.stream.write_all(data)?;
        let mut buf = vec![0u8; 4096];
        let n = self.stream.read(&mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }
    fn is_open(&self) -> bool {
        !self.closed
    }
    fn close(&mut self) -> io::Result<()> {
        self.closed = true;
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

pub struct UdpTransport {
    socket: UdpSocket,
    peer: std::net::SocketAddr,
}

impl UdpTransport {
    pub fn bind_and_connect(bind_addr: &str, peer_addr: &str, timeout_ms: u64) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        let peer = peer_addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address resolved"))?;
        socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
        socket.set_write_timeout(Some(Duration::from_millis(timeout_ms)))?;
        Ok(UdpTransport { socket, peer })
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        self.socket.send_to(data, self.peer)?;
        let mut buf = vec![0u8; 4096];
        let (n, _) = self.socket.recv_from(&mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }
    fn is_open(&self) -> bool {
        true
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::thread;

    #[test]
    fn test_tcp_connect_refused() {
        assert!(TcpTransport::connect("127.0.0.1:19999", 100).is_err());
    }
    #[test]
    fn test_udp_bind_ok() {
        assert!(UdpTransport::bind_and_connect("127.0.0.1:0", "127.0.0.1:19999", 100).is_ok());
    }

    #[test]
    fn test_tcp_echo_roundtrip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            loop {
                match sock.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => sock.write_all(&buf[..n]).unwrap(),
                }
            }
        });
        let mut t = TcpTransport::connect(addr, 1000).unwrap();
        assert!(t.is_open());
        let resp = t.send(b"ping").unwrap();
        assert_eq!(resp, b"ping");
        assert!(t.close().is_ok());
        assert!(!t.is_open());
        server.join().unwrap();
    }

    #[test]
    fn test_tcp_connect_refused_on_recently_freed_port() {
        // Grab a free port, release it, then connect: the port is closed.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        assert!(TcpTransport::connect(addr, 1000).is_err());
    }

    #[test]
    fn test_tcp_connect_unresolvable_address() {
        assert!(TcpTransport::connect("256.256.256.256:1", 100).is_err());
    }

    #[test]
    fn test_tcp_echo_then_server_close() {
        // Server echoes one packet then closes the connection.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let n = sock.read(&mut buf).unwrap();
            sock.write_all(&buf[..n]).unwrap();
        });
        let mut t = TcpTransport::connect(addr, 1000).unwrap();
        assert_eq!(t.send(b"hello").unwrap(), b"hello");
        server.join().unwrap();
        // Peer is gone: no echo comes back (EOF or a connection error).
        if let Ok(v) = t.send(b"again") {
            assert!(v.is_empty(), "closed peer must not echo data");
        }
        // Close is best-effort on a reset socket, but the transport
        // must report closed, and rejects further writes.
        let _ = t.close();
        assert!(!t.is_open());
        assert!(t.send(b"again").is_err());
    }

    #[test]
    fn test_tcp_send_times_out_when_no_peer_responds() {
        // Listener never accepts: connection succeeds via the backlog,
        // but the read must time out.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let mut t = TcpTransport::connect(addr, 100).unwrap();
        assert!(t.send(b"nobody-home").is_err());
        drop(listener);
    }

    #[test]
    fn test_udp_echo_roundtrip() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let (n, from) = server.recv_from(&mut buf).unwrap();
            server.send_to(&buf[..n], from).unwrap();
        });
        let mut t =
            UdpTransport::bind_and_connect("127.0.0.1:0", &server_addr.to_string(), 1000).unwrap();
        assert!(t.is_open());
        let resp = t.send(b"udp-ping").unwrap();
        assert_eq!(resp, b"udp-ping");
        assert!(t.close().is_ok());
        handle.join().unwrap();
    }

    #[test]
    fn test_udp_close_idempotent() {
        let mut t = UdpTransport::bind_and_connect("127.0.0.1:0", "127.0.0.1:9", 100).unwrap();
        assert!(t.close().is_ok());
        assert!(t.close().is_ok());
    }
}
