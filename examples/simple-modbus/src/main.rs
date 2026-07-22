use industrial_protocols_kernel::*;
use industrial_protocols_modbus::ModbusProtocol;

fn main() {
    let config_toml = r#"
[devices.plc-001]
protocol = "modbus"
variant = "tcp"
host = "127.0.0.1"
port = 502
unit_id = 1
timeout_ms = 3000
"#;

    let mut kernel = Kernel::builder()
        .config_from_str(config_toml)
        .with_protocol(Box::new(ModbusProtocol::tcp()))
        .build();

    match kernel.boot() {
        Ok(()) => println!("Kernel booted successfully"),
        Err(e) => println!("Boot warning (no real device expected): {e}"),
    }

    if let Some(proto) = kernel.protocol_registry().get("modbus") {
        println!("Loaded protocol: {} v{}", proto.name(), proto.version());
    }

    let _ = kernel.shutdown();
    println!("Kernel shut down.");
}
