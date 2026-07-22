use industrial_protocols_mqtt::MqttProtocol;
use industrial_protocols_kernel::Kernel;

#[test]
fn test_kernel() {
    let kernel = Kernel::builder()
        .with_protocol(Box::new(MqttProtocol::new("test")))
        .build();
    assert!(kernel.protocol_registry().get("mqtt").is_some());
}
