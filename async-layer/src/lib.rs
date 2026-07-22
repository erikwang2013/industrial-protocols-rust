// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! # 异步层（可选）
//!
//! 基于 tokio 的异步包装层，将同步内核 API 包装为 async 方法。
//!
//! ## 核心组件
//!
//! - [`AsyncKernel`] — 使用 `tokio::sync::Mutex` 包装 [`Kernel`]，提供异步 boot/shutdown/read/write
//! - [`AsyncEventStream`] — 将内核的同步 [`EventBus`] 事件转为 tokio 异步流
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use industrial_protocols_async_layer::*;
//! use industrial_protocols_kernel::*;
//!
//! let kernel = Kernel::builder()
//!     .with_protocol(Box::new(my_protocol))
//!     .build();
//!
//! let ak = AsyncKernel::new(kernel);
//! ak.boot().await?;
//!
//! // 异步事件流
//! let mut stream = AsyncEventStream::subscribe(
//!     ak.kernel().blocking_lock(),
//!     EventTopic::Connection,
//! );
//! while let Some(event) = stream.recv().await {
//!     println!("{:?}", event);
//! }
//! ```

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::Result;
use industrial_protocols_kernel::sdk::{ProtocolRead, ProtocolWrite};
use industrial_protocols_kernel::Kernel;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AsyncKernel {
    kernel: Arc<Mutex<Kernel>>,
}

impl AsyncKernel {
    pub fn new(kernel: Kernel) -> Self {
        AsyncKernel { kernel: Arc::new(Mutex::new(kernel)) }
    }

    pub async fn boot(&self) -> std::io::Result<()> {
        let mut k = self.kernel.lock().await;
        k.boot()
    }

    pub async fn shutdown(&self) -> std::io::Result<()> {
        let mut k = self.kernel.lock().await;
        k.shutdown()
    }

    pub async fn read<P: ProtocolRead + ?Sized>(
        &self, protocol: &P, conn: &mut (dyn Connection + Send), addr: &P::Address,
    ) -> Result<P::Value> {
        protocol.read(conn, addr)
    }

    pub async fn write<P: ProtocolWrite + ?Sized>(
        &self, protocol: &P, conn: &mut (dyn Connection + Send), addr: &P::Address, value: &P::Value,
    ) -> Result<()> {
        protocol.write(conn, addr, value)
    }

    pub fn kernel(&self) -> &Arc<Mutex<Kernel>> { &self.kernel }
}

pub struct AsyncEventStream {
    receiver: tokio::sync::mpsc::UnboundedReceiver<industrial_protocols_kernel::event::ProtocolEvent>,
}

impl AsyncEventStream {
    pub fn from_channel(
        receiver: tokio::sync::mpsc::UnboundedReceiver<industrial_protocols_kernel::event::ProtocolEvent>,
    ) -> Self {
        AsyncEventStream { receiver }
    }

    pub fn subscribe(
        kernel: &Kernel,
        topic: industrial_protocols_kernel::event::EventTopic,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        kernel.event_bus().subscribe(topic, move |event| { let _ = tx.send(event); });
        AsyncEventStream { receiver: rx }
    }

    pub async fn recv(&mut self) -> Option<industrial_protocols_kernel::event::ProtocolEvent> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use industrial_protocols_kernel::sdk::*;
    use industrial_protocols_kernel::connection::Connection;
    use std::io;

    struct TestProtocol;
    impl Protocol for TestProtocol {
        fn name(&self) -> &'static str { "test" }
        fn version(&self) -> &'static str { "1.0" }
        fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
        fn supported_variants(&self) -> &'static [&'static str] { &["tcp"] }
    }
    impl ProtocolRead for TestProtocol {
        type Address = u16; type Value = u16;
        fn read(&self, _c: &mut dyn Connection, a: &Self::Address) -> Result<Self::Value> { Ok(*a * 2) }
        fn read_batch(&self, _c: &mut dyn Connection, a: &[Self::Address]) -> Result<Vec<Self::Value>> { Ok(a.iter().map(|x| x * 2).collect()) }
        fn read_range(&self, _c: &mut dyn Connection, s: &Self::Address, n: u16) -> Result<Vec<Self::Value>> { Ok((*s..*s + n).map(|x| x * 2).collect()) }
    }

    struct TestConn;
    impl Connection for TestConn {
        fn send(&mut self, _: &[u8]) -> io::Result<Vec<u8>> { Ok(vec![42]) }
        fn close(&mut self) -> io::Result<()> { Ok(()) }
        fn is_alive(&self) -> bool { true }
    }

    #[tokio::test]
    async fn test_async_read() {
        let kernel = Kernel::builder().with_protocol(Box::new(TestProtocol)).build();
        let ak = AsyncKernel::new(kernel);
        let mut conn = TestConn;
        let val = ak.read(&TestProtocol, &mut conn, &21).await.unwrap();
        assert_eq!(val, 42);
    }

    #[tokio::test]
    async fn test_async_boot_shutdown() {
        let kernel = Kernel::builder().with_protocol(Box::new(TestProtocol)).build();
        let ak = AsyncKernel::new(kernel);
        assert!(ak.boot().await.is_ok());
        assert!(ak.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_event_stream() {
        // Create kernel and subscribe BEFORE entering async context
        let kernel = Kernel::builder().with_protocol(Box::new(TestProtocol)).build();

        let mut stream = AsyncEventStream::subscribe(
            &kernel,
            industrial_protocols_kernel::event::EventTopic::Connection,
        );

        // Publish synchronously (before wrapping in Mutex)
        kernel.event_bus().publish(industrial_protocols_kernel::event::ProtocolEvent::Connected {
            device_id: "test".into(), protocol: "test".into(), timestamp: std::time::Instant::now(),
        });

        let event = stream.recv().await;
        assert!(event.is_some());
    }
}
