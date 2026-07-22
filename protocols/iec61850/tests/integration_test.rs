use industrial_protocols_iec61850::Iec61850Protocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(Iec61850Protocol::new()))
        .build();
    assert!(kernel.protocol_registry().get("iec61850").is_some());
}
