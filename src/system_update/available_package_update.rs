use std::fmt::{Display, Formatter, Result};

pub struct AvailablePackageUpdate {
    pub name: String,
    pub from_version: String,
    pub to_version: String,
}

impl Display for AvailablePackageUpdate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} {} -> {}",
            self.name, self.from_version, self.to_version
        )
    }
}
