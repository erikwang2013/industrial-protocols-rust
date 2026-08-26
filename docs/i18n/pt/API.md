# Referência de API — Industrial Protocols Rust

Este documento contém as principais APIs públicas do conjunto de protocolos industriais: Traits do Protocol SDK, componentes do kernel, camada de bridge, tipos de erro e camada assíncrona.

Voltar ao [README](../README.md) (versões em vários idiomas em [Diretório de idiomas](../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Traits

Cada protocolo integra-se ao kernel implementando os seguintes traits:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // 协议特定地址类型
    type Value: ValueLike;      // 协议特定值类型
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

**Tipos associados** tornam endereços e valores seguros por tipo: Modbus usa `u16`, OPC UA usa `String`, BACnet usa `f32`. **A divisão em traits** permite que protocolos implementem apenas o necessário — um sensor somente leitura não precisa de `ProtocolWrite`.

## 2. Kernel — Microkernel

O kernel não implementa nenhum protocolo; fornece apenas a infraestrutura compartilhada:

| Componente | Responsabilidade | API principal |
|------|------|----------|
| `ProtocolRegistry` | Registro/consulta/classificação de protocolos | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Ciclo de vida das conexões de dispositivos | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | Configuração TOML + sobreposição em runtime | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Publicação/assinatura de eventos | `subscribe()`, `publish()` |
| `CircuitBreaker` | Disjuntor Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fixo/exponencial/linear/nenhum | `run()`, `start()` |
| `GatewayEngine` | Regras de encaminhamento de dados entre protocolos | `add_rule()`, `start()/stop()` |

O gerenciamento de conexões suporta três estratégias:
```rust
pub enum ConnectionStrategy {
    Simple,      // 每次新建，用完释放
    Persistent,  // 单连接复用，断开自动重连
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — Abstração de transporte

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implementação | Dependência |
|--------|------|------|
| `bridge-socket` | Conexão TCP + envio/recepção UDP | std |
| `bridge-serial` | Porta serial real, feature `serial` opcional | `serialport` |
| `bridge-can` | SocketCAN do Linux (CAN_RAW) | `libc` |
| `bridge-ffi` | Carregamento dinâmico do SDK C do fornecedor | `libloading` |
| `bridge-vendor` | Predefinições Siemens/AB/Schneider | — |

## 4. Error — Cadeia de erros estruturada

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 非法地址"
    CircuitOpen { device, retry_after },  // 熔断器已断开
    Context { context, source },  // 附加上文，形成错误链
    // ...
}
```

Exemplo de cadeia de erros:
```
Falha ao ler o registrador de retenção 40001 do PLC-001
  → timeout: device=plc-001, timeout=3000ms
    → falha de conexão: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — wrapper tokio opcional

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// 异步事件流
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
