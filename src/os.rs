use crate::os::info_source::OsInfoSource;

pub mod os_info;
pub mod info_source;
#[cfg(not(target_os = "macos"))]
pub mod linux_info_source;

pub fn get_os_info_source() -> Box<dyn OsInfoSource> {
    Box::new(linux_info_source::LinuxInfoSource)
}
