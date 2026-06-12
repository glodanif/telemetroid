use crate::smart_check::drive_info::DriveInfo;
use crate::smart_check::smart_check_result::{SmartCheckFailure, SmartCheckResult};
use crate::smart_check::smart_ctl_interface::{get_drive_info, launch_short_test};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(300);

pub fn start_shot_ata_self_test(drive: &DriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let result = launch_short_test(drive.device.name.as_str());
    match result {
        Ok(()) => wait_for_result(drive),
        Err(e) => Err(e),
    }
}

fn wait_for_result(drive: &DriveInfo) -> Result<SmartCheckResult, SmartCheckFailure> {
    let check_result = get_drive_info(drive.device.name.as_str());
    match check_result {
        Ok(info) => {}
        Err(e) => {
            eprintln!("Failed to get drive info: {}", e);
        }
    }
    todo!()
}
