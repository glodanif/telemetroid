use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::common::device_info_parts::{Device, PowerOnTime, UserCapacity};
use crate::text_utils::format_bytes;
use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize)]
pub struct NvmeDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub user_capacity: UserCapacity,
    pub device: Device,
}

impl Display for NvmeDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<b>{}</b>\n{} {} ({})",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name
        )
    }
}
