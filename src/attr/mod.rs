use crate::SensorType;
use crate::pb::sns_std_sensor::*;
use strum_macros::{Display, EnumIter};

#[derive(Debug, Clone, PartialEq, Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum SensorAttr {
    Uuid(u128),
    Name(String),
    Vendor(String),
    Type(SensorType),
    Available(bool),
    Version(u32),
    Api(Vec<String>),
    Rates(Vec<f32>),
    Resolutions(Vec<f32>),
    FifoSize(u32),
    ActiveCurrent(Vec<u32>),
    SleepCurrent(u32),
    Ranges(Vec<(f32, f32)>),
    OpModes(Vec<String>),
    Dri(bool),
    StreamSync(bool),
    EventSize(u32),
    StreamType(SnsStdSensorStreamType),
    Dynamic(bool),
    HwId(u32),
    RigidBodyType(SnsStdSensorRigidBodyType),
    Placement(Vec<f32>),
    PhysicalSensor(bool),
}

macro_rules! attr_id_mapping {
    ($($attr:ident => $id:ident),* $(,)?) => {
        impl SensorAttr {
            pub fn id(&self) -> SnsStdSensorAttrId {
                match self {
                    $(SensorAttr::$attr(_) => SnsStdSensorAttrId::$id,)*
                }
            }
        }
    };
}

attr_id_mapping! {
    Uuid => SnsStdSensorAttridSuid,
    Name => SnsStdSensorAttridName,
    Vendor => SnsStdSensorAttridVendor,
    Type => SnsStdSensorAttridType,
    Available => SnsStdSensorAttridAvailable,
    Version => SnsStdSensorAttridVersion,
    Api => SnsStdSensorAttridApi,
    Rates => SnsStdSensorAttridRates,
    Resolutions => SnsStdSensorAttridResolutions,
    FifoSize => SnsStdSensorAttridFifoSize,
    ActiveCurrent => SnsStdSensorAttridActiveCurrent,
    SleepCurrent => SnsStdSensorAttridSleepCurrent,
    Ranges => SnsStdSensorAttridRanges,
    OpModes => SnsStdSensorAttridOpModes,
    Dri => SnsStdSensorAttridDri,
    StreamSync => SnsStdSensorAttridStreamSync,
    EventSize => SnsStdSensorAttridEventSize,
    StreamType => SnsStdSensorAttridStreamType,
    Dynamic => SnsStdSensorAttridDynamic,
    HwId => SnsStdSensorAttridHwId,
    RigidBodyType => SnsStdSensorAttridRigidBody,
    Placement => SnsStdSensorAttridPlacement,
    PhysicalSensor => SnsStdSensorAttridPhysicalSensor,
}

impl From<SensorAttr> for SnsStdSensorAttrId {
    fn from(attr: SensorAttr) -> Self {
        attr.id()
    }
}
