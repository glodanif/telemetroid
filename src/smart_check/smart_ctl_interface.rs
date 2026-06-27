use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::basic_device_info::BasicDeviceInfo;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use std::process::Command;
use crate::smart_check::scan_result::ScanResult;

const SMARTCTL: &str = "smartctl";

pub fn get_ata_drive_info(drive_name: &str) -> Result<AtaDriveInfo, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--json", "-a", drive_name])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: AtaDriveInfo =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result)
}

pub fn get_nvme_drive_info(drive_name: &str) -> Result<NvmeDriveInfo, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--json", "-a", drive_name])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: NvmeDriveInfo =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result)
}

pub fn scan_drives() -> Result<Vec<BasicDeviceInfo>, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--scan"])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: ScanResult =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result.devices)
}

fn execute_command(command: &str, arguments: &[&str]) -> Result<Vec<u8>, SmartCheckError> {
    let cmd_str = format!("{} {}", command, arguments.join(" "));
    let output = Command::new("sudo")
        .arg(command)
        .args(arguments)
        .output()
        .map_err(|e| SmartCheckError::CommandExecutionError(cmd_str.clone(), e.to_string()))?;

    // smartctl encodes SMART health flags in exit bits 2-7; only bits 0-1 are real failures
    let exit_code = match output.status.code() {
        Some(code) => code,
        None => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let detail = if stderr.is_empty() {
                "process terminated by signal".to_string()
            } else {
                stderr
            };
            return Err(SmartCheckError::CommandExecutionError(cmd_str, detail));
        }
    };
    if exit_code & 0b11 != 0 {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let detail = if stderr.is_empty() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            stderr
        };
        return Err(SmartCheckError::CommandExecutionError(cmd_str, detail));
    }

    Ok(output.stdout)
}
