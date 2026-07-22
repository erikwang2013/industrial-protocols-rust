use industrial_protocols_isa100::Isa100Protocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(Isa100Protocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("isa100").is_some());
}
