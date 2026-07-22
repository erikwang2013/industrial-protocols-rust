use industrial_protocols_vme::VmeProtocol;
use industrial_protocols_vme::VmeSpace;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(VmeProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("vme").is_some());
}
