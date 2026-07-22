use industrial_protocols_devicenet::DevicenetProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(DevicenetProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("devicenet").is_some());
}
