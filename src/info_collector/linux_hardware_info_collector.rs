use std::fs;
use std::path::Path;
use std::process::Command;

use crate::info_collector::hardware_info::{
    CpuInfo, DiskInfo, GpuInfo, GpuType, HardwareInfo, MemoryInfo,
};

pub fn collect() -> HardwareInfo {
    HardwareInfo {
        cpu: read_cpu_info(),
        gpu: read_gpu_info(),
        swap: read_swap_info(),
        memory: read_memory_info(),
        disks: read_disks_info(),
    }
}

fn read_cpu_info() -> Option<CpuInfo> {
    let content = fs::read_to_string("/proc/cpuinfo").ok()?;

    let raw_name = content
        .lines()
        .find(|l| l.starts_with("model name"))?
        .split(':')
        .nth(1)?
        .trim()
        .to_string();

    let name = simplify_cpu_name(&raw_name);

    let cores = content
        .lines()
        .filter(|l| l.starts_with("processor"))
        .count() as u32;

    let frequency = read_max_cpu_freq()
        .or_else(|| read_current_cpu_freq(&content))
        .unwrap_or_default();

    let temperature = read_cpu_temperature();

    Some(CpuInfo {
        name,
        cores,
        frequency,
        temperature,
    })
}

fn simplify_cpu_name(name: &str) -> String {
    let mut name = name
        .replace("(R)", "")
        .replace("(TM)", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    for suffix in [" with Radeon Graphics", " Processor", " CPU"] {
        if let Some(pos) = name.rfind(suffix) {
            let trimmed = name[..pos].trim_end().to_string();
            if !trimmed.is_empty() {
                name = trimmed;
                break;
            }
        }
    }

    if let Some(pos) = name.rfind(" @") {
        let trimmed = name[..pos].trim_end().to_string();
        if !trimmed.is_empty() {
            name = trimmed;
        }
    }

    if let Some(core_pos) = name.rfind("-Core") {
        let prefix = &name[..core_pos];
        let digit_count = prefix
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .count();
        if digit_count > 0 {
            name = prefix[..prefix.len() - digit_count].trim_end().to_string();
        }
    }

    name
}

fn read_max_cpu_freq() -> Option<String> {
    let freq_khz = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()?;
    Some(format!("{:.2} GHz", freq_khz / 1_000_000.0))
}

fn read_current_cpu_freq(cpuinfo: &str) -> Option<String> {
    let mhz = cpuinfo
        .lines()
        .find(|l| l.starts_with("cpu MHz"))?
        .split(':')
        .nth(1)?
        .trim()
        .parse::<f64>()
        .ok()?;
    Some(format!("{:.2} GHz", mhz / 1000.0))
}

fn read_cpu_temperature() -> Option<f32> {
    if let Some(t) = read_hwmon_temp("k10temp", &["Tdie", "Tctl", "Tccd1"]) {
        return Some(t);
    }
    if let Some(t) = read_hwmon_temp("coretemp", &["Package id 0"]) {
        return Some(t);
    }
    read_thermal_zone_cpu_temp()
}

fn read_hwmon_temp(driver: &str, labels: &[&str]) -> Option<f32> {
    let entries = fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match fs::read_to_string(path.join("name")) {
            Ok(n) => n,
            Err(_) => continue,
        };
        if name.trim() != driver {
            continue;
        }
        for i in 1..=20u8 {
            let temp_path = path.join(format!("temp{}_input", i));
            if !temp_path.exists() {
                break;
            }
            let label_path = path.join(format!("temp{}_label", i));
            if let Ok(label) = fs::read_to_string(&label_path) {
                if labels.contains(&label.trim()) {
                    if let Ok(t) = fs::read_to_string(&temp_path) {
                        if let Ok(millideg) = t.trim().parse::<f32>() {
                            return Some(millideg / 1000.0);
                        }
                    }
                }
            }
        }
    }
    None
}

fn read_thermal_zone_cpu_temp() -> Option<f32> {
    let entries = fs::read_dir("/sys/class/thermal").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let zone_type = match fs::read_to_string(path.join("type")) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let zone_type = zone_type.trim();
        if zone_type == "x86_pkg_temp" || zone_type.contains("cpu") {
            if let Ok(t) = fs::read_to_string(path.join("temp")) {
                if let Ok(millideg) = t.trim().parse::<f32>() {
                    return Some(millideg / 1000.0);
                }
            }
        }
    }
    None
}

fn read_gpu_info() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    gpus.extend(read_nvidia_gpus());
    gpus.extend(read_drm_gpus());
    gpus
}

fn read_nvidia_gpus() -> Vec<GpuInfo> {
    let output = match Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(2, ',');
            let name = parts.next().unwrap_or("").trim().to_string();
            let temperature = parts.next().and_then(|s| s.trim().parse::<f32>().ok());
            GpuInfo {
                name,
                temperature,
                gpu_type: Some(GpuType::Discrete),
            }
        })
        .collect()
}

fn read_drm_gpus() -> Vec<GpuInfo> {
    let entries = match fs::read_dir("/sys/class/drm") {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut cards: Vec<_> = entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.starts_with("card") && !s.contains('-')
        })
        .collect();

    cards.sort_by_key(|e| e.file_name());

    let mut gpus = Vec::new();

    for entry in cards {
        let device_path = entry.path().join("device");

        let vendor = match fs::read_to_string(device_path.join("vendor")) {
            Ok(v) => v.trim().to_string(),
            Err(_) => continue,
        };

        match vendor.as_str() {
            "0x10de" => continue, // NVIDIA handled by nvidia-smi
            "0x1002" | "0x8086" => {}
            _ => continue,
        }

        let name = gpu_name_from_lspci(&device_path).unwrap_or_else(|| match vendor.as_str() {
            "0x8086" => "Intel GPU".to_string(),
            _ => "AMD GPU".to_string(),
        });

        let temperature = read_drm_gpu_temp(&device_path);
        let gpu_type = determine_gpu_type(&device_path, &vendor);

        gpus.push(GpuInfo {
            name,
            temperature,
            gpu_type,
        });
    }

    gpus
}

fn gpu_name_from_lspci(device_path: &Path) -> Option<String> {
    let real_path = device_path.canonicalize().ok()?;
    let pci_full = real_path.file_name()?.to_str()?.to_string();
    // sysfs format: "0000:03:00.0", lspci format: "03:00.0"
    let pci_addr = if pci_full.len() > 7 && pci_full.as_bytes().get(4) == Some(&b':') {
        pci_full[5..].to_string()
    } else {
        pci_full
    };

    let output = Command::new("lspci")
        .args(["-s", &pci_addr])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().next()?;

    // Format: "03:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Raphael (rev c1)"
    let after_addr = line.splitn(2, ' ').nth(1)?;
    let name_part = after_addr.splitn(2, ": ").nth(1)?.trim();

    Some(simplify_lspci_name(name_part))
}

fn simplify_lspci_name(name: &str) -> String {
    let name = if let Some(pos) = name.rfind(" (rev ") {
        name[..pos].trim()
    } else {
        name.trim()
    };

    // Extract from "[AMD/ATI] Device" bracket notation
    if let Some(end) = name.rfind(']') {
        if let Some(start) = name[..end].rfind('[') {
            let bracket = &name[start + 1..end];
            let device = name[end + 1..].trim();
            if !device.is_empty() {
                let vendor = if bracket.contains("AMD") {
                    "AMD"
                } else if bracket.contains("Intel") {
                    "Intel"
                } else {
                    return device.to_string();
                };
                return format!("{} {}", vendor, device);
            }
        }
    }

    // Strip verbose vendor prefixes
    for prefix in ["Intel Corporation ", "Advanced Micro Devices, Inc. "] {
        if let Some(stripped) = name.strip_prefix(prefix) {
            return stripped.to_string();
        }
    }

    name.to_string()
}

fn read_drm_gpu_temp(device_path: &Path) -> Option<f32> {
    let entries = match fs::read_dir(device_path.join("hwmon")) {
        Ok(e) => e,
        Err(_) => return None,
    };
    for entry in entries.flatten() {
        let hwmon = entry.path();
        for label_name in ["edge", "junction"] {
            for i in 1..=5u8 {
                let label_path = hwmon.join(format!("temp{}_label", i));
                if let Ok(label) = fs::read_to_string(&label_path) {
                    if label.trim() == label_name {
                        let temp_path = hwmon.join(format!("temp{}_input", i));
                        if let Ok(t) = fs::read_to_string(&temp_path) {
                            if let Ok(millideg) = t.trim().parse::<f32>() {
                                return Some(millideg / 1000.0);
                            }
                        }
                    }
                }
            }
        }
        // Fallback: first available temperature
        if let Ok(t) = fs::read_to_string(hwmon.join("temp1_input")) {
            if let Ok(millideg) = t.trim().parse::<f32>() {
                return Some(millideg / 1000.0);
            }
        }
    }
    None
}

fn determine_gpu_type(device_path: &Path, vendor: &str) -> Option<GpuType> {
    if vendor == "0x8086" {
        return Some(GpuType::Integrated);
    }
    // AMD APUs share system memory as VRAM (typically 256 MiB–2 GiB).
    // All modern AMD discrete GPUs have at least 4 GiB of dedicated GDDR.
    if let Ok(vram_str) = fs::read_to_string(device_path.join("mem_info_vram_total")) {
        if let Ok(vram) = vram_str.trim().parse::<u64>() {
            return Some(if vram >= 4 * 1024 * 1024 * 1024 {
                GpuType::Discrete
            } else {
                GpuType::Integrated
            });
        }
    }
    None
}

fn read_memory_info() -> Option<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let total = parse_meminfo_kb(&content, "MemTotal")? * 1024;
    let available = parse_meminfo_kb(&content, "MemAvailable")? * 1024;
    Some(MemoryInfo {
        total,
        used: total.saturating_sub(available),
    })
}

fn read_swap_info() -> Option<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let total = parse_meminfo_kb(&content, "SwapTotal")? * 1024;
    if total == 0 {
        return None;
    }
    let free = parse_meminfo_kb(&content, "SwapFree").unwrap_or(0) * 1024;
    Some(MemoryInfo {
        total,
        used: total.saturating_sub(free),
    })
}

fn parse_meminfo_kb(content: &str, key: &str) -> Option<u64> {
    content
        .lines()
        .find(|l| l.starts_with(key))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

fn read_disks_info() -> Vec<DiskInfo> {
    let output = match Command::new("df")
        .args(["-B1", "--output=source,size,used,target,fstype"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return vec![],
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    stdout
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                return None;
            }
            if !parts[0].starts_with("/dev/") {
                return None;
            }
            Some(DiskInfo {
                total: parts[1].parse().ok()?,
                used: parts[2].parse().ok()?,
                mount: parts[3].to_string(),
                filesystem: parts[4].to_string(),
            })
        })
        .collect()
}
