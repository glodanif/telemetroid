use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tokio::task;

use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::basic_device_info::DeviceInterface;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::self_test_report::{SelfTestOutcome, SelfTestReport};
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_ctl_interface::{check_ata_drive, check_nvme_drive, launch_self_test, scan_drives};

/// How long to wait between polls of a running self-test.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Multiple of the drive's own time estimate to wait before giving up.
const TIMEOUT_GRACE: u32 = 2;

/// Lower bound on a test's timeout, so a tiny reported estimate still leaves headroom.
const MIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);

// `static`, not `const`: a const is inlined at every use site, so each access would get
// its own fresh atomic and the guard would never actually see another run in progress.
static SELF_TEST_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Which self-test to launch. Maps to smartctl's `-t` argument.
#[derive(Debug, Clone, Copy)]
pub enum SmartTestKind {
    Short,
    Long,
}

impl SmartTestKind {
    fn arg(self) -> &'static str {
        match self {
            SmartTestKind::Short => "short",
            SmartTestKind::Long => "long",
        }
    }

    /// Timeout to use when the drive reports no time estimate (NVMe). A short test is a couple
    /// of minutes; an extended test can run for hours on a large drive.
    fn fallback_timeout(self) -> Duration {
        match self {
            SmartTestKind::Short => Duration::from_secs(15 * 60),
            SmartTestKind::Long => Duration::from_secs(4 * 60 * 60),
        }
    }
}

impl Display for SmartTestKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SmartTestKind::Short => f.write_str("short"),
            SmartTestKind::Long => f.write_str("long"),
        }
    }
}

/// RAII guard that marks a self-test run as in progress and clears the flag on drop, so the
/// slot is always released — including on early returns and future cancellation.
struct SelfTestGuard;

impl SelfTestGuard {
    fn acquire() -> Result<Self, SmartCheckError> {
        if SELF_TEST_IN_PROGRESS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(SmartCheckError::AlreadyRunningError());
        }
        Ok(SelfTestGuard)
    }
}

impl Drop for SelfTestGuard {
    fn drop(&mut self) {
        SELF_TEST_IN_PROGRESS.store(false, Ordering::Release);
    }
}

pub async fn run_short_self_test() -> Result<SelfTestReport, SmartCheckError> {
    run_self_test(SmartTestKind::Short).await
}

pub async fn run_long_self_test() -> Result<SelfTestReport, SmartCheckError> {
    run_self_test(SmartTestKind::Long).await
}

async fn run_self_test(kind: SmartTestKind) -> Result<SelfTestReport, SmartCheckError> {
    // Held across the await; cleared on drop however this function exits.
    let _guard = SelfTestGuard::acquire()?;
    let report = task::spawn_blocking(move || execute_self_test(kind)).await??;
    Ok(report)
}

fn execute_self_test(kind: SmartTestKind) -> Result<SelfTestReport, SmartCheckError> {
    log::info!("Scanning for drives to run {} self-test", kind);
    let testable: Vec<_> = scan_drives()?
        .into_iter()
        .filter(|drive| !matches!(drive.protocol, DeviceInterface::Unsupported))
        .collect();

    if testable.is_empty() {
        log::info!("No testable drives found");
        return Ok(SelfTestReport { results: Vec::new() });
    }

    let names: Vec<&str> = testable.iter().map(|d| d.name.as_str()).collect();
    log::info!(
        "Running {} self-test concurrently on {} drive(s): {}",
        kind,
        testable.len(),
        names.join(", "),
    );

    // Each drive's test is independent and runs on its own controller, so launch them on
    // separate threads and poll in parallel. Scoped threads let the closures borrow the
    // drive list, and joining in order keeps the report deterministic regardless of which
    // drive finishes first.
    let results = thread::scope(|scope| {
        let handles: Vec<_> = testable
            .iter()
            .map(|drive| {
                scope.spawn(move || {
                    let result = match drive.protocol {
                        DeviceInterface::Ata => run_on_drive::<AtaDriveInfo>(&drive.name, kind),
                        DeviceInterface::Nvme => run_on_drive::<NvmeDriveInfo>(&drive.name, kind),
                        DeviceInterface::Unsupported => unreachable!("filtered out above"),
                    };
                    if let Err(err) = &result {
                        log::error!("{} self-test failed on {}: {}", kind, drive.name, err);
                    }
                    (drive.name.clone(), result)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("self-test thread panicked"))
            .collect()
    });

    log::info!("{} self-test run complete on all drives", kind);
    Ok(SelfTestReport { results })
}

/// Launches a self-test on one drive and polls until it finishes (or the timeout elapses).
fn run_on_drive<T: SelfTestDrive>(name: &str, kind: SmartTestKind) -> Result<SelfTestOutcome, SmartCheckError> {
    log::info!("Starting {} self-test on {}", kind, name);
    // Snapshot the existing log (and the drive's own time estimate) before launching.
    let baseline = T::fetch(name)?;
    let before = baseline.table_snapshot();
    let timeout = match baseline.estimated_duration(kind) {
        // Allow twice the drive's own estimate, but never less than a small floor.
        Some(estimate) => (estimate * TIMEOUT_GRACE).max(MIN_TIMEOUT),
        // No estimate reported (NVMe): fall back to a fixed ceiling.
        None => kind.fallback_timeout(),
    };
    launch_self_test(name, kind.arg())?;
    log::info!(
        "{} self-test launched on {}, waiting up to {} min for completion",
        kind,
        name,
        timeout.as_secs() / 60,
    );

    let deadline = Instant::now() + timeout;
    loop {
        thread::sleep(POLL_INTERVAL);
        let info = T::fetch(name)?;

        if info.is_running() {
            match info.progress() {
                Some(percent) => log::info!("{} self-test on {}: {}% complete", kind, name, percent),
                None => log::info!("{} self-test on {}: in progress", kind, name),
            }
        } else if info.table_snapshot() != before {
            let outcome = info
                .outcome()
                .ok_or_else(|| SmartCheckError::TestIncomplete(name.to_string(), "no result logged".to_string()))?;
            log::info!(
                "{} self-test on {} finished: {}",
                kind,
                name,
                if outcome.passed { "passed" } else { "FAILED" },
            );
            return Ok(outcome);
        }

        if Instant::now() >= deadline {
            log::error!("{} self-test on {} timed out", kind, name);
            return Err(SmartCheckError::TestIncomplete(
                name.to_string(),
                "timed out waiting for completion".to_string(),
            ));
        }
    }
}

/// Common operations the polling loop needs from a drive, regardless of protocol.
trait SelfTestDrive: Sized {
    fn fetch(name: &str) -> Result<Self, SmartCheckError>;
    fn is_running(&self) -> bool;
    fn table_snapshot(&self) -> Vec<(u64, u64)>;
    fn outcome(&self) -> Option<SelfTestOutcome>;
    /// The drive's own estimate of how long the test will take, if it reports one.
    fn estimated_duration(&self, kind: SmartTestKind) -> Option<Duration>;
    /// Percent of the running test completed (0–100), if the drive reports progress.
    fn progress(&self) -> Option<u8>;
}

impl SelfTestDrive for AtaDriveInfo {
    fn fetch(name: &str) -> Result<Self, SmartCheckError> {
        check_ata_drive(name)
    }

    fn is_running(&self) -> bool {
        AtaDriveInfo::is_running(self)
    }

    fn table_snapshot(&self) -> Vec<(u64, u64)> {
        AtaDriveInfo::table_snapshot(self)
    }

    fn outcome(&self) -> Option<SelfTestOutcome> {
        self.latest_result().map(|entry| SelfTestOutcome {
            passed: entry.status.passed,
            status: entry.status.string.clone(),
            power_on_hours: self.power_on_time.hours,
        })
    }

    fn estimated_duration(&self, kind: SmartTestKind) -> Option<Duration> {
        let polling = &self.ata_smart_data.self_test.polling_minutes;
        let minutes = match kind {
            SmartTestKind::Short => polling.short,
            SmartTestKind::Long => polling.extended,
        };
        Some(Duration::from_secs(minutes * 60))
    }

    fn progress(&self) -> Option<u8> {
        // smartctl reports percent *remaining* while a test runs; invert it to percent done.
        self.ata_smart_data
            .self_test
            .status
            .remaining_percent
            .map(|remaining| 100u8.saturating_sub(remaining))
    }
}

impl SelfTestDrive for NvmeDriveInfo {
    fn fetch(name: &str) -> Result<Self, SmartCheckError> {
        check_nvme_drive(name)
    }

    fn is_running(&self) -> bool {
        NvmeDriveInfo::is_running(self)
    }

    fn table_snapshot(&self) -> Vec<(u64, u64)> {
        NvmeDriveInfo::table_snapshot(self)
    }

    fn outcome(&self) -> Option<SelfTestOutcome> {
        self.latest_result().map(|entry| SelfTestOutcome {
            passed: entry.has_completed_without_error(),
            status: entry.self_test_result.string.clone(),
            power_on_hours: entry.power_on_hours,
        })
    }

    // NVMe self-test output carries no time estimate; the caller uses a fixed timeout.
    fn estimated_duration(&self, _kind: SmartTestKind) -> Option<Duration> {
        None
    }

    fn progress(&self) -> Option<u8> {
        self.nvme_self_test_log.current_self_test_completion_percent
    }
}
