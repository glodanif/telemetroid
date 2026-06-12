use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_drive_info, launch_short_test};
use std::thread::sleep;
use std::time::Duration;

const SECONDS_BETWEEN_PROGRESS_CHECKS: Duration = Duration::from_secs(15);
const TIMEOUT: Duration = Duration::from_secs(300);

pub fn start_shot_nvme_self_test(drive: &DriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let result = launch_short_test(drive.device.name.as_str());
    match result {
        Ok(()) => check_progress(drive),
        Err(e) => Err(e),
    }
}

fn check_progress(drive: &DriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let mut time_passed = Duration::from_secs(0);
    loop {
        sleep(SECONDS_BETWEEN_PROGRESS_CHECKS);
        time_passed += SECONDS_BETWEEN_PROGRESS_CHECKS;
        if time_passed >= TIMEOUT {
            return Err(SmartCheckFailure {
                drive_name: drive.device.name.clone(),
                message: "Timeout".to_string(),
            });
        }
        let progress_check_result = get_drive_info(drive.device.name.as_str());
        match progress_check_result {
            Ok(info) => {

            }
            Err(e) => {
                eprintln!("Failed to get drive info: {}", e);
            }
        }
    }
}
