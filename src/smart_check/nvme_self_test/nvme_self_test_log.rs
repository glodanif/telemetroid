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
    // Only emitted by smartctl when the corresponding field is valid (i.e. on a failure).
    pub failing_lba: Option<u64>,
    pub segment: Option<u32>,
}

impl LogEntry {
    pub fn has_completed_without_error(&self) -> bool {
        self.self_test_result.string == COMPLETED_WITHOUT_ERROR
    }

    /// Detail about a failure (failing LBA and/or segment), if reported.
    pub fn failure_detail(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(lba) = self.failing_lba {
            parts.push(format!("LBA {lba}"));
        }
        if let Some(segment) = self.segment {
            parts.push(format!("segment {segment}"));
        }
        (!parts.is_empty()).then(|| parts.join(", "))
    }
}

#[derive(Debug, Deserialize)]
pub struct ValueString {
    pub value: u64,
    pub string: String,
}
