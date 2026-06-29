pub struct HardwareInfo {
    pub cpu: Option<CpuInfo>,
    pub gpu: Vec<GpuInfo>,
    pub swap: Option<MemoryInfo>,
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskInfo>,
    pub network: Vec<NetworkInfo>,
}

pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub frequency: String,
    pub temperature: Option<f32>,
}

pub enum GpuType {
    Discrete,
    Integrated,
}

pub struct GpuInfo {
    pub name: String,
    pub temperature: Option<f32>,
    pub gpu_type: Option<GpuType>,
}

pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
}

pub struct DiskInfo {
    /// Mountpoint for a mounted filesystem, or the device name for an unmounted drive.
    pub label: String,
    pub filesystem: String,
    /// Filesystem size when mounted, otherwise the raw device size.
    pub size: u64,
    /// Used bytes when mounted; `None` for an unmounted drive (no usage stats).
    pub used: Option<u64>,
}

pub struct NetworkInfo {
    pub name: String,
    pub ip_address: Option<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
