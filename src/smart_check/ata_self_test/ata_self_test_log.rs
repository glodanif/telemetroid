use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AtaSmartSelfTestLog {
    pub standard: Standard,
}

#[derive(Debug, Deserialize)]
pub struct Standard {
    pub revision: u32,
    pub table: Option<Vec<LogEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    #[serde(rename = "type")]
    pub test_type: TestType,
    pub status: Status,
    pub lifetime_hours: u64,
    // The LBA of the first error; smartctl only emits this for a failed test.
    pub lba: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TestType {
    // e.g. "Short offline", "Extended offline", "Conveyance offline".
    pub string: String,
}

impl LogEntry {
    /// Detail about a failure (the LBA of the first read error), if recorded.
    pub fn failure_detail(&self) -> Option<String> {
        self.lba.map(|lba| format!("first error at LBA {lba}"))
    }
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub value: u32,
    pub string: String,
    // smartctl omits `passed` for tests that didn't complete (e.g. "Aborted by host"
    // carries `remaining_percent` instead). Treat a missing value as not-passed.
    #[serde(default)]
    pub passed: bool,
}
