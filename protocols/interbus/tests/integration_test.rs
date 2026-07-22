use industrial_protocols_interbus::InterbusProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(InterbusProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("interbus").is_some());
}
