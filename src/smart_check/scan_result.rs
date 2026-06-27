use serde::Deserialize;
use crate::smart_check::basic_device_info::BasicDeviceInfo;

#[derive(Debug, Deserialize)]
pub struct ScanResult {
    pub devices: Vec<BasicDeviceInfo>,
}
