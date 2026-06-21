use crate::smart_check::basic_device_info::DeviceInterface;

#[derive(Debug)]
pub enum SelfTestOutcome {
    CompletedOk,
    CompletedWithError { failing_lba: Option<u64> },
    Aborted,
}

#[derive(Debug)]
pub struct AtaSmartCheckResult {
    pub drive_name: String,
    pub model_name: String,
    pub user_capacity_bytes: u64,   // was String; this is a number, store it as one
    pub interface: DeviceInterface,

    pub current_temperature: i32,
    pub max_temperature: Option<i32>,

    pub endurance_used_percentage: u8,
    pub power_on_time: u64,
    pub power_cycle_count: u64,
    pub unsafe_shutdowns: Option<u64>,   // attr 174, your UPS/power proxy

    pub smart_status: bool,

    pub self_test_outcome: SelfTestOutcome,
    pub last_self_test_hours: Option<u64>,   // to compute "how long ago"

    // the actual early-warning signals
    pub reallocated_sectors: Option<u64>,    // attr 5
    pub pending_sectors: Option<u64>,        // attr 197
    pub offline_uncorrectable: Option<u64>,  // attr 198
    pub reported_uncorrectable: Option<u64>, // attr 187
    pub crc_errors: Option<u64>,             // attr 199, bad cable signal
}