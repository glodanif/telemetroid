use crate::smart_check::common::device_info_parts::{Device, PowerOnTime, UserCapacity};
use crate::text_utils::format_bytes;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize)]
pub struct AtaDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub user_capacity: UserCapacity,
    pub ata_smart_data: AtaSmartData,
    pub device: Device,
}

#[derive(Debug, Deserialize)]
pub struct AtaSmartData {
    pub self_test: SelfTest,
}

#[derive(Debug, Deserialize)]
pub struct SelfTest {
    pub polling_minutes: PollingMinutes,
}

#[derive(Debug, Deserialize)]
pub struct PollingMinutes {
    pub short: u32,
}

impl Display for AtaDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "<b>{}</b>\n{} ({})",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.name
        )
    }
}
