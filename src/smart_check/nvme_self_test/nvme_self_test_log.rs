use serde::Deserialize;

const COMPLETED_WITHOUT_ERROR: &str = "Completed without error";

#[derive(Debug, Deserialize)]
pub struct NvmeSelfTestLog {
    pub nsid: i64,
    pub current_self_test_operation: ValueString,
    pub table: Option<Vec<LogEntry>>,
    pub current_self_test_completion_percent: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub self_test_result: ValueString,
    pub self_test_code: ValueString,
    pub power_on_hours: u64,
}

impl LogEntry {
    pub fn has_completed_without_error(&self) -> bool {
        self.self_test_result.string == COMPLETED_WITHOUT_ERROR
    }
}

#[derive(Debug, Deserialize)]
pub struct ValueString {
    pub value: u64,
    pub string: String,
}
