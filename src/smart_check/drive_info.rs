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

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    const PB: u64 = TB * 1024;

    match bytes {
        b if b >= PB => format!("{:.2} PB", b as f64 / PB as f64),
        b if b >= TB => format!("{:.2} TB", b as f64 / TB as f64),
        b if b >= GB => format!("{:.2} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KB", b as f64 / KB as f64),
        b => format!("{} B", b),
    }
}

pub fn format_number(n: f32) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as u32)
    } else {
        format!("{:.1}", n)
    }
}
