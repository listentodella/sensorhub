use log::error;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use sensorhub_rs::{
    Sensor, SensorAttr, SensorModule, SensorModuleOps, SensorOps, SensorType, Suid, log::debug,
    register_sensor,
};

#[derive(Debug)]
#[allow(dead_code)]
struct AccelSensor {
    sensor: Arc<Mutex<Sensor>>,
}

#[allow(dead_code)]
impl AccelSensor {
    fn new(sensor: Arc<Mutex<Sensor>>) -> Self {
        Self { sensor }
    }

    fn set_attr(&self, attr: SensorAttr) {
        if let Ok(mut sensor) = self.sensor.lock() {
            sensor.set_attr(attr);
        }
    }

    fn get_attr(&self, attr: SensorAttr) -> Option<SensorAttr> {
        if let Ok(sensor) = self.sensor.lock() {
            sensor.get_attr(attr).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct GyroSensor {
    sensor: Arc<Mutex<Sensor>>,
}

#[allow(dead_code)]
impl GyroSensor {
    fn new(sensor: Arc<Mutex<Sensor>>) -> Self {
        Self { sensor }
    }

    fn set_attr(&self, attr: SensorAttr) {
        if let Ok(mut sensor) = self.sensor.lock() {
            sensor.set_attr(attr);
        }
    }

    fn get_attr(&self, attr: SensorAttr) -> Option<SensorAttr> {
        if let Ok(sensor) = self.sensor.lock() {
            sensor.get_attr(attr).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Imu {
    name: String,
    accel: Option<AccelSensor>,
    gyro: Option<GyroSensor>,
    accel_suid: Option<Suid>,
    gyro_suid: Option<Suid>,
}

impl Imu {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            accel: None,
            gyro: None,
            accel_suid: None,
            gyro_suid: None,
        }
    }

    fn read_chip_id(&self) -> bool {
        debug!("read_chip_id");
        true
    }

    // 设置共享的传感器对象
    fn set_accel_sensor(&mut self, sensor: Arc<Mutex<Sensor>>, suid: Suid) {
        self.accel = Some(AccelSensor::new(sensor));
        self.accel_suid = Some(suid);
    }

    fn set_gyro_sensor(&mut self, sensor: Arc<Mutex<Sensor>>, suid: Suid) {
        self.gyro = Some(GyroSensor::new(sensor));
        self.gyro_suid = Some(suid);
    }

    // 更新传感器状态的方法
    fn update_accel_status(&self, available: bool) {
        if let Some(accel) = &self.accel {
            accel.set_attr(SensorAttr::Available(available));
        }
    }

    fn update_gyro_status(&self, available: bool) {
        if let Some(gyro) = &self.gyro {
            gyro.set_attr(SensorAttr::Available(available));
        }
    }

    fn set_accel_sample_rate(&self, rate: f32) {
        if let Some(accel) = &self.accel {
            accel.set_attr(SensorAttr::Rates(vec![rate]));
        }
    }

    fn set_gyro_sample_rate(&self, rate: f32) {
        if let Some(gyro) = &self.gyro {
            gyro.set_attr(SensorAttr::Rates(vec![rate]));
        }
    }
}

// 实现 IMU 模块的具体操作
struct ImuModuleOps {
    imu: &'static Lazy<Arc<Mutex<Imu>>>,
}

impl SensorModuleOps for ImuModuleOps {
    fn install(&self, module_name: &str) -> bool {
        debug!("Probing IMU module: {module_name}");
        let mut imu = self.imu.lock().unwrap();
        imu.name = module_name.to_string();

        // 这里可以添加实际的硬件检测逻辑
        // 例如：检查 I2C 地址、读取设备 ID 等
        imu.read_chip_id()
    }

    fn uninstall(&self) {
        debug!("Removing IMU module: {}", self.imu.lock().unwrap().name);
    }

    fn create_sensor(&self, hw_id: u8) -> Vec<Sensor> {
        let imu = self.imu.lock().unwrap();
        debug!(
            "Creating sensors for IMU module: {}, hw_id: {hw_id}",
            imu.name
        );

        let mut accel_sensor = Sensor::new();
        accel_sensor.set_attr(SensorAttr::Name("acc".to_string()));
        accel_sensor.set_attr(SensorAttr::Type(SensorType::Accelerometer));
        accel_sensor.set_attr(SensorAttr::Vendor("mVendor".to_string()));
        accel_sensor.set_attr(SensorAttr::HwId(hw_id as u32));

        let mut gyro_sensor = Sensor::new();
        gyro_sensor.set_attr(SensorAttr::Name("gyro".to_string()));
        gyro_sensor.set_attr(SensorAttr::Type(SensorType::Gyroscope));
        gyro_sensor.set_attr(SensorAttr::Vendor("mVendor".to_string()));
        gyro_sensor.set_attr(SensorAttr::HwId(hw_id as u32));

        vec![accel_sensor, gyro_sensor]
    }

    fn create_sensor_instance(&self) -> sensorhub_rs::SensorInstance {
        debug!(
            "Creating sensor instance for IMU module: {}",
            self.imu.lock().unwrap().name
        );
        sensorhub_rs::SensorInstance
    }
}

static IMU_INSTANCE: Lazy<Arc<Mutex<Imu>>> =
    Lazy::new(|| Arc::new(Mutex::new(Imu::new("mVendor-000x"))));
static IMU_MODULE_OPS: ImuModuleOps = ImuModuleOps { imu: &IMU_INSTANCE };

register_sensor! {
    SensorModule {
        name: "mVendor-0000",
        hw_id: 0,
        sub_sensor: 2,
        ops: &IMU_MODULE_OPS,
    }
}

register_sensor! {
    SensorModule {
        name: "mVendor-0000",
        hw_id: 1,
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

    error!("IMU module sensor SUIDs: {suids:?}");

    // 将共享的传感器对象传递给 IMU 实例
    if suids.len() >= 2 {
        let mut imu = IMU_INSTANCE.lock().unwrap();

        // 获取加速度计传感器
        if let Some(accel_sensor_arc) = sensor_manager.get_sensor_arc(&suids[0]) {
            imu.set_accel_sensor(accel_sensor_arc.clone(), suids[0]);
            debug!("Accel sensor connected with SUID: {}", suids[0]);
        }

        // 获取陀螺仪传感器
        if let Some(gyro_sensor_arc) = sensor_manager.get_sensor_arc(&suids[1]) {
            imu.set_gyro_sensor(gyro_sensor_arc.clone(), suids[1]);
            debug!("Gyro sensor connected with SUID: {}", suids[1]);
        }
    }

    // 演示 IMU 驱动如何修改传感器状态
    {
        let imu = IMU_INSTANCE.lock().unwrap();

        // IMU 驱动更新传感器状态
        imu.update_accel_status(true);
        imu.update_gyro_status(true);

        // IMU 驱动设置采样率
        imu.set_accel_sample_rate(100.0);
        imu.set_gyro_sample_rate(200.0);

        debug!("IMU driver updated sensor status");
    }

    // 验证 SensorManager 能够观察到 IMU 驱动的修改
    for (i, suid) in suids.iter().enumerate() {
        if let Some(sensor) = sensor_manager.get_sensor(suid) {
            debug!("Sensor {i} with SUID: {suid}");
            debug!("Sensor {} attributes: {:?}", i, sensor.attrs());

            // 检查是否能看到 IMU 驱动的修改
            if let Some(SensorAttr::Available(available)) =
                sensor.get_attr(SensorAttr::Available(false))
            {
                debug!("Sensor {i} available: {available}");
            }

            if let Some(SensorAttr::Rates(rates)) = sensor.get_attr(SensorAttr::Rates(vec![])) {
                debug!("Sensor {i} rates: {rates:?}");
            }
        }
    }

    debug!("IMU example completed - demonstrating shared sensor access");
}
