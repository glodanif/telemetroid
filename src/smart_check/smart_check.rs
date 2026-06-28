use crate::smart_check::basic_device_info::DeviceInterface;
use crate::smart_check::drives::Drives;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_ctl_interface::{
    check_ata_drive, check_nvme_drive, scan_drives,
};

pub fn get_drives_info() -> Result<Drives, SmartCheckError> {
    let drives = scan_drives()?;
    let mut ata_results = Vec::new();
    let mut nvme_results = Vec::new();
    for drive in drives.iter() {
        let name = drive.name.as_str();
        match drive.protocol {
            DeviceInterface::Ata => ata_results.push(check_ata_drive(name)),
            DeviceInterface::Nvme => nvme_results.push(check_nvme_drive(name)),
            DeviceInterface::Unsupported => {}
        }
    }
    Ok(Drives {
        ata: ata_results,
        nvme: nvme_results,
    })
}
