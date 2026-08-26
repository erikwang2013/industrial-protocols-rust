# Informe de pruebas unitarias — industrial-protocols-rust

- **Fecha**: 2026-08-27
- **Versión**: 1.1.2 (workspace)
- **Resultado**: ✅ Todo correcto — 926 tests passed, 0 failed, 3 ignored (restringidos por hardware)

## Resumen

| Elemento | Resultado |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Correcto |
| Crates cubiertos | 46 / 46 |
| Funciones de prueba añadidas en esta ocasión | Aprox. 727 |

## Número de pruebas por crate

| crate | Pruebas | crate | Pruebas |
|---|---|---|---|
| kernel | 123 | protocols/kline | 24 |
| async-layer | 9 | protocols/lightbus | 26 |
| bridge/can | 5 | protocols/lin | 24 |
| bridge/ffi | 5 | protocols/lonworks | 22 |
| bridge/serial | 9 | protocols/modbus | 36 |
| bridge/socket | 9 | protocols/modbusplus | 22 |
| bridge/vendor | 6 | protocols/most | 21 |
| protocols/asinterface | 20 | protocols/mqtt | 23 |
| protocols/bacnet | 20 | protocols/opcua | 13 |
| protocols/canopen | 25 | protocols/pci | 22 |
| protocols/cclink | 20 | protocols/powerlink | 20 |
| protocols/cclinkie | 18 | protocols/profibus | 22 |
| protocols/controlnet | 18 | protocols/profinet | 16 |
| protocols/cpci | 19 | protocols/saej1850 | 15 |
| protocols/dali | 24 | protocols/sercos | 17 |
| protocols/devicenet | 21 | protocols/vme | 22 |
| protocols/dnp3 | 15 | protocols/wirelesshart | 18 |
| protocols/ethercat | 12 | protocols/worldfip | 19 |
| protocols/ethernet-ip | 16 | protocols/hartip | 18 |
| protocols/flexray | 20 | protocols/iec61850 | 18 |
| protocols/foundationfieldbus | 12 | protocols/interbus | 22 |
| protocols/hart | 14 | protocols/iolink | 24 |
| protocols/isa100 | 23 | | |

## Cobertura

- **Construcción de tramas**: secuencias de bytes exactas, longitud, endianness, sumas de comprobación (CRC-16/LRC/XOR verificadas de forma cruzada e independiente)
- **Analizadores**: ruta normal + casos límite como truncados/entradas cortas/offsets fuera de rango (todos devuelven error, sin panics)
- **Metadatos**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range mediante un `Connection` simulado (respuestas predefinidas + modos de fallo `io::Error` + registro de tramas enviadas)
- **ProtocolHealth**: rutas healthy / degraded / unhealthy
- **ProtocolDiagnostics**: integridad de los campos
- **Kernel**: transición half-open del disyuntor, límite de reintentos, valores por defecto de configuración, filtrado de eventos, pool de conexiones, contador de errores del motor de pasarela
- **Bridge**: loopback real de socket, rechazo de escrituras tras el cierre, loopback UDP; rutas de error de serial/can/ffi

## Defectos corregidos (detectados en estas pruebas)

| Ubicación | Defecto | Corrección |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` usaba la negación de bits `!x` para calcular el bit de paridad P1, siempre resultaba 0x80, todos los PID de LIN quedaban inválidos | Cambiado a negación lógica `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` hacía un slice fuera de límites con pánico en tramas malformadas con len<2 | Cambiado a `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` entraba en pánico con entrada truncada (longitud/tema/ID de paquete) | Añadidas 3 comprobaciones de límites |
| protocols/opcua/src/lib.rs | `build_read_request` contaba 4 bytes de menos en el campo MessageSize | Cambiado a `8 + 4 + body.len()` |
| kernel/src/config.rs | Sin la tabla `[defaults]`, la deserialización daba ceros en lugar de los valores por defecto documentados | Añadida la función serde `default_defaults()` |
| bridge/socket/src/lib.rs | `is_open()` seguía devolviendo true después de `close()` | Añadida la marca `closed` |

## Reproducción

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
