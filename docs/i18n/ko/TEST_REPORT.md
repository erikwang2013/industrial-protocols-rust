# 단위 테스트 보고서 — industrial-protocols-rust

- **날짜**: 2026-08-27
- **버전**: 1.1.2 (workspace)
- **결론**: ✅ 전체 통과 — 926 tests passed, 0 failed, 3 ignored (하드웨어 게이팅)

## 요약

| 항목 | 결과 |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ 통과 |
| 테스트 대상 crate 수 | 46 / 46 |
| 이번에 추가된 테스트 함수 | 약 727 |

## crate별 테스트 수

| crate | 테스트 수 | crate | 테스트 수 |
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

## 테스트 범위

- **프레임 구성**: 정확한 바이트 시퀀스, 길이, 엔디언, 체크섬 (CRC-16/LRC/XOR 독립 교차 검증)
- **파서**: 정상 경로 + 잘림/짧은 입력/범위를 벗어난 offset 등 경계 조건 (모두 오류 반환, panic 없음)
- **메타데이터**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: mock `Connection`을 통한 read/read_batch/read_range (사전 설정된 응답 + `io::Error` 장애 모드 + 전송 프레임 기록)
- **ProtocolHealth**: healthy / degraded / unhealthy 경로
- **ProtocolDiagnostics**: 필드 무결성
- **커널**: 서킷 브레이커 HalfOpen 전환, 재시도 상한, 설정 기본값, 이벤트 필터링, 연결 풀, 게이트웨이 엔진 오류 카운트
- **브리지**: socket 실제 loopback 회신, 닫힌 후 쓰기 거부, UDP 루프백; serial/can/ffi 오류 경로

## 수정된 결함 (이번 테스트에서 발견)

| 위치 | 결함 | 수정 |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header`가 비트 반전 `!x`로 P1 패리티 비트를 계산하여 항상 0x80으로 설정, LIN PID가 모두 무효 | 논리 부정 `1 - x`로 변경 |
| protocols/kline/src/lib.rs | `parse_kwp2000_response`가 len<2인 비정상 프레임에서 슬라이스 범위 초과로 panic | `data.get(..).unwrap_or_default()`로 변경 |
| protocols/mqtt/src/lib.rs | `parse_publish`가 잘린 입력(길이/토픽/패킷 ID)에서 panic | 경계 검사 3곳 추가 |
| protocols/opcua/src/lib.rs | `build_read_request`의 MessageSize 필드가 4바이트 부족 | `8 + 4 + body.len()`으로 변경 |
| kernel/src/config.rs | `[defaults]` 테이블이 없으면 문서 기본값 대신 전부 0으로 역직렬화 | `default_defaults()` serde 함수 추가 |
| bridge/socket/src/lib.rs | `close()` 이후에도 `is_open()`이 true 반환 | `closed` 플래그 추가 |

## 재현

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
