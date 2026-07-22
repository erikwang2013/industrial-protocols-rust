use industrial_protocols_most::MostProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(MostProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("most").is_some());
}
