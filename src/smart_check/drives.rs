use crate::smart_check::ata_self_test::ata_drive_info::AtaDriveInfo;
use crate::smart_check::nvme_self_test::nvme_drive_info::NvmeDriveInfo;
use crate::smart_check::smart_check_result::SmartCheckFailure;

pub struct Drives {
    pub ata: Vec<Result<AtaDriveInfo, SmartCheckFailure>>,
    pub nvme: Vec<Result<NvmeDriveInfo, SmartCheckFailure>>,
}
