use industrial_protocols_worldfip::WorldfipProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(WorldfipProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("worldfip").is_some());
}
