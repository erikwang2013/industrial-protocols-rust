# यूनिट टेस्ट रिपोर्ट — industrial-protocols-rust

- **दिनांक**: 2026-08-27
- **संस्करण**: 1.1.2 (workspace)
- **निष्कर्ष**: ✅ सभी पास — 926 tests passed, 0 failed, 3 ignored (हार्डवेयर-गेटेड)

## सारांश

| मद | परिणाम |
|---|---|
| `cargo test --workspace` | ✅ 926 passed / 0 failed / 3 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all --check` | ✅ पास |
| कवर किए गए crates | 46 / 46 |
| इस बार जोड़े गए परीक्षण फ़ंक्शन | लगभग 727 |

## प्रति-crate परीक्षण संख्या

| crate | परीक्षण संख्या | crate | परीक्षण संख्या |
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

## कवरेज

- **फ्रेम निर्माण**: सटीक बाइट अनुक्रम, लंबाई, एंडियननेस, चेकसम (CRC-16/LRC/XOR स्वतंत्र क्रॉस-सत्यापन)
- **पार्सर**: सामान्य पथ + ट्रंकेटेड/छोटे इनपुट/आउट-ऑफ-बाउंड offset जैसे एज केस (सभी त्रुटि लौटाते हैं, कोई panic नहीं)
- **मेटाडेटा**: name()/version()/category()/supported_variants()
- **ProtocolRead/Write**: read/read_batch/read_range मॉक `Connection` के माध्यम से (पूर्व-निर्धारित प्रतिक्रियाएं + `io::Error` विफलता मोड + भेजे गए फ्रेम की रिकॉर्डिंग)
- **ProtocolHealth**: healthy / degraded / unhealthy पथ
- **ProtocolDiagnostics**: फ़ील्ड पूर्णता
- **कर्नेल**: सर्किट ब्रेकर हाफ-ओपन संक्रमण, रीट्राई सीमा, कॉन्फ़िगरेशन डिफ़ॉल्ट, इवेंट फ़िल्टरिंग, कनेक्शन पूल, गेटवे इंजन त्रुटि गणना
- **ब्रिज**: socket वास्तविक loopback राउंड-ट्रिप, बंद करने के बाद लिखने से इनकार, UDP loopback; serial/can/ffi त्रुटि पथ

## इस परीक्षण में पाए गए दोष (निर्धारित)

| स्थान | दोष | समाधान |
|---|---|---|
| protocols/lin/src/lib.rs | `build_header` P1 पैरिटी बिट के लिए बिट-नॉट `!x` उपयोग करता है, हमेशा 0x80 सेट करता है, LIN PID सभी अमान्य | लॉजिकल नॉट `1 - x` में बदला |
| protocols/kline/src/lib.rs | `parse_kwp2000_response` len<2 वाले मैलफॉर्म्ड फ्रेम पर स्लाइस आउट-ऑफ-बाउंड panic | `data.get(..).unwrap_or_default()` में बदला |
| protocols/mqtt/src/lib.rs | `parse_publish` ट्रंकेटेड इनपुट (लंबाई/विषय/पैकेट ID) पर panic | 3 स्थानों पर बाउंड्री जांच जोड़ी |
| protocols/opcua/src/lib.rs | `build_read_request` में MessageSize फ़ील्ड 4 बाइट कम गिना गया | `8 + 4 + body.len()` में बदला |
| kernel/src/config.rs | `[defaults]` टेबल की अनुपस्थिति में दस्तावेज़ डिफ़ॉल्ट के बजाय सभी-शून्य डिसेरिएलाइज़ेशन | `default_defaults()` serde फ़ंक्शन जोड़ा |
| bridge/socket/src/lib.rs | `close()` के बाद `is_open()` अभी भी true लौटाता है | `closed` फ़्लैग जोड़ा |

## पुनरुत्पादन

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```
