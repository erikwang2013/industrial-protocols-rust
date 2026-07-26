// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::Kernel;
use industrial_protocols_kline::KlineProtocol;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(KlineProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("kline").is_some());
}
