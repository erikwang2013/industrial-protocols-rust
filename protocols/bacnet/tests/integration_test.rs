use industrial_protocols_bacnet::BacnetProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(BacnetProtocol::new(1001)))
        .build();
    assert!(kernel.protocol_registry().get("bacnet").is_some());
}
