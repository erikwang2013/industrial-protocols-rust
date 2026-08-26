# 单元测试报告 — industrial-protocols-rust

- **日期**: 2026-08-27
- **版本**: 1.1.2（workspace）
- **结论**: ✅ 全部通过 — 926 tests passed, 0 failed, 3 ignored（硬件门控）

## 汇总

| 项 | 结果 |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ 通过 |
| 覆盖 crate 数 | 46 / 46 |
| 本次新增测试函数 | 约 727 |

## 各 crate 测试数

| crate | 测试数 | crate | 测试数 |
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

## 覆盖范围

- **帧构建**: 精确字节序列、长度、大小端、校验和（CRC-16/LRC/XOR 独立交叉验证）
- **解析器**: 正常路径 + 截断/短输入/越界 offset 等边界（全部返回错误，无 panic）
- **元数据**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range 经 mock `Connection`（预设响应 + `io::Error` 故障模式 + 发送帧记录）
- **ProtocolHealth**: healthy / degraded / unhealthy 路径
- **ProtocolDiagnostics**: 字段完整性
- **内核**: 熔断器半开转换、重试上限、配置默认值、事件过滤、连接池、网关引擎错误计数
- **桥接**: socket 真实 loopback 回环、关闭后拒绝写入、UDP 回环；serial/can/ffi 错误路径

## 修复的缺陷（本次测试发现）

| 位置 | 缺陷 | 修复 |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` 用位反 `!x` 计算 P1 校验位，恒置 0x80，LIN PID 全部无效 | 改逻辑非 `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` 对 len<2 的畸形帧切片越界 panic | 改 `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` 对截断输入（长度/主题/包 ID）panic | 补 3 处边界检查 |
| protocols/opcua/src/lib.rs | `build_read_request` MessageSize 字段少计 4 字节 | 改为 `8 + 4 + body.len()` |
| kernel/src/config.rs | 缺 `[defaults]` 表时反序列化为全零而非文档默认值 | 加 `default_defaults()` serde 函数 |
| bridge/socket/src/lib.rs | `close()` 后 `is_open()` 仍返回 true | 加 `closed` 标志 |

## 复现

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
