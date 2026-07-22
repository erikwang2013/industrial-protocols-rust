use industrial_protocols_canopen::CanopenProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(CanopenProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("canopen").is_some());
}
