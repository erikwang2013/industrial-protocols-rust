# Relatório de Testes Unitários — industrial-protocols-rust

- **Data**: 2026-08-27
- **Versão**: 1.1.2 (workspace)
- **Resultado**: ✅ Tudo aprovado — 926 tests passed, 0 failed, 3 ignored (gated por hardware)

## Resumo

| Item | Resultado |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ Aprovado |
| Crates cobertos | 46 / 46 |
| Funções de teste adicionadas desta vez | aproximadamente 727 |

## Contagem de testes por crate

| crate | Testes | crate | Testes |
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

- **Construção de quadros**: sequências exatas de bytes, comprimento, endianness, somas de verificação (CRC-16/LRC/XOR validadas de forma cruzada e independente)
- **Parsers**: caminho normal + bordas como entrada truncada/curta, offset fora dos limites (todos retornam erro, sem panic)
- **Metadados**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range por meio de `Connection` mock (respostas predefinidas + modos de falha `io::Error` + registro de quadros enviados)
- **ProtocolHealth**: caminhos healthy / degraded / unhealthy
- **ProtocolDiagnostics**: integridade dos campos
- **Kernel**: transição half-open do disjuntor, limite máximo de retries, valores padrão de configuração, filtragem de eventos, pool de conexões, contagem de erros do gateway engine
- **Bridge**: loopback real do socket, rejeição de escrita após fechamento, loopback UDP; caminhos de erro de serial/can/ffi

## Defeitos corrigidos (descobertos por estes testes)

| Local | Defeito | Correção |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` calculava o bit de paridade P1 com negação bit a bit `!x`, sempre definindo 0x80, invalidando todos os PIDs LIN | Alterado para negação lógica `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` causava panic por fatiamento fora dos limites em quadros malformados com len<2 | Alterado para `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` causava panic em entradas truncadas (comprimento/tópico/packet ID) | Adicionadas 3 verificações de borda |
| protocols/opcua/src/lib.rs | O campo MessageSize de `build_read_request` subcontava 4 bytes | Alterado para `8 + 4 + body.len()` |
| kernel/src/config.rs | Sem a tabela `[defaults]`, a desserialização resultava em tudo zero em vez dos valores padrão documentados | Adicionada função serde `default_defaults()` |
| bridge/socket/src/lib.rs | `is_open()` ainda retornava true após `close()` | Adicionada a flag `closed` |

## Reprodução

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
