//! Modbus frame building and parsing for TCP, RTU, and ASCII variants.

// TCP frame format: [TID 2B] [PID 2B] [Len 2B] [UID 1B] [FC 1B] [Data nB]
// RTU frame format: [Addr 1B] [FC 1B] [Data nB] [CRC 2B]
// ASCII frame: :[Addr 2H][FC 2H][Data nH][LRC 2H]\r\n

pub const CRC16_TABLE: [u16; 256] = [
    0x0000, 0xC0C1, 0xC181, 0x0140, 0xC301, 0x03C0, 0x0280, 0xC241,
    0xC601, 0x06C0, 0x0780, 0xC741, 0x0500, 0xC5C1, 0xC481, 0x0440,
    0xCC01, 0x0CC0, 0x0D80, 0xCD41, 0x0F00, 0xCFC1, 0xCE81, 0x0E40,
    0x0A00, 0xCAC1, 0xCB81, 0x0B40, 0xC901, 0x09C0, 0x0880, 0xC841,
    0xD801, 0x18C0, 0x1980, 0xD941, 0x1B00, 0xDBC1, 0xDA81, 0x1A40,
    0x1E00, 0xDEC1, 0xDF81, 0x1F40, 0xDD01, 0x1DC0, 0x1C80, 0xDC41,
    0x1400, 0xD4C1, 0xD581, 0x1540, 0xD701, 0x17C0, 0x1680, 0xD641,
    0xD201, 0x12C0, 0x1380, 0xD341, 0x1100, 0xD1C1, 0xD081, 0x1040,
    0xF001, 0x30C0, 0x3180, 0xF141, 0x3300, 0xF3C1, 0xF281, 0x3240,
    0x3600, 0xF6C1, 0xF781, 0x3740, 0xF501, 0x35C0, 0x3480, 0xF441,
    0x3C00, 0xFCC1, 0xFD81, 0x3D40, 0xFF01, 0x3FC0, 0x3E80, 0xFE41,
    0xFA01, 0x3AC0, 0x3B80, 0xFB41, 0x3900, 0xF9C1, 0xF881, 0x3840,
    0x2800, 0xE8C1, 0xE981, 0x2940, 0xEB01, 0x2BC0, 0x2A80, 0xEA41,
    0xEE01, 0x2EC0, 0x2F80, 0xEF41, 0x2D00, 0xEDC1, 0xEC81, 0x2C40,
    0xE401, 0x24C0, 0x2580, 0xE541, 0x2700, 0xE7C1, 0xE681, 0x2640,
    0x2200, 0xE2C1, 0xE381, 0x2340, 0xE101, 0x21C0, 0x2080, 0xE041,
    0xA001, 0x60C0, 0x6180, 0xA141, 0x6300, 0xA3C1, 0xA281, 0x6240,
    0x6600, 0xA6C1, 0xA781, 0x6740, 0xA501, 0x65C0, 0x6480, 0xA441,
    0x6C00, 0xACC1, 0xAD81, 0x6D40, 0xAF01, 0x6FC0, 0x6E80, 0xAE41,
    0xAA01, 0x6AC0, 0x6B80, 0xAB41, 0x6900, 0xA9C1, 0xA881, 0x6840,
    0x7800, 0xB8C1, 0xB981, 0x7940, 0xBB01, 0x7BC0, 0x7A80, 0xBA41,
    0xBE01, 0x7EC0, 0x7F80, 0xBF41, 0x7D00, 0xBDC1, 0xBC81, 0x7C40,
    0xB401, 0x74C0, 0x7580, 0xB541, 0x7700, 0xB7C1, 0xB681, 0x7640,
    0x7200, 0xB2C1, 0xB381, 0x7340, 0xB101, 0x71C0, 0x7080, 0xB041,
    0x5000, 0x90C1, 0x9181, 0x5140, 0x9301, 0x53C0, 0x5280, 0x9241,
    0x9601, 0x56C0, 0x5780, 0x9741, 0x5500, 0x95C1, 0x9481, 0x5440,
    0x9C01, 0x5CC0, 0x5D80, 0x9D41, 0x5F00, 0x9FC1, 0x9E81, 0x5E40,
    0x5A00, 0x9AC1, 0x9B81, 0x5B40, 0x9901, 0x59C0, 0x5880, 0x9841,
    0x8801, 0x48C0, 0x4980, 0x8941, 0x4B00, 0x8BC1, 0x8A81, 0x4A40,
    0x4E00, 0x8EC1, 0x8F81, 0x4F40, 0x8D01, 0x4DC0, 0x4C80, 0x8C41,
    0x4400, 0x84C1, 0x8581, 0x4540, 0x8701, 0x47C0, 0x4680, 0x8641,
    0x8201, 0x42C0, 0x4380, 0x8341, 0x4100, 0x81C1, 0x8081, 0x4040,
];


/// Calculate CRC-16/XMODEM for Modbus RTU frames.
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// Calculate LRC (Longitudinal Redundancy Check) for Modbus ASCII.
pub fn lrc(data: &[u8]) -> u8 {
    let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    0u8.wrapping_sub(sum) // two's complement
}

// ── TCP frame builders (with MBAP header) ──

pub fn build_read_holding_registers_tcp(tid: u16, unit_id: u8, start: u16, qty: u16) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]); // protocol id
    f.extend_from_slice(&0x0006u16.to_be_bytes()); // remaining len
    f.push(unit_id);
    f.push(0x03); // FC: Read Holding Registers
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&qty.to_be_bytes());
    f
}

pub fn build_write_single_register_tcp(tid: u16, unit_id: u8, addr: u16, value: u16) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&0x0006u16.to_be_bytes());
    f.push(unit_id);
    f.push(0x06);
    f.extend_from_slice(&addr.to_be_bytes());
    f.extend_from_slice(&value.to_be_bytes());
    f
}

pub fn build_write_multiple_registers_tcp(tid: u16, unit_id: u8, start: u16, values: &[u16]) -> Vec<u8> {
    let bc = (values.len() * 2) as u8;
    let remaining = 5 + bc as u16;
    let mut f = Vec::with_capacity(7 + remaining as usize);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&remaining.to_be_bytes());
    f.push(unit_id);
    f.push(0x10);
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&(values.len() as u16).to_be_bytes());
    f.push(bc);
    for v in values { f.extend_from_slice(&v.to_be_bytes()); }
    f
}

pub fn build_read_coils_tcp(tid: u16, unit_id: u8, start: u16, qty: u16) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&0x0006u16.to_be_bytes());
    f.push(unit_id);
    f.push(0x01);
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&qty.to_be_bytes());
    f
}

pub fn build_read_discrete_inputs_tcp(tid: u16, unit_id: u8, start: u16, qty: u16) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&0x0006u16.to_be_bytes());
    f.push(unit_id);
    f.push(0x02);
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&qty.to_be_bytes());
    f
}

pub fn build_read_input_registers_tcp(tid: u16, unit_id: u8, start: u16, qty: u16) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&0x0006u16.to_be_bytes());
    f.push(unit_id);
    f.push(0x04);
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&qty.to_be_bytes());
    f
}

// ── RTU frame builders (address + FC + data + CRC16, no MBAP header) ──

pub fn build_read_holding_registers_rtu(addr: u8, start: u16, qty: u16) -> Vec<u8> {
    let mut f = vec![addr, 0x03];
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&qty.to_be_bytes());
    let crc = crc16(&f);
    f.extend_from_slice(&crc.to_le_bytes()); // CRC is little-endian
    f
}

pub fn build_write_single_register_rtu(addr: u8, reg: u16, value: u16) -> Vec<u8> {
    let mut f = vec![addr, 0x06];
    f.extend_from_slice(&reg.to_be_bytes());
    f.extend_from_slice(&value.to_be_bytes());
    let crc = crc16(&f);
    f.extend_from_slice(&crc.to_le_bytes());
    f
}

pub fn build_write_multiple_registers_rtu(addr: u8, start: u16, values: &[u16]) -> Vec<u8> {
    let bc = (values.len() * 2) as u8;
    let mut f = vec![addr, 0x10];
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&(values.len() as u16).to_be_bytes());
    f.push(bc);
    for v in values { f.extend_from_slice(&v.to_be_bytes()); }
    let crc = crc16(&f);
    f.extend_from_slice(&crc.to_le_bytes());
    f
}

// ── ASCII frame builders (colon-delimited hex with LRC) ──

fn to_hex_ascii(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("")
}

pub fn build_read_holding_registers_ascii(addr: u8, start: u16, qty: u16) -> Vec<u8> {
    let payload = [addr, 0x03, (start >> 8) as u8, start as u8, (qty >> 8) as u8, qty as u8];
    let lrc_byte = lrc(&payload);
    let frame = format!(":{}{:02X}\r\n", to_hex_ascii(&payload), lrc_byte);
    frame.into_bytes()
}

pub fn build_write_single_register_ascii(addr: u8, reg: u16, value: u16) -> Vec<u8> {
    let payload = [addr, 0x06, (reg >> 8) as u8, reg as u8, (value >> 8) as u8, value as u8];
    let lrc_byte = lrc(&payload);
    let frame = format!(":{}{:02X}\r\n", to_hex_ascii(&payload), lrc_byte);
    frame.into_bytes()
}

// ── Backward-compatible TCP builders ──

pub fn build_read_holding_registers(tid: u16, start: u16, qty: u16) -> Vec<u8> {
    build_read_holding_registers_tcp(tid, 1, start, qty)
}
pub fn build_write_single_register(tid: u16, addr: u16, value: u16) -> Vec<u8> {
    build_write_single_register_tcp(tid, 1, addr, value)
}
pub fn build_write_multiple_registers(tid: u16, start: u16, values: &[u16]) -> Vec<u8> {
    build_write_multiple_registers_tcp(tid, 1, start, values)
}
pub fn build_read_coils(tid: u16, start: u16, qty: u16) -> Vec<u8> {
    build_read_coils_tcp(tid, 1, start, qty)
}
pub fn build_read_discrete_inputs(tid: u16, start: u16, qty: u16) -> Vec<u8> {
    build_read_discrete_inputs_tcp(tid, 1, start, qty)
}
pub fn build_read_input_registers(tid: u16, start: u16, qty: u16) -> Vec<u8> {
    build_read_input_registers_tcp(tid, 1, start, qty)
}

// ── Response parsing ──

/// Parse a Modbus TCP read response (FC 01/02/03/04).
pub fn parse_read_response_tcp(data: &[u8]) -> Result<Vec<u16>, String> {
    if data.len() < 9 { return Err(format!("TCP response too short: {} bytes", data.len())); }
    let byte_count = data[8] as usize;
    let expected = 9 + byte_count;
    if data.len() < expected {
        return Err(format!("TCP response truncated: expected {} bytes, got {}", expected, data.len()));
    }
    let mut values = Vec::with_capacity(byte_count / 2);
    let mut i = 9;
    while i + 1 < data.len() && i < expected {
        values.push(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    Ok(values)
}

/// Parse a Modbus RTU read response (FC 01/02/03/04).
pub fn parse_read_response_rtu(data: &[u8]) -> Result<Vec<u16>, String> {
    if data.len() < 5 { return Err(format!("RTU response too short: {} bytes", data.len())); }
    // Verify CRC
    let crc_received = u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]]);
    let crc_calc = crc16(&data[..data.len() - 2]);
    if crc_received != crc_calc {
        return Err(format!("RTU CRC mismatch: received 0x{:04X}, calculated 0x{:04X}", crc_received, crc_calc));
    }
    let byte_count = data[2] as usize;
    let mut values = Vec::with_capacity(byte_count / 2);
    let mut i = 3;
    while i + 1 < data.len() - 2 && values.len() < byte_count / 2 {
        values.push(u16::from_be_bytes([data[i], data[i + 1]]));
        i += 2;
    }
    Ok(values)
}

/// Parse read response (auto-detect TCP vs RTU).
pub fn parse_read_response(data: &[u8]) -> Result<Vec<u16>, String> {
    if data.len() >= 7 && data[2] == 0x00 && data[3] == 0x00 {
        parse_read_response_tcp(data)
    } else {
        parse_read_response_rtu(data)
    }
}

/// Parse exception from TCP response.
pub fn parse_exception_tcp(data: &[u8]) -> Option<u8> {
    if data.len() >= 9 && (data[7] & 0x80) != 0 { Some(data[8]) } else { None }
}

/// Parse exception from RTU response.
pub fn parse_exception_rtu(data: &[u8]) -> Option<u8> {
    if data.len() >= 3 && (data[1] & 0x80) != 0 { Some(data[2]) } else { None }
}

/// Parse exception (auto-detect).
pub fn parse_exception(data: &[u8]) -> Option<u8> {
    parse_exception_tcp(data).or_else(|| parse_exception_rtu(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_known_vector() {
        let data = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
        assert_eq!(crc16(&data), 0x0A84);
    }

    #[test]
    fn test_crc16_empty() {
        assert_eq!(crc16(&[]), 0xFFFF);
    }

    #[test]
    fn test_lrc_calculation() {
        // Example: address 0x01, FC 0x03, data 0x00 0x0A 0x00 0x01
        let data = [0x01, 0x03, 0x00, 0x0A, 0x00, 0x01];
        let l = lrc(&data);
        assert_eq!(l, 0xF1); // (01+03+00+0A+00+01)=0F, 0x100-0x0F=0xF1
    }

    #[test]
    fn test_rtu_frame_has_crc() {
        let frame = build_read_holding_registers_rtu(1, 0, 10);
        assert_eq!(frame.len(), 8); // addr(1) + fc(1) + start(2) + qty(2) + crc(2)
        assert_eq!(frame[0], 1);
        assert_eq!(frame[1], 0x03);
        // Verify CRC
        let crc = crc16(&frame[..6]);
        assert_eq!(&frame[6..], &crc.to_le_bytes());
    }

    #[test]
    fn test_ascii_frame_format() {
        let frame = build_read_holding_registers_ascii(1, 0, 10);
        let s = String::from_utf8(frame).unwrap();
        assert!(s.starts_with(':'));
        assert!(s.ends_with("\r\n"));
    }

    #[test]
    fn test_rtu_response_parse() {
        // RTU response: addr=1, fc=3, count=4, values=[42,66], crc=...
        let mut data = vec![0x01, 0x03, 0x04, 0x00, 0x2A, 0x00, 0x42];
        let crc = crc16(&data);
        data.extend_from_slice(&crc.to_le_bytes());
        assert_eq!(parse_read_response_rtu(&data).unwrap(), vec![42, 66]);
    }

    #[test]
    fn test_rtu_tcp_auto_detect() {
        // RTU
        let mut rtu = vec![0x01, 0x03, 0x02, 0x00, 0x2A];
        let crc = crc16(&rtu);
        rtu.extend_from_slice(&crc.to_le_bytes());
        assert_eq!(parse_read_response(&rtu).unwrap(), vec![42]);

        // TCP
        let tcp = vec![0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x01, 0x03, 0x02, 0x00, 0x2A];
        assert_eq!(parse_read_response(&tcp).unwrap(), vec![42]);
    }
}

// ── FC 05: Write Single Coil ──

pub fn build_write_single_coil_tcp(tid: u16, unit_id: u8, addr: u16, value: bool) -> Vec<u8> {
    let mut f = Vec::with_capacity(12);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&0x0006u16.to_be_bytes());
    f.push(unit_id);
    f.push(0x05);
    f.extend_from_slice(&addr.to_be_bytes());
    f.extend_from_slice(&if value { 0xFF00u16 } else { 0x0000u16 }.to_be_bytes());
    f
}

pub fn build_write_single_coil_rtu(addr: u8, coil_addr: u16, value: bool) -> Vec<u8> {
    let mut f = vec![addr, 0x05];
    f.extend_from_slice(&coil_addr.to_be_bytes());
    f.extend_from_slice(&if value { 0xFF00u16 } else { 0x0000u16 }.to_be_bytes());
    let crc = crc16(&f);
    f.extend_from_slice(&crc.to_le_bytes());
    f
}

// ── FC 15: Write Multiple Coils ──

pub fn build_write_multiple_coils_tcp(tid: u16, unit_id: u8, start: u16, values: &[bool]) -> Vec<u8> {
    let byte_count = ((values.len() + 7) / 8) as u8;
    let remaining = 5 + byte_count as u16;
    let mut f = Vec::with_capacity(7 + remaining as usize);
    f.extend_from_slice(&tid.to_be_bytes());
    f.extend_from_slice(&[0x00, 0x00]);
    f.extend_from_slice(&remaining.to_be_bytes());
    f.push(unit_id);
    f.push(0x0F);
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&(values.len() as u16).to_be_bytes());
    f.push(byte_count);
    for chunk in values.chunks(8) {
        let mut byte = 0u8;
        for (i, &v) in chunk.iter().enumerate() {
            if v { byte |= 1 << i; }
        }
        f.push(byte);
    }
    f
}

pub fn build_write_multiple_coils_rtu(addr: u8, start: u16, values: &[bool]) -> Vec<u8> {
    let byte_count = ((values.len() + 7) / 8) as u8;
    let mut f = vec![addr, 0x0F];
    f.extend_from_slice(&start.to_be_bytes());
    f.extend_from_slice(&(values.len() as u16).to_be_bytes());
    f.push(byte_count);
    for chunk in values.chunks(8) {
        let mut byte = 0u8;
        for (i, &v) in chunk.iter().enumerate() {
            if v { byte |= 1 << i; }
        }
        f.push(byte);
    }
    let crc = crc16(&f);
    f.extend_from_slice(&crc.to_le_bytes());
    f
}

#[test]
fn test_write_coil_frame() {
    let f = build_write_single_coil_tcp(1, 1, 100, true);
    assert_eq!(f[7], 0x05);
    assert_eq!(f[10], 0xFF);
    assert_eq!(f[11], 0x00);
}

#[test]
fn test_write_multiple_coils_frame() {
    let f = build_write_multiple_coils_tcp(1, 1, 0, &[true, false, true, false, true]);
    assert_eq!(f[7], 0x0F);
    assert_eq!(f[13], 0b00010101);
}
