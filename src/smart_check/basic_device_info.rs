use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct BasicDeviceInfo {
    pub name: String,
    pub protocol: DeviceInterface,
}

#[derive(Debug)]
pub enum DeviceInterface {
    Ata,
    Nvme,
    Unsupported,
}

impl<'de> Deserialize<'de> for DeviceInterface {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.to_lowercase().as_str() {
            "scsi" => DeviceInterface::Ata,
            "nvme" => DeviceInterface::Nvme,
            _ => DeviceInterface::Unsupported,
        })
    }
}
