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

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let os = &self.os;
        let hw = &self.hardware;

        match &os.name {
            Some(name) => writeln!(f, "OS: {}", name)?,
            None => writeln!(f, "OS: Unknown")?,
        }
        if let Some(host) = &os.host { writeln!(f, "Host: {}", host)?; }
        if let Some(kernel) = &os.kernel { writeln!(f, "Kernel: {}", kernel)?; }
        if let Some(age) = &os.age { writeln!(f, "Age: {}", age)?; }
        if let Some(uptime) = &os.uptime { writeln!(f, "Uptime: {}", uptime)?; }

        match &hw.cpu {
            Some(cpu) => {
                write!(f, "CPU: {} ({}) @ {}", cpu.name, cpu.cores, cpu.frequency)?;
                if let Some(temp) = cpu.temperature { write!(f, " - {:.1}°C", temp)?; }
                writeln!(f)?;
            }
            None => writeln!(f, "CPU: Unknown")?,
        }

        let gpu_count = hw.gpu.len();
        for (i, gpu) in hw.gpu.iter().enumerate() {
            if gpu_count > 1 { write!(f, "GPU {}: {}", i + 1, gpu.name)?; }
            else { write!(f, "GPU: {}", gpu.name)?; }
            if let Some(temp) = gpu.temperature { write!(f, " - {:.1}°C", temp)?; }
            if let Some(gpu_type) = &gpu.gpu_type {
                let label = match gpu_type { GpuType::Discrete => "Discrete", GpuType::Integrated => "Integrated" };
                write!(f, " [{}]", label)?;
            }
            writeln!(f)?;
        }

        match &hw.memory {
            Some(mem) => {
                let pct = if mem.total > 0 { mem.used * 100 / mem.total } else { 0 };
                writeln!(f, "Memory: {} / {} ({}%)", format_bytes(mem.used), format_bytes(mem.total), pct)?;
            }
            None => writeln!(f, "Memory: Unknown")?,
        }

        if let Some(swap) = &hw.swap {
            let pct = if swap.total > 0 { swap.used * 100 / swap.total } else { 0 };
            writeln!(f, "Swap: {} / {} ({}%)", format_bytes(swap.used), format_bytes(swap.total), pct)?;
        }

        if hw.disks.is_empty() {
            writeln!(f, "Disk: Unknown")?;
        } else {
            for disk in &hw.disks {
                let pct = if disk.total > 0 { disk.used * 100 / disk.total } else { 0 };
                writeln!(f, "Disk ({}): {} / {} ({}%) - {}", disk.mount, format_bytes(disk.used), format_bytes(disk.total), pct, disk.filesystem)?;
            }
        }

        Ok(())
    }
}
