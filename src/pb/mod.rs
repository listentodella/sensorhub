pub mod sns_fw;
pub mod sns_raw;
pub mod sns_std;
pub mod sns_std_sensor;
pub mod sns_std_type;
pub mod sns_suid;

// magic + len + payload + crc
// 2B + 2B + N + 2B
// use USB CRC16
fn crc16(data: &[u8]) -> u16 {
    crc::Crc::<u16>::new(&crc::CRC_16_USB).checksum(data)
}

pub fn build_packet(payload: &[u8]) -> Vec<u8> {
    let mut packet = vec![0xAA, 0x55];
    let len = payload.len() as u16;
    packet.extend_from_slice(&len.to_le_bytes());
    packet.extend_from_slice(payload);
    let crc = crc16(&packet);
    packet.extend_from_slice(&crc.to_le_bytes());
    packet
}

pub fn parse_packet(buf: &[u8]) -> Option<&[u8]> {
    if buf.len() < 6 {
        return None;
    }
    if buf[0..2] != [0xAA, 0x55] {
        return None;
    }
    let len = u16::from_le_bytes([buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len + 2 {
        return None;
    }
    let crc = u16::from_le_bytes([buf[4 + len], buf[5 + len]]);
    if crc16(&buf[..buf.len() - 2]) != crc {
        return None;
    }

    let payload = &buf[4..buf.len() - 2];
    Some(payload)
}

#[cfg(test)]
mod tests {
    use crate::pb::{build_packet, parse_packet, sns_raw};
    use crate::*;
    use prost::Message;
    #[test]
    fn sns_raw_test() {
        log::init();
        let mut read_seq = sns_raw::SnsRawRegisterSequenceReq::default();
        read_seq.reqs.push(sns_raw::SnsRawRegisterReq {
            op: Some(sns_raw::SnsRawRegisterOp::Read as i32),
            addr_len: Some(1),
            data_len: Some(1),
            duration: Some(1000),
            addr: Some(0x00),
            data: Some(vec![0x00]),
        });

        read_seq.reqs.push(sns_raw::SnsRawRegisterReq {
            op: Some(sns_raw::SnsRawRegisterOp::Read as i32),
            addr_len: Some(1),
            data_len: Some(1),
            duration: Some(1000),
            addr: Some(0x02),
            data: Some(vec![0x00]),
        });

        let mut write_seq = sns_raw::SnsRawRegisterSequenceReq::default();
        write_seq.reqs.push(sns_raw::SnsRawRegisterReq {
            op: Some(sns_raw::SnsRawRegisterOp::Write as i32),
            addr_len: Some(1),
            data_len: Some(2),
            duration: Some(1000),
            addr: Some(0x00),
            data: Some(vec![0x01, 0x02]),
        });

        trace!("Read sequence: {read_seq:?}");
        trace!("Write sequence: {write_seq:?}");

        assert_eq!(
            read_seq.reqs[0].op,
            Some(sns_raw::SnsRawRegisterOp::Read as i32)
        );
        assert_eq!(
            read_seq.reqs[1].op,
            Some(sns_raw::SnsRawRegisterOp::Read as i32)
        );
        assert_eq!(
            write_seq.reqs[0].op,
            Some(sns_raw::SnsRawRegisterOp::Write as i32)
        );
        assert_eq!(write_seq.reqs[0].data, Some(vec![0x01, 0x02]));
        assert_eq!(write_seq.reqs[0].data_len, Some(2));

        let mut rsp = sns_raw::SnsRawRegisterRsp::default();
        rsp.data.push(0x01);
        rsp.data.push(0x02);
        trace!("Read response: {rsp:#x?}");
        assert_ne!(rsp.data, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn sns_raw_mock_tx_rx_channel_test() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        // sender thread
        let sender = thread::spawn(move || {
            let req = sns_raw::SnsRawRegisterReq {
                op: Some(sns_raw::SnsRawRegisterOp::Read as i32),
                addr_len: Some(1),
                data_len: Some(1),
                duration: Some(1000),
                addr: Some(0x10),
                data: Some(vec![0x00]),
            };
            tx.send(req).expect("send failed!");
        });
        // receiver thread
        let receiver = thread::spawn(move || {
            if let Ok(req) = rx.recv() {
                trace!("Received request: {req:?}");
                assert_eq!(req.op, Some(sns_raw::SnsRawRegisterOp::Read as i32));
                assert_eq!(req.addr, Some(0x10));
                assert_eq!(req.data, Some(vec![0x00]));
            } else {
                error!("Failed to receive request");
            }
        });

        sender.join().expect("Sender thread panicked");
        receiver.join().expect("Receiver thread panicked");
    }

    #[test]
    fn sns_raw_mock_tx_rx_test() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        // sender thread
        let sender = thread::spawn(move || {
            let req = sns_raw::SnsRawRegisterReq {
                op: Some(sns_raw::SnsRawRegisterOp::Write as i32),
                addr_len: Some(1),
                data_len: Some(2),
                duration: Some(1000),
                addr: Some(0x10),
                data: Some(vec![0x01, 0x02]),
            };
            let mut buf = Vec::new();
            req.encode(&mut buf).unwrap();
            tx.send(buf).expect("send failed!");
        });
        // receiver thread
        let receiver = thread::spawn(move || {
            if let Ok(buf) = rx.recv() {
                let req = sns_raw::SnsRawRegisterReq::decode(&buf[..]).unwrap();
                trace!("Recv req(encoded): {req:?}");
                assert_eq!(req.op, Some(sns_raw::SnsRawRegisterOp::Write as i32));
                assert_eq!(req.addr, Some(0x10));
                assert_eq!(req.data, Some(vec![0x01, 0x02]));
            } else {
                error!("Failed to receive request");
            }
        });

        sender.join().expect("Sender thread panicked");
        receiver.join().expect("Receiver thread panicked");
    }

    #[test]
    fn sns_raw_mock_crc_tx_rx_test() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        // sender thread
        let sender = thread::spawn(move || {
            let req = sns_raw::SnsRawRegisterReq {
                op: Some(sns_raw::SnsRawRegisterOp::Write as i32),
                addr_len: Some(1),
                data_len: Some(2),
                duration: Some(1000),
                addr: Some(0x10),
                data: Some(vec![0x01, 0x02]),
            };
            let mut buf = Vec::new();
            req.encode(&mut buf).unwrap();
            let buf = build_packet(&buf);
            tx.send(buf).expect("send failed!");
        });
        // receiver thread
        let receiver = thread::spawn(move || {
            if let Ok(buf) = rx.recv() {
                if let Some(payload) = parse_packet(&buf) {
                    let req = sns_raw::SnsRawRegisterReq::decode(payload).unwrap();
                    trace!("Recv req(encoded+crc): {req:?}");
                    assert_eq!(req.op, Some(sns_raw::SnsRawRegisterOp::Write as i32));
                    assert_eq!(req.addr, Some(0x10));
                    assert_eq!(req.data, Some(vec![0x01, 0x02]));
                } else {
                    error!("Failed to parse packet");
                }
            } else {
                error!("Failed to receive request");
            }
        });

        sender.join().expect("Sender thread panicked");
        receiver.join().expect("Receiver thread panicked");
    }
}
