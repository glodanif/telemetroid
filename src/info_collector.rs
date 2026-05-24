use crate::info_collector::info::Info;

pub mod hardware_info;
pub mod info;
pub mod linux_info_source;
pub mod os_info;
pub mod linux_os_info_collector;
pub mod linux_hardware_info_collector;

pub fn collect() -> Info {
    linux_info_source::collect_info()
}
