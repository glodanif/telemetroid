use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use crate::btrfs::btrfs_error::BtrfsError;
use crate::text_utils::format_bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrubStatus {
    Running,
    Finished,
    Aborted,
    Interrupted,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct ScrubResult {
    pub uuid: String,
    pub started: String,
    pub status: ScrubStatus,
    pub duration: Duration,
    pub data_bytes_scrubbed: u64,
    pub tree_bytes_scrubbed: u64,
    pub read_errors: u64,
    pub csum_errors: u64,
    pub verify_errors: u64,
    pub super_errors: u64,
    pub corrected_errors: u64,
    pub uncorrectable_errors: u64,
}

impl ScrubResult {
    /// Parses the output of `btrfs scrub status -R <mount>`.
    pub fn parse(output: &str) -> Result<Self, BtrfsError> {
        let fields: HashMap<&str, &str> = output
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.trim(), value.trim()))
            .collect();

        Ok(ScrubResult {
            uuid: field(&fields, "UUID")?.to_string(),
            started: field(&fields, "Scrub started")?.to_string(),
            status: parse_status(field(&fields, "Status")?),
            duration: parse_duration(field(&fields, "Duration")?)?,
            data_bytes_scrubbed: number(&fields, "data_bytes_scrubbed")?,
            tree_bytes_scrubbed: number(&fields, "tree_bytes_scrubbed")?,
            read_errors: number(&fields, "read_errors")?,
            csum_errors: number(&fields, "csum_errors")?,
            verify_errors: number(&fields, "verify_errors")?,
            super_errors: number(&fields, "super_errors")?,
            corrected_errors: number(&fields, "corrected_errors")?,
            uncorrectable_errors: number(&fields, "uncorrectable_errors")?,
        })
    }

    /// Total errors that btrfs could not repair (data loss / unredundant corruption).
    pub fn has_unrecoverable_errors(&self) -> bool {
        self.uncorrectable_errors > 0
            || self.super_errors > 0
            || self.verify_errors > 0
    }

    /// True only when the scrub ran to completion with nothing left broken.
    pub fn is_healthy(&self) -> bool {
        self.status == ScrubStatus::Finished && !self.has_unrecoverable_errors()
    }

    /// Human-readable list of the non-zero error counters.
    pub fn error_summary(&self) -> Vec<String> {
        let counters = [
            ("uncorrectable", self.uncorrectable_errors),
            ("csum", self.csum_errors),
            ("read", self.read_errors),
            ("verify", self.verify_errors),
            ("super", self.super_errors),
            ("corrected", self.corrected_errors),
        ];
        counters
            .iter()
            .filter(|(_, count)| *count > 0)
            .map(|(label, count)| format!("{}: {}", label, count))
            .collect()
    }
}

impl Display for ScrubStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrubStatus::Running => f.write_str("running"),
            ScrubStatus::Finished => f.write_str("finished"),
            ScrubStatus::Aborted => f.write_str("aborted"),
            ScrubStatus::Interrupted => f.write_str("interrupted"),
            ScrubStatus::Unknown(other) => f.write_str(other),
        }
    }
}

impl Display for ScrubResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let health = if self.has_unrecoverable_errors() {
            "❌ Errors"
        } else if self.corrected_errors > 0 {
            "⚠️ Repaired"
        } else if self.status == ScrubStatus::Finished {
            "✅ Healthy"
        } else {
            "ℹ️ Incomplete"
        };

        write!(
            f,
            "{} · {} · {} scrubbed",
            health,
            self.status,
            format_bytes(self.data_bytes_scrubbed + self.tree_bytes_scrubbed),
        )?;

        let errors = self.error_summary();
        if !errors.is_empty() {
            write!(f, "\n⚠️ {}", errors.join(" · "))?;
        }

        Ok(())
    }
}

fn field<'a>(fields: &HashMap<&str, &'a str>, key: &str) -> Result<&'a str, BtrfsError> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| BtrfsError::FormatError(format!("missing field `{key}`")))
}

fn number(fields: &HashMap<&str, &str>, key: &str) -> Result<u64, BtrfsError> {
    field(fields, key)?
        .parse()
        .map_err(|_| BtrfsError::FormatError(format!("`{key}` is not a valid number")))
}

fn parse_status(value: &str) -> ScrubStatus {
    match value {
        "running" => ScrubStatus::Running,
        "finished" => ScrubStatus::Finished,
        "aborted" => ScrubStatus::Aborted,
        "interrupted" => ScrubStatus::Interrupted,
        other => ScrubStatus::Unknown(other.to_string()),
    }
}

/// Parses the `H:MM:SS` duration btrfs prints.
fn parse_duration(value: &str) -> Result<Duration, BtrfsError> {
    let invalid = || BtrfsError::FormatError(format!("invalid duration `{value}`"));
    let [hours, minutes, seconds] = value.split(':').collect::<Vec<_>>()[..] else {
        return Err(invalid());
    };
    let hours: u64 = hours.parse().map_err(|_| invalid())?;
    let minutes: u64 = minutes.parse().map_err(|_| invalid())?;
    let seconds: u64 = seconds.parse().map_err(|_| invalid())?;
    Ok(Duration::from_secs(hours * 3600 + minutes * 60 + seconds))
}
