use crate::info_collector::info::Info;
use crate::info_collector::{linux_hardware_info_collector, linux_os_info_collector};

pub fn collect_info() -> Info {
    Info {
        os: linux_os_info_collector::collect(),
        hardware: linux_hardware_info_collector::collect(),
    }
}
