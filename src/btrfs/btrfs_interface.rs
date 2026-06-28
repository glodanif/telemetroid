use std::collections::HashSet;
use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::btrfs_filesystem::{BtrfsFilesystem, FindmntOutput};
use crate::btrfs::scrub_result::ScrubResult;
use crate::command_runner::run_command;

const BTRFS: &str = "btrfs";
const FINDMNT: &str = "findmnt";

/// Lists mounted btrfs filesystems, one entry per filesystem (deduped by UUID).
pub fn scan_btrfs_filesystems() -> Result<Vec<BtrfsFilesystem>, BtrfsError> {
    // findmnt exits 1 with no output when nothing matches; treat only >1 as a real error.
    let output = run_command(
        FINDMNT,
        &["-J", "-l", "-t", "btrfs", "-o", "TARGET,SOURCE,UUID"],
        |code| code > 1,
    )?;

    let json = String::from_utf8(output).map_err(|e| BtrfsError::FormatError(e.to_string()))?;
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: FindmntOutput =
        serde_json::from_str(&json).map_err(|e| BtrfsError::FormatError(e.to_string()))?;
    let filesystems = dedupe_by_uuid(parsed.filesystems);
    log::info!("Found {} btrfs filesystem(s) to scrub", filesystems.len());
    for fs in &filesystems {
        log::debug!("  {} ({}, {})", fs.target, fs.source, fs.uuid);
    }
    Ok(filesystems)
}

/// Runs a scrub on a single mount point and returns its parsed result.
pub fn run_scrub(target: &str) -> Result<ScrubResult, BtrfsError> {
    // `-B` keeps the scrub in the foreground until it finishes. Exit code 3 means the scrub
    // found errors, which is a valid result we want to report, not a command failure.
    log::info!("Starting scrub on {} (this may take a while)...", target);
    run_command(BTRFS, &["scrub", "start", "-B", target], |code| {
        code != 0 && code != 3
    })?;

    log::debug!("Scrub finished on {}, reading status", target);
    let output = run_command(BTRFS, &["scrub", "status", "-R", target], |code| code != 0)?;
    let status = String::from_utf8(output).map_err(|e| BtrfsError::FormatError(e.to_string()))?;
    let result = ScrubResult::parse(&status)?;
    log::info!(
        "Scrub done on {}: {} ({} corrected, {} uncorrectable)",
        target,
        result.status,
        result.corrected_errors,
        result.uncorrectable_errors,
    );
    Ok(result)
}

fn dedupe_by_uuid(filesystems: Vec<BtrfsFilesystem>) -> Vec<BtrfsFilesystem> {
    let mut seen = HashSet::new();
    filesystems
        .into_iter()
        .filter(|fs| seen.insert(fs.uuid.clone()))
        .collect()
}
