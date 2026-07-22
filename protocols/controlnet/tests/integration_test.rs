use industrial_protocols_controlnet::ControlnetProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(ControlnetProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("controlnet").is_some());
}
