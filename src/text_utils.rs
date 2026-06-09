pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    const PB: u64 = TB * 1024;

    match bytes {
        b if b >= PB => format!("{:.2}P", b as f64 / PB as f64),
        b if b >= TB => format!("{:.2}T", b as f64 / TB as f64),
        b if b >= GB => format!("{:.2}G", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2}M", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2}K", b as f64 / KB as f64),
        b => format!("{}B", b),
    }
}

pub fn format_number(n: f32) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as u32)
    } else {
        format!("{:.1}", n)
    }
}
