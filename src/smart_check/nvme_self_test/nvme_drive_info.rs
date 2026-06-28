use crate::smart_check::common::device_info_parts::{
    Device, PowerOnTime, SmartStatus, Temperature, UserCapacity,
};
use crate::smart_check::nvme_self_test::nvme_self_test_log::{LogEntry, NvmeSelfTestLog};
use crate::text_utils::{format_bytes, pluralize};
use serde::Deserialize;
use std::fmt::{Display, Formatter};

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
    pub available_spare: u8,
    pub available_spare_threshold: u8,
    pub media_errors: u64,
    pub power_cycles: u64,
}

impl NvmeDriveInfo {
    /// True while any self-test (short or extended) is executing. A `current_self_test_operation`
    /// value of 0 means "No self-test in progress".
    pub fn is_running(&self) -> bool {
        self.nvme_self_test_log.current_self_test_operation.value != 0
    }

    /// Identity of each logged self-test, as (power_on_hours, result value). Used to detect
    /// that a freshly launched test has produced a new log entry.
    pub fn table_snapshot(&self) -> Vec<(u64, u64)> {
        self.nvme_self_test_log
            .table
            .as_ref()
            .map(|table| {
                table
                    .iter()
                    .map(|entry| (entry.power_on_hours, entry.self_test_result.value))
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

    /// Pre-failure conditions worth surfacing, formatted for display.
    pub fn warnings(&self) -> Vec<String> {
        let health = &self.nvme_smart_health_information_log;
        let mut warnings = Vec::new();
        if health.available_spare < health.available_spare_threshold {
            warnings.push(format!(
                "Spare {}% (below {}% threshold)",
                health.available_spare, health.available_spare_threshold
            ));
        }
        warnings
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
            "<b>{}</b>\n{} {} ({})\n{} · {}°C · wear {}%\n{} h · {} · {}",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name,
            status,
            self.temperature.current,
            health.percentage_used,
            self.power_on_time.hours,
            pluralize(health.power_cycles, "cycle"),
            pluralize(health.media_errors, "media error")
        )?;

        match self.latest_result() {
            Some(result) => write!(
                f,
                "\nLast self-test: {} ({} h)",
                result.self_test_result.string, result.power_on_hours
            )?,
            None => write!(f, "\nLast self-test: none recorded")?,
        }

        let warnings = self.warnings();
        if !warnings.is_empty() {
            write!(f, "\n⚠️ {}", warnings.join(" · "))?;
        }

        Ok(())
    }
}
