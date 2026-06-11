use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::SmartCheckFailure;
use std::process::Command;

const SMARTCTL: &str = "smartctl";

pub fn get_drive_info(drive_name: &str) -> Result<DriveInfo, SmartCheckError> {
    let output = execute_command(SMARTCTL, &["--json", "-a", drive_name])?;
    let json_str =
        String::from_utf8(output).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    let result: DriveInfo =
        serde_json::from_str(&json_str).map_err(|e| SmartCheckError::FormatError(e.to_string()))?;
    Ok(result)
}

pub fn scan_drives() -> Result<Vec<String>, SmartCheckError> {
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

pub fn launch_short_test(drive_name: &str) -> Result<(), SmartCheckFailure> {
    let _ =
        execute_command(SMARTCTL, &["-t", "short", drive_name]).map_err(|e| SmartCheckFailure {
            drive_name: drive_name.to_string(),
            message: e.to_string(),
        });
    Ok(())
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
