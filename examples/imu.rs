use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use sensorhub_rs::{
    Sensor, SensorAttr, SensorModule, SensorModuleOps, SensorOps, SensorType, log::debug,
    register_sensor,
};

#[derive(Debug)]
#[allow(dead_code)]
struct AccelSensor {
    sensor: Sensor,
}

#[allow(dead_code)]
impl AccelSensor {
    fn new() -> Self {
        Self {
            sensor: Sensor::new(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct GyroSensor {
    sensor: Sensor,
}

#[allow(dead_code)]
impl GyroSensor {
    fn new() -> Self {
        Self {
            sensor: Sensor::new(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Imu {
    accel: AccelSensor,
    gyro: GyroSensor,
}

impl Imu {
    fn new() -> Self {
        Self {
            accel: AccelSensor::new(),
            gyro: GyroSensor::new(),
        }
    }

    fn read_chip_id(&self) -> bool {
        debug!("read_chip_id");
        true
    }
}

// 实现 IMU 模块的具体操作
struct ImuModuleOps {
    imu: &'static Lazy<Arc<Mutex<Imu>>>,
}

impl SensorModuleOps for ImuModuleOps {
    fn probe(&self) -> bool {
        debug!("Probing IMU module: mVendor-0000");
        let imu = self.imu.lock().unwrap();

        // 这里可以添加实际的硬件检测逻辑
        // 例如：检查 I2C 地址、读取设备 ID 等
        imu.read_chip_id()
    }

    fn remove(&self) {
        debug!("Removing IMU module: mVendor-0000");
    }

    fn create_sensor(&self) -> Vec<Sensor> {
        debug!("Creating sensors for IMU module: mVendor-0000");

        let mut accel_sensor = Sensor::new();
        accel_sensor.set_attr(SensorAttr::Name("acc".to_string()));
        accel_sensor.set_attr(SensorAttr::Type(SensorType::Accelerometer));
        accel_sensor.set_attr(SensorAttr::Vendor("mVendor".to_string()));

        let mut gyro_sensor = Sensor::new();
        gyro_sensor.set_attr(SensorAttr::Name("gyro".to_string()));
        gyro_sensor.set_attr(SensorAttr::Type(SensorType::Gyroscope));
        gyro_sensor.set_attr(SensorAttr::Vendor("mVendor".to_string()));

        vec![accel_sensor, gyro_sensor]
    }

    fn create_sensor_instance(&self) -> sensorhub_rs::SensorInstance {
        debug!("Creating sensor instance for IMU module: mVendor-0000");
        sensorhub_rs::SensorInstance
    }
}

static IMU_INSTANCE: Lazy<Arc<Mutex<Imu>>> = Lazy::new(|| Arc::new(Mutex::new(Imu::new())));
static IMU_MODULE_OPS: ImuModuleOps = ImuModuleOps { imu: &IMU_INSTANCE };

register_sensor! {
    SensorModule {
        name: "mVendor-0000",
        sub_sensor: 2,
        ops: &IMU_MODULE_OPS,
    }
}

fn main() {
    sensorhub_rs::log::init();
    sensorhub_rs::init();

    let mut fw = sensorhub_rs::FW.lock().unwrap();
    let sensor_manager = fw.get_sensor_manager();

    // 先获取 IMU 模块的传感器 UUID 列表
    let suids = sensor_manager
        .get_module_sensor_suids("mVendor-0000")
        .map(|suids| suids.to_vec())
        .unwrap_or_default();

    debug!("IMU module sensor SUIDs: {suids:?}");

    // 演示如何通过 SUID 获取传感器并进行操作
    for (i, suid) in suids.iter().enumerate() {
        if let Some(sensor) = sensor_manager.get_sensor_mut(suid) {
            debug!("Operating on sensor {i} with SUID: {suid}");

            // 这里可以进行各种传感器操作
            // 例如：设置采样率、开启数据流等
            sensor.set_attr(SensorAttr::Available(true));

            debug!("Sensor {} attributes: {:?}", i, sensor.attrs());
        }
    }

    debug!("IMU example completed");
}
