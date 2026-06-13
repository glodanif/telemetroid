use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PowerOnTime {
    pub hours: u32,
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
