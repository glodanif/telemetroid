use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_ata_drive_info, launch_short_test};
use std::thread::sleep;
use std::time::Duration;

const GRACE_PERIOD_MULTIPLIER: f32 = 0.3;

pub fn start_shot_ata_self_test(
    drive: &AtaDriveInfo,
) -> Result<SmartCheckResult, SmartCheckFailure> {
    let result = launch_short_test(drive.device.name.as_str());
    match result {
        Ok(()) => wait_for_result(drive),
        Err(e) => Err(e),
    }
}

fn wait_for_result(drive: &AtaDriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let estimated_duration = Duration::from_mins(drive.ata_smart_data.get_polling_minutes());
    let result = wait_and_check_result(drive, estimated_duration);
    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            let grace_period =
                (estimated_duration.as_secs() as f32 * GRACE_PERIOD_MULTIPLIER) as u64;
            wait_and_check_result(drive, Duration::from_secs(grace_period))
        }
    }
}

fn wait_and_check_result(
    drive: &AtaDriveInfo,
    wait_duration: Duration,
) -> Result<SmartCheckResult, SmartCheckFailure> {
    sleep(wait_duration);
    let check_result = get_ata_drive_info(drive.device.name.as_str());
    match check_result {
        Ok(info) => {
            let result = info
                .get_result_by_hour(drive.power_on_time.hours)
                .or_else(|| info.get_result_by_hour(info.power_on_time.hours));
            match result {
                None => Err(SmartCheckFailure {
                    drive_name: drive.device.name.clone(),
                    message: "Test failed to start".to_string(),
                }),
                Some(log) => Ok(SmartCheckResult {
                    drive_name: drive.device.name.clone(),
                    power_on_time: info.power_on_time.hours,
                    passed: log.status.passed,
                }),
            }
        }
        Err(e) => Err(SmartCheckFailure {
            drive_name: drive.device.name.clone(),
            message: e.to_string(),
        }),
    }
}
