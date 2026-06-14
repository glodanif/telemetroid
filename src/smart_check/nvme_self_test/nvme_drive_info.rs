use crate::smart_check::common::device_info_parts::{Device, PowerOnTime, UserCapacity};
use crate::smart_check::nvme_self_test::nvme_self_test_log::{LogEntry, NvmeSelfTestLog};
use crate::text_utils::format_bytes;
use serde::Deserialize;
use std::fmt::{Display, Formatter};

const TEST_IN_PROGRESS: &str = "Short self-test in progress";

#[derive(Debug, Deserialize)]
pub struct NvmeDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub user_capacity: UserCapacity,
    pub device: Device,
    pub nvme_self_test_log: NvmeSelfTestLog,
}

impl NvmeDriveInfo {
    pub fn is_running(&self) -> bool {
        self.nvme_self_test_log.current_self_test_operation.string == TEST_IN_PROGRESS
            && self
                .nvme_self_test_log
                .current_self_test_completion_percent
                .is_some()
    }

    pub fn get_result_by_hour(&self, power_on_hours: u64) -> Option<&LogEntry> {
        self.nvme_self_test_log.table.as_ref().and_then(|table| {
            table
                .iter()
                .find(|entry| entry.power_on_hours == power_on_hours)
        })
    }
}

impl Display for NvmeDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<b>{}</b>\n{} {} ({})",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name
        )
    }
}
