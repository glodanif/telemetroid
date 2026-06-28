use serde::Deserialize;

/// Top-level shape of `findmnt -J` output.
#[derive(Debug, Deserialize)]
pub struct FindmntOutput {
    pub filesystems: Vec<BtrfsFilesystem>,
}

/// A single mounted btrfs filesystem as reported by findmnt.
#[derive(Debug, Clone, Deserialize)]
pub struct BtrfsFilesystem {
    pub target: String,
    pub source: String,
    pub uuid: String,
}
