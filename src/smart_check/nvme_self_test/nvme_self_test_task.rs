use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_nvme_drive_info, launch_short_test};
use std::thread::sleep;
use std::time::Duration;

const SECONDS_BETWEEN_PROGRESS_CHECKS: Duration = Duration::from_secs(15);
const TIMEOUT: Duration = Duration::from_secs(300);
const MAX_CONSECUTIVE_ERRORS: u32 = 3;

pub fn start_shot_nvme_self_test(
    drive: &NvmeDriveInfo,
) -> Result<SmartCheckResult, SmartCheckFailure> {
    let result = launch_short_test(drive.device.name.as_str());
    match result {
        Ok(()) => check_progress(drive),
        Err(e) => Err(e),
    }
}

fn check_progress(drive: &NvmeDriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let mut time_passed = Duration::from_secs(0);
    let mut consecutive_errors = 0;
    loop {
        let progress_check_result = get_nvme_drive_info(drive.device.name.as_str());
        match progress_check_result {
            Ok(info) => {
                consecutive_errors = 0;
                if info.is_running() {
                    println!(
                        "Test for {} is in progress: {}%",
                        drive.device.name
                        info.nvme_self_test_log.current_self_test_completion_percent.unwrap_or(0),
                    )
                } else {
                    let result = info
                        .get_result_by_hour(drive.power_on_time.hours)
                        .or_else(|| info.get_result_by_hour(info.power_on_time.hours));
                    return match result {
                        None => Err(SmartCheckFailure {
                            drive_name: drive.device.name.clone(),
                            message: "Test failed to start".to_string(),
                        }),
                        Some(log) => Ok(SmartCheckResult {
                            drive_name: drive.device.name.clone(),
                            power_on_time: info.power_on_time.hours,
                            passed: log.has_completed_without_error(),
                        }),
                    };
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return Err(SmartCheckFailure {
                        drive_name: drive.device.name.clone(),
                        message: format!("Repeated poll failures: {}", e),
                    });
                }
                eprintln!("Failed to get drive info: {}", e);
            }
        }

        if time_passed >= TIMEOUT {
            return Err(SmartCheckFailure {
                drive_name: drive.device.name.clone(),
                message: "Timeout".to_string(),
            });
        }
        sleep(SECONDS_BETWEEN_PROGRESS_CHECKS);
        time_passed += SECONDS_BETWEEN_PROGRESS_CHECKS;
    }
}
