use industrial_protocols_lonworks::LonworksProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(LonworksProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("lonworks").is_some());
}
