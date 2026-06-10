use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NvmeSelfTestLog {
    pub nsid: i64,
    pub table: Vec<LogEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub self_test_result: SelfTestResult,
    pub power_on_hours: u64,
}

#[derive(Debug, Deserialize)]
pub struct SelfTestResult {
    pub value: u32,
    pub string: String,
}
