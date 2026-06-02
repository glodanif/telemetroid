use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize)]
pub struct DriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub ata_smart_data: AtaSmartData,
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

impl Display for DriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} ({})\nTime to test: {:.1} min",
            self.model_name,
            self.device.name,
            self.ata_smart_data.self_test.polling_minutes.short as f32 * 1.5
        )
    }
}
