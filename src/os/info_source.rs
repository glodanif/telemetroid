use crate::os::os_info::OsInfo;

pub trait OsInfoSource: Send {
    fn collect_info(&self) -> OsInfo;
}
