// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_foundationfieldbus::FfProtocol;
use industrial_protocols_foundationfieldbus::FfVariant;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(FfProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("foundationfieldbus").is_some());
}
