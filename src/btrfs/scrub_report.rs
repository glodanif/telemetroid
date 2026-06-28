use std::fmt::{Display, Formatter};
use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::scrub_result::ScrubResult;

/// Outcome of scrubbing every mounted btrfs filesystem. Each entry keeps its mount
/// point so a failed scrub is still attributable to a filesystem.
pub struct ScrubReport {
    pub results: Vec<(String, Result<ScrubResult, BtrfsError>)>,
}

impl ScrubReport {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }
}

impl Display for ScrubReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            writeln!(f, "No btrfs filesystems found")?;
        }
        for (index, (target, result)) in self.results.iter().enumerate() {
            match result {
                Ok(info) => writeln!(f, "{}. <b>{}</b>\n{}", index + 1, target, info)?,
                Err(err) => writeln!(f, "{}. <b>{}</b>\nFailed: {}", index + 1, target, err)?,
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
