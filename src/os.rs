use crate::os::info_source::OsInfoSource;

pub mod os_info;
pub mod info_source;
pub mod age;
pub mod uptime;
#[cfg(not(target_os = "macos"))]
pub mod unix_info_source;
#[cfg(target_os = "macos")]
pub mod mac_info_source;

pub fn get_os_info_source() -> Box<dyn OsInfoSource> {
    #[cfg(target_os = "macos")]
    return Box::new(mac_info_source::MacInfoSource);
    #[cfg(not(target_os = "macos"))]
    return Box::new(unix_info_source::UnixInfoSource);
}