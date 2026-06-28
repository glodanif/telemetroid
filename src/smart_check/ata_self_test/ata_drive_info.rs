use crate::smart_check::ata_self_test::ata_self_test_log::{AtaSmartSelfTestLog, LogEntry};
use crate::smart_check::common::device_info_parts::{
    Device, EnduranceUsed, PowerOnTime, SmartStatus, Temperature, UserCapacity,
};
use crate::text_utils::{format_bytes, pluralize};
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

// SMART attributes that predict drive failure (Backblaze canary set), as
// (id, label). A non-zero raw value on any of these warrants attention.
const CANARY_ATTRIBUTES: &[(u32, &str)] = &[
    (5, "Reallocated"),
    (197, "Pending"),
    (198, "Uncorrectable"),
    (187, "Reported uncorrect"),
    (188, "Command timeout"),
];

#[derive(Debug, Deserialize)]
pub struct AtaDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub power_cycle_count: u64,
    pub user_capacity: UserCapacity,
    pub smart_status: SmartStatus,
    pub temperature: Temperature,
    pub endurance_used: Option<EnduranceUsed>,
    pub ata_smart_data: AtaSmartData,
    pub ata_smart_attributes: Option<AtaSmartAttributes>,
    pub device: Device,
    pub ata_smart_self_test_log: AtaSmartSelfTestLog,
}

impl AtaDriveInfo {
    pub fn latest_result(&self) -> Option<&LogEntry> {
        self.ata_smart_self_test_log
            .standard
            .table
            .as_ref()
            .and_then(|table| table.first())
    }

    /// True while a self-test is executing (smartctl reports a remaining percentage).
    pub fn is_running(&self) -> bool {
        self.ata_smart_data.self_test.status.remaining_percent.is_some()
    }

    /// Identity of each logged self-test, as (lifetime_hours, status value). Used to
    /// detect that a freshly launched test has produced a new log entry.
    pub fn table_snapshot(&self) -> Vec<(u64, u64)> {
        self.ata_smart_self_test_log
            .standard
            .table
            .as_ref()
            .map(|table| {
                table
                    .iter()
                    .map(|entry| (entry.lifetime_hours, entry.status.value as u64))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Non-zero canary attributes, formatted as "<label> <count>".
    pub fn warnings(&self) -> Vec<String> {
        let table = match self.ata_smart_attributes.as_ref().and_then(|a| a.table.as_ref()) {
            Some(table) => table,
            None => return Vec::new(),
        };
        CANARY_ATTRIBUTES
            .iter()
            .filter_map(|(id, label)| {
                table
                    .iter()
                    .find(|attr| attr.id == *id)
                    .filter(|attr| attr.raw.value > 0)
                    .map(|attr| format!("{} {}", label, attr.raw.value))
            })
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct AtaSmartAttributes {
    pub table: Option<Vec<SmartAttribute>>,
}

#[derive(Debug, Deserialize)]
pub struct SmartAttribute {
    pub id: u32,
    pub raw: AttributeRaw,
}

#[derive(Debug, Deserialize)]
pub struct AttributeRaw {
    pub value: u64,
}

#[derive(Debug, Deserialize)]
pub struct AtaSmartData {
    pub self_test: SelfTest,
}

impl AtaSmartData {
    pub fn get_polling_minutes(&self) -> u64 {
        self.self_test.polling_minutes.short
    }
}

#[derive(Debug, Deserialize)]
pub struct SelfTest {
    pub status: SelfTestStatus,
    pub polling_minutes: PollingMinutes,
}

#[derive(Debug, Deserialize)]
pub struct SelfTestStatus {
    // Present only while a self-test is in progress; absent once it has finished.
    pub remaining_percent: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct PollingMinutes {
    pub short: u64,
    pub extended: u64,
}

impl Display for AtaDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let health = if self.smart_status.passed {
            "✅ Healthy"
        } else {
            "❌ FAILING"
        };

        write!(
            f,
            "<b>{}</b>\n{} {} ({})\n{} · {}°C",
            self.model_name,
            format_bytes(self.user_capacity.bytes),
            self.device.protocol,
            self.device.name,
            health,
            self.temperature.current
        )?;
        if let Some(endurance) = &self.endurance_used {
            write!(f, " · wear {}%", endurance.current_percent)?;
        }

        write!(
            f,
            "\n{} h · {}",
            self.power_on_time.hours,
            pluralize(self.power_cycle_count, "cycle")
        )?;

        match self.latest_result() {
            Some(result) => write!(
                f,
                "\nLast self-test: {} — {} ({} h)",
                result.test_type.string, result.status.string, result.lifetime_hours
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
