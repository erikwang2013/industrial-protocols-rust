# Отчёт о модульных тестах — industrial-protocols-rust

- **Дата**: 2026-08-27
- **Версия**: 1.1.2 (workspace)
- **Результат**: ✅ Все пройдено — 926 tests passed, 0 failed, 3 ignored (аппаратно-гейтированные)

## Сводка

| Пункт | Результат |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Пройдено |
| Число покрытых crate-ов | 46 / 46 |
| Новых тестовых функций добавлено | около 727 |

## Число тестов по crate-ам

| crate | тестов | crate | тестов |
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

## Покрытие

- **Построение кадров**: точные байтовые последовательности, длины, порядок байтов (big/little endian), контрольные суммы (CRC-16/LRC/XOR — независимая перекрёстная проверка)
- **Парсеры**: штатный путь + граничные случаи (усечённые/короткие входы, offset за границами и т. п. — все возвращают ошибку, без panic)
- **Метаданные**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range через mock `Connection` (предустановленные ответы + режимы отказа `io::Error` + запись отправленных кадров)
- **ProtocolHealth**: пути healthy / degraded / unhealthy
- **ProtocolDiagnostics**: целостность полей
- **Ядро**: переход предохранителя в half-open, лимиты повторов, значения конфигурации по умолчанию, фильтрация событий, пул подключений, счётчик ошибок gateway-движка
- **Мост**: реальная петля loopback через socket, отказ записи после close, UDP-петля; пути ошибок serial/can/ffi

## Исправленные дефекты (обнаружены этим набором тестов)

| Место | Дефект | Исправление |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` вычислял бит чётности P1 через побитовое отрицание `!x`, всегда выставляя 0x80, из-за чего все LIN PID были недействительны | заменено на логическое отрицание `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` паниковал при срезе за границы для повреждённых кадров с len<2 | заменено на `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` паниковал на усечённых входных данных (длина/тема/ID пакета) | добавлены 3 проверки границ |
| protocols/opcua/src/lib.rs | поле MessageSize в `build_read_request` недосчитывало 4 байта | заменено на `8 + 4 + body.len()` |
| kernel/src/config.rs | при отсутствии таблицы `[defaults]` данные десериализовались в нули вместо значений по умолчанию из документации | добавлена serde-функция `default_defaults()` |
| bridge/socket/src/lib.rs | после `close()` `is_open()` по-прежнему возвращал true | добавлен флаг `closed` |

## Воспроизведение

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
