use std::fs;

pub struct BotMetrics {
    pub ram_mb: f64,
    pub cpu_percent: f64,
    pub uptime_str: String,
}

/// Mengambil data statistik RAM murni, CPU, dan Uptime bot secara akurat
pub fn get_bot_metrics() -> BotMetrics {
    let (cpu_percent, process_age_sec) = get_cpu_and_uptime();

    BotMetrics {
        ram_mb: get_ram_usage_mb(),
        cpu_percent,
        uptime_str: format_uptime(process_age_sec as u64),
    }
}

fn get_ram_usage_mb() -> f64 {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        return kb / 1024.0;
                    }
                }
            }
        }
    }
    0.0
}

fn get_cpu_and_uptime() -> (f64, f64) {
    let stat_str = match fs::read_to_string("/proc/self/stat") {
        Ok(s) => s,
        Err(_) => return (0.0, 0.0),
    };
    let uptime_str = match fs::read_to_string("/proc/uptime") {
        Ok(s) => s,
        Err(_) => return (0.0, 0.0),
    };

    let sys_uptime: f64 = uptime_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);

    let parts: Vec<&str> = stat_str.split_whitespace().collect();
    if parts.len() < 22 {
        return (0.0, 0.0);
    }

    let utime: f64 = parts[13].parse().unwrap_or(0.0);
    let stime: f64 = parts[14].parse().unwrap_or(0.0);
    let starttime: f64 = parts[21].parse().unwrap_or(0.0);

    let clock_ticks = 100.0; // Hz bawaan Linux
    let total_time_sec = (utime + stime) / clock_ticks;
    let process_age_sec = (sys_uptime - (starttime / clock_ticks)).max(0.0);

    let cpu_percent = if process_age_sec > 0.0 {
        ((total_time_sec / process_age_sec) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    (cpu_percent, process_age_sec)
}

fn format_uptime(total_seconds: u64) -> String {
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
