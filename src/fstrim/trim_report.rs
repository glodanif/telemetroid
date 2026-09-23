use std::fmt::{Display, Formatter};
use crate::fstrim::trim_result::TrimResult;
use crate::text_utils::format_bytes;

/// Outcome of trimming every mounted filesystem that supports discard.
///
/// Unlike the scrub and self-test reports there is no per-entry error: `fstrim -a` either
/// succeeds for every filesystem it selected or fails as a whole, and the caller reports that
/// failure. Filesystems without discard support — spinning disks, ntfs-3g (fuseblk) mounts,
/// which implement no `FITRIM` ioctl — are skipped by fstrim and never appear here.
pub struct TrimReport {
    pub results: Vec<TrimResult>,
}

impl TrimReport {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Total bytes reported trimmed across every filesystem.
    pub fn total_bytes(&self) -> u64 {
        self.results.iter().map(|result| result.bytes).sum()
    }
}

impl Display for TrimReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            writeln!(f, "No filesystems supporting discard found")?;
        }
        for (index, result) in self.results.iter().enumerate() {
            writeln!(f, "{}. <b>{}</b>\n{}", index + 1, result.mount_point, result)?;
            writeln!(f)?;
        }
        if self.len() > 1 {
            writeln!(f, "<b>Total:</b> {}", format_bytes(self.total_bytes()))?;
        }
        Ok(())
    }
}
