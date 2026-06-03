use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use std::process::Command;
use tokio::task;

const SMARTCTL: &str = "smartctl";

pub async fn prepare_smart_check()
-> Result<Vec<Result<DriveInfo, SmartCheckFailure>>, SmartCheckError> {
    let result = task::spawn_blocking(get_drives_info)
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))??;
    Ok(result)
}

fn get_drives_info() -> Result<Vec<Result<DriveInfo, SmartCheckFailure>>, SmartCheckError> {
    let drives = scan_drives()?;
    let mut results = Vec::new();
    for drive in drives.iter() {
        let result = get_drive_info(drive);
        match result {
            Ok(info) => results.push(Ok(info)),
            Err(e) => results.push(Err(SmartCheckFailure {
                drive_name: drive.clone(),
                message: e.to_string(),
            })),
        }
    }
    Ok(results)
}

pub async fn smart_check(drives: Vec<DriveInfo>)
-> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    task::spawn_blocking(|| scan_and_check_drives(drives))
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?
}

fn scan_and_check_drives(drives: Vec<DriveInfo>)
-> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    let mut results = Vec::new();
    // for drive in drives.iter() {
    //     let result = get_power_on_time(drive);
    //     match result {
    //         Ok(hours) => results.push(Ok(SmartCheckResult {
    //             drive_name: drive.clone(),
    //             power_on_time: hours,
    //         })),
    //         Err(e) => results.push(Err(SmartCheckFailure {
    //             message: e.to_string(),
    //         })),
    //     }
    // }
    Ok(results)
}

fn scan_drives() -> Result<Vec<String>, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--scan"])?;
    output
        .split(|&c| c == b'\n')
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            line.split_whitespace().next().map(String::from)
        })
        .map(Ok)
        .collect()
}

fn get_drive_info(drive_name: &str) -> Result<DriveInfo, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--json", "-a", drive_name])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: DriveInfo =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result)
}

fn execute_command(command: &str, arguments: &[&str]) -> Result<Vec<u8>, SmartCheckError> {
    let cmd_str = format!("{} {}", command, arguments.join(" "));
    let output = Command::new("sudo")
        .arg(command)
        .args(arguments)
        .output()
        .map_err(|e| SmartCheckError::CommandExecutionError(cmd_str.clone(), e.to_string()))?;

    // smartctl encodes SMART health flags in exit bits 2-7; only bits 0-1 are real failures
    let exit_code = output.status.code().unwrap_or(0);
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
