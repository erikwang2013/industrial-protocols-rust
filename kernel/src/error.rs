use std::time::Duration;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProtocolError {
    #[error("connection failed: {0}")]
    Connection(String),

    #[error("timeout: device={device}, timeout={timeout_ms}ms")]
    Timeout { device: String, timeout_ms: u64 },

    #[error("frame parse error: protocol={protocol}, reason={reason}")]
    Parse { protocol: String, reason: String },

    #[error("device error: device={device}, code={code:#04X}, message={message}")]
    DeviceError { device: String, code: u16, message: String },

    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("operation unsupported: protocol={protocol}, operation={operation}")]
    Unsupported { protocol: String, operation: String },

    #[error("circuit breaker open: device={device}, retry_after={retry_after:?}")]
    CircuitOpen { device: String, retry_after: Duration },

    #[error("config error: {0}")]
    Config(String),

    #[error("protocol-specific error: {0}")]
    ProtocolSpecific(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<ProtocolError>,
    },
}

impl ProtocolError {
    pub fn context(self, ctx: impl Into<String>) -> Self {
        ProtocolError::Context { context: ctx.into(), source: Box::new(self) }
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        ProtocolError::Connection(e.to_string())
    }
}

impl From<String> for ProtocolError {
    fn from(s: String) -> Self {
        ProtocolError::ProtocolSpecific(Box::new(std::io::Error::new(std::io::ErrorKind::Other, s)))
    }
}

impl From<&str> for ProtocolError {
    fn from(s: &str) -> Self {
        ProtocolError::ProtocolSpecific(Box::new(std::io::Error::new(std::io::ErrorKind::Other, s.to_string())))
    }
}
