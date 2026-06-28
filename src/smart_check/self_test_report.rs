use std::fmt::{Display, Formatter};
use crate::smart_check::smart_check_error::SmartCheckError;

/// Result of a single drive's self-test, distilled to what gets reported.
pub struct SelfTestOutcome {
    pub passed: bool,
    pub status: String,
    pub power_on_hours: u64,
}

/// Outcome of running a self-test on every drive. Each entry keeps its device name so a
/// failed test (or one that never logged a result) is still attributable to a drive.
pub struct SelfTestReport {
    pub results: Vec<(String, Result<SelfTestOutcome, SmartCheckError>)>,
}

impl SelfTestReport {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }
}

impl Display for SelfTestOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let badge = if self.passed { "✅ Passed" } else { "❌ FAILED" };
        write!(f, "{} · {} ({} h)", badge, self.status, self.power_on_hours)
    }
}

impl Display for SelfTestReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            writeln!(f, "No drives found")?;
        }
        for (index, (name, result)) in self.results.iter().enumerate() {
            match result {
                Ok(outcome) => writeln!(f, "{}. <b>{}</b>\n{}", index + 1, name, outcome)?,
                Err(err) => writeln!(f, "{}. <b>{}</b>\nFailed: {}", index + 1, name, err)?,
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
