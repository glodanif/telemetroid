use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SmartctlJson {
    pub power_on_time: PowerOnTime,
}

#[derive(Debug, Deserialize)]
pub struct PowerOnTime {
    pub hours: u32,
}
