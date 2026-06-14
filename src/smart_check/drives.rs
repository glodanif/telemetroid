use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::smart_check_result::SmartCheckFailure;
use std::fmt::{Display, Formatter};

pub struct Drives {
    pub ata: Vec<Result<AtaDriveInfo, SmartCheckFailure>>,
    pub nvme: Vec<Result<NvmeDriveInfo, SmartCheckFailure>>,
}

impl Display for Drives {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.ata.is_empty() {
            writeln!(f, "ATA Drives:")?;
            for (i, drive) in self.ata.iter().enumerate() {
                match drive {
                    Ok(info) => writeln!(f, "{}. {}", i + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", i + 1, err)?,
                }
            }
            Ok(())
        } else if !self.nvme.is_empty() {
            writeln!(f, "NVMe Drives:")?;
            for (i, drive) in self.nvme.iter().enumerate() {
                match drive {
                    Ok(info) => writeln!(f, "{}. NVMe Drive Info: {:?}", i + 1, info)?,
                    Err(err) => writeln!(f, "{}. Failed: {}", i + 1, err)?,
                }
            }
            Ok(())
        } else {
            writeln!(f, "No drives found")
        }
    }
}
