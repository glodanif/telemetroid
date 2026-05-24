use crate::info_collector::os_info::OsInfo;
use std::fs;
use std::time::SystemTime;

pub fn collect() -> OsInfo {
    OsInfo {
        name: read_os_name(),
        host: read_host(),
        kernel: read_kernel(),
        age: read_os_age(),
        uptime: read_uptime(),
    }
}

fn read_os_name() -> Option<String> {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .map(|line| {
                    line.trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string()
                })
        })
}

fn read_kernel() -> Option<String> {
    fs::read_to_string("/proc/version")
        .ok()
        .and_then(|content| content.split_whitespace().nth(2).map(|v| v.to_string()))
}

fn is_dmi_placeholder(s: &str) -> bool {
    matches!(
        s.to_ascii_lowercase().as_str(),
        "system product name"
            | "system version"
            | "to be filled by o.e.m."
            | "default string"
            | "not specified"
            | "none"
            | "n/a"
    )
}

fn read_host() -> Option<String> {
    let name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !is_dmi_placeholder(s))
        .unwrap_or_default();

    let version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !is_dmi_placeholder(s))
        .unwrap_or_default();

    match (name.is_empty(), version.is_empty()) {
        (false, false) => Some(format!("{name} {version}")),
        (false, true) => Some(name),
        _ => None,
    }
}

fn read_os_age() -> Option<String> {
    let meta = fs::metadata("/").ok();
    let created = meta.and_then(|m| m.created().ok());
    let elapsed = created.and_then(|c| SystemTime::now().duration_since(c).ok())?;

    let total_days = elapsed.as_secs() / 86400;
    let years = total_days / 365;
    let months = (total_days % 365) / 30;
    let days = total_days % 30;
    let string = match (years, months, days) {
        (0, 0, d) => format!("{d} days"),
        (0, m, d) => format!("{m} months, {d} days"),
        (y, m, d) => format!("{y} years, {m} months, {d} days"),
    };
    Some(string)
}

fn read_uptime() -> Option<String> {
    let content = fs::read_to_string("/proc/uptime").ok();
    let seconds = content
        .as_deref()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<f64>().ok())?;

    let total_minutes = seconds as u64 / 60;
    let days = total_minutes / 1440;
    let hours = (total_minutes % 1440) / 60;
    let minutes = total_minutes % 60;
    let string = match (days, hours, minutes) {
        (0, 0, m) => format!("{m} mins"),
        (0, h, m) => format!("{h} hours, {m} mins"),
        (d, h, m) => format!("{d} days, {h} hours, {m} mins"),
    };
    Some(string)
}
