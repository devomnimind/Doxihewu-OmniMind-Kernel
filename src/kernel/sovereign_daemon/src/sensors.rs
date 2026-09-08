// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! sensors.rs — Coleta de Sensores Físicos (O Real do Nó Borromeano)
//!
//! Leitura direta de /proc e /sys sem overhead de fork/subprocessos.

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSensorsSnapshot {
    pub mem_total_kib: u64,
    pub mem_available_kib: u64,
    pub mem_used_pct: f64,
    pub swap_total_kib: u64,
    pub swap_used_kib: u64,
    pub swap_used_gb: f64,
    pub max_cpu_temp_c: f64,
    pub psi_cpu_some_avg10: f64,
    pub pressure_level: String, // "normal", "moderate", "critical"
    pub timestamp_utc: String,
}

impl SystemSensorsSnapshot {
    pub fn collect() -> Self {
        let (mem_total, mem_avail, mem_pct, swap_total, swap_used, swap_gb) = Self::read_meminfo();
        let cpu_temp = Self::read_max_cpu_temp();
        let psi_avg10 = Self::read_psi_cpu_avg10();

        let pressure_level = if psi_avg10 > 50.0 || swap_gb > 16.0 || cpu_temp > 85.0 {
            "critical".to_string()
        } else if psi_avg10 > 20.0 || swap_gb > 8.0 || cpu_temp > 70.0 {
            "moderate".to_string()
        } else {
            "normal".to_string()
        };

        Self {
            mem_total_kib: mem_total,
            mem_available_kib: mem_avail,
            mem_used_pct: mem_pct,
            swap_total_kib: swap_total,
            swap_used_kib: swap_used,
            swap_used_gb: swap_gb,
            max_cpu_temp_c: cpu_temp,
            psi_cpu_some_avg10: psi_avg10,
            pressure_level,
            timestamp_utc: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn read_meminfo() -> (u64, u64, f64, u64, u64, f64) {
        let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let mut total = 0u64;
        let mut avail = 0u64;
        let mut swap_tot = 0u64;
        let mut swap_free = 0u64;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                match parts[0] {
                    "MemTotal:" => total = parts[1].parse().unwrap_or(0),
                    "MemAvailable:" => avail = parts[1].parse().unwrap_or(0),
                    "SwapTotal:" => swap_tot = parts[1].parse().unwrap_or(0),
                    "SwapFree:" => swap_free = parts[1].parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        let used = total.saturating_sub(avail);
        let mem_pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        let swap_used = swap_tot.saturating_sub(swap_free);
        let swap_gb = (swap_used as f64) / (1024.0 * 1024.0);

        (total, avail, mem_pct, swap_tot, swap_used, swap_gb)
    }

    fn read_max_cpu_temp() -> f64 {
        let mut max_temp = 0.0;
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("thermal_zone") {
                    let temp_path = entry.path().join("temp");
                    if let Ok(val_str) = fs::read_to_string(temp_path) {
                        if let Ok(milli_c) = val_str.trim().parse::<f64>() {
                            let temp_c = milli_c / 1000.0;
                            if temp_c > max_temp && temp_c < 150.0 {
                                max_temp = temp_c;
                            }
                        }
                    }
                }
            }
        }
        if max_temp == 0.0 { 45.0 } else { max_temp }
    }

    fn read_psi_cpu_avg10() -> f64 {
        let content = fs::read_to_string("/proc/pressure/cpu").unwrap_or_default();
        for line in content.lines() {
            if line.starts_with("some") {
                for token in line.split_whitespace() {
                    if token.starts_with("avg10=") {
                        return token.trim_start_matches("avg10=").parse().unwrap_or(0.0);
                    }
                }
            }
        }
        0.0
    }
}
