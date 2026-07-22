use industrial_protocols_pci::PciProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(PciProtocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("pci").is_some());
}
