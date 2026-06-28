use crate::command_runner::run_command;
use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::basic_device_info::BasicDeviceInfo;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::scan_result::ScanResult;
use crate::smart_check::smart_check_error::SmartCheckError;
use serde::de::DeserializeOwned;

const SMARTCTL: &str = "smartctl";

fn run_smartctl<T: DeserializeOwned>(args: &[&str]) -> Result<T, SmartCheckError> {
    let output = run_command(SMARTCTL, args, |code| code & 0b11 != 0)?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))
}

pub fn check_ata_drive(drive_name: &str) -> Result<AtaDriveInfo, SmartCheckError> {
    run_smartctl(&["--json", "-a", drive_name])
}

pub fn check_nvme_drive(drive_name: &str) -> Result<NvmeDriveInfo, SmartCheckError> {
    run_smartctl(&["--json", "-a", drive_name])
}

pub fn scan_drives() -> Result<Vec<BasicDeviceInfo>, SmartCheckError> {
    let result: ScanResult = run_smartctl(&["--scan", "--json"])?;
    Ok(result.devices)
}
