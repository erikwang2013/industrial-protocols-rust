# Unit-Testbericht — industrial-protocols-rust

- **Datum**: 2026-08-27
- **Version**: 1.1.2 (workspace)
- **Ergebnis**: ✅ Alle bestanden — 926 tests passed, 0 failed, 3 ignored (hardware-gated)

## Zusammenfassung

| Posten | Ergebnis |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Bestanden |
| Abgedeckte Crates | 46 / 46 |
| Neue Testfunktionen in diesem Durchgang | ca. 727 |

## Testanzahl je Crate

| crate | Testanzahl | crate | Testanzahl |
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

## Abdeckung

- **Frame-Aufbau**: exakte Byte-Sequenzen, Länge, Endianness, Prüfsummen (CRC-16/LRC/XOR unabhängig kreuzvalidiert)
- **Parser**: Normalpfad + Randfälle wie abgeschnittene/kurze Eingaben und Offsets außerhalb des Bereichs (alle geben Fehler zurück, kein Panic)
- **Metadaten**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range über mock `Connection` (vordefinierte Antworten + `io::Error`-Fehlermodi + Aufzeichnung gesendeter Frames)
- **ProtocolHealth**: Pfade healthy / degraded / unhealthy
- **ProtocolDiagnostics**: Feldvollständigkeit
- **Kernel**: Half-Open-Übergang des Unterbrechers, Retry-Obergrenzen, Konfigurationsstandards, Ereignisfilterung, Verbindungspool, Fehlerzähler der Gateway-Engine
- **Brücken**: socket mit echtem Loopback-Roundtrip, Schreibverweigerung nach close, UDP-Loopback; Fehlerpfade für serial/can/ffi

## Behobene Fehler (in diesem Durchgang gefunden)

| Ort | Fehler | Fix |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` berechnete das P1-Paritätsbit mit bitweisem Komplement `!x`, setzte es stets auf 0x80, alle LIN-PIDs ungültig | geändert auf logisches Komplement `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` panic durch Slice-Out-of-Bounds bei defekten Frames mit len<2 | geändert auf `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` panic bei abgeschnittenen Eingaben (Länge/Thema/Paket-ID) | 3 Grenzprüfungen ergänzt |
| protocols/opcua/src/lib.rs | `build_read_request` unterschätzte das MessageSize-Feld um 4 Byte | geändert auf `8 + 4 + body.len()` |
| kernel/src/config.rs | Fehlende `[defaults]`-Tabelle führte zu Alles-Null statt der dokumentierten Standardwerte | serde-Funktion `default_defaults()` ergänzt |
| bridge/socket/src/lib.rs | `is_open()` lieferte nach `close()` weiterhin true | `closed`-Flag ergänzt |

## Reproduktion

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
