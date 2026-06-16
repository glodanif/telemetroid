use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_ata_drive_info, launch_short_test};
use std::thread::sleep;
use std::time::Duration;

const GRACE_PERIOD_MULTIPLIER: f32 = 0.3;

pub fn start_shot_ata_self_test(
    drive: &AtaDriveInfo,
) -> Result<SmartCheckResult, SmartCheckFailure> {
    let name = drive.device.name.as_str();
    log::info!("Starting ATA short self-test for {}", name);
    match launch_short_test(name) {
        Ok(()) => wait_for_result(drive),
        Err(e) => {
            log::error!("Failed to launch ATA self-test for {}: {}", name, e);
            Err(e)
        }
    }
}

fn wait_for_result(drive: &AtaDriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let estimated_duration = Duration::from_mins(drive.ata_smart_data.get_polling_minutes());
    log::info!(
        "ATA self-test launched for {}, waiting estimated {}s for completion",
        drive.device.name,
        estimated_duration.as_secs()
    );
    match wait_and_check_result(drive, estimated_duration) {
        Ok(r) => Ok(r),
        Err(e) => {
            let grace_period =
                (estimated_duration.as_secs() as f32 * GRACE_PERIOD_MULTIPLIER) as u64;
            log::warn!(
                "First check for {} failed ({}), retrying after grace period of {}s",
                drive.device.name,
                e,
                grace_period
            );
            wait_and_check_result(drive, Duration::from_secs(grace_period))
        }
    }
}

fn wait_and_check_result(
    drive: &AtaDriveInfo,
    wait_duration: Duration,
) -> Result<SmartCheckResult, SmartCheckFailure> {
    log::debug!(
        "Sleeping {}s before checking self-test result for {}",
        wait_duration.as_secs(),
        drive.device.name
    );
    sleep(wait_duration);
    match get_ata_drive_info(drive.device.name.as_str()) {
        Ok(info) => {
            let new_result = if info.table_snapshot() != drive.table_snapshot() {
                info.latest_result()
            } else {
                None
            };
            match new_result {
                None => {
                    log::warn!(
                        "No new self-test entry found for {} after waiting",
                        drive.device.name
                    );
                    Err(SmartCheckFailure::FailedToStart(
                        drive.device.name.clone(),
                        "Test failed to start".to_string(),
                    ))
                }
                Some(log_entry) => {
                    log::info!(
                        "ATA self-test for {} completed: {} ({})",
                        drive.device.name,
                        if log_entry.status.passed {
                            "passed"
                        } else {
                            "failed"
                        },
                        log_entry.status.string
                    );
                    Ok(SmartCheckResult {
                        drive_name: drive.device.name.clone(),
                        power_on_time: info.power_on_time.hours,
                        passed: log_entry.status.passed,
                    })
                }
            }
        }
        Err(e) => {
            log::error!(
                "Failed to read drive info for {}: {}",
                drive.device.name,
                e
            );
            Err(SmartCheckFailure::DeviceUnavailable(
                drive.device.name.clone(),
                e.to_string(),
            ))
        }
    }
}
