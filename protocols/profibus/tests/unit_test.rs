// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz

use industrial_protocols_kernel::connection::Connection;
use industrial_protocols_kernel::error::ProtocolError;
use industrial_protocols_kernel::sdk::*;
use industrial_protocols_profibus::*;
use std::io;

/// Mock connection with a canned response and a failure mode.
struct MockConn {
    response: Vec<u8>,
    fail: bool,
    sent: Vec<Vec<u8>>,
}

impl MockConn {
    fn new(response: Vec<u8>) -> Self {
        MockConn {
            response,
            fail: false,
            sent: Vec::new(),
        }
    }
    fn failing() -> Self {
        MockConn {
            response: Vec::new(),
            fail: true,
            sent: Vec::new(),
        }
    }
}

impl Connection for MockConn {
    fn send(&mut self, data: &[u8]) -> io::Result<Vec<u8>> {
        if self.fail {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "mock failure"));
        }
        self.sent.push(data.to_vec());
        Ok(self.response.clone())
    }
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn is_alive(&self) -> bool {
        true
    }
}

/// Re-implements the PROFIBUS FCS (wrapping sum of all bytes before FCS/ED).
fn fcs_of(f: &[u8]) -> u8 {
    f[..f.len() - 2].iter().fold(0u8, |a, &b| a.wrapping_add(b))
}

// ---------- frame builders ----------

#[test]
fn profibus_dp_read_exact_bytes_and_fcs() {
    let f = build_dp_read(1, 2, 3, 4);
    assert_eq!(f[0], 0x68); // SD2
    assert_eq!(f[1], 0x05); // LE
    assert_eq!(f[2], 0x05); // LEr
    assert_eq!(f[3], 0x81); // DA = 0x80 | slave
    assert_eq!(f[4], 0x00); // SA = master
    assert_eq!(f[5], 0x5C); // FC data exchange read
    assert_eq!(&f[6..9], &[2, 3, 4]); // slot, index, len
    assert_eq!(f[f.len() - 2], fcs_of(&f)); // FCS is correct
    assert_eq!(*f.last().unwrap(), 0x16); // ED
}

#[test]
fn profibus_dp_read_edge_slave_addr() {
    let f = build_dp_read(0x7F, 0, 0, 0);
    assert_eq!(f[3], 0xFF);
    let f = build_dp_read(0, 0, 0, 0);
    assert_eq!(f[3], 0x80);
}

#[test]
fn profibus_dp_write_exact_bytes_and_fcs() {
    let f = build_dp_write(1, 2, 3, &[0xAA, 0xBB]);
    assert_eq!(
        &f[0..9],
        &[0x68, 0x05, 0x05, 0x81, 0x00, 0x5D, 0x02, 0x03, 0x02]
    );
    assert_eq!(&f[9..11], &[0xAA, 0xBB]);
    assert_eq!(f[f.len() - 2], fcs_of(&f));
    assert_eq!(*f.last().unwrap(), 0x16);
}

#[test]
fn profibus_dp_write_empty_data() {
    let f = build_dp_write(1, 0, 0, &[]);
    assert_eq!(f[1], 0x03); // LE = 0 + 3
    assert_eq!(f.len(), 11); // 9 header + 0 data + FCS + ED
    assert_eq!(f[8], 0x00); // data length byte
    assert_eq!(f[f.len() - 2], fcs_of(&f));
}

#[test]
fn profibus_dp_write_max_len_field() {
    let data = vec![0x11; 0xFC]; // 252 bytes keeps len field = 255
    let f = build_dp_write(1, 0, 0, &data);
    assert_eq!(f[1], 255);
    assert_eq!(f[8], 0xFC);
    assert_eq!(f.len(), 9 + 252 + 2);
    assert_eq!(f[f.len() - 2], fcs_of(&f));
}

#[test]
fn profibus_diag_request_exact_bytes() {
    assert_eq!(
        build_diag_request(1),
        vec![0x68, 0x03, 0x03, 0x81, 0x00, 0x5A, 0xA0, 0x16]
    );
    assert_eq!(build_diag_request(0)[3], 0x80);
}

// ---------- protocol metadata & builders ----------

#[test]
fn profibus_metadata() {
    let p = ProfibusProtocol::new();
    assert_eq!(p.name(), "profibus");
    assert_eq!(p.version(), "1.0.0");
    assert_eq!(p.category(), ProtocolCategory::Fieldbus);
    assert_eq!(p.supported_variants(), &["dp", "pa", "fms"]);
}

#[test]
fn profibus_default_and_builders() {
    let p = ProfibusProtocol::default();
    assert_eq!(p.slave_addr, 1);
    assert_eq!(p.gateway_host, "192.168.1.100");
    let p = ProfibusProtocol::new()
        .with_slave(9)
        .with_gateway("10.0.0.1");
    assert_eq!(p.slave_addr, 9);
    assert_eq!(p.gateway_host, "10.0.0.1");
}

// ---------- ProtocolRead ----------

#[test]
fn profibus_read_sends_dp_read_frame() {
    let p = ProfibusProtocol::new().with_slave(1);
    let mut conn = MockConn::new(vec![0x00, 0x01]);
    let v = ProtocolRead::read(&p, &mut conn, &0x0102).unwrap(); // slot 1, index 2
    assert_eq!(v, vec![0x00, 0x01]);
    assert_eq!(conn.sent, vec![build_dp_read(1, 1, 2, 4)]);
}

#[test]
fn profibus_read_connection_failure() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::failing();
    let err = ProtocolRead::read(&p, &mut conn, &0).unwrap_err();
    assert!(matches!(err, ProtocolError::Connection(_)));
}

#[test]
fn profibus_read_batch_and_range() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::new(vec![0xAB]);
    let vals = ProtocolRead::read_batch(&p, &mut conn, &[0x0101, 0x0202]).unwrap();
    assert_eq!(vals, vec![vec![0xAB], vec![0xAB]]);
    let mut conn = MockConn::new(vec![0xAB]);
    let vals = ProtocolRead::read_range(&p, &mut conn, &0x0101, 3).unwrap();
    assert_eq!(vals, vec![vec![0xAB]; 3]);
}

#[test]
fn profibus_read_batch_empty_and_range_zero() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    assert_eq!(
        ProtocolRead::read_batch(&p, &mut conn, &[]).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(
        ProtocolRead::read_range(&p, &mut conn, &0, 0).unwrap(),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(conn.sent.len(), 0);
}

#[test]
fn profibus_read_batch_connection_failure() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolRead::read_batch(&p, &mut conn, &[0]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolWrite ----------

#[test]
fn profibus_write_sends_dp_write_frame() {
    let p = ProfibusProtocol::new().with_slave(2);
    let mut conn = MockConn::new(vec![]);
    ProtocolWrite::write(&p, &mut conn, &0x0102, &vec![0xDE, 0xAD]).unwrap();
    assert_eq!(conn.sent, vec![build_dp_write(2, 1, 2, &[0xDE, 0xAD])]);
}

#[test]
fn profibus_write_batch_multiple() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::new(vec![]);
    ProtocolWrite::write_batch(&p, &mut conn, &[(0x0101, vec![0x01]), (0x0202, vec![0x02])])
        .unwrap();
    assert_eq!(conn.sent.len(), 2);
    assert_eq!(conn.sent[0], build_dp_write(1, 1, 1, &[0x01]));
    assert_eq!(conn.sent[1], build_dp_write(1, 2, 2, &[0x02]));
}

#[test]
fn profibus_write_connection_failure() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::failing();
    assert!(matches!(
        ProtocolWrite::write(&p, &mut conn, &0, &vec![0x01]).unwrap_err(),
        ProtocolError::Connection(_)
    ));
}

// ---------- ProtocolHealth ----------

#[test]
fn profibus_health_healthy() {
    let p = ProfibusProtocol::new().with_slave(5);
    let mut conn = MockConn::new(vec![0x00]);
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Healthy);
    assert!(h.details.contains("diag"));
    assert_eq!(conn.sent, vec![build_diag_request(5)]);
}

#[test]
fn profibus_health_unhealthy() {
    let p = ProfibusProtocol::new();
    let mut conn = MockConn::failing();
    let h = ProtocolHealth::health(&p, &mut conn).unwrap();
    assert_eq!(h.state, HealthState::Unhealthy);
    assert!(h.details.contains("mock failure"));
}

// ---------- ProtocolDiagnostics ----------

#[test]
fn profibus_diagnostics_fields() {
    let p = ProfibusProtocol::new().with_slave(7);
    let mut conn = MockConn::new(vec![]);
    let d = ProtocolDiagnostics::diagnostics(&p, &mut conn).unwrap();
    assert_eq!(d.device_id, "pb:slave7");
    assert_eq!(d.uptime_secs, None);
    assert_eq!(d.error_count, 0);
    assert_eq!(d.last_error, None);
    assert!(d.custom_fields.is_empty());
}
