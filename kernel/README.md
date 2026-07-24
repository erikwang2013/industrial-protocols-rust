# industrial-protocols-kernel

Micro-kernel for industrial protocol implementations in Rust.

## Features

- **ProtocolRegistry** — register and look up protocols by name or category
- **ConnectionManager** — device connection management with 3 strategies (Persistent, OnDemand, Pooled)
- **ConfigRepository** — TOML-based device configuration with runtime overrides
- **EventBus** — publish/subscribe event system
- **CircuitBreaker** — fault protection with configurable thresholds
- **GatewayEngine** — cross-protocol data forwarding engine
- **SDK Traits** — 6 traits defining what a protocol is

## Usage

```rust
use industrial_protocols_kernel::*;

let mut kernel = Kernel::builder()
    .config_from_str(r#"
        [devices.plc-001]
        protocol = "modbus"
        host = "192.168.1.10"
        port = 502
    "#)
    .build();

kernel.boot()?;
// ... use protocols ...
kernel.shutdown()?;
```

License: MIT
