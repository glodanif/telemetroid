use crate::system_update::available_package_update::AvailablePackageUpdate;
use crate::system_update::update_check_error::UpdateCheckError;
use std::collections::HashMap;
use std::process::Command;

pub fn check_updates() -> Result<Vec<AvailablePackageUpdate>, UpdateCheckError> {
    // Sync the repodata as its own step. In dry-run mode (`-n`) xbps-install
    // skips the `-S` sync, so a combined `-Sun` checks against the stale local
    // index and can report "up to date" while updates are actually pending.
    let sync = Command::new("xbps-install")
        .arg("-S")
        .output()
        .map_err(|e| {
            UpdateCheckError::CommandExecutionError("xbps-install -S".to_string(), e.to_string())
        })?;

    if !sync.status.success() {
        let stderr = String::from_utf8_lossy(&sync.stderr);
        return Err(UpdateCheckError::CommandExecutionError(
            "xbps-install -S".to_string(),
            stderr.trim().to_string(),
        ));
    }

    // Check pending updates against the freshly-synced index (read-only, no -S).
    let output = Command::new("xbps-install")
        .args(["-un"])
        .output()
        .map_err(|e| {
            UpdateCheckError::CommandExecutionError("xbps-install -un".to_string(), e.to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UpdateCheckError::CommandExecutionError(
            "xbps-install -un".to_string(),
            stderr.trim().to_string(),
        ));
    }

    let pending: Vec<(String, String)> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let pkg_ver = line.split_whitespace().next()?;
            split_pkg_ver(pkg_ver)
        })
        .collect();

    if pending.is_empty() {
        return Ok(vec![]);
    }

    let installed = query_installed()?;

    Ok(pending
        .into_iter()
        .map(|(name, to_version)| {
            let from_version = installed.get(&name).cloned().unwrap_or_default();
            AvailablePackageUpdate {
                name,
                from_version,
                to_version,
            }
        })
        .collect())
}

fn split_pkg_ver(s: &str) -> Option<(String, String)> {
    let bytes = s.as_bytes();
    for i in (0..bytes.len().saturating_sub(1)).rev() {
        if bytes[i] == b'-' && bytes[i + 1].is_ascii_digit() {
            return Some((s[..i].to_string(), s[i + 1..].to_string()));
        }
    }
    None
}

fn query_installed() -> Result<HashMap<String, String>, UpdateCheckError> {
    let output = Command::new("xbps-query")
        .args(["-l"])
        .output()
        .map_err(|e| {
            UpdateCheckError::CommandExecutionError("xbps-query -l".to_string(), e.to_string())
        })?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let pkg_ver = line.split_whitespace().nth(1)?;
            split_pkg_ver(pkg_ver)
        })
        .collect())
}
