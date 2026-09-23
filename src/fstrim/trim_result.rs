use std::fmt::{Display, Formatter};
use crate::fstrim::fstrim_error::FstrimError;
use crate::text_utils::format_bytes;

/// One mount point's trim result, parsed from a single `fstrim -v` output line.
#[derive(Debug, Clone)]
pub struct TrimResult {
    pub mount_point: String,
    /// Backing device, reported only when fstrim discovered the mount itself (`-a`/`--fstab`).
    pub device: Option<String>,
    /// Bytes the kernel reports as trimmed. This is the length of the ranges handed to the
    /// device, not space reclaimed — on most filesystems it is an upper bound.
    pub bytes: u64,
}

impl TrimResult {
    /// Parses one line of `fstrim -v` output, in either shape it emits:
    ///   `/home: 1.5 GiB (1610612736 bytes) trimmed on /dev/sda2`
    ///   `/home: 1.5 GiB (1610612736 bytes) trimmed`
    pub fn parse(line: &str) -> Result<Self, FstrimError> {
        let invalid = || FstrimError::FormatError(format!("unrecognised output line `{line}`"));

        // Split from the right around the byte count, so a mount point containing `": "`
        // (or a space) still comes out whole.
        let (head, tail) = line.split_once(" bytes)").ok_or_else(invalid)?;
        let (head, bytes) = head.rsplit_once(" (").ok_or_else(invalid)?;
        let (mount_point, _human_size) = head.rsplit_once(": ").ok_or_else(invalid)?;

        let tail = tail.trim();
        if !tail.starts_with("trimmed") {
            return Err(invalid());
        }

        Ok(TrimResult {
            mount_point: mount_point.to_string(),
            device: tail.strip_prefix("trimmed on ").map(str::to_string),
            bytes: bytes.parse().map_err(|_| invalid())?,
        })
    }
}

impl Display for TrimResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "✂️ {} trimmed", format_bytes(self.bytes))?;
        if let Some(device) = &self.device {
            write!(f, " · {}", device)?;
        }
        Ok(())
    }
}
