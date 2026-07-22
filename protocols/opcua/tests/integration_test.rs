use industrial_protocols_opcua::OpcuaProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(OpcuaProtocol::new("test")))
        .build();
    assert!(kernel.protocol_registry().get("opcua").is_some());
}
