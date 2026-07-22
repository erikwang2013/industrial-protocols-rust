// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # FFI 桥接层
//!
//! 通过 `libloading` 动态加载厂商 C SDK（.so/.dll/.dylib），
//! 将 unsafe FFI 调用封装在 [`SafeVendorTransport`] 中，对外暴露安全的 [`Transport`] trait。
//! 支持可配置的函数符号名和初始化参数。

//! FFI bridge — safe wrapper for vendor C SDK dynamic loading.
//!
//! Uses `libloading` to dynamically load vendor-provided shared libraries
//! and wraps unsafe FFI calls in safe Rust traits.

use industrial_protocols_kernel::bridge::Transport;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

pub struct FfiConfig {
    pub library_path: PathBuf,
    pub vendor: String,
    pub init_params: HashMap<String, String>,
    pub symbols: FfiSymbols,
}

#[derive(Debug, Clone)]
pub struct FfiSymbols {
    pub init: String,
    pub read: String,
    pub write: String,
    pub shutdown: String,
}

impl Default for FfiSymbols {
    fn default() -> Self {
        FfiSymbols {
            init: "vendor_init".into(),
            read: "vendor_read".into(),
            write: "vendor_write".into(),
            shutdown: "vendor_shutdown".into(),
        }
    }
}

type FfiInitFn = unsafe extern "C" fn(*const libc::c_char) -> libc::c_int;
type FfiReadFn = unsafe extern "C" fn(libc::c_uint, *mut u8, libc::c_uint) -> libc::c_int;
type FfiWriteFn = unsafe extern "C" fn(libc::c_uint, *const u8, libc::c_uint) -> libc::c_int;
type FfiShutdownFn = unsafe extern "C" fn() -> libc::c_int;

/// Safe wrapper around a dynamically loaded vendor driver.
/// Uses raw function pointers (transmuted from libloading symbols)
/// to avoid lifetime coupling between Library and Symbol.
pub struct SafeVendorTransport {
    library: libloading::Library,
    read_fn: unsafe extern "C" fn(libc::c_uint, *mut u8, libc::c_uint) -> libc::c_int,
    write_fn: unsafe extern "C" fn(libc::c_uint, *const u8, libc::c_uint) -> libc::c_int,
    shutdown_fn: unsafe extern "C" fn() -> libc::c_int,
    vendor: String,
    open: bool,
}

impl SafeVendorTransport {
    pub fn load(config: FfiConfig) -> io::Result<Self> {
        unsafe {
            let lib = libloading::Library::new(&config.library_path)
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("failed to load: {e}")))?;

            // Get init symbol and call it
            let init_sym: libloading::Symbol<FfiInitFn> = lib.get(config.symbols.init.as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("init symbol: {e}")))?;
            let params = std::ffi::CString::new(
                config.init_params.get("config").cloned().unwrap_or_default()
            ).unwrap_or_default();
            let rc = init_sym(params.as_ptr());
            if rc != 0 {
                return Err(io::Error::new(io::ErrorKind::Other, format!("vendor init failed: code {rc}")));
            }
            drop(init_sym);

            // Get function symbols and transmute to raw pointers for storage
            let read_sym: libloading::Symbol<FfiReadFn> = lib.get(config.symbols.read.as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("read: {e}")))?;
            let read_fn: FfiReadFn = std::mem::transmute_copy(&*read_sym);
            drop(read_sym);

            let write_sym: libloading::Symbol<FfiWriteFn> = lib.get(config.symbols.write.as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("write: {e}")))?;
            let write_fn: FfiWriteFn = std::mem::transmute_copy(&*write_sym);
            drop(write_sym);

            let shutdown_sym: libloading::Symbol<FfiShutdownFn> = lib.get(config.symbols.shutdown.as_bytes())
                .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("shutdown: {e}")))?;
            let shutdown_fn: FfiShutdownFn = std::mem::transmute_copy(&*shutdown_sym);
            drop(shutdown_sym);

            Ok(SafeVendorTransport {
                library: lib,
                read_fn, write_fn, shutdown_fn,
                vendor: config.vendor,
                open: true,
            })
        }
    }

    pub fn read_raw(&self, offset: u32, len: u32) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len as usize];
        let rc = unsafe { (self.read_fn)(offset, buf.as_mut_ptr(), len) };
        if rc < 0 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("read failed: {rc}")));
        }
        Ok(buf)
    }

    pub fn write_raw(&self, offset: u32, data: &[u8]) -> io::Result<()> {
        let rc = unsafe { (self.write_fn)(offset, data.as_ptr(), data.len() as u32) };
        if rc < 0 {
            return Err(io::Error::new(io::ErrorKind::Other, format!("write failed: {rc}")));
        }
        Ok(())
    }
}

impl Transport for SafeVendorTransport {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if data.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "FFI frame too short"));
        }
        let offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let payload = &data[4..];
        if !payload.is_empty() {
            self.write_raw(offset, payload)?;
        }
        self.read_raw(offset, 256)
    }
    fn is_open(&self) -> bool { self.open }
    fn close(&mut self) -> io::Result<()> {
        self.open = false;
        unsafe { (self.shutdown_fn)(); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_config_defaults() {
        let syms = FfiSymbols::default();
        assert_eq!(syms.init, "vendor_init");
        assert_eq!(syms.read, "vendor_read");
    }

    #[test]
    fn test_load_nonexistent_library() {
        let config = FfiConfig {
            library_path: PathBuf::from("/tmp/nonexistent_lib.so"),
            vendor: "test".into(),
            init_params: HashMap::new(),
            symbols: FfiSymbols::default(),
        };
        assert!(SafeVendorTransport::load(config).is_err());
    }
}
