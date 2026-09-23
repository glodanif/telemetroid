use crate::command_runner::run_command;
use crate::fstrim::fstrim_error::FstrimError;
use crate::fstrim::trim_report::TrimReport;
use crate::fstrim::trim_result::TrimResult;

const FSTRIM: &str = "fstrim";

/// Trims every mounted filesystem that supports discard.
///
/// `-a` leaves the selection to fstrim: it walks the mount table, skips read-only and duplicate
/// mounts, and skips devices with no discard support, which `--quiet-unsupported` (util-linux
/// 2.36+) also keeps out of the output. Any non-zero exit — including 64, "some filesystems
/// failed" — is surfaced as an error rather than reported as a partial success, so a half-done
/// run is never mistaken for a clean one.
pub(crate) fn execute_trim() -> Result<TrimReport, FstrimError> {
    log::info!("Trimming all mounted filesystems that support discard");
    let output = run_command(FSTRIM, &["-a", "-v", "--quiet-unsupported"], |code| code != 0)?;

    let text = String::from_utf8(output).map_err(|e| FstrimError::FormatError(e.to_string()))?;
    let results = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(TrimResult::parse)
        .collect::<Result<Vec<_>, _>>()?;

    log::info!("Trim complete on {} filesystem(s)", results.len());
    for result in &results {
        log::debug!("  {}: {} bytes", result.mount_point, result.bytes);
    }
    Ok(TrimReport { results })
}
