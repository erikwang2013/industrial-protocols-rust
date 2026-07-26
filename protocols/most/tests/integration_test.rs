// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::Kernel;
use industrial_protocols_most::MostProtocol;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(MostProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("most").is_some());
}
