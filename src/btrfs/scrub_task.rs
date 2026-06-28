use std::sync::atomic::{AtomicBool, Ordering};
use tokio::task;
use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::btrfs_interface::{run_scrub, scan_btrfs_filesystems};
use crate::btrfs::scrub_report::ScrubReport;

// `static`, not `const`: a const is inlined at every use site, so each access would get
// its own fresh atomic and the guard would never actually see another run in progress.
static SCRUB_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// RAII guard that marks a scrub as running and clears the flag on drop, so the slot is
/// always released — including on early returns and future cancellation.
struct ScrubGuard;

impl ScrubGuard {
    fn acquire() -> Result<Self, BtrfsError> {
        if SCRUB_IN_PROGRESS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(BtrfsError::AlreadyRunningError());
        }
        Ok(ScrubGuard)
    }
}

impl Drop for ScrubGuard {
    fn drop(&mut self) {
        SCRUB_IN_PROGRESS.store(false, Ordering::Release);
    }
}

pub async fn run_btrfs_scrub() -> Result<ScrubReport, BtrfsError> {
    // Held across the await; cleared on drop however this function exits.
    let _guard = ScrubGuard::acquire()?;
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
