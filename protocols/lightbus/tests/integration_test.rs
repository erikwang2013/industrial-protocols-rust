use industrial_protocols_lightbus::LightbusProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(LightbusProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("lightbus").is_some());
}
