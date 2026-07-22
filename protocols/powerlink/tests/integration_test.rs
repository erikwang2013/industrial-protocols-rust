use industrial_protocols_powerlink::PowerlinkProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(PowerlinkProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("powerlink").is_some());
}
