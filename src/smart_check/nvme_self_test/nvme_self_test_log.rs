use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NvmeSelfTestLog {
    pub nsid: i64,
    pub current_self_test_operation: Option<ValueString>,
    pub table: Option<Vec<LogEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub self_test_result: ValueString,
    pub self_test_code: ValueString,
    pub power_on_hours: u64,
}

#[derive(Debug, Deserialize)]
pub struct ValueString {
    pub value: u64,
    pub string: String,
}

