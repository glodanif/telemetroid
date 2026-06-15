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
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub value: u32,
    pub string: String,
    pub passed: bool,
}
