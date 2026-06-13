use crate::smart_check::basic_device_info::DeviceInterface;
use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::drives::Drives;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{
    get_ata_drive_info, get_nvme_drive_info, scan_drives,
};
use tokio::task;

pub async fn prepare_smart_check() -> Result<Drives, SmartCheckError> {
    let result = task::spawn_blocking(get_drives_info)
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))??;
    Ok(result)
}

fn get_drives_info() -> Result<Drives, SmartCheckError> {
    let drives = scan_drives()?;
    let mut ata_results = Vec::new();
    let mut nvme_results = Vec::new();
    for drive in drives.iter() {
        match drive.interface {
            DeviceInterface::Ata => {
                let result = get_ata_drive_info(drive.name.as_str());
                match result {
                    Ok(info) => ata_results.push(Ok(info)),
                    Err(e) => ata_results.push(Err(SmartCheckFailure {
                        drive_name: drive.name.clone(),
                        message: e.to_string(),
                    })),
                }
            }
            DeviceInterface::Nvme => {
                let result = get_nvme_drive_info(drive.name.as_str());
                match result {
                    Ok(info) => nvme_results.push(Ok(info)),
                    Err(e) => nvme_results.push(Err(SmartCheckFailure {
                        drive_name: drive.name.clone(),
                        message: e.to_string(),
                    })),
                }
            }
            DeviceInterface::Unsupported => {}
        }
    }
    Ok(Drives {
        ata: ata_results,
        nvme: nvme_results,
    })
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
