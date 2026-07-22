//! BACnet/IP protocol implementation.
//!
//! Implements BVLC (BACnet Virtual Link Control) over UDP,
//! Who-Is/I-Am device discovery, and ReadProperty service.

use industrial_protocols_kernel::bridge::Transport;
use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::{ProtocolError, Result};
use industrial_protocols_kernel::sdk::*;

pub struct BacnetProtocol {
    pub device_instance: u32,
    vendor_id: u16,
}

impl BacnetProtocol {
    pub fn new(device_instance: u32) -> Self {
        BacnetProtocol { device_instance, vendor_id: 0 }
    }

    pub fn with_vendor(mut self, vendor_id: u16) -> Self { self.vendor_id = vendor_id; self }
}

// BVLC header: [Type 1B] [Function 1B] [Length 2B] [Data nB]
const BVLC_TYPE: u8 = 0x81; // BACnet/IP

fn build_bvlc_header(payload_len: u16) -> Vec<u8> {
    let mut hdr = Vec::with_capacity(4);
    hdr.push(BVLC_TYPE);
    hdr.push(0x0A); // Original-Unicast-NPDU
    hdr.extend_from_slice(&(4 + payload_len).to_be_bytes()); // total length including header
    hdr
}

/// Build a Who-Is request (unconfirmed service).
pub fn build_who_is(device_instance_low: u32, device_instance_high: u32) -> Vec<u8> {
    // NPDU: [Version 1B] [Control 1B]
    // APDU: [Type 1B=unconfirmed] [Service 1B=Who-Is] [Params...]
    let apdu = vec![
        0x10, // Unconfirmed-Request, no segmentation
        0x08, // Who-Is service choice (unconfirmed)
        0x1C, // tag: context-specific, 4, value
        0x04, // length 4
        (device_instance_low >> 24) as u8,
        (device_instance_low >> 16) as u8,
        (device_instance_low >> 8) as u8,
        device_instance_low as u8,
        0x1C, 0x04,
        (device_instance_high >> 24) as u8,
        (device_instance_high >> 16) as u8,
        (device_instance_high >> 8) as u8,
        device_instance_high as u8,
    ];
    let npdu = [0x01, 0x04]; // Version 1, no network layer
    let payload: Vec<u8> = npdu.iter().chain(apdu.iter()).copied().collect();
    let mut frame = build_bvlc_header(payload.len() as u16);
    frame.extend(&payload);
    frame
}

/// Build an I-Am response (unconfirmed service).
pub fn build_i_am(device_instance: u32) -> Vec<u8> {
    let apdu = vec![
        0x10, // Unconfirmed-Request
        0x00, // I-Am service
        0xC4, 0x02, // tag: application, unsigned
        0x00, 0x16, // device object type (8)
        0x22, 0x04, // tag: application, unsigned 32
        (device_instance >> 24) as u8,
        (device_instance >> 16) as u8,
        (device_instance >> 8) as u8,
        device_instance as u8,
    ];
    let npdu = [0x01, 0x04];
    let payload: Vec<u8> = npdu.iter().chain(apdu.iter()).copied().collect();
    let mut frame = build_bvlc_header(payload.len() as u16);
    frame.extend(&payload);
    frame
}

/// Build a ReadProperty request (confirmed service).
pub fn build_read_property(
    invoke_id: u8,
    device_instance: u32,
    object_type: u16,
    object_instance: u32,
    property_id: u16,
) -> Vec<u8> {
    let apdu = vec![
        0x00, // Confirmed-Request, no segmentation
        0x04, // max APDU length accepted (206)
        invoke_id, // invoke ID
        0x0C, // ReadProperty service
        0x0C, // context tag: object specifier
        0x00, 0x00, 0x16, // object type (DEVICE=8)
        0x22, 0x04, // app tag, unsigned 32
        (device_instance >> 24) as u8,
        (device_instance >> 16) as u8,
        (device_instance >> 8) as u8,
        device_instance as u8,
        0x19, (property_id & 0xFF) as u8, // context tag, property ID
    ];
    let npdu = [0x01, 0x04];
    let payload: Vec<u8> = npdu.iter().chain(apdu.iter()).copied().collect();
    let mut frame = build_bvlc_header(payload.len() as u16);
    frame.extend(&payload);
    frame
}

/// Parse a ReadProperty response, extracting the value.
pub fn parse_read_property_response(data: &[u8]) -> std::result::Result<f32, String> {
    // Skip BVLC (4) + NPDU (2) + APDU header (4) + service ack (1) + object id (7)
    // Look for opening tag 0x3E (property value), then application tag 0x44 (real)
    let payload = if data.len() > 6 { &data[6..] } else { return Err("response too short".into()) };
    if let Some(pos) = payload.windows(2).position(|w| w == [0x44, 0x04]) {
        let start = pos + 2;
        if start + 4 <= payload.len() {
            let bytes: [u8; 4] = payload[start..start+4].try_into().unwrap();
            return Ok(f32::from_be_bytes(bytes));
        }
    }
    // Try parsing as unsigned integer if real not found
    if let Some(pos) = payload.windows(2).position(|w| w == [0x22, 0x04]) {
        let start = pos + 2;
        if start + 4 <= payload.len() {
            let bytes: [u8; 4] = payload[start..start+4].try_into().unwrap();
            return Ok(u32::from_be_bytes(bytes) as f32);
        }
    }
    Err("could not parse property value".into())
}

/// Build a generic Who-Is without device instance range limits.
pub fn build_who_is_any() -> Vec<u8> {
    build_who_is(0, 4194303) // full range
}

impl Protocol for BacnetProtocol {
    fn name(&self) -> &'static str { "bacnet" }
    fn version(&self) -> &'static str { "1.0.0" }
    fn category(&self) -> ProtocolCategory { ProtocolCategory::IndustrialEthernet }
    fn supported_variants(&self) -> &'static [&'static str] { &["ip", "ethernet"] }
}

impl ProtocolRead for BacnetProtocol {
    type Address = String;
    type Value = f32;

    fn read(&self, conn: &mut dyn Connection, addr: &Self::Address) -> Result<Self::Value> {
        // Parse address as "object_type:instance:property_id" e.g. "0:0:85" (DEVICE:0:present-value)
        let parts: Vec<&str> = addr.split(':').collect();
        let obj_type: u16 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let obj_inst: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(self.device_instance);
        let prop_id: u16 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(85); // present-value

        let frame = build_read_property(1, obj_inst, obj_type, obj_inst, prop_id);
        let resp = conn.send(&frame).map_err(ProtocolError::from)?;
        parse_read_property_response(&resp)
            .map_err(|e| ProtocolError::Parse { protocol: "bacnet".into(), reason: e })
    }

    fn read_batch(&self, conn: &mut dyn Connection, addrs: &[Self::Address]) -> Result<Vec<Self::Value>> {
        addrs.iter().map(|a| self.read(conn, a)).collect()
    }

    fn read_range(&self, conn: &mut dyn Connection, start: &Self::Address, count: u16) -> Result<Vec<Self::Value>> {
        (0..count).map(|_| self.read(conn, start)).collect()
    }
}

impl ProtocolWrite for BacnetProtocol {
    type Address = String;
    type Value = f32;

    fn write(&self, _conn: &mut dyn Connection, _addr: &Self::Address, _value: &Self::Value) -> Result<()> {
        Err(ProtocolError::Unsupported { protocol: "bacnet".into(), operation: "write — use WriteProperty service".into() })
    }

    fn write_batch(&self, conn: &mut dyn Connection, items: &[(Self::Address, Self::Value)]) -> Result<()> {
        for (a, v) in items { self.write(conn, a, v)?; }
        Ok(())
    }
}

impl ProtocolDiscover for BacnetProtocol {
    fn discover(&self, transport: &mut dyn Transport) -> Result<Vec<DiscoveredDevice>> {
        let frame = build_who_is_any();
        match transport.send(&frame) {
            Ok(resp) => {
                let device_id = format!("bacnet:{}", self.device_instance);
                Ok(vec![DiscoveredDevice {
                    device_id,
                    protocol: "bacnet".into(),
                    address: format!("instance={}", self.device_instance),
                    metadata: std::collections::HashMap::from([
                        ("vendor_id".into(), self.vendor_id.to_string()),
                    ]),
                }])
            }
            Err(e) => Err(ProtocolError::Connection(format!("BACnet discovery failed: {e}"))),
        }
    }
}

impl ProtocolHealth for BacnetProtocol {
    fn health(&self, conn: &mut dyn Connection) -> Result<HealthStatus> {
        let start = std::time::Instant::now();
        // Try reading the device object name (property 77)
        match self.read(conn, &format!("8:{}:77", self.device_instance)) {
            Ok(_) => Ok(HealthStatus {
                state: HealthState::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: "bacnet: ReadProperty DEVICE.object-name ok".into(),
            }),
            Err(e) => Ok(HealthStatus {
                state: HealthState::Unhealthy,
                latency_ms: start.elapsed().as_millis() as u64,
                last_seen: std::time::Instant::now(),
                details: format!("bacnet health check failed: {e}"),
            }),
        }
    }
}

impl ProtocolDiagnostics for BacnetProtocol {
    fn diagnostics(&self, _conn: &mut dyn Connection) -> Result<DiagnosticsReport> {
        Ok(DiagnosticsReport {
            device_id: format!("bacnet:{}", self.device_instance),
            uptime_secs: None, error_count: 0, last_error: None,
            custom_fields: std::collections::HashMap::from([
                ("device_instance".into(), self.device_instance.to_string()),
                ("vendor_id".into(), self.vendor_id.to_string()),
            ]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_who_is_frame() {
        let frame = build_who_is_any();
        assert_eq!(frame[0], 0x81); // BVLC type
        assert_eq!(frame[1], 0x0A); // function
        assert!(frame.len() > 4);
    }

    #[test]
    fn test_read_property_frame() {
        let frame = build_read_property(1, 1001, 0, 1001, 85);
        assert_eq!(frame[0], 0x81);
        assert!(frame.len() > 10);
    }

    #[test]
    fn test_bacnet_metadata() {
        let p = BacnetProtocol::new(1001).with_vendor(42);
        assert_eq!(p.name(), "bacnet");
        assert_eq!(p.device_instance, 1001);
        assert_eq!(p.vendor_id, 42);
    }
}
