use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
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

pub async fn smart_check(
    drives: Vec<DriveInfo>,
) -> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    task::spawn_blocking(|| scan_and_check_drives(drives))
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?
}

fn scan_and_check_drives(
    drives: Vec<DriveInfo>,
) -> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    let mut results = Vec::new();
    for drive in drives.iter() {
        let result = launch_short_test(&drive.device.name);
        match result {
            Ok(_) => {
                let _ = sleep(Duration::from_secs(drive.get_time_to_test_secs()));
                let test_result = get_drive_info(&drive.device.name);
                match test_result {
                    Ok(r) => {
                        if r.power_on_time.hours == drive.power_on_time.hours {
                            results.push(Ok(SmartCheckResult {
                                drive_name: drive.device.name.clone(),
                                power_on_time: r.power_on_time.hours,
                                passed: r.smart_status.passed,
                            }));
                        }
                    }
                    Err(e) => {
                        results.push(Err(SmartCheckFailure {
                            drive_name: drive.device.name.clone(),
                            message: e.to_string(),
                        }));
                    }
                }
            }
            Err(e) => results.push(Err(e)),
        }
    }
    Ok(results)
}

fn launch_short_test(drive_name: &str) -> Result<(), SmartCheckFailure> {
    let _ =
        execute_command(SMARTCTL, &["-t", "short", drive_name]).map_err(|e| SmartCheckFailure {
            drive_name: drive_name.to_string(),
            message: e.to_string(),
        });
    Ok(())
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
