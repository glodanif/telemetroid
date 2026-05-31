use crate::smart_check::power_on_time::PowerOnTimeResult;
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
    let power_on_time: PowerOnTimeResult =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(power_on_time.hours)
}

fn execute_command(command: &str, arguments: &[&str]) -> Result<Vec<u8>, SmartCheckError> {
    let output = Command::new(command)
        .args(arguments)
        .output()
        .map_err(|e| {
            SmartCheckError::CommandExecutionError(
                format!("{} {}", command, arguments.join(" ")).to_string(),
                e.to_string(),
            )
        })?;

    if !output.status.success() {
        let error = SmartCheckError::CommandExecutionError(
            format!(
                "Failed to execute command {} {}",
                command,
                arguments.join(" ")
            )
            .to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        );
        return Err(error);
    }

    Ok(output.stdout)
}
