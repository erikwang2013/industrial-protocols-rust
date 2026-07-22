use industrial_protocols_hart::HartProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(HartProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("hart").is_some());
}
