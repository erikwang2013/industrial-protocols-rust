use industrial_protocols_dnp3::Dnp3Protocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(Dnp3Protocol::new(1, 1)))
        .build();
    assert!(kernel.protocol_registry().get("dnp3").is_some());
}
