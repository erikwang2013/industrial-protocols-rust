use industrial_protocols_profinet::ProfinetProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(ProfinetProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("profinet").is_some());
}
