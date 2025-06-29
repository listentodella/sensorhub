pub mod attr;
pub mod log;
pub mod pb;
pub mod suid;

pub use attr::*;
pub use inventory as sensor_inventory;
pub use log::{debug, error, info, trace, warn};

pub use suid::Suid;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

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

// 传感器管理器 - 使用 Arc<Mutex<Sensor>> 来共享传感器
#[derive(Debug, Default)]
pub struct SensorManager {
    sensors: Vec<Arc<Mutex<Sensor>>>,
    module_sensors: std::collections::HashMap<String, Vec<Suid>>, // key: "module_name_hw_id"
    suid_to_index: std::collections::HashMap<Suid, usize>,
}

impl SensorManager {
    // 生成模块的唯一标识符
    fn module_key(module: &SensorModule) -> String {
        format!("{}_{}", module.name, module.hw_id)
    }

    // 添加传感器并返回 SUID 列表
    fn add_sensors(&mut self, module: &SensorModule, mut sensors: Vec<Sensor>) -> Vec<Suid> {
        let mut suids = Vec::new();

        for (index, sensor) in sensors.iter_mut().enumerate() {
            // 为每个传感器生成 SUID
            let suid = match sensor.get_suid() {
                Some(existing_suid) => *existing_suid,
                None => {
                    let suid =
                        suid::generate(&format!("{}_{}_{}", module.name, module.hw_id, index))
                            .unwrap_or_else(|_| {
                                error!(
                                    "suid already exists, module_name: {}, hw_id: {}, index: {}",
                                    module.name, module.hw_id, index
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

        self.sensors
            .extend(sensors.into_iter().map(|s| Arc::new(Mutex::new(s))));

        // 使用 module_name + hw_id 作为唯一标识符
        let module_key = Self::module_key(module);
        self.module_sensors.insert(module_key, suids.clone());

        suids
    }

    // 获取模块的传感器 UUID 列表
    pub fn get_module_sensor_suids(&self, module_name: &str, hw_id: u8) -> Option<&[Suid]> {
        let module_key = format!("{module_name}_{hw_id}");
        self.module_sensors
            .get(&module_key)
            .map(|uuids| uuids.as_slice())
    }

    // 获取模块的传感器 UUID 列表（向后兼容，仅使用 module_name）
    pub fn get_module_sensor_suids_by_name(&self, module_name: &str) -> Option<&[Suid]> {
        // 查找所有匹配的模块
        for (key, suids) in &self.module_sensors {
            if key.starts_with(&format!("{module_name}_")) {
                return Some(suids.as_slice());
            }
        }
        None
    }

    // 获取所有匹配 module_name 的传感器 UUID 列表
    pub fn get_all_module_sensor_suids(&self, module_name: &str) -> Vec<Suid> {
        let mut all_suids = Vec::new();
        for (key, suids) in &self.module_sensors {
            if key.starts_with(&format!("{module_name}_")) {
                all_suids.extend(suids.iter().cloned());
            }
        }
        all_suids
    }

    // 通过 SUID 获取传感器的 Arc<Mutex<Sensor>> 引用
    pub fn get_sensor_arc(&self, suid: &Suid) -> Option<&Arc<Mutex<Sensor>>> {
        self.suid_to_index
            .get(suid)
            .and_then(|&index| self.sensors.get(index))
    }

    // 通过 SUID 获取传感器（可变引用）
    pub fn get_sensor_mut(&mut self, suid: &Suid) -> Option<std::sync::MutexGuard<'_, Sensor>> {
        self.suid_to_index
            .get(suid)
            .and_then(|&index| self.sensors.get(index))
            .and_then(|arc_sensor| arc_sensor.lock().ok())
    }

    // 通过 SUID 获取传感器（不可变引用）
    pub fn get_sensor(&self, suid: &Suid) -> Option<std::sync::MutexGuard<'_, Sensor>> {
        self.suid_to_index
            .get(suid)
            .and_then(|&index| self.sensors.get(index))
            .and_then(|arc_sensor| arc_sensor.lock().ok())
    }

    // 获取所有传感器的 Arc<Mutex<Sensor>> 引用
    fn get_all_sensor_arcs(&self) -> &[Arc<Mutex<Sensor>>] {
        &self.sensors
    }

    // 更新传感器属性
    pub fn update_sensor_attr(&mut self, suid: &Suid, attr: SensorAttr) {
        if let Some(mut sensor) = self.get_sensor_mut(suid) {
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

    // 获取模块的传感器（通过 module_name + hw_id）
    pub fn get_module_sensors(&self, module_name: &str, hw_id: u8) -> Vec<Arc<Mutex<Sensor>>> {
        let module_key = format!("{module_name}_{hw_id}");
        if let Some(suids) = self.module_sensors.get(&module_key) {
            suids
                .iter()
                .filter_map(|suid| self.get_sensor_arc(suid))
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    // 获取模块的传感器（向后兼容，仅使用 module_name）
    pub fn get_module_sensors_by_name(&self, module_name: &str) -> Vec<Arc<Mutex<Sensor>>> {
        let mut all_sensors = Vec::new();
        for (key, suids) in &self.module_sensors {
            if key.starts_with(&format!("{module_name}_")) {
                for suid in suids {
                    if let Some(sensor_arc) = self.get_sensor_arc(suid) {
                        all_sensors.push(sensor_arc.clone());
                    }
                }
            }
        }
        all_sensors
    }

    // 获取所有模块的传感器（通过 module_name）
    pub fn get_all_module_sensors(&self, module_name: &str) -> Vec<Arc<Mutex<Sensor>>> {
        let mut all_sensors = Vec::new();
        for (key, suids) in &self.module_sensors {
            if key.starts_with(&format!("{module_name}_")) {
                for suid in suids {
                    if let Some(sensor_arc) = self.get_sensor_arc(suid) {
                        all_sensors.push(sensor_arc.clone());
                    }
                }
            }
        }
        all_sensors
    }
}

pub trait SensorModuleOps: Sync {
    //TODO: maybe we should abstract a data struct contains more info
    // from the SensorModule
    fn install(&self, module_name: &str) -> bool {
        trace!("default detect: {module_name}");
        false
    }

    fn uninstall(&self) {
        trace!("default uninstall");
    }

    fn create_sensor(&self, hw_id: u8) -> Vec<Sensor> {
        info!("default create sensor: {hw_id}");
        vec![]
    }

    fn create_sensor_instance(&self) -> SensorInstance {
        info!("default create sensor instance");
        SensorInstance
    }
}

pub struct SensorModule {
    pub name: &'static str,
    pub hw_id: u8,
    pub sub_sensor: u8,
    pub ops: &'static dyn SensorModuleOps,
}

impl SensorModule {
    pub fn probe(&self) -> bool {
        self.ops.install(self.name)
    }

    pub fn remove(&self) {
        self.ops.uninstall();
    }

    pub fn create_sensor(&self) -> Vec<Sensor> {
        self.ops.create_sensor(self.hw_id)
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
                let sensor_suids = self.sensor_manager.add_sensors(module, sensors);
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
        sensor_hub_fw.sensor_manager.get_all_sensor_arcs()
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
