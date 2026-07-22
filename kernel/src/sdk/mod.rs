pub mod types;

use crate::error::Result;
use crate::connection::Connection;
use crate::bridge::Transport;
pub use types::*;

pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>>;
    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol {
    type Address: AddressLike;
    type Value: ValueLike;
    fn write(&self, conn: &mut dyn Connection, addr: &Self::Address, value: &Self::Value) -> Result<()>;
    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()>;
}

pub trait ProtocolSubscribe: Protocol {
    type SubscriptionId: Clone + Send + Sync;
    type Address: AddressLike;
    fn subscribe(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Self::SubscriptionId>;
    fn unsubscribe(&self, conn: &mut dyn Connection, sub_id: &Self::SubscriptionId) -> Result<()>;
    fn poll(&self, conn: &mut dyn Connection) -> Result<Vec<SubscriptionUpdate<Self::SubscriptionId>>>;
}

pub trait ProtocolDiscover {
    fn discover(&self, transport: &mut dyn Transport) -> Result<Vec<DiscoveredDevice>>;
}

pub trait ProtocolHealth: Protocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus>;
}

pub trait ProtocolDiagnostics: Protocol {
    fn diagnostics(&self, conn: &mut dyn Connection) -> Result<DiagnosticsReport>;
}
