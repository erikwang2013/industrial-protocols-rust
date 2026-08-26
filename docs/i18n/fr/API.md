# Référence API — Industrial Protocols Rust

Ce document décrit les API publiques principales de la suite de protocoles industriels : traits du SDK Protocol, composants du noyau, couche de pont (bridge), types d'erreur et couche asynchrone.

Retour au [README](../README.md) (versions multilingues dans le [répertoire des langues](../README.md#multilingue--languages)).

## 1. SDK Protocol — 6 traits

Chaque protocole s'intègre au noyau en implémentant les traits suivants :

```rust
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn category(&self) -> ProtocolCategory;
    fn supported_variants(&self) -> &'static [&'static str];
}

pub trait ProtocolRead: Protocol {
    type Address: AddressLike;   // Type d'adresse spécifique au protocole
    type Value: ValueLike;      // Type de valeur spécifique au protocole
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

**Les types associés** rendent les types d'adresse et de valeur sûrs : Modbus utilise `u16`, OPC UA `String`, BACnet `f32`. **La division en traits** permet aux protocoles d'implémenter seulement ce dont ils ont besoin — un capteur en lecture seule n'a pas besoin de `ProtocolWrite`.

## 2. Kernel — micro-noyau

Le noyau n'implémente aucun protocole, il fournit uniquement l'infrastructure partagée :

| Composant | Responsabilité | API clés |
|------|------|----------|
| `ProtocolRegistry` | Enregistrement/recherche/classification des protocoles | `register()`, `get()`, `list_by_category()` |
| `ConnectionManager` | Cycle de vie des connexions aux équipements | `connect()`, `health()`, `shutdown()` |
| `ConfigRepository` | Configuration TOML + surcharge à l'exécution | `load()`, `get_device()`, `set_override()` |
| `EventBus` | Publication/abonnement aux événements | `subscribe()`, `publish()` |
| `CircuitBreaker` | Disjoncteur Closed→Open→HalfOpen | `call()`, `state()` |
| `RetryConfig` | Fixe/exponentielle/linéaire/aucune | `run()`, `start()` |
| `GatewayEngine` | Règles de transfert de données entre protocoles | `add_rule()`, `start()/stop()` |

La gestion des connexions prend en charge trois stratégies :
```rust
pub enum ConnectionStrategy {
    Simple,      // Nouvelle connexion à chaque fois, libérée après usage
    Persistent,  // Connexion unique réutilisée, reconnexion automatique en cas de coupure
    Pooled { min: usize, max: usize, timeout_ms: u64 },
}
```

## 3. Bridge Layer — abstraction de transport

```rust
pub trait Transport: Send {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>>;
    fn is_open(&self) -> bool;
    fn close(&mut self) -> io::Result<()>;
}
```

| Bridge | Implémentation | Dépendance |
|--------|------|------|
| `bridge-socket` | Connexion TCP + émission/réception UDP | std |
| `bridge-serial` | Port série réel, feature `serial` optionnelle | `serialport` |
| `bridge-can` | SocketCAN Linux (CAN_RAW) | `libc` |
| `bridge-ffi` | Chargement dynamique des SDK C des fournisseurs | `libloading` |
| `bridge-vendor` | Préréglages Siemens/AB/Schneider | — |

## 4. Error — chaîne d'erreurs structurée

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    Connection(String),           // "192.168.1.10:502: Connection refused"
    Timeout { device, timeout_ms }, // "plc-001: 3000ms"
    Parse { protocol, reason },   // "modbus: CRC mismatch"
    DeviceError { device, code, message }, // "plc-001: 0x02 adresse illégale"
    CircuitOpen { device, retry_after },  // Disjoncteur ouvert
    Context { context, source },  // Ajoute du contexte, formant une chaîne d'erreurs
    // ...
}
```

Exemple de chaîne d'erreurs :
```
Échec de la lecture du registre de maintien 40001 de PLC-001
  → Expiration du délai: device=plc-001, timeout=3000ms
    → Échec de connexion: 192.168.1.10:502: Connection refused
```

## 5. Async Layer — wrapper tokio optionnel

```rust
let kernel = Kernel::builder()
    .with_protocol(Box::new(ModbusProtocol::tcp()))
    .build();

let ak = AsyncKernel::new(kernel);
ak.boot().await?;

// Flux d'événements asynchrones
let mut stream = AsyncEventStream::subscribe(
    ak.kernel().blocking_lock(), EventTopic::Connection
);
while let Some(event) = stream.recv().await {
    println!("{:?}", event);
}
```
