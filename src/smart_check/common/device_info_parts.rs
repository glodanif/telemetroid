use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PowerOnTime {
    pub hours: u64,
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

#[derive(Debug, Deserialize)]
pub struct SmartStatus {
    pub passed: bool,
}

#[derive(Debug, Deserialize)]
pub struct Temperature {
    pub current: i64,
}

#[derive(Debug, Deserialize)]
pub struct EnduranceUsed {
    pub current_percent: u8,
}
