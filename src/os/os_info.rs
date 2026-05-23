use std::fmt::{Display, Formatter, Result};

pub struct
OsInfo {
    pub name: String,
    pub host: Option<String>,
    pub kernel: String,
    pub age: String,
    pub uptime: String,
}

impl Display for OsInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(host) = &self.host {
            writeln!(f, "Host: {host}")?;
        }
        write!(
            f,
            "OS: {}\nKernel: {}\nAge: {}\nUptime: {}",
            self.name, self.kernel, self.age, self.uptime
        )
    }
}