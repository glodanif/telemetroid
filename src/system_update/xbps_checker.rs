use crate::system_update::available_package_update::AvailablePackageUpdate;
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    let updates = stdout
        .lines()
        .skip(1) // skip header line
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[1] == "update" {
                Some(AvailablePackageUpdate {
                    name: parts[0].to_string(),
                    from_version: parts[2].to_string(),
                    to_version: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(updates)
}
