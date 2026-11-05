// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # CAN 总线传输
//!
//! Linux SocketCAN (CAN_RAW) 实现。通过 `libc` 直接操作 CAN 套接字，
//! 支持 CAN 2.0A（11-bit ID）和 CAN 2.0B（29-bit ID）帧格式。
//! 非 Linux 平台使用存根实现。

//! CAN bus transport via Linux SocketCAN.
//!
//! Uses raw CAN_RAW sockets to send/receive CAN frames.
//! Requires the `socketcan` feature (Linux only).

use industrial_protocols_kernel::bridge::Transport;
use std::io;

/// A single CAN frame.
#[derive(Debug, Clone)]
pub struct CanFrame {
    pub id: u32,
    pub extended: bool,
    pub data: Vec<u8>,
}

/// CAN transport using Linux SocketCAN.
#[cfg(target_os = "linux")]
pub struct CanTransport {
    socket: std::os::fd::RawFd,
}

#[cfg(target_os = "linux")]
impl CanTransport {
    /// Open a CAN socket on the given interface.
    /// Requires CAP_NET_RAW or root.
    pub fn open(interface: &str) -> io::Result<Self> {
        // Create CAN_RAW socket
        let fd = unsafe {
            libc::socket(
                libc::PF_CAN,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC,
                libc::CAN_RAW,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        // Get interface index
        let ifname = std::ffi::CString::new(interface).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "interface name contains a NUL byte",
            )
        })?;
        let ifindex = unsafe { libc::if_nametoindex(ifname.as_ptr()) };
        if ifindex == 0 {
            unsafe { libc::close(fd) };
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("interface '{}' not found", interface),
            ));
        }

        // Bind to interface
        let mut addr: libc::sockaddr_can = unsafe { std::mem::zeroed() };
        addr.can_family = libc::AF_CAN as u16;
        addr.can_ifindex = ifindex as i32;

        let ret = unsafe {
            libc::bind(
                fd,
                &addr as *const _ as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_can>() as u32,
            )
        };
        if ret < 0 {
            unsafe { libc::close(fd) };
            return Err(io::Error::last_os_error());
        }

        Ok(CanTransport { socket: fd })
    }

    /// Send a raw CAN frame.
    pub fn send_frame(&self, id: u32, data: &[u8], extended: bool) -> io::Result<()> {
        let mut frame: libc::can_frame = unsafe { std::mem::zeroed() };
        frame.can_id = id;
        if extended {
            frame.can_id |= libc::CAN_EFF_FLAG;
        }
        frame.can_dlc = data.len().min(8) as u8;
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                frame.data.as_mut_ptr(),
                frame.can_dlc as usize,
            );
        }
        let ret = unsafe {
            libc::write(
                self.socket,
                &frame as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::can_frame>(),
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Receive a CAN frame.
    pub fn recv_frame(&self, timeout_ms: u64) -> io::Result<CanFrame> {
        use std::time::{Duration, Instant};
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let _tv = libc::timeval {
                tv_sec: remaining.as_secs() as libc::time_t,
                tv_usec: remaining.subsec_micros() as libc::suseconds_t,
            };
            let mut fds = libc::pollfd {
                fd: self.socket,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = unsafe { libc::poll(&mut fds, 1, (remaining.as_millis() as i32).min(1000)) };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
            if ret == 0 {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "CAN recv timeout"));
            }
            if fds.revents & libc::POLLIN != 0 {
                let mut frame: libc::can_frame = unsafe { std::mem::zeroed() };
                let ret = unsafe {
                    libc::read(
                        self.socket,
                        &mut frame as *mut _ as *mut libc::c_void,
                        std::mem::size_of::<libc::can_frame>(),
                    )
                };
                if ret >= 0 {
                    let id = frame.can_id & !libc::CAN_EFF_FLAG;
                    let extended = frame.can_id & libc::CAN_EFF_FLAG != 0;
                    let data = frame.data[..frame.can_dlc as usize].to_vec();
                    return Ok(CanFrame { id, extended, data });
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for CanTransport {
    fn drop(&mut self) {
        if self.socket >= 0 {
            unsafe { libc::close(self.socket) };
        }
    }
}

#[cfg(target_os = "linux")]
impl Transport for CanTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if data.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "CAN frame too short",
            ));
        }
        let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let payload = &data[4..];
        self.send_frame(id, payload, false)?;
        match self.recv_frame(1000) {
            Ok(frame) => Ok(frame.data),
            Err(_) => Ok(vec![]),
        }
    }
    fn is_open(&self) -> bool {
        self.socket >= 0
    }
    fn close(&mut self) -> io::Result<()> {
        if self.socket >= 0 {
            unsafe { libc::close(self.socket) };
            self.socket = -1;
        }
        Ok(())
    }
}

/// Stub for non-Linux platforms.
#[cfg(not(target_os = "linux"))]
pub struct CanTransport;

#[cfg(not(target_os = "linux"))]
impl CanTransport {
    pub fn open(_interface: &str) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SocketCAN only available on Linux",
        ))
    }
}

#[cfg(not(target_os = "linux"))]
impl Transport for CanTransport {
    fn send(&mut self, _data: &[u8]) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SocketCAN only available on Linux",
        ))
    }
    fn is_open(&self) -> bool {
        false
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame_default() {
        let frame = CanFrame {
            id: 0x123,
            extended: false,
            data: vec![1, 2, 3],
        };
        assert_eq!(frame.id, 0x123);
        assert!(!frame.extended);
        assert_eq!(frame.data.len(), 3);
    }

    #[test]
    fn test_can_frame_extended_29bit() {
        let frame = CanFrame {
            id: 0x1FFFFFFF,
            extended: true,
            data: vec![0u8; 8],
        };
        assert!(frame.extended);
        assert_eq!(frame.id, 0x1FFFFFFF);
        assert_eq!(frame.data.len(), 8);
    }

    #[test]
    fn test_can_frame_clone_debug() {
        let frame = CanFrame {
            id: 0x55,
            extended: false,
            data: vec![1, 2],
        };
        let clone = frame.clone();
        assert_eq!(clone.id, frame.id);
        assert_eq!(clone.data, frame.data);
        let _dbg = format!("{frame:?}");
    }

    #[test]
    fn test_open_bogus_interface_fails() {
        // Linux: socket creation or if_nametoindex fails; other platforms:
        // the stub rejects outright. Either way open must error.
        assert!(CanTransport::open("definitely_not_an_iface_12345").is_err());
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_stub_rejects() {
        assert!(CanTransport::open("can0").is_err());
    }
}
