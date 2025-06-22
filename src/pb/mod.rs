pub mod sns_fw;
pub mod sns_raw;
pub mod sns_std;
pub mod sns_std_sensor;
pub mod sns_std_type;
pub mod sns_suid;

#[cfg(test)]
mod tests {
    use crate::pb::sns_raw;
    #[test]
    fn sns_raw_test() {
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

        println!("Read sequence: {read_seq:?}");
        println!("Write sequence: {write_seq:?}");

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
        println!("Read response: {rsp:#x?}");
        assert_ne!(rsp.data, vec![0x01, 0x02, 0x03]);
    }
}
