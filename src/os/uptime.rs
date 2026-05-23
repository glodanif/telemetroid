pub struct Uptime {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
}

impl Uptime {
    pub fn collect() -> Option<Self> {
        let total_minutes = read_uptime_seconds()? / 60;
        Some(Self {
            days: total_minutes / 1440,
            hours: (total_minutes % 1440) / 60,
            minutes: total_minutes % 60,
        })
    }
}

#[cfg(target_os = "linux")]
fn read_uptime_seconds() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/uptime").ok()?;
    let seconds: f64 = content.split_whitespace().next()?.parse().ok()?;
    Some(seconds as u64)
}

#[cfg(target_os = "macos")]
fn read_uptime_seconds() -> Option<u64> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let output = std::process::Command::new("sysctl")
        .args(["-n", "kern.boottime"])
        .output()
        .ok()?;
    let content = String::from_utf8_lossy(&output.stdout);
    // Format: { sec = 1748111111, usec = 0 }
    let boot_sec: u64 = content.split("sec = ").nth(1)?.split(',').next()?.trim().parse().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(now.saturating_sub(boot_sec))
}

impl std::fmt::Display for Uptime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.days, self.hours, self.minutes) {
            (0, 0, m) => write!(f, "{m} mins"),
            (0, h, m) => write!(f, "{h} hours, {m} mins"),
            (d, h, m) => write!(f, "{d} days, {h} hours, {m} mins"),
        }
    }
}
