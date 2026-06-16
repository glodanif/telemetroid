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
    let name = drive.device.name.as_str();
    log::info!("Starting NVMe short self-test for {}", name);
    match launch_short_test(name) {
        Ok(()) => check_progress(drive),
        Err(e) => {
            log::error!("Failed to launch NVMe self-test for {}: {}", name, e);
            Err(e)
        }
    }
}

fn check_progress(drive: &NvmeDriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let mut time_passed = Duration::from_secs(0);
    let mut consecutive_errors = 0;
    let mut last_progress: Option<u8> = None;
    loop {
        match get_nvme_drive_info(drive.device.name.as_str()) {
            Ok(info) => {
                consecutive_errors = 0;
                if info.is_running() {
                    last_progress = info.nvme_self_test_log.current_self_test_completion_percent;
                    log::info!(
                        "NVMe self-test for {} in progress: {}%",
                        drive.device.name,
                        last_progress.unwrap_or(0)
                    );
                } else {
                    let new_result = if info.table_snapshot() != drive.table_snapshot() {
                        info.latest_result()
                    } else {
                        None
                    };
                    return match new_result {
                        None => {
                            log::warn!(
                                "No new self-test entry found for {} after test finished",
                                drive.device.name
                            );
                            Err(SmartCheckFailure::FailedToStart(
                                drive.device.name.clone(),
                                "Test failed to start".to_string(),
                            ))
                        }
                        Some(log_entry) => {
                            let passed = log_entry.has_completed_without_error();
                            log::info!(
                                "NVMe self-test for {} completed: {} ({})",
                                drive.device.name,
                                if passed { "passed" } else { "failed" },
                                log_entry.self_test_result.string
                            );
                            Ok(SmartCheckResult {
                                drive_name: drive.device.name.clone(),
                                power_on_time: info.power_on_time.hours,
                                passed,
                            })
                        }
                    };
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    log::error!(
                        "Repeated poll failures for {} ({} consecutive): {}",
                        drive.device.name,
                        consecutive_errors,
                        e
                    );
                    return Err(SmartCheckFailure::DeviceUnavailable(
                        drive.device.name.clone(),
                        format!("Repeated poll failures: {}", e),
                    ));
                }
                log::warn!(
                    "Failed to get drive info for {} (attempt {}/{}): {}",
                    drive.device.name,
                    consecutive_errors,
                    MAX_CONSECUTIVE_ERRORS,
                    e
                );
            }
        }

        if time_passed >= TIMEOUT {
            log::error!(
                "Timeout waiting for {} self-test after {}s (last progress: {:?})",
                drive.device.name,
                time_passed.as_secs(),
                last_progress
            );
            return Err(SmartCheckFailure::TestNotLogged(
                drive.device.name.clone(),
                match last_progress {
                    Some(percent) => format!("Timeout (last progress: {}%)", percent),
                    None => "Timeout (no progress reported)".to_string(),
                },
            ));
        }
        sleep(SECONDS_BETWEEN_PROGRESS_CHECKS);
        time_passed += SECONDS_BETWEEN_PROGRESS_CHECKS;
    }
}
