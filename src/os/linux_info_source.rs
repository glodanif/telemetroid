use crate::os::info_source::OsInfoSource;
use crate::os::os_info::OsInfo;
use std::fs;
use std::time::SystemTime;

pub struct LinuxInfoSource;

impl OsInfoSource for LinuxInfoSource {
    fn collect_info(&self) -> OsInfo {
        OsInfo {
            name: read_os_name(),
            kernel: read_kernel(),
            age: read_os_age(),
            uptime: read_uptime(),
        }
    }
}

fn read_os_name() -> String {
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
        .unwrap_or_else(|| "Unknown".into())
}

fn read_kernel() -> String {
    fs::read_to_string("/proc/version")
        .ok()
        .and_then(|content| content.split_whitespace().nth(2).map(|v| v.to_string()))
        .unwrap_or_else(|| "Unknown".into())
}

fn read_os_age() -> String {
    let meta = fs::metadata("/").ok();
    let created = meta.and_then(|m| m.created().ok());
    let elapsed = created.and_then(|c| SystemTime::now().duration_since(c).ok());

    match elapsed {
        None => "Unknown".into(),
        Some(e) => {
            let total_days = e.as_secs() / 86400;
            let years = total_days / 365;
            let months = (total_days % 365) / 30;
            let days = total_days % 30;
            match (years, months, days) {
                (0, 0, d) => format!("{d} days"),
                (0, m, d) => format!("{m} months, {d} days"),
                (y, m, d) => format!("{y} years, {m} months, {d} days"),
            }
        }
    }
}

fn read_uptime() -> String {
    let content = fs::read_to_string("/proc/uptime").ok();
    let seconds = content
        .as_deref()
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<f64>().ok());

    match seconds {
        None => "Unknown".into(),
        Some(secs) => {
            let total_minutes = secs as u64 / 60;
            let days = total_minutes / 1440;
            let hours = (total_minutes % 1440) / 60;
            let minutes = total_minutes % 60;
            match (days, hours, minutes) {
                (0, 0, m) => format!("{m} mins"),
                (0, h, m) => format!("{h} hours, {m} mins"),
                (d, h, m) => format!("{d} days, {h} hours, {m} mins"),
            }
        }
    }
}