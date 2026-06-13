pub struct BasicDeviceInfo {
    pub name: String,
    pub interface: DeviceInterface,
}

pub enum DeviceInterface {
    Ata,
    Nvme,
    Unsupported,
}

impl DeviceInterface {
    pub fn from(string: &str) -> Self {
        match string {
            "scsi" => DeviceInterface::Ata,
            "nvme" => DeviceInterface::Nvme,
            _ => DeviceInterface::Unsupported,
        }
    }
}
