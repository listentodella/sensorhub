pub mod pb;

#[derive(Debug, Clone, Eq, PartialEq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SensorType {
    #[strum(serialize = "accel")]
    Accelermeter,
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

pub trait SensorOps {
    fn detect(&mut self) -> bool {
        println!("default detect");
        false
    }

    fn open(&mut self, req_odr: u32) {
        println!("default open: {req_odr}");
    }

    fn hw_open(&mut self) {
        println!("forget to impl or not?");
    }

    fn close(&mut self) {
        println!("default close");
    }
    fn flush(&mut self) {
        println!("default flush");
    }
    fn batch(&mut self) {
        println!("default batch");
    }
}
