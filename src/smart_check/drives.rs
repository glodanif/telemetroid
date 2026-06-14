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

    pub fn len(&self) -> usize {
        self.ata.len() + self.nvme.len()
    }

    pub fn is_any_not_failed(&self) -> bool {
        self.ata.iter().any(|r| r.is_ok()) || self.nvme.iter().any(|r| r.is_ok())
    }
}

impl Display for Drives {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            writeln!(f, "No drives found")?;
        }
        let mut has_ata = false;
        if !self.ata.is_empty() {
            has_ata = true;
            writeln!(f, "ATA Drives:")?;
            for (i, drive) in self.ata.iter().enumerate() {
                match drive {
                    Ok(info) => writeln!(f, "{}. {}", i + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", i + 1, err)?,
                }
            }
        }
        if !self.nvme.is_empty() {
            if has_ata {
                writeln!(f, "\n")?;
            }
            writeln!(f, "NVMe Drives:")?;
            for (i, drive) in self.nvme.iter().enumerate() {
                match drive {
                    Ok(info) => writeln!(f, "{}. {}", i + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", i + 1, err)?,
                }
            }
        }
        Ok(())
    }
}
