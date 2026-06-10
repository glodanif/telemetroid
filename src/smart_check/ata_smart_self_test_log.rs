pub struct AtaSmartSelfTestLog {
    pub short: u32,
}

pub struct Standard {
    pub revision: u32,
    pub table: Vec<LogEntry>,
}

pub struct LogEntry {

    pub lifetime_hours: u64,
}

pub struct Status {
    pub value: u32,
    pub string: String,
    pub passed: bool,
}