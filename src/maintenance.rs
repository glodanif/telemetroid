use tokio::task;

use crate::btrfs::btrfs_error::BtrfsError;
use crate::btrfs::scrub_report::ScrubReport;
use crate::btrfs::scrub_task::{execute_scrub, ScrubRun};
use crate::smart_check::self_test_report::SelfTestReport;
use crate::smart_check::self_test_task::{execute_self_test, SelfTestRun, SmartTestKind};
use crate::smart_check::smart_check_error::SmartCheckError;

/// Returned when a scrub or self-test is already running, so a combined maintenance run cannot
/// reserve both slots.
pub struct MaintenanceBusy;

/// A combined monthly maintenance run: a btrfs scrub followed by a long SMART self-test.
///
/// It holds both the scrub slot and the self-test slot for its entire duration, so while it is
/// active every manual `/scrub`, `/smart_test_long` and `/smart_test_short` is rejected with
/// "already in progress" (their `begin()` calls see the slots taken).
pub struct MaintenanceRun {
    _scrub: ScrubRun,
    _selftest: SelfTestRun,
}

impl MaintenanceRun {
    /// Reserves both the scrub and self-test slots, or returns `MaintenanceBusy` if either is
    /// already taken. If the second reservation fails, the first guard is dropped (released)
    /// automatically.
    pub fn begin() -> Result<Self, MaintenanceBusy> {
        let selftest = SelfTestRun::begin().map_err(|_| MaintenanceBusy)?;
        let scrub = ScrubRun::begin().map_err(|_| MaintenanceBusy)?;
        Ok(MaintenanceRun {
            _scrub: scrub,
            _selftest: selftest,
        })
    }

    /// Runs the scrub on every filesystem to completion, off the async runtime. Borrows `self` so
    /// both slots stay reserved until the whole maintenance run is dropped.
    pub async fn run_scrub(&self) -> Result<ScrubReport, BtrfsError> {
        task::spawn_blocking(execute_scrub).await?
    }

    /// Runs the long SMART self-test on every drive to completion, off the async runtime. Borrows
    /// `self` so both slots stay reserved until the whole maintenance run is dropped.
    pub async fn run_long_test(&self) -> Result<SelfTestReport, SmartCheckError> {
        task::spawn_blocking(|| execute_self_test(SmartTestKind::Long)).await?
    }
}
