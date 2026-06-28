use tokio::task;
use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::btrfs_interface::{run_scrub, scan_btrfs_filesystems};
use crate::btrfs::scrub_report::ScrubReport;

pub async fn run_btrfs_scrub() -> Result<ScrubReport, BtrfsError> {
    let result = task::spawn_blocking(execute_scrub).await??;
    Ok(result)
}

fn execute_scrub() -> Result<ScrubReport, BtrfsError> {
    log::info!("Scanning for btrfs filesystems to scrub");
    let filesystems = scan_btrfs_filesystems()?;
    let results = filesystems
        .into_iter()
        .map(|filesystem| {
            let result = run_scrub(&filesystem.target);
            if let Err(err) = &result {
                log::error!("Scrub failed on {}: {}", filesystem.target, err);
            }
            (filesystem.target, result)
        })
        .collect();
    log::info!("Btrfs scrub run complete");
    Ok(ScrubReport { results })
}
