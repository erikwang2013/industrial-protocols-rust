# Laporan Pengujian Unit — industrial-protocols-rust

- **Tanggal**: 2026-08-27
- **Versi**: 1.1.2 (workspace)
- **Kesimpulan**: ✅ Semua lulus — 926 tests passed, 0 failed, 3 ignored (dibatasi perangkat keras)

## Ringkasan

| Item | Hasil |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Lulus |
| Jumlah crate yang dicakup | 46 / 46 |
| Fungsi pengujian baru kali ini | Sekitar 727 |

## Jumlah Pengujian per Crate

| crate | Jumlah pengujian | crate | Jumlah pengujian |
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

## Cakupan

- **Pembuatan frame**: urutan byte presisi, panjang, endianness, checksum (verifikasi silang independen CRC-16/LRC/XOR)
- **Parser**: jalur normal + kasus tepi seperti input terpotong/pendek/offset di luar batas (semua mengembalikan error, tanpa panic)
- **Metadata**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range melalui mock `Connection` (respons preset + mode kegagalan `io::Error` + pencatatan frame yang dikirim)
- **ProtocolHealth**: jalur healthy / degraded / unhealthy
- **ProtocolDiagnostics**: kelengkapan field
- **Kernel**: transisi half-open pemutus sirkuit, batas retry, default konfigurasi, filter event, connection pool, penghitung error gateway engine
- **Bridge**: loopback socket nyata, penolakan tulis setelah ditutup, loopback UDP; jalur error serial/can/ffi

## Defect yang Diperbaiki (ditemukan oleh pengujian kali ini)

| Lokasi | Defect | Perbaikan |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` menghitung bit P1 dengan komplemen bit `!x`, selalu menghasilkan 0x80, seluruh PID LIN tidak valid | Diubah menjadi negasi logis `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` mengiris melebihi batas (panic) pada frame malformed dengan len<2 | Diubah menjadi `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` panic pada input terpotong (panjang/topik/paket ID) | Ditambah 3 pemeriksaan batas |
| protocols/opcua/src/lib.rs | Field MessageSize di `build_read_request` kurang menghitung 4 byte | Diubah menjadi `8 + 4 + body.len()` |
| kernel/src/config.rs | Tanpa tabel `[defaults]`, deserialisasi menghasilkan nol semua alih-alih nilai default dokumen | Ditambah fungsi serde `default_defaults()` |
| bridge/socket/src/lib.rs | Setelah `close()`, `is_open()` masih mengembalikan true | Ditambah flag `closed` |

## Reproduksi

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
