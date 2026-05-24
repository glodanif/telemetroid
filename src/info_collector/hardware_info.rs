pub struct HardwareInfo {
    pub cpu: Option<CpuInfo>,
    pub gpu: Vec<GpuInfo>,
    pub swap: Option<MemoryInfo>,
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskInfo>,
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
    pub total: u64,
    pub used: u64,
    pub mount: String,
    pub filesystem: String,
}
