// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_cclink::CclinkProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(CclinkProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("cclink").is_some());
}
