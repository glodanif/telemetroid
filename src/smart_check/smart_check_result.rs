use std::fmt::{Display, Formatter, Result};
use thiserror::Error;

#[derive(Debug)]
pub struct SmartCheckResult {
    pub drive_name: String,
    pub power_on_time: u64,
    pub passed: bool,
}

#[derive(Debug, Error)]
pub enum SmartCheckFailure {
    #[error("Failed to launch self-test on \"{0}\": {1}")]
    CommandFailed (String, String),
    #[error("Self-test on \"{0}\" did not start: {1}")]
    FailedToStart (String, String),
    #[error("Self-test result for \"{0}\" was not logged: {1}")]
    TestNotLogged (String, String),
    #[error("Device \"{0}\" is unavailable: {1}")]
    DeviceUnavailable (String, String),
}

impl Display for SmartCheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let pass = if self.passed { "passed" } else { "failed" };
        write!(f, "{} - {}", self.drive_name, pass)
    }
}
