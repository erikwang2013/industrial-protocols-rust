# تقرير اختبارات الوحدة — industrial-protocols-rust

- **التاريخ**: 2026-08-27
- **الإصدار**: 1.1.2 (workspace)
- **النتيجة**: ✅ الكل ناجح — 926 tests passed, 0 failed, 3 ignored (مقيدة بالعتاد)

## الملخص

| البند | النتيجة |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ ناجح |
| عدد crates المغطاة | 46 / 46 |
| دوال اختبار مضافة حديثًا | حوالي 727 |

## عدد اختبارات كل crate

| crate | عدد الاختبارات | crate | عدد الاختبارات |
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

## التغطية

- **بناء الإطارات**: تسلسلات بايت دقيقة، الأطوال، ترتيب البايتات، المجاميع الاختبارية (CRC-16/LRC/XOR تم التحقق منها بشكل متقاطع ومستقل)
- **المحللات**: المسار الطبيعي + حالات الحدود مثل المدخلات المقطوعة/القصيرة/offset خارج النطاق (جميعها تُرجع أخطاء، دون panic)
- **البيانات الوصفية**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range عبر `Connection` وهمي (استجابات محددة مسبقًا + أوضاع فشل `io::Error` + تسجيل الإطارات المرسلة)
- **ProtocolHealth**: مسارات healthy / degraded / unhealthy
- **ProtocolDiagnostics**: اكتمال الحقول
- **النواة**: انتقالات نصف-فتح قاطع الدائرة، حدود إعادة المحاولة، القيم الافتراضية للإعدادات، تصفية الأحداث، تجمع الاتصالات، عدّاد أخطاء محرك البوابة
- **الجسر**: حلقة رجوع loopback حقيقية عبر socket، رفض الكتابة بعد الإغلاق، حلقة رجوع UDP؛ مسارات الخطأ لـ serial/can/ffi

## العيوب المُصلَحة (اكتُشفت في هذه الجولة من الاختبارات)

| الموقع | العيب | الإصلاح |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` يحسب بت التحقق P1 بالنفي الثنائي `!x`، فيُثبّت 0x80 دائمًا، فتصبح جميع LIN PID غير صالحة | تغييره إلى النفي المنطقي `1 - x` |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` يتجاوز الحدود (slice out of bounds) مع إطارات مشوهة len<2 | تغييره إلى `data.get(..).unwrap_or_default()` |
| protocols/mqtt/src/lib.rs | `parse_publish` يحدث panic مع مدخلات مقطوعة (الطول/الموضوع/معرف الحزمة) | إضافة 3 فحوصات حدود |
| protocols/opcua/src/lib.rs | حقل MessageSize في `build_read_request` ينقصه 4 بايتات | تغييره إلى `8 + 4 + body.len()` |
| kernel/src/config.rs | عند غياب جدول `[defaults]` يتم إلغاء التسلسل إلى أصفار بدلًا من القيم الافتراضية الموثقة | إضافة دالة serde `default_defaults()` |
| bridge/socket/src/lib.rs | بعد `close()` لا يزال `is_open()` يعيد true | إضافة علامة `closed` |

## إعادة الإنتاج

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
