use industrial_protocols_flexray::FlexrayProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(FlexrayProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("flexray").is_some());
}
