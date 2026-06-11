use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use std::thread::sleep;
use std::time::Duration;
use tokio::task;
use crate::smart_check::smart_ctl_interface::{get_drive_info, launch_short_test, scan_drives};

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


