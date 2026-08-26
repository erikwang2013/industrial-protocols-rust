# Referencia de API — Industrial Protocols Rust

Este documento contiene las API públicas principales del conjunto de protocolos industriales: traits del Protocol SDK, componentes del kernel, capa de puente (bridge), tipos de error y capa asíncrona.

Volver al [README](../README.md) (versiones en otros idiomas en el [índice de idiomas](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Traits

Cada protocolo se integra en el kernel implementando los siguientes traits:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // Tipo de dirección específico del protocolo
    type Value: ValueLike;      // Tipo de valor específico del protocolo
    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value>;
    fn read_batch(&self, ...) -> Result<Vec<Self::Value>>;
    fn read_range(&self, ...) -> Result<Vec<Self::Value>>;
}

pub trait ProtocolWrite: Protocol { ... }
pub trait ProtocolSubscribe: Protocol { ... }
pub trait ProtocolDiscover { ... }
pub trait ProtocolHealth: Protocol { ... }
pub trait ProtocolDiagnostics: Protocol { ... }
```

Los **tipos asociados** hacen que direcciones y valores sean seguros por tipo: Modbus usa `u16`, OPC UA usa `String`, BACnet usa `f32`. La **separación de traits** permite implementar los protocolos según la necesidad: un sensor solo de lectura no necesita `ProtocolWrite`.

## 2. Kernel — Microkernel

El kernel no implementa ningún protocolo, solo proporciona la infraestructura compartida:

| Componente | Responsabilidad | API clave |
|------|------|----------|
| `ProtocolRegistry` | Registro/búsqueda/clasificación de protocolos | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Ciclo de vida de las conexiones de dispositivos | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | Configuración TOML + anulación en tiempo de ejecución | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Publicación/suscripción de eventos | `subscribe()`, `publish()` |
| `CircuitBreaker` | Disyuntor Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fijo/exponencial/lineal/ninguno | `run()`, `start()` |
| `GatewayEngine` | Reglas de reenvío de datos entre protocolos | `add_rule()`, `start()/stop()` |

La gestión de conexiones admite tres estrategias:
```rust
pub enum ConnectionStrategy {
    Simple,      // Nueva en cada uso, se libera al terminar
    Persistent,  // Reutiliza una única conexión, reconexión automática al desconectarse
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — Abstracción de transporte

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementación | Dependencias |
|--------|------|------|
| `bridge-socket` | Conexión TCP + envío/recepción UDP | std |
| `bridge-serial` | Puerto serie real, feature opcional `serial` | `serialport` |
| `bridge-can` | SocketCAN de Linux (CAN_RAW) | `libc` |
| `bridge-ffi` | Carga dinámica del SDK C del proveedor | `libloading` |
| `bridge-vendor` | Preajustes Siemens/AB/Schneider | — |

## 4. Error — Cadena de errores estructurada

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 dirección no válida"
    CircuitOpen { device, retry_after },  // El disyuntor está abierto
    Context { context, source },  // Añade contexto para formar la cadena de errores
    // ...
}
```

Ejemplo de cadena de errores:
```
Falló la lectura del registro de retención 40001 del PLC-001
  → Tiempo de espera agotado: device=plc-001, timeout=3000ms
    → Error de conexión: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — Envoltura tokio opcional

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// Flujo de eventos asíncronos
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
