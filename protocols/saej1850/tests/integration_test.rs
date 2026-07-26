// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::Kernel;
use industrial_protocols_saej1850::J1850Variant;
use industrial_protocols_saej1850::SaeJ1850Protocol;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(SaeJ1850Protocol::new(J1850Variant::Vpw)))
        .build();
    assert!(kernel.protocol_registry().get("saej1850").is_some());
}
