use crate::text_utils::{format_bytes, format_number};
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize)]
pub struct DriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub ata_smart_data: AtaSmartData,
    pub user_capacity: UserCapacity,
    pub device: Device,
}

#[derive(Debug, Deserialize)]
pub struct PowerOnTime {
    pub hours: u32,
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

#[derive(Debug, Deserialize)]
pub struct Device {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UserCapacity {
    pub bytes: u64,
}

impl Display for DriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "<b>{}</b>\n{} ({})\nTime to test: {} min",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.name,
            format_number(self.ata_smart_data.self_test.polling_minutes.short as f32 * 1.5)
        )
    }
}
