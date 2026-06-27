use crate::smart_check::common::device_info_parts::{
    Device, PowerOnTime, SmartStatus, Temperature, UserCapacity,
};
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
    pub smart_status: SmartStatus,
    pub temperature: Temperature,
    pub nvme_smart_health_information_log: NvmeHealth,
    pub nvme_self_test_log: NvmeSelfTestLog,
}

#[derive(Debug, Deserialize)]
pub struct NvmeHealth {
    pub critical_warning: u8,
    pub percentage_used: u8,
    pub media_errors: u64,
    pub power_cycles: u64,
}

impl NvmeDriveInfo {
    pub fn is_running(&self) -> bool {
        self.nvme_self_test_log.current_self_test_operation.string == TEST_IN_PROGRESS
    }

    pub fn table_snapshot(&self) -> Vec<(u64, u64, u64)> {
        self.nvme_self_test_log
            .table
            .as_ref()
            .map(|table| {
                table
                    .iter()
                    .map(|entry| {
                        (
                            entry.power_on_hours,
                            entry.self_test_result.value,
                            entry.self_test_code.value,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn latest_result(&self) -> Option<&LogEntry> {
        self.nvme_self_test_log
            .table
            .as_ref()
            .and_then(|table| table.first())
    }
}

impl Display for NvmeDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let health = &self.nvme_smart_health_information_log;
        let status = if self.smart_status.passed && health.critical_warning == 0 {
            "✅ Healthy"
        } else {
            "❌ FAILING"
        };

        write!(
            f,
            "<b>{}</b>\n{} {} ({})\n{} · {}°C · wear {}%\n{} h · {} cycles · {} media errors",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name,
            status,
            self.temperature.current,
            health.percentage_used,
            self.power_on_time.hours,
            health.power_cycles,
            health.media_errors
        )?;

        match self.latest_result() {
            Some(result) => write!(
                f,
                "\nLast self-test: {} ({} h)",
                result.self_test_result.string, result.power_on_hours
            ),
            None => write!(f, "\nLast self-test: none recorded"),
        }
    }
}
