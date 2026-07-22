// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_hartip::HartIpProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(HartIpProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("hartip").is_some());
}
