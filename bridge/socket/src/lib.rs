use industrial_protocols_kernel::bridge::Transport;
use std::io::{self, Read, Write};
use std::net::{TcpStream, UdpSocket, ToSocketAddrs};
use std::time::Duration;

pub struct TcpTransport { stream: TcpStream }

impl TcpTransport {
    pub fn connect(addr: impl ToSocketAddrs, timeout_ms: u64) -> io::Result<Self> {
        let timeout = Duration::from_millis(timeout_ms);
        let addr = addr.to_socket_addrs()?.next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no address resolved"))?;
        let stream = TcpStream::connect_timeout(&addr, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(TcpTransport { stream })
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
    fn is_open(&self) -> bool { true }
    fn close(&mut self) -> io::Result<()> { self.stream.shutdown(std::net::Shutdown::Both) }
}

pub struct UdpTransport { socket: UdpSocket, peer: std::net::SocketAddr }

impl UdpTransport {
    pub fn bind_and_connect(bind_addr: &str, peer_addr: &str, timeout_ms: u64) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        let peer = peer_addr.to_socket_addrs()?.next()
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
    fn is_open(&self) -> bool { true }
    fn close(&mut self) -> io::Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tcp_connect_refused() { assert!(TcpTransport::connect("127.0.0.1:19999", 100).is_err()); }
    #[test]
    fn test_udp_bind_ok() { assert!(UdpTransport::bind_and_connect("127.0.0.1:0", "127.0.0.1:19999", 100).is_ok()); }
}
