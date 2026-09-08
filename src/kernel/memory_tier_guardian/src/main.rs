// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LOG_PREFIX: &str = "[memory-tier-rust]";

fn body_analysis_path() -> PathBuf {
    env::var("PROJECT_ROOT")
        .map(PathBuf::from)
        .ok()
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("data/monitor/body_calibration/pressure_analysis.json")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MemMetrics {
    total_gb: f64,
    available_gb: f64,
    swap_total_gb: f64,
    swap_used_gb: f64,
    zram_total_gb: f64,
    zram_used_gb: f64,
    psi_some_avg10: f64,
    psi_full_avg10: f64,
    vram_total_mb: i32,
    vram_used_mb: i32,
    gpu_util_percent: i32,
    gpu_temp_c: i32,
    swap_activity_pages_sec: f64,
    major_faults_per_sec: f64,
    oom_kills_delta: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BodyBaseline {
    active: bool,
    snapshots: i32,
    swap_p75_pages_sec: f64,
    swap_p95_pages_sec: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StatePayload {
    timestamp: u64,
    stage: String,
    reasons: Vec<String>,
    body_baseline: Option<BodyBaseline>,
    metrics: MemMetrics,
    state_path: String,
}

fn env_int(name: &str, default: i32) -> i32 {
    env::var(name)
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default)
}

fn env_float(name: &str, default: f64) -> f64 {
    env::var(name)
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(default)
}

fn env_flag(name: &str, default: bool) -> bool {
    match env::var(name).ok() {
        Some(val) => {
            let v = val.trim().to_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        None => default,
    }
}

fn log(msg: &str) {
    println!("{} {}", LOG_PREFIX, msg);
    let _ = io::stdout().flush();
}

fn read_meminfo() -> (f64, f64) {
    let mut total_kb = 0.0;
    let mut available_kb = 0.0;
    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                total_kb = parse_kb(&line);
            } else if line.starts_with("MemAvailable:") {
                available_kb = parse_kb(&line);
            }
        }
    }
    (total_kb / 1024.0 / 1024.0, available_kb / 1024.0 / 1024.0)
}

fn parse_kb(line: &str) -> f64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn read_psi_memory() -> (f64, f64) {
    let mut some = 0.0;
    let mut full = 0.0;
    if let Ok(file) = File::open("/proc/pressure/memory") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("some ") {
                some = parse_avg10(&line);
            } else if line.starts_with("full ") {
                full = parse_avg10(&line);
            }
        }
    }
    (some, full)
}

fn parse_avg10(line: &str) -> f64 {
    for field in line.split_whitespace() {
        if field.starts_with("avg10=") {
            if let Some(val_str) = field.split('=').nth(1) {
                return val_str.parse().unwrap_or(0.0);
            }
        }
    }
    0.0
}

fn read_swaps() -> (f64, f64, f64, f64) {
    let mut total_b = 0.0;
    let mut used_b = 0.0;
    let mut ztotal_b = 0.0;
    let mut zused_b = 0.0;

    if let Ok(file) = File::open("/proc/swaps") {
        let reader = BufReader::new(file);
        // Skip header line
        for line in reader.lines().map_while(Result::ok).skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let filename = parts[0];
                let size_kb = parts[2].parse::<f64>().unwrap_or(0.0);
                let used_kb = parts[3].parse::<f64>().unwrap_or(0.0);
                
                let size_b = size_kb * 1024.0;
                let used_b_val = used_kb * 1024.0;

                total_b += size_b;
                used_b += used_b_val;

                if filename.contains("zram") {
                    ztotal_b += size_b;
                    zused_b += used_b_val;
                }
            }
        }
    }
    (
        total_b / 1024.0 / 1024.0 / 1024.0,
        used_b / 1024.0 / 1024.0 / 1024.0,
        ztotal_b / 1024.0 / 1024.0 / 1024.0,
        zused_b / 1024.0 / 1024.0 / 1024.0,
    )
}

fn read_gpu() -> (i32, i32, i32, i32) {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=memory.used,memory.total,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = text.trim().split(',').map(|s| s.trim()).collect();
            if parts.len() == 4 {
                let used = parts[0].parse().unwrap_or(0);
                let total = parts[1].parse().unwrap_or(0);
                let util = parts[2].parse().unwrap_or(0);
                let temp = parts[3].parse().unwrap_or(0);
                return (used, total, util, temp);
            }
        }
    }
    (0, 0, 0, 0)
}

fn read_vmstat_counters() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open("/proc/vmstat") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let key = parts[0];
                if key == "pswpin" || key == "pswpout" || key == "pgmajfault" || key == "oom_kill" {
                    if let Ok(val) = parts[1].parse() {
                        map.insert(key.to_string(), val);
                    }
                }
            }
        }
    }
    map
}

fn load_body_baseline(enabled: bool, min_snapshots: i32) -> Option<serde_json::Value> {
    if !enabled {
        return None;
    }
    let data = fs::read_to_string(body_analysis_path()).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    let snapshots = json.get("total_snapshots")?.as_i64()? as i32;
    if snapshots < min_snapshots {
        return None;
    }
    Some(json)
}

fn resolve_state_path() -> PathBuf {
    let mut candidates = vec![PathBuf::from("/run/omnimind_memory_tier_state.json")];
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        candidates.push(PathBuf::from(runtime_dir).join("omnimind_memory_tier_state.json"));
    }
    candidates.push(PathBuf::from("/tmp/omnimind_memory_tier_state.json"));

    for candidate in candidates {
        if let Some(parent) = candidate.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::OpenOptions::new().write(true).create(true).truncate(false).open(&candidate).is_ok() {
            return candidate;
        }
    }
    PathBuf::from("/tmp/omnimind_memory_tier_state.json")
}

fn stop_service(unit: &str) {
    let output = Command::new("systemctl").args(["stop", unit]).output();
    match output {
        Ok(out) => {
            if out.status.success() {
                log(&format!("stopped service: {}", unit));
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                log(&format!("failed to stop {}: {}", unit, err.trim()));
            }
        }
        Err(e) => {
            log(&format!("failed to execute systemctl stop for {}: {}", unit, e));
        }
    }
}

fn maybe_apply_actions(stage: &str, enable_heavy_stop: bool) {
    if stage != "critical" || !enable_heavy_stop {
        return;
    }
    for unit in &[
        "omnimind-autonomous-loop.service",
        "omnimind-cosmic.service",
        "omnimind-p2p-circulation.service",
    ] {
        stop_service(unit);
    }
}

fn main() {
    let poll_sec = env_int("OMNIMIND_MEMORY_POLL_SEC", 30) as u64;
    let mem_warn_gb = env_float("OMNIMIND_MEMORY_WARN_GB", 4.0);
    let mem_crit_gb = env_float("OMNIMIND_MEMORY_CRIT_GB", 2.5);
    let swap_warn_pct = env_float("OMNIMIND_SWAP_WARN_PCT", 50.0);
    let swap_crit_pct = env_float("OMNIMIND_SWAP_CRIT_PCT", 75.0);
    let zram_warn_pct = env_float("OMNIMIND_ZRAM_WARN_PCT", 80.0);
    let zram_crit_pct = env_float("OMNIMIND_ZRAM_CRIT_PCT", 90.0);
    let psi_warn = env_float("OMNIMIND_PSI_MEM_WARN", 3.0);
    let psi_crit = env_float("OMNIMIND_PSI_MEM_CRIT", 6.0);
    let vram_warn_pct = env_float("OMNIMIND_VRAM_WARN_PCT", 75.0);
    let vram_crit_pct = env_float("OMNIMIND_VRAM_CRIT_PCT", 90.0);
    let enable_heavy_stop = env_flag("OMNIMIND_MEMORY_STOP_HEAVY", false);
    let body_baseline_enable = env_flag("OMNIMIND_BODY_BASELINE_ENABLE", true);
    let body_min_snapshots = env_int("OMNIMIND_BODY_BASELINE_MIN_SNAPSHOTS", 100);

    log(&format!(
        "started Rust guardian (warn_mem={}GB, crit_mem={}GB, swap_warn={}%, swap_crit={}%, zram_warn={}%, zram_crit={}%, psi_warn={}, psi_crit={}, vram_warn={}%, vram_crit={}%, stop_heavy={}, body_baseline={}, min_body_snapshots={})",
        mem_warn_gb, mem_crit_gb, swap_warn_pct, swap_crit_pct, zram_warn_pct, zram_crit_pct, psi_warn, psi_crit, vram_warn_pct, vram_crit_pct, enable_heavy_stop, body_baseline_enable, body_min_snapshots
    ));

    let state_path = resolve_state_path();
    let mut prev_vmstat = read_vmstat_counters();
    let mut prev_instant = Instant::now();

    loop {
        thread::sleep(Duration::from_secs(poll_sec.max(5)));

        let curr_vmstat = read_vmstat_counters();
        let now = Instant::now();
        let elapsed = now.duration_since(prev_instant).as_secs_f64().max(1.0);
        prev_instant = now;

        // Calculate dynamics
        let pswpin = (curr_vmstat.get("pswpin").unwrap_or(&0) - prev_vmstat.get("pswpin").unwrap_or(&0)) as f64 / elapsed;
        let pswpout = (curr_vmstat.get("pswpout").unwrap_or(&0) - prev_vmstat.get("pswpout").unwrap_or(&0)) as f64 / elapsed;
        let pgmajfault = (curr_vmstat.get("pgmajfault").unwrap_or(&0) - prev_vmstat.get("pgmajfault").unwrap_or(&0)) as f64 / elapsed;
        let oom_kills = (curr_vmstat.get("oom_kill").unwrap_or(&0) - prev_vmstat.get("oom_kill").unwrap_or(&0)) as i32;

        let swap_activity = (pswpin + pswpout).max(0.0);
        let major_faults = pgmajfault.max(0.0);

        prev_vmstat = curr_vmstat;

        // Gather statistics
        let (total_gb, available_gb) = read_meminfo();
        let (swap_total_gb, swap_used_gb, zram_total_gb, zram_used_gb) = read_swaps();
        let (psi_some_avg10, psi_full_avg10) = read_psi_memory();
        let (vram_used_mb, vram_total_mb, gpu_util_percent, gpu_temp_c) = read_gpu();

        let metrics = MemMetrics {
            total_gb: (total_gb * 100.0).round() / 100.0,
            available_gb: (available_gb * 100.0).round() / 100.0,
            swap_total_gb: (swap_total_gb * 100.0).round() / 100.0,
            swap_used_gb: (swap_used_gb * 100.0).round() / 100.0,
            zram_total_gb: (zram_total_gb * 100.0).round() / 100.0,
            zram_used_gb: (zram_used_gb * 100.0).round() / 100.0,
            psi_some_avg10,
            psi_full_avg10,
            vram_total_mb,
            vram_used_mb,
            gpu_util_percent,
            gpu_temp_c,
            swap_activity_pages_sec: (swap_activity * 100.0).round() / 100.0,
            major_faults_per_sec: (major_faults * 100.0).round() / 100.0,
            oom_kills_delta: oom_kills.max(0),
        };

        let swap_used_pct = if metrics.swap_total_gb <= 0.0 { 0.0 } else { (metrics.swap_used_gb / metrics.swap_total_gb) * 100.0 };
        let zram_used_pct = if metrics.zram_total_gb <= 0.0 { 0.0 } else { (metrics.zram_used_gb / metrics.zram_total_gb) * 100.0 };
        let vram_used_pct = if metrics.vram_total_mb <= 0 { 0.0 } else { (metrics.vram_used_mb as f64 / metrics.vram_total_mb as f64) * 100.0 };

        // Determine decision
        let body_baseline = load_body_baseline(body_baseline_enable, body_min_snapshots);
        let mut reasons = Vec::new();
        let mut stage = String::from("normal");
        let baseline_repr: Option<BodyBaseline>;

        if let Some(baseline) = body_baseline {
            let swap_rate_stats = baseline.get("swap_rate_stats");
            let swap_p75 = swap_rate_stats.and_then(|s| s.get("p75")).and_then(|v| v.as_f64()).unwrap_or(500.0);
            let swap_p95 = swap_rate_stats.and_then(|s| s.get("p95")).and_then(|v| v.as_f64()).unwrap_or(swap_p75 * 2.0).max(500.0);

            baseline_repr = Some(BodyBaseline {
                active: true,
                snapshots: baseline.get("total_snapshots").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                swap_p75_pages_sec: (swap_p75 * 100.0).round() / 100.0,
                swap_p95_pages_sec: (swap_p95 * 100.0).round() / 100.0,
            });

            if metrics.oom_kills_delta > 0 {
                reasons.push(String::from("oom_kill_observed"));
                stage = String::from("critical");
            } else if metrics.available_gb <= mem_crit_gb {
                reasons.push(String::from("mem_available_below_critical_floor"));
                stage = String::from("critical");
            } else if metrics.psi_full_avg10 >= psi_crit && metrics.swap_activity_pages_sec > swap_p95 {
                reasons.push(String::from("psi_full_and_swap_outlier"));
                stage = String::from("critical");
            } else if metrics.available_gb <= mem_warn_gb {
                reasons.push(String::from("mem_available_below_warning_floor"));
                stage = String::from("high");
            } else if metrics.swap_activity_pages_sec > swap_p95 {
                reasons.push(String::from("swap_activity_above_body_p95"));
                stage = String::from("high");
            } else if metrics.psi_full_avg10 >= psi_crit {
                reasons.push(String::from("psi_full_above_critical_absolute"));
                stage = String::from("high");
            } else if metrics.psi_full_avg10 >= psi_warn && metrics.swap_activity_pages_sec > swap_p75 {
                reasons.push(String::from("psi_full_and_swap_above_body_p75"));
                stage = String::from("high");
            } else if swap_used_pct >= swap_crit_pct && metrics.swap_activity_pages_sec > swap_p75 {
                reasons.push(String::from("swap_capacity_and_churn_high"));
                stage = String::from("high");
            } else if zram_used_pct >= zram_crit_pct && metrics.swap_activity_pages_sec > swap_p75 {
                reasons.push(String::from("zram_capacity_and_churn_high"));
                stage = String::from("high");
            } else if vram_used_pct >= vram_crit_pct && metrics.gpu_temp_c >= 85 {
                reasons.push(String::from("vram_and_gpu_thermal_high"));
                stage = String::from("high");
            } else if metrics.psi_some_avg10 >= 1.0
                || swap_used_pct >= 25.0
                || zram_used_pct >= zram_warn_pct
                || vram_used_pct >= vram_warn_pct
            {
                reasons.push(String::from("habitual_pressure_within_body_envelope"));
                stage = String::from("elevated");
            } else {
                reasons.push(String::from("body_nominal"));
                stage = String::from("normal");
            }
        } else {
            baseline_repr = Some(BodyBaseline {
                active: false,
                snapshots: 0,
                swap_p75_pages_sec: 0.0,
                swap_p95_pages_sec: 0.0,
            });

            if metrics.available_gb <= mem_crit_gb {
                reasons.push(String::from("mem_available_below_critical_floor"));
            }
            if swap_used_pct >= swap_crit_pct {
                reasons.push(String::from("swap_capacity_above_critical_absolute"));
            }
            if zram_used_pct >= zram_crit_pct {
                reasons.push(String::from("zram_capacity_above_critical_absolute"));
            }
            if metrics.psi_full_avg10 >= psi_crit {
                reasons.push(String::from("psi_full_above_critical_absolute"));
            }
            if vram_used_pct >= vram_crit_pct {
                reasons.push(String::from("vram_capacity_above_critical_absolute"));
            }

            if !reasons.is_empty() {
                stage = String::from("critical");
            } else {
                if metrics.available_gb <= mem_warn_gb {
                    reasons.push(String::from("mem_available_below_warning_floor"));
                }
                if swap_used_pct >= swap_warn_pct {
                    reasons.push(String::from("swap_capacity_above_warning_absolute"));
                }
                if zram_used_pct >= zram_warn_pct {
                    reasons.push(String::from("zram_capacity_above_warning_absolute"));
                }
                if metrics.psi_full_avg10 >= psi_warn {
                    reasons.push(String::from("psi_full_above_warning_absolute"));
                }
                if vram_used_pct >= vram_warn_pct {
                    reasons.push(String::from("vram_capacity_above_warning_absolute"));
                }

                if !reasons.is_empty() {
                    stage = String::from("high");
                } else if metrics.psi_some_avg10 >= 1.0 || swap_used_pct >= 25.0 {
                    stage = String::from("elevated");
                    reasons.push(String::from("psi_some_or_swap_capacity_elevated_absolute"));
                } else {
                    reasons.push(String::from("absolute_nominal"));
                }
            }
        }

        // Read cache to check transition
        let mut changed = true;
        if let Ok(data) = fs::read_to_string(&state_path) {
            if let Ok(cached_json) = serde_json::from_str::<serde_json::Value>(&data) {
                if let (Some(c_stage), Some(c_reasons)) = (cached_json.get("stage").and_then(|s| s.as_str()), cached_json.get("reasons").and_then(|r| r.as_array())) {
                    let c_reasons_vec: Vec<String> = c_reasons.iter().flat_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    if c_stage == stage && c_reasons_vec == reasons {
                        changed = false;
                    }
                }
            }
        }

        if changed {
            log(&format!(
                "stage={} mem_avail={:.2}GB swap={:.1}% zram={:.1}% psi_full10={:.2} swap_rate={:.1}/s vram={:.1}% gpu_util={}% gpu_temp={}C reasons={}",
                stage, metrics.available_gb, swap_used_pct, zram_used_pct, metrics.psi_full_avg10, metrics.swap_activity_pages_sec, vram_used_pct, metrics.gpu_util_percent, metrics.gpu_temp_c, reasons.join(",")
            ));
        }

        maybe_apply_actions(&stage, enable_heavy_stop);

        let payload = StatePayload {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            stage: stage.clone(),
            reasons,
            body_baseline: baseline_repr,
            metrics,
            state_path: state_path.to_string_lossy().into_owned(),
        };

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&state_path, json_str);
        }
    }
}
