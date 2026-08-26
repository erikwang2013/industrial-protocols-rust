# ইউনিট টেস্ট রিপোর্ট — industrial-protocols-rust

- **তারিখ**: 2026-08-27
- **ভার্সন**: 1.1.2 (workspace)
- **ফলাফল**: ✅ সব পাস — 926 tests passed, 0 failed, 3 ignored (হার্ডওয়্যার-গেটেড)

## সারাংশ

| আইটেম | ফলাফল |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ পাস |
| কভার করা crate সংখ্যা | 46 / 46 |
| এইবার নতুন টেস্ট ফাংশন | প্রায় 727 |

## প্রতি crate টেস্ট সংখ্যা

| crate | টেস্ট সংখ্যা | crate | টেস্ট সংখ্যা |
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

## কভারেজ

- **ফ্রেম নির্মাণ**: সঠিক বাইট সিকোয়েন্স, দৈর্ঘ্য, এন্ডিয়ানেস, চেকসাম (CRC-16/LRC/XOR স্বাধীন ক্রস-ভেরিফিকেশন)
- **পার্সার**: স্বাভাবিক পাথ + ট্রাঙ্কেটেড/ছোট ইনপুট/আউট-অফ-বাউন্ড offset ইত্যাদি এজ কেস (সব এরর রিটার্ন করে, কোনো panic নেই)
- **মেটাডেটা**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range mock `Connection` দিয়ে (প্রিসেট রেসপন্স + `io::Error` ফল্ট মোড + পাঠানো ফ্রেম রেকর্ডিং)
- **ProtocolHealth**: healthy / degraded / unhealthy পাথ
- **ProtocolDiagnostics**: ফিল্ড সম্পূর্ণতা
- **কার্নেল**: সার্কিট ব্রেকার হাফ-ওপেন ট্রানজিশন, রিট্রাই লিমিট, কনফিগ ডিফল্ট, ইভেন্ট ফিল্টারিং, কানেকশন পুল, গেটওয়ে ইঞ্জিন এরর কাউন্ট
- **ব্রিজ**: socket বাস্তব loopback, বন্ধের পর লেখা প্রত্যাখ্যান, UDP loopback; serial/can/ffi এরর পাথ

## এইবার টেস্টে পাওয়া বাগ (স্থির করা হয়েছে)

| অবস্থান | বাগ | সমাধান |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` বিট-নেগেশন `!x` দিয়ে P1 প্যারিটি বিট হিসাব করে, সবসময় 0x80 সেট করে, LIN PID সব অবৈধ | লজিক্যাল নেগেশন `1 - x`-এ পরিবর্তন |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` len<2 বিশিষ্ট খারাপ ফ্রেমে স্লাইস আউট-অফ-বাউন্ড panic | `data.get(..).unwrap_or_default()`-এ পরিবর্তন |
| protocols/mqtt/src/lib.rs | `parse_publish` ট্রাঙ্কেটেড ইনপুটে (দৈর্ঘ্য/টপিক/প্যাকেট ID) panic | ৩টি বাউন্ডারি চেক যোগ করা হয়েছে |
| protocols/opcua/src/lib.rs | `build_read_request` MessageSize ফিল্ডে ৪ বাইট কম গণনা | `8 + 4 + body.len()`-এ পরিবর্তন |
| kernel/src/config.rs | `[defaults]` টেবিল না থাকলে ডিফল্ট মানের বদলে শূন্য ডিসিরিয়ালাইজ হয় | `default_defaults()` serde ফাংশন যোগ করা হয়েছে |
| bridge/socket/src/lib.rs | `close()`-এর পর `is_open()` এখনও true ফেরত দেয় | `closed` ফ্ল্যাগ যোগ করা হয়েছে |

## রিপ্রোডাকশন

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
