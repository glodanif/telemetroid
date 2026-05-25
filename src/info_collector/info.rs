use crate::info_collector::hardware_info::{GpuType, HardwareInfo};
use crate::info_collector::os_info::OsInfo;
use std::fmt::{Display, Formatter, Result};

pub struct Info {
    pub os: OsInfo,
    pub hardware: HardwareInfo,
}

fn format_bytes(bytes: u64) -> String {
    const TIB: u64 = 1 << 40;
    const GIB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;
    const KIB: u64 = 1 << 10;
    if bytes >= TIB { format!("{:.2} TiB", bytes as f64 / TIB as f64) }
    else if bytes >= GIB { format!("{:.2} GiB", bytes as f64 / GIB as f64) }
    else if bytes >= MIB { format!("{:.2} MiB", bytes as f64 / MIB as f64) }
    else if bytes >= KIB { format!("{:.2} KiB", bytes as f64 / KIB as f64) }
    else { format!("{} B", bytes) }
}

fn temp_indicator(temp: f32) -> &'static str {
    if temp < 60.0 { "🟢" } else if temp < 80.0 { "🟡" } else { "🔴" }
}

fn usage_bar(used: u64, total: u64, blocks: usize) -> String {
    if total == 0 {
        return "⬜".repeat(blocks);
    }
    let pct = (used * 100 / total) as usize;
    let filled = ((pct * blocks + 50) / 100).min(blocks);
    let icon = if pct >= 80 { "🟥" } else if pct >= 60 { "🟨" } else { "🟩" };
    format!("{}{}", icon.repeat(filled), "⬜".repeat(blocks - filled))
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let os = &self.os;
        let hw = &self.hardware;

        match &os.name {
            Some(name) => writeln!(f, "<b>OS:</b> {}", name)?,
            None => writeln!(f, "<b>OS:</b> Unknown")?,
        }
        if let Some(host) = &os.host { writeln!(f, "<b>Host:</b> {}", host)?; }
        if let Some(kernel) = &os.kernel { writeln!(f, "<b>Kernel:</b> {}", kernel)?; }
        if let Some(age) = &os.age { writeln!(f, "<b>Age:</b> {}", age)?; }
        if let Some(uptime) = &os.uptime { writeln!(f, "<b>Uptime:</b> {}", uptime)?; }
        if let Some((a, b, c)) = os.load_avg {
            let cores = hw.cpu.as_ref().map(|c| c.cores as f32).unwrap_or(1.0);
            let pct = |v: f32| (v / cores * 100.0).round() as u32;
            writeln!(f, "<b>Load:</b> {}% · {}% · {}%  <i>(1/5/15 min)</i>", pct(a), pct(b), pct(c))?;
        }

        writeln!(f, "\n──────────────────────\n")?;

        match &hw.cpu {
            Some(cpu) => {
                write!(f, "<b>CPU:</b> {} ({} cores) @ {}", cpu.name, cpu.cores, cpu.frequency)?;
                if let Some(temp) = cpu.temperature {
                    write!(f, "  {} {:.1}°C", temp_indicator(temp), temp)?;
                }
                writeln!(f)?;
            }
            None => writeln!(f, "<b>CPU:</b> Unknown")?,
        }

        let gpu_count = hw.gpu.len();
        for (i, gpu) in hw.gpu.iter().enumerate() {
            let label = if gpu_count > 1 { format!("GPU {}:", i + 1) } else { "GPU:".to_string() };
            write!(f, "<b>{}</b> {}", label, gpu.name)?;
            if let Some(gpu_type) = &gpu.gpu_type {
                let t = match gpu_type { GpuType::Discrete => "Discrete", GpuType::Integrated => "Integrated" };
                write!(f, " [{}]", t)?;
            }
            if let Some(temp) = gpu.temperature {
                write!(f, "  {} {:.1}°C", temp_indicator(temp), temp)?;
            }
            writeln!(f)?;
        }

        match &hw.memory {
            Some(mem) => {
                let pct = if mem.total > 0 { mem.used * 100 / mem.total } else { 0 };
                writeln!(f, "<b>Memory:</b> {} / {} ({}%)", format_bytes(mem.used), format_bytes(mem.total), pct)?;
                writeln!(f, "{}", usage_bar(mem.used, mem.total, 10))?;
            }
            None => writeln!(f, "<b>Memory:</b> Unknown")?,
        }

        if let Some(swap) = &hw.swap {
            let pct = if swap.total > 0 { swap.used * 100 / swap.total } else { 0 };
            writeln!(f, "<b>Swap:</b> {} / {} ({}%)", format_bytes(swap.used), format_bytes(swap.total), pct)?;
            writeln!(f, "{}", usage_bar(swap.used, swap.total, 10))?;
        }

        writeln!(f)?;

        if hw.disks.is_empty() {
            writeln!(f, "<b>Disk:</b> Unknown")?;
        } else {
            for disk in &hw.disks {
                let pct = if disk.total > 0 { disk.used * 100 / disk.total } else { 0 };
                writeln!(f, "<b>Disk {}:</b> {} / {} ({}%) — {}", disk.mount, format_bytes(disk.used), format_bytes(disk.total), pct, disk.filesystem)?;
                writeln!(f, "{}", usage_bar(disk.used, disk.total, 10))?;
            }
        }

        if !hw.network.is_empty() {
            writeln!(f)?;
            writeln!(f, "<b>Network:</b>")?;
            for net in &hw.network {
                let ip = net.ip_address.as_deref().unwrap_or("—");
                writeln!(f, "{} {}  ↓ {}  ↑ {}", net.name, ip, format_bytes(net.rx_bytes), format_bytes(net.tx_bytes))?;
            }
        }

        Ok(())
    }
}
