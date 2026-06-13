use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_drive_info, scan_drives};
use tokio::task;

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
        let result = get_drive_info(drive.name.as_str());
        match result {
            Ok(info) => results.push(Ok(info)),
            Err(e) => results.push(Err(SmartCheckFailure {
                drive_name: drive.name.clone(),
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
    for drive in drives.iter() {}
    Ok(results)
}
