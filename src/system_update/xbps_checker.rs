use crate::system_update::available_package_update::AvailablePackageUpdate;
use std::collections::HashMap;
use std::process::Command;

pub fn check_updates() -> Result<Vec<AvailablePackageUpdate>, String> {
    let output = Command::new("xbps-install")
        .args(["-Sun"])
        .output()
        .map_err(|e| format!("Failed to run xbps-install: {}", e))?;

    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("xbps-install error: {}", stderr.trim()));
    }

    // Each line: "pkgname-newver arch -> repourl"
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
            AvailablePackageUpdate { name, from_version, to_version }
        })
        .collect())
}

// Splits "pkgname-1.2.3_1" into ("pkgname", "1.2.3_1").
// xbps convention: split at the last '-' followed by a digit.
fn split_pkg_ver(s: &str) -> Option<(String, String)> {
    let bytes = s.as_bytes();
    for i in (0..bytes.len().saturating_sub(1)).rev() {
        if bytes[i] == b'-' && bytes[i + 1].is_ascii_digit() {
            return Some((s[..i].to_string(), s[i + 1..].to_string()));
        }
    }
    None
}

fn query_installed() -> Result<HashMap<String, String>, String> {
    let output = Command::new("xbps-query")
        .args(["-l"])
        .output()
        .map_err(|e| format!("Failed to run xbps-query: {}", e))?;

    // Each line: "ii pkgname-ver  description"
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let pkg_ver = line.split_whitespace().nth(1)?;
            split_pkg_ver(pkg_ver)
        })
        .collect())
}
