use industrial_protocols_dali::DaliProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(DaliProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("dali").is_some());
}
