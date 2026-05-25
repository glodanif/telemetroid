pub struct OsInfo {
    pub name: Option<String>,
    pub host: Option<String>,
    pub kernel: Option<String>,
    pub age: Option<String>,
    pub uptime: Option<String>,
    pub load_avg: Option<(f32, f32, f32)>,
}
