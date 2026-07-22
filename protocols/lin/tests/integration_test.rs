use industrial_protocols_lin::LinProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(LinProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("lin").is_some());
}
