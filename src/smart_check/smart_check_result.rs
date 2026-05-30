use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct SmartCheckResult {
    pub drive_name: String,
}

impl Display for SmartCheckResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.drive_name)
    }
}
