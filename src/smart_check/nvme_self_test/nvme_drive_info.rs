use crate::smart_check::common::device_info_parts::{Device, PowerOnTime, UserCapacity};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NvmeDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub user_capacity: UserCapacity,
    pub device: Device,
}
