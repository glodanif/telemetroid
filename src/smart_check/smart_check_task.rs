use crate::smart_check::basic_device_info::DeviceInterface;
use crate::smart_check::drives::Drives;
use crate::smart_check::nvme_self_test::nvme_self_test_task::start_shot_nvme_self_test;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{
    get_ata_drive_info, get_nvme_drive_info, scan_drives,
};
use tokio::task;
use crate::smart_check::ata_self_test::ata_self_test_task::start_shot_ata_self_test;

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
    drives: Drives,
) -> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    task::spawn_blocking(|| scan_and_check_drives(drives))
        .await
        .map_err(|e| SmartCheckError::SpawnError(e.to_string()))?
}

fn scan_and_check_drives(
    drives: Drives,
) -> Result<Vec<Result<SmartCheckResult, SmartCheckFailure>>, SmartCheckError> {
    let mut results = Vec::new();
    for drive in drives.nvme.iter() {
        if let Ok(drive) = drive {
            let result = start_shot_nvme_self_test(drive);
            results.push(result);
        }
    }
    for drive in drives.ata.iter() {
        if let Ok(drive) = drive {
            let result = start_shot_ata_self_test(drive);
            results.push(result);
        }
    }
    Ok(results)
}
