use crate::smart_check::power_on_time::SmartctlJson;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use std::process::Command;
use tokio::task;

const SMARTCTL: &str = "smartctl";

pub async fn smart_check()
-> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    task::spawn_blocking(scan_and_check_drives)
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?
}

fn scan_and_check_drives()
-> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    let drives = scan_drives()?;
    let mut results = Vec::new();
    for drive in drives.iter() {
        let result = get_power_on_time(drive);
        match result {
            Ok(hours) => results.push(Ok(SmartCheckResult {
                drive_name: drive.clone(),
                power_on_time: hours,
            })),
            Err(e) => results.push(Err(SmartCheckFailure {
                message: e.to_string(),
            })),
        }
    }
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

fn get_power_on_time(drive_name: &str) -> Result<u32, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--json", "-a", drive_name])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: SmartctlJson =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result.power_on_time.hours)
}

fn execute_command(command: &str, arguments: &[&str]) -> Result<Vec<u8>, SmartCheckError> {
    let cmd_str = format!("{} {}", command, arguments.join(" "));
    let output = Command::new(command)
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
