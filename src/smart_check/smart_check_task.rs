use crate::smart_check::basic_device_info::DeviceInterface;
use crate::smart_check::drives::Drives;
use crate::smart_check::smart_check_error::SmartCheckError;
use crate::smart_check::smart_ctl_interface::{
    get_ata_drive_info, get_nvme_drive_info, scan_drives,
};

pub fn get_drives_info() -> Result<Drives, SmartCheckError> {
    let drives = scan_drives()?;
    let mut ata_results = Vec::new();
    let mut nvme_results = Vec::new();
    for drive in drives.iter() {
        let name = drive.name.as_str();
        match drive.protocol {
            DeviceInterface::Ata => ata_results.push(run(name, get_ata_drive_info)),
            DeviceInterface::Nvme => nvme_results.push(run(name, get_nvme_drive_info)),
            DeviceInterface::Unsupported => {}
        }
    }
    Ok(Drives {
        ata: ata_results,
        nvme: nvme_results,
    })
}

fn run<T, E: ToString>(
    name: &str,
    f: impl FnOnce(&str) -> Result<T, E>,
) -> Result<T, SmartCheckError> {
    f(name).map_err(|e| SmartCheckError::CommandExecutionError(name.to_string(), e.to_string()))
}
