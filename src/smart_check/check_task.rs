use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::SmartCheckResult;
use std::process::Command;
use tokio::task;

const SMARTCTL: &str = "smartctl";

pub async fn smart_check() -> Result<Vec<SmartCheckResult>, SmartCheckError> {
    task::spawn_blocking(scan_and_check_drives)
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?
}

fn scan_and_check_drives() -> Result<Vec<SmartCheckResult>, SmartCheckError> {
    let drives = scan_drives()?;
    drives
        .into_iter()
        .map(|d| Ok(SmartCheckResult { drive_name: d }))
        .collect()
}

fn scan_drives() -> Result<Vec<String>, SmartCheckError> {
    let output = Command::new(SMARTCTL)
        .args(&["--scan"])
        .output()
        .map_err(|e| {
            SmartCheckError::CommandExecutionError("SMARTCTL --scan".to_string(), e.to_string())
        })?;

    if !output.status.success() {
        return Err(SmartCheckError::CommandExecutionError(
            "SMARTCTL --scan".to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    output
        .stdout
        .split(|&c| c == b'\n')
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            line.split_whitespace().next().map(String::from)
        })
        .map(Ok)
        .collect()
}
