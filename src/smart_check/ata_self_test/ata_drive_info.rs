use crate::smart_check::ata_self_test::ata_self_test_log::{AtaSmartSelfTestLog, LogEntry};
use crate::smart_check::common::device_info_parts::{Device, PowerOnTime, UserCapacity};
use crate::text_utils::format_bytes;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Deserialize)]
pub struct AtaDriveInfo {
    pub model_name: String,
    pub power_on_time: PowerOnTime,
    pub user_capacity: UserCapacity,
    pub ata_smart_data: AtaSmartData,
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
    pub polling_minutes: PollingMinutes,
}

#[derive(Debug, Deserialize)]
pub struct PollingMinutes {
    pub short: u64,
}

impl Display for AtaDriveInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
