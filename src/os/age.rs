use std::fs;
use std::time::SystemTime;

pub struct Age {
    pub years: u64,
    pub months: u64,
    pub days: u64,
}

impl Age {
    pub fn collect() -> Option<Self> {
        let path = if cfg!(target_os = "macos") { "/var/db/.AppleSetupDone" } else { "/" };
        let meta = fs::metadata(path).ok()?;
        let created = meta.created().ok()?;
        let elapsed = SystemTime::now().duration_since(created).ok()?;

        let total_days = elapsed.as_secs() / 86400;

        Some(Self {
            years: total_days / 365,
            months: (total_days % 365) / 30,
            days: total_days % 30,
        })
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.years, self.months, self.days) {
            (0, 0, d) => write!(f, "{d} days"),
            (0, m, d) => write!(f, "{m} months, {d} days"),
            (y, m, d) => write!(f, "{y} years, {m} months, {d} days"),
        }
    }
}
