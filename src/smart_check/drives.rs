use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::smart_check_result::SmartCheckFailure;
use std::fmt::{Display, Formatter};

pub struct Drives {
    pub ata: Vec<Result<AtaDriveInfo, SmartCheckFailure>>,
    pub nvme: Vec<Result<NvmeDriveInfo, SmartCheckFailure>>,
}

impl Drives {
    pub fn is_empty(&self) -> bool {
        self.ata.is_empty() && self.nvme.is_empty()
    }

    pub fn is_any_not_failed(&self) -> bool {
        self.ata.iter().any(|r| r.is_ok()) || self.nvme.iter().any(|r| r.is_ok())
    }

    pub fn len(&self) -> usize {
        self.ata.len() + self.nvme.len()
    }
}

impl Display for Drives {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            writeln!(f, "No drives found")?;
        }
        let mut index = 0;
        if !self.ata.is_empty() {
            for drive in self.ata.iter() {
                match drive {
                    Ok(info) => writeln!(f, "{}. {}", index + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", index + 1, err)?,
                }
                writeln!(f, "")?;
                index += 1;
            }
        }
        if !self.nvme.is_empty() {
            for drive in self.nvme.iter() {
                match drive {
                    Ok(info) => writeln!(f, "{}. {}", index + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", index + 1, err)?,
                }
                writeln!(f, "")?;
                index += 1;
            }
        }
        Ok(())
    }
}
