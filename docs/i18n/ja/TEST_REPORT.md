# 単体テストレポート — industrial-protocols-rust

- **日付**: 2026-08-27
- **バージョン**: 1.1.2（workspace）
- **結果**: ✅ すべて合格 — 926 tests passed, 0 failed, 3 ignored（ハードウェアゲート付き）

## サマリー

| 項目 | 結果 |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ 合格 |
| カバーした crate 数 | 46 / 46 |
| 今回追加したテスト関数 | 約 727 |

## 各 crate のテスト数

| crate | テスト数 | crate | テスト数 |
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

## カバレッジ範囲

- **フレーム構築**: 正確なバイト列、長さ、エンディアン、チェックサム（CRC-16/LRC/XOR を独立に相互検証）
- **パーサー**: 正常パス + 切り詰め/短い入力/範囲外 offset などの境界ケース（すべてエラーを返し、panic なし）
- **メタデータ**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range を mock の `Connection` 経由で検証（事前定義レスポンス + `io::Error` 障害モード + 送信フレームの記録）
- **ProtocolHealth**: healthy / degraded / unhealthy パス
- **ProtocolDiagnostics**: フィールドの完全性
- **カーネル**: サーキットブレーカーの半開放遷移、リトライ上限、設定デフォルト値、イベントフィルタリング、コネクションプール、ゲートウェイエンジンのエラーカウント
- **ブリッジ**: socket の実 loopback エコー、close 後の書き込み拒否、UDP ループバック；serial/can/ffi のエラーパス

## 修正された欠陥（今回のテストで発見）

| 場所 | 欠陥 | 修正 |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` がビット反転 `!x` で P1 チェックビットを計算し、常に 0x80 が立ち、LIN PID がすべて無効 | 論理否定 `1 - x` に変更 |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` が len<2 の不正フレームでスライス範囲外 panic | `data.get(..).unwrap_or_default()` に変更 |
| protocols/mqtt/src/lib.rs | `parse_publish` が切り詰め入力（長さ/トピック/パケット ID）で panic | 3 箇所の境界チェックを追加 |
| protocols/opcua/src/lib.rs | `build_read_request` の MessageSize フィールドが 4 バイト不足 | `8 + 4 + body.len()` に変更 |
| kernel/src/config.rs | `[defaults]` テーブルがない場合、ドキュメント記載のデフォルト値ではなく全ゼロに逆シリアライズ | `default_defaults()` serde 関数を追加 |
| bridge/socket/src/lib.rs | `close()` 後も `is_open()` が true を返す | `closed` フラグを追加 |

## 再現手順

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
