# Справочник по API — Industrial Protocols Rust

В этом документе описаны основные публичные API набора промышленных протоколов: Trait-ы Protocol SDK, компоненты ядра, мостовой слой, типы ошибок и асинхронный слой.

Вернуться к [README](../../README.md) (многоязычные версии — в разделе [«Многоязычность»](../../README.md#多语言--languages)).

## 1. Protocol SDK — 6 Trait-ов

Каждый протокол подключается к ядру через реализацию следующих trait-ов:

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // тип адреса, специфичный для протокола
    type Value: ValueLike;      // тип значения, специфичный для протокола
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

**Ассоциированные типы** обеспечивают типобезопасность адресов и значений: Modbus использует `u16`, OPC UA — `String`, BACnet — `f32`. **Разделение на trait-ы** позволяет реализовывать протоколы по мере необходимости — датчику только для чтения не нужен `ProtocolWrite`.

## 2. Kernel — микроядро

Ядро не реализует ни одного протокола, а предоставляет лишь общую инфраструктуру:

| Компонент | Обязанность | Ключевые API |
|------|------|----------|
| `ProtocolRegistry` | Регистрация/поиск/классификация протоколов | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Жизненный цикл подключений к устройствам | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | TOML-конфигурация + переопределение в рантайме | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Публикация/подписка на события | `subscribe()`, `publish()` |
| `CircuitBreaker` | Предохранитель Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Фиксированный/экспоненциальный/линейный/без повтора | `run()`, `start()` |
| `GatewayEngine` | Правила пересылки данных между протоколами | `add_rule()`, `start()/stop()` |

Управление подключениями поддерживает три стратегии:
```rust
pub enum ConnectionStrategy {
    Simple,      // новое подключение каждый раз, освобождается после использования
    Persistent,  // переиспользование одного подключения, авто-переподключение при обрыве
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — абстракция транспорта

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Реализация | Зависимости |
|--------|------|------|
| `bridge-socket` | TCP-подключение + приём/передача UDP | std |
| `bridge-serial` | Настоящий последовательный порт, опциональный feature `serial` | `serialport` |
| `bridge-can` | Linux SocketCAN (CAN_RAW) | `libc` |
| `bridge-ffi` | Динамическая загрузка вендорских C SDK | `libloading` |
| `bridge-vendor` | Пресеты Siemens/AB/Schneider | — |

## 4. Error — структурированная цепочка ошибок

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 недопустимый адрес"
    CircuitOpen { device, retry_after },  // предохранитель разомкнут
    Context { context, source },  // добавление контекста, образует цепочку ошибок
    // ...
}
```

Пример цепочки ошибок:
```
Не удалось прочитать регистры хранения 40001 устройства PLC-001
  → Тайм-аут: device=plc-001, timeout=3000ms
    → Ошибка подключения: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — опциональная обёртка tokio

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// асинхронный поток событий
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
