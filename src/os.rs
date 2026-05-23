use crate::os::os_info::OsInfo;

pub mod linux_info_source;
pub mod os_info;

pub fn collect_os_info() -> OsInfo {
    linux_info_source::collect_info()
}
