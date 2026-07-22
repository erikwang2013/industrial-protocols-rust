use industrial_protocols_ethercat::EthercatProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(EthercatProtocol::new(1)))
        .build();
    assert!(kernel.protocol_registry().get("ethercat").is_some());
}
