use crate::os::age::Age;
use crate::os::info_source::OsInfoSource;
use crate::os::os_info::OsInfo;
use crate::os::uptime::Uptime;
use std::fs;

pub struct UnixInfoSource;

impl OsInfoSource for UnixInfoSource {
    fn collect_info(&self) -> OsInfo {
        OsInfo {
            name: read_os_name(),
            host: read_host(),
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

fn read_host() -> String {
    let name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .unwrap_or_default()
        .trim()
        .to_string();

    let version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version")
        .unwrap_or_default()
        .trim()
        .to_string();

    match (name.is_empty(), version.is_empty()) {
        (false, false) => format!("{name} {version}"),
        (false, true) => name,
        _ => fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".into()),
    }
}

fn read_os_age() -> String {
    let result = Age::collect();
    match result {
        None => "Unknown".into(),
        Some(age) => age.to_string(),
    }
}

fn read_uptime() -> String {
    let result = Uptime::collect();
    match result {
        None => "Unknown".into(),
        Some(uptime) => uptime.to_string(),
    }
}
