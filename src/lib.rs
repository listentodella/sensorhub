pub mod attr;
pub mod log;
pub mod pb;
pub mod suid;

pub use attr::*;
pub use inventory as sensor_inventory;
pub use log::{debug, error, info, trace, warn};

pub use suid::Suid;

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[macro_export]
macro_rules! collect_sensors {
    ($t:ty) => {
        $crate::sensor_inventory::collect!($t);
    };
}

#[macro_export]
macro_rules! register_sensor {
    ($expr:expr) => {
        $crate::sensor_inventory::submit!($expr);
    };
}

#[derive(Default, Debug, Clone, Eq, PartialEq, strum_macros::Display, strum_macros::EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum SensorType {
    #[default]
    #[strum(serialize = "unknown")]
    Unknown,
    #[strum(serialize = "accel")]
    Accelerometer,
    #[strum(serialize = "gyro")]
    Gyroscope,
    #[strum(serialize = "mag")]
    Magnetic,
    #[strum(serialize = "sensor_temperature")]
    Temperature,
    #[strum(serialize = "ambient_light")]
    AmbientLight,
    #[strum(serialize = "proximity")]
    Proximity,
    #[strum(serialize = "resampler")]
    Resampler,
}

pub trait SensorOps: Sync {
    fn open(&mut self, req_odr: u32) {
        trace!("default open: {req_odr}");
    }

    fn hw_open(&mut self) {
        trace!("forget to impl or not?");
    }

    fn close(&mut self) {
        trace!("default close");
    }
    fn flush(&mut self) {
        trace!("default flush");
    }
    fn batch(&mut self) {
        trace!("default batch");
    }

    fn attrs(&self) -> &Vec<SensorAttr>;
    fn attrs_mut(&mut self) -> &mut Vec<SensorAttr>;

    /// try to get the sensor attr from the sensor
    fn get_attr(&self, attr: SensorAttr) -> Option<&SensorAttr> {
        self.attrs().iter().find(|&a| a.id() == attr.id())
    }

    /// try to set the sensor attr to the sensor
    /// if the sensor already has the attr, it will be updated
    /// if the sensor doesn't have the attr, it will be added
    fn set_attr(&mut self, attr: SensorAttr) {
        if let Some(old) = self.attrs_mut().iter_mut().find(|a| a.id() == attr.id()) {
            *old = attr;
        } else {
            self.attrs_mut().push(attr);
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Sensor {
    pub suid: Option<Suid>,
    pub attrs: Vec<SensorAttr>,
}

impl Sensor {
    pub fn new() -> Self {
        Self {
            suid: None,
            attrs: vec![],
        }
    }

    fn set_suid(&mut self, suid: Suid) {
        self.suid = Some(suid);
    }

    pub fn get_suid(&self) -> Option<&Suid> {
        self.suid.as_ref()
    }
}

impl SensorOps for Sensor {
    fn attrs(&self) -> &Vec<SensorAttr> {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<SensorAttr> {
        &mut self.attrs
    }
}

// impl SensorOps for Sensor;
#[derive(Debug, PartialEq)]
pub struct SensorInstance;

// 传感器管理器
#[derive(Debug, Default)]
pub struct SensorManager {
    sensors: Vec<Sensor>,
    module_sensors: std::collections::HashMap<String, Vec<Suid>>,
    suid_to_index: std::collections::HashMap<Suid, usize>,
}

impl SensorManager {
    // 添加传感器并返回 SUID 列表
    fn add_sensors(&mut self, module_name: &str, mut sensors: Vec<Sensor>) -> Vec<Suid> {
        let mut suids = Vec::new();

        for (index, sensor) in sensors.iter_mut().enumerate() {
            // 为每个传感器生成 SUID
            let suid = match sensor.get_suid() {
                Some(existing_suid) => *existing_suid,
                None => {
                    let suid =
                        suid::generate(&format!("{module_name}_{index}")).unwrap_or_else(|_| {
                            error!(
                                "suid already exists, module_name: {module_name}, index: {index}"
                            );
                            suid::generate_fixed("0xdeadbeef")
                        });
                    sensor.set_suid(suid);
                    suid
                }
            };

            suids.push(suid);
        }

        // 记录索引映射
        let start_index = self.sensors.len();
        for (i, suid) in suids.iter().enumerate() {
            self.suid_to_index.insert(*suid, start_index + i);
        }

        self.sensors.extend(sensors);
        self.module_sensors
            .insert(module_name.to_string(), suids.clone());

        suids
    }

    // 获取模块的传感器 UUID 列表
    pub fn get_module_sensor_suids(&self, module_name: &str) -> Option<&[Suid]> {
        self.module_sensors
            .get(module_name)
            .map(|uuids| uuids.as_slice())
    }

    // 通过 SUID 获取传感器（可变引用）
    pub fn get_sensor_mut(&mut self, suid: &Suid) -> Option<&mut Sensor> {
        self.suid_to_index
            .get(suid)
            .and_then(|&index| self.sensors.get_mut(index))
    }

    // 通过 SUID 获取传感器（不可变引用）
    pub fn get_sensor(&self, suid: &Suid) -> Option<&Sensor> {
        self.suid_to_index
            .get(suid)
            .and_then(|&index| self.sensors.get(index))
    }

    // 获取所有传感器
    pub fn get_all_sensors(&self) -> &[Sensor] {
        &self.sensors
    }

    // 更新传感器属性
    pub fn update_sensor_attr(&mut self, suid: &Suid, attr: SensorAttr) {
        if let Some(sensor) = self.get_sensor_mut(suid) {
            sensor.set_attr(attr);
        }
    }

    // 通过 SUID 查找传感器是否存在
    pub fn has_sensor(&self, suid: &Suid) -> bool {
        self.suid_to_index.contains_key(suid)
    }

    // 获取传感器总数
    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }

    // 获取模块的传感器（通过 SUID）
    pub fn get_module_sensors(&self, module_name: &str) -> Vec<&Sensor> {
        if let Some(suids) = self.module_sensors.get(module_name) {
            suids
                .iter()
                .filter_map(|suid| self.get_sensor(suid))
                .collect()
        } else {
            vec![]
        }
    }
}

pub trait SensorModuleOps: Sync {
    fn probe(&self) -> bool {
        trace!("default detect");
        false
    }

    fn remove(&self) {
        trace!("default remove");
    }

    fn create_sensor(&self) -> Vec<Sensor> {
        info!("default create sensor");
        vec![]
    }

    fn create_sensor_instance(&self) -> SensorInstance {
        info!("default create sensor instance");
        SensorInstance
    }
}

pub struct SensorModule {
    pub name: &'static str,
    pub sub_sensor: u8,
    pub ops: &'static dyn SensorModuleOps,
}

impl SensorModule {
    pub fn probe(&self) -> bool {
        self.ops.probe()
    }

    pub fn remove(&self) {
        self.ops.remove();
    }

    pub fn create_sensor(&self) -> Vec<Sensor> {
        self.ops.create_sensor()
    }

    pub fn create_sensor_instance(&self) -> SensorInstance {
        self.ops.create_sensor_instance()
    }
}

collect_sensors! {SensorModule}

#[derive(Debug, Default)]
pub struct SensorHubFw {
    sensor_manager: SensorManager,
    sensor_instances: Vec<SensorInstance>,
}

pub static FW: Lazy<Mutex<SensorHubFw>> = Lazy::new(|| {
    let mut fw = SensorHubFw::default();
    fw.probe_all_sensors();
    Mutex::new(fw)
});

impl SensorHubFw {
    fn probe_all_sensors(&mut self) {
        for module in sensor_inventory::iter::<SensorModule>() {
            info!("Probing sensor: {}", module.name);
            if module.probe() {
                let sensors = module.create_sensor();
                let sensor_suids = self.sensor_manager.add_sensors(module.name, sensors);
                self.sensor_instances.push(module.create_sensor_instance());

                info!(
                    "Module {} registered {} sensors with SUIDs: {:?}",
                    module.name,
                    sensor_suids.len(),
                    sensor_suids
                );
            }
        }
    }

    // 获取传感器管理器
    pub fn get_sensor_manager(&mut self) -> &mut SensorManager {
        &mut self.sensor_manager
    }
}

pub fn init() {
    let sensor_hub_fw = FW.lock().unwrap();
    info!("=========after probe=========");
    info!(
        "registered sensors: {:?}",
        sensor_hub_fw.sensor_manager.get_all_sensors()
    );
    info!("registered instances: {:?}", sensor_hub_fw.sensor_instances);
}

#[cfg(test)]
mod tests {
    use crate::{
        SensorAttr, SensorOps, SensorType, collect_sensors, register_sensor, sensor_inventory,
    };
    use log::info;
    use strum::IntoEnumIterator;

    #[test]
    fn sensor_type_test() {
        for sensor in SensorType::iter() {
            info!("Sensor type: {sensor:?}, string type = {sensor}");
        }
    }

    struct SensorDriverTest {
        pub sensor_type: SensorType,
        pub _ops: &'static dyn SensorOps,
    }

    collect_sensors! {SensorDriverTest}
    struct MySensor1 {
        attrs: Vec<SensorAttr>,
    }
    impl SensorOps for MySensor1 {
        fn attrs(&self) -> &Vec<SensorAttr> {
            &self.attrs
        }
        fn attrs_mut(&mut self) -> &mut Vec<SensorAttr> {
            &mut self.attrs
        }
    }
    static MY_SENSOR1: MySensor1 = MySensor1 { attrs: vec![] };
    register_sensor! {SensorDriverTest {
        sensor_type: SensorType::Accelerometer,
        _ops: &MY_SENSOR1,
    }}

    struct MySensor2 {
        _val: u32,
        attrs: Vec<SensorAttr>,
    }
    impl SensorOps for MySensor2 {
        fn attrs(&self) -> &Vec<SensorAttr> {
            &self.attrs
        }
        fn attrs_mut(&mut self) -> &mut Vec<SensorAttr> {
            &mut self.attrs
        }
    }
    static MY_SENSOR2: MySensor2 = MySensor2 {
        _val: 233,
        attrs: vec![],
    };
    register_sensor! {SensorDriverTest {
        sensor_type: SensorType::Gyroscope,
        _ops: &MY_SENSOR2,
    }}

    #[test]
    fn probe_all_sensors() {
        for sensor in sensor_inventory::iter::<SensorDriverTest>() {
            info!("====>>>>Probing sensor: {:?}", sensor.sensor_type);
        }
    }
}
