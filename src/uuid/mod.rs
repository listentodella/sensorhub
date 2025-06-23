use once_cell::sync::Lazy;
use std::sync::Mutex;
use uuid::Uuid;

// 静态Vec存储已生成的UUID
static GENERATED_UUIDS: Lazy<Mutex<Vec<Uuid>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// 生成UUID：如果检测到重复则返回Err
pub fn generate(input: &str) -> Result<Uuid, String> {
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_DNS, input.as_bytes());

    let mut generated_uuids = GENERATED_UUIDS.lock().unwrap();

    // 检查是否已经生成过这个UUID
    if generated_uuids.contains(&uuid) {
        Err(format!("Duplicate UUID for {input}->{uuid}"))
    } else {
        // 记录新生成的UUID
        generated_uuids.push(uuid);
        Ok(uuid)
    }
}

pub fn get_generated_count() -> usize {
    GENERATED_UUIDS.lock().unwrap().len()
}

pub fn get_u64_pair(uuid: &Uuid) -> (u64, u64) {
    uuid.as_u64_pair()
}

#[allow(dead_code)]
/// skip check duplicate, only for test
fn generate_fixed(input: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::trace;
    use serial_test::serial;

    fn reset_state() {
        GENERATED_UUIDS.lock().unwrap().clear();
    }

    #[test]
    #[serial]
    fn uuid_generate_test() {
        reset_state();

        let input = "example.com";
        let uuid1 = generate(input).unwrap();
        trace!("First uuid: {uuid1:?}");

        let result = generate(input);
        assert!(result.is_err());
        trace!("Second call result: {result:?}");

        let fixed_uuid = generate_fixed(input);
        assert_eq!(uuid1, fixed_uuid);

        assert_eq!(get_generated_count(), 1);
    }

    #[test]
    #[serial]
    fn uuid_multiple_inputs_test() {
        reset_state();

        let uuid1 = generate("input1").unwrap();
        let uuid2 = generate("input2").unwrap();
        let uuid3 = generate("input3").unwrap();

        trace!("uuid1: {uuid1:?}");
        trace!("uuid2: {uuid2:?}");
        trace!("uuid3: {uuid3:?}");

        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid2, uuid3);
        assert_ne!(uuid1, uuid3);

        assert_eq!(get_generated_count(), 3);

        assert!(generate("input1").is_err());
        assert!(generate("input2").is_err());
        assert!(generate("input3").is_err());
    }

    #[test]
    fn uuid_fixed_generate_test() {
        let input = "example.com";
        let uuid = generate_fixed(input);
        let uuid2 = generate_fixed(input);
        trace!("Fixed uuid: {uuid:?}, uuid2: {uuid2:?}");
        assert_eq!(uuid, uuid2);
        assert_eq!(uuid.to_string(), "cfbff0d1-9375-5685-968c-48ce8b15ae17");
    }

    #[test]
    fn uuid_u64_pair_test() {
        let input = "example.com";
        let uuid = generate_fixed(input);
        let (high, low) = get_u64_pair(&uuid);
        trace!("pair high - {high:#x}, low - {low:#x}");
        assert_eq!(high, 0xcfbff0d193755685);
        assert_eq!(low, 0x968c48ce8b15ae17);
    }
}
