use crate::text_utils::{format_bytes, format_number};
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize)]
pub struct DriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub ata_smart_data: Option<AtaSmartData>,
    pub user_capacity: UserCapacity,
    pub device: Device,
}

impl DriveInfo {
    pub fn get_time_to_test(&self) -> f32 {
        if self.device.drive_type == "nvme" {
            1.5
        } else {
            match &self.ata_smart_data {
                Some(data) => data.self_test.polling_minutes.short as f32 * 1.5,
                None => 5.0,
            }
        }
    }
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
    #[serde(rename = "type")]
    pub drive_type: String,
    pub protocol: String,
}

#[derive(Debug, Deserialize)]
pub struct UserCapacity {
    pub bytes: u64,
}

impl Display for DriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "<b>{}</b>\n{} {} ({})\nTime to test: {} min",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name,
            format_number(self.get_time_to_test())
        )
    }
}
