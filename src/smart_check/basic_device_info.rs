pub struct BasicDeviceInfo {
    pub name: String,
    pub interface: DeviceInterface,
}

pub enum DeviceInterface {
    Sata,
    Nvme,
    Unsupported,
}

impl DeviceInterface {
    pub fn from(string: &str) -> Self {
        match string {
            "sata" => DeviceInterface::Sata,
            "nvme" => DeviceInterface::Nvme,
            _ => DeviceInterface::Unsupported,
        }
    }
}
