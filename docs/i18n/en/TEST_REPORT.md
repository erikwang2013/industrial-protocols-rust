# Unit Test Report — industrial-protocols-rust

- **Date**: 2026-08-27
- **Version**: 1.1.2 (workspace)
- **Result**: ✅ All passed — 926 tests passed, 0 failed, 3 ignored (hardware-gated)

## Summary

| Item | Result |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Passed |
| Crates covered | 46 / 46 |
| New test functions added | ~727 |

## Test Count per Crate

| crate | tests | crate | tests |
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

## Coverage

- **Frame building**: exact byte sequences, lengths, endianness, checksums (CRC-16/LRC/XOR cross-validated independently)
- **Parsers**: happy path + edge cases such as truncated/short input and out-of-range offsets (all return errors, no panics)
- **Metadata**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range via mock `Connection` (preset responses + `io::Error` fault modes + sent-frame recording)
- **ProtocolHealth**: healthy / degraded / unhealthy paths
- **ProtocolDiagnostics**: field completeness
- **Kernel**: circuit breaker half-open transitions, retry caps, config defaults, event filtering, connection pool, gateway engine error counts
- **Bridges**: socket real loopback echo, write-after-close rejection, UDP loopback; serial/can/ffi error paths

## Defects Fixed (discovered by this test run)

| Location | Defect | Fix |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` computed the P1 parity bit with bitwise NOT `!x`, always setting 0x80, making all LIN PIDs invalid | Changed to logical NOT `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` panicked with an out-of-bounds slice on malformed frames with len<2 | Changed to `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` panicked on truncated input (length/topic/package ID) | Added 3 boundary checks |
| protocols/opcua/src/lib.rs | `build_read_request` under-counted the MessageSize field by 4 bytes | Changed to `8 + 4 + body.len()` |
| kernel/src/config.rs | Missing `[defaults]` table deserialized to all zeros instead of documented defaults | Added `default_defaults()` serde function |
| bridge/socket/src/lib.rs | `is_open()` still returned true after `close()` | Added `closed` flag |

## Reproduction

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
