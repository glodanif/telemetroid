use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct SmartCheckResult {
    pub drive_name: String,
    pub power_on_time: u32,
    pub passed: bool,
}

pub struct SmartCheckFailure {
    pub drive_name: String,
    pub message: String,
}

impl Display for SmartCheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}\n{}", self.drive_name, self.power_on_time)
    }
}

impl Display for SmartCheckFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}\n{}", self.drive_name, format!("Preparation failed: {}", self.message))
    }
}
