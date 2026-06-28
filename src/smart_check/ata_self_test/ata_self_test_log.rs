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
    pub status: Status,
    pub lifetime_hours: u64,
    // The LBA of the first error; smartctl only emits this for a failed test.
    pub lba: Option<u64>,
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
    pub passed: bool,
}
