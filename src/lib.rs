pub mod attr;
pub mod log;
pub mod pb;
pub mod uuid;

pub use attr::*;
pub use inventory as sensor_inventory;
pub use log::{debug, error, info, trace, warn};

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
    fn probe(&self) -> bool {
        trace!("default detect");
        false
    }

    fn remove(&self) {
        trace!("default remove");
    }

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

pub struct SensorDriver {
    pub sensor_type: SensorType,
    pub ops: &'static dyn SensorOps,
}

collect_sensors! {SensorDriver}

fn probe_all_sensors() {
    for sensor in sensor_inventory::iter::<SensorDriver>() {
        info!("Probing sensor: {:?}", sensor.sensor_type);
        sensor.ops.probe();
    }
}

pub fn init() {
    probe_all_sensors();
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
        pub ops: &'static dyn SensorOps,
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
        ops: &MY_SENSOR1,
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

        fn probe(&self) -> bool {
            info!("MySensor2 detected with value: {}", self._val);
            true
        }
    }
    static MY_SENSOR2: MySensor2 = MySensor2 {
        _val: 233,
        attrs: vec![],
    };
    register_sensor! {SensorDriverTest {
        sensor_type: SensorType::Gyroscope,
        ops: &MY_SENSOR2,
    }}

    #[test]
    fn probe_all_sensors() {
        for sensor in sensor_inventory::iter::<SensorDriverTest>() {
            info!("====>>>>Probing sensor: {:?}", sensor.sensor_type);
            sensor.ops.probe();
        }
    }
}
