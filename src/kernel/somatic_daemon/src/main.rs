// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
#![allow(clippy::type_complexity)]
//
// omnimind-somatic-daemon v0.2.0 — A-4 Fase 1 (Shadow Mode)
//
// Contrato: tasks/A-4-somatic-daemon-rust/CONTRACT.md
// Alinhamentos: Seção 16 (Deep) + Seção 17 (AGY)
//
// Shadow mode: escreve em arquivos *_rust_shadow.* e usa source
// "somatic_daemon_rust_shadow" no SQLite. NUNCA escreve no banco
// real com source do daemon Python (alinhamento 16.4).
//
// Cadências reais (alinhamento 16.3 / drop-ins systemd):
//   CYCLE=15s, MIN_SLEEP=3.0s, HEALTH=120s
//
// Hooks Python (IntegrationLoop, SomaticBridge, SharedWorkspace)
// ficam em processo separado — este daemon é apenas sensoriamento
// + persistência + telemetria (alinhamento 17.3.5).

use chrono::{Local, Utc};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// ─── Constants ──────────────────────────────────────────────────────────────

const SHADOW_SOURCE: &str = "somatic_daemon_rust_shadow";
const LOG_PREFIX: &str = "[somatic-daemon-rust-shadow]";

/// Calculate age in seconds from an ISO 8601 timestamp string to now.
fn age_seconds_from_iso(iso_str: &str) -> Option<f64> {
    if iso_str.trim().is_empty() {
        return None;
    }
    let clean = iso_str.replace("Z", "+00:00");
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&clean) {
        let now = chrono::Utc::now();
        let secs = (now.timestamp_millis() - dt.with_timezone(&chrono::Utc).timestamp_millis()) as f64 / 1000.0;
        Some(secs)
    } else {
        None
    }
}

// ─── Config (from env, matching Python daemon + drop-ins) ───────────────────

struct Config {
    project_root: PathBuf,
    cycle_interval: f64,
    min_sleep: f64,
    health_interval: f64,
    db_path: PathBuf,
    shadow_state_file: PathBuf,
    shadow_mesh_json: PathBuf,
    shadow_mesh_md: PathBuf,
    log_file: PathBuf,
    storage_oracle_path: PathBuf,
}

impl Config {
    fn from_env() -> Self {
        let project_root = env::var("PROJECT_ROOT")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                env::var("CARGO_MANIFEST_DIR").ok().and_then(|d| {
                    PathBuf::from(d).join("../../..").canonicalize().ok()
                })
            })
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let cycle_interval = env::var("OMNIMIND_SOMATIC_CYCLE_S")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(15.0); // Alinhamento 16.3: 15s (não 5s)

        let min_sleep = env::var("OMNIMIND_SOMATIC_MIN_SLEEP_S")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(3.0); // Alinhamento 16.3: 3.0s (não 1.0s)

        let health_interval = env::var("OMNIMIND_SOMATIC_HEALTH_INTERVAL_S")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(120.0); // Alinhamento 16.3: 120s (não 60s)

        let db_path = project_root.join("data/monitor/somatic_mesh_runtime.sqlite");
        let data_somatic = project_root.join("data/somatic");

        Self {
            project_root: project_root.clone(),
            cycle_interval,
            min_sleep,
            health_interval,
            db_path,
            shadow_state_file: data_somatic.join("daemon_state_rust_shadow.json"),
            shadow_mesh_json: project_root.join("runtime_config/somatic_mesh_runtime_rust_shadow_latest.json"),
            shadow_mesh_md: project_root.join("reports_runtime/somatic_mesh_runtime_rust_shadow_latest.md"),
            log_file: data_somatic.join("somatic_daemon_rust_shadow.log"),
            storage_oracle_path: project_root.join("runtime_config/storage_mount_health_oracle_latest.json"),
        }
    }
}

// ─── Logging ────────────────────────────────────────────────────────────────

fn log(cfg: &Config, msg: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("{} {} {}", now, LOG_PREFIX, msg);
    println!("{}", line);
    // Append to log file (best-effort)
    if let Some(parent) = cfg.log_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&cfg.log_file)
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "{}", line)
        });
}

// ─── UUID (alinhamento 17.2: usar crate uuid, não subprocess uuidgen) ───────

fn gen_uuid() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

// ─── RAPL Energy Telemetry (alinhamento 16.6: leitura via sudo cat) ─────────

fn read_rapl_energy() -> Value {
    let rapl_paths = [
        "/sys/class/powercap/intel-rapl:0/energy_uj",
        "/sys/class/powercap/intel-rapl:0:0/energy_uj",
        "/sys/class/powercap/intel-rapl:0:1/energy_uj",
        "/sys/class/powercap/intel-rapl:1/energy_uj",
    ];

    let mut domains = Vec::new();

    for path in &rapl_paths {
        // Try direct read first
        let content = fs::read_to_string(path)
            .ok()
            .or_else(|| {
                // Fallback: sudo -n cat (alinhamento 16.6)
                process::Command::new("sudo")
                    .args(["-n", "cat", path])
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            String::from_utf8_lossy(&o.stdout).trim().to_string().into()
                        } else {
                            None
                        }
                    })
            });

        if let Some(content) = content {
            let energy_uj: i64 = content.trim().parse().unwrap_or(0);
            let domain_name = if path.contains(":0:0") {
                "core"
            } else if path.contains(":0:1") {
                "uncore"
            } else if path.contains(":1") {
                "psys"
            } else {
                "package-0"
            };
            domains.push(json!({
                "path": path,
                "domain_name": domain_name,
                "energy_uj": energy_uj,
            }));
        }
    }

    json!({
        "service_name": SHADOW_SOURCE,
        "rapl_available": !domains.is_empty(),
        "rapl_domains": domains,
    })
}

// ─── Storage Oracle (ler JSON do runtime_config) ────────────────────────────

fn read_storage_oracle(cfg: &Config) -> Value {
    if !cfg.storage_oracle_path.exists() {
        return json!({});
    }
    match fs::read_to_string(&cfg.storage_oracle_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(json!({})),
        Err(_) => json!({}),
    }
}

// ─── System Sensors (meminfo, swap, PSI) ────────────────────────────────────

fn read_meminfo() -> (f64, f64) {
    // Returns (mem_available_gb, swap_used_ratio)
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut mem_available_kb: f64 = 0.0;
    let mut swap_total_kb: f64 = 0.0;
    let mut swap_free_kb: f64 = 0.0;

    for line in meminfo.lines() {
        if line.starts_with("MemAvailable:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                mem_available_kb = parts[1].parse().unwrap_or(0.0);
            }
        } else if line.starts_with("SwapTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                swap_total_kb = parts[1].parse().unwrap_or(0.0);
            }
        } else if line.starts_with("SwapFree:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                swap_free_kb = parts[1].parse().unwrap_or(0.0);
            }
        }
    }

    let mem_available_gb = mem_available_kb / 1024.0 / 1024.0;
    let swap_used_ratio = if swap_total_kb > 0.0 {
        (swap_total_kb - swap_free_kb) / swap_total_kb
    } else {
        0.0
    };

    (mem_available_gb, swap_used_ratio)
}

fn read_disk_usage() -> (f64, f64, f64) {
    // Returns (root_used_pct, home_used_pct, var_used_pct)
    fn disk_used_pct(mount: &str) -> f64 {
        let output = process::Command::new("df")
            .args(["-h", "--output=pcent", mount])
            .output()
            .ok();
        if let Some(o) = output {
            let text = String::from_utf8_lossy(&o.stdout);
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() >= 2 {
                let pct_str = lines[1].trim().trim_end_matches('%');
                return pct_str.parse::<f64>().unwrap_or(0.0);
            }
        }
        0.0
    }
    (disk_used_pct("/"), disk_used_pct("/home"), disk_used_pct("/var"))
}

fn read_cpu_temp() -> f64 {
    // Read max CPU temperature from /sys/class/thermal
    let mut max_temp: f64 = 0.0;
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("thermal_zone") {
                let temp_path = path.join("temp");
                if let Ok(content) = fs::read_to_string(&temp_path) {
                    let temp_mc: f64 = content.trim().parse().unwrap_or(0.0);
                    let temp_c = temp_mc / 1000.0;
                    if temp_c > max_temp {
                        max_temp = temp_c;
                    }
                }
            }
        }
    }
    max_temp
}

/// Read CPU usage percentage from /proc/stat (first line = aggregate).
/// Returns 0-100. Reads two samples with a tiny sleep for delta.
fn read_cpu_usage_pct() -> f64 {
    fn read_proc_stat() -> Option<(u64, u64)> {
        let content = fs::read_to_string("/proc/stat").ok()?;
        let first_line = content.lines().next()?;
        if !first_line.starts_with("cpu ") {
            return None;
        }
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }
        // user, nice, system, idle, iowait...
        let user: u64 = parts[1].parse().ok()?;
        let nice: u64 = parts[2].parse().ok()?;
        let system: u64 = parts[3].parse().ok()?;
        let idle: u64 = parts[4].parse().ok()?;
        let iowait: u64 = if parts.len() > 5 { parts[5].parse().unwrap_or(0) } else { 0 };
        let total = user + nice + system + idle + iowait;
        let busy = user + nice + system;
        Some((busy, total))
    }
    let (busy1, total1) = match read_proc_stat() {
        Some(v) => v,
        None => return 0.0,
    };
    std::thread::sleep(Duration::from_millis(50));
    let (busy2, total2) = match read_proc_stat() {
        Some(v) => v,
        None => return 0.0,
    };
    let total_delta = (total2 as f64) - (total1 as f64);
    let busy_delta = (busy2 as f64) - (busy1 as f64);
    if total_delta > 0.0 {
        (busy_delta / total_delta * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}

/// Calculate somatic state matching Python somatic_cell.py logic:
/// - energy_level = (memory_available + cpu_available) / 2
/// - pain_signals: sobreaquecimento, exaustão, erro_crítico
/// - pleasure_signals: vitalidade, temperatura_ideal, bem_estar
fn calculate_somatic_state(cpu_temp: f64, mem_available_gb: f64, cpu_usage_pct: f64) -> (f64, Vec<String>, Vec<String>) {
    // Memory available as percentage (assume 32GB total for this host)
    let mem_total_gb = 32.0;
    let mem_available_pct = ((mem_available_gb / mem_total_gb) * 100.0).clamp(0.0, 100.0);
    let cpu_available_pct = 100.0 - cpu_usage_pct;
    let energy_level = (mem_available_pct + cpu_available_pct) / 2.0;

    // Pain signals (matching Python thresholds)
    let mut pain_signals = Vec::new();
    if cpu_temp > 85.0 {
        pain_signals.push("sobreaquecimento".to_string());
    }
    if energy_level < 20.0 {
        pain_signals.push("exaustão".to_string());
    }

    // Pleasure signals
    let mut pleasure_signals = Vec::new();
    if energy_level > 80.0 {
        pleasure_signals.push("vitalidade".to_string());
    }
    if cpu_temp > 30.0 && cpu_temp < 70.0 {
        pleasure_signals.push("temperatura_ideal".to_string());
    }

    (energy_level, pain_signals, pleasure_signals)
}

// ─── SQLite Writer (alinhamento 16.4: source=somatic_daemon_rust_shadow) ────

fn ensure_schema(conn: &Connection) {
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS somatic_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_uid TEXT NOT NULL UNIQUE,
            timestamp_utc TEXT NOT NULL,
            source TEXT NOT NULL,
            cycle INTEGER,
            phi_estimate REAL,
            body_temperature_c REAL,
            energy_pct REAL,
            resonance REAL,
            pain_signals_n INTEGER,
            pleasure_signals_n INTEGER,
            fragment_count INTEGER,
            mem_available_gb REAL,
            swap_used_ratio REAL,
            root_used_pct REAL,
            home_used_pct REAL,
            var_used_pct REAL,
            oracle_overall_level TEXT,
            oracle_status TEXT,
            service_states_json TEXT,
            notes_json TEXT,
            runtime_energy_json TEXT,
            raw_json TEXT,
            created_utc TEXT NOT NULL
        )",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_somatic_snapshots_time
            ON somatic_snapshots(timestamp_utc DESC)",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_somatic_snapshots_source
            ON somatic_snapshots(source)",
        [],
    );
}

struct SomaticMetrics {
    phi_estimate: f64,
    temperature: f64,
    energy: f64,
    resonance: f64,
    pain_signals: Vec<String>,
    pleasure_signals: Vec<String>,
    mem_available_gb: f64,
    swap_used_ratio: f64,
    root_used_pct: f64,
    home_used_pct: f64,
    var_used_pct: f64,
    rapl: Value,
    oracle: Value,
}

fn record_snapshot(cfg: &Config, cycle: i64, metrics: &SomaticMetrics) -> Result<(), String> {
    if let Some(parent) = cfg.db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(&cfg.db_path)
        .map_err(|e| format!("SQLite open: {}", e))?;
    ensure_schema(&conn);

    let snap_uid = gen_uuid();
    let now_utc = Utc::now().to_rfc3339();
    let oracle_level = metrics.oracle.get("overall_level")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let oracle_status = metrics.oracle.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("shadow");

    let rapl_str = serde_json::to_string(&metrics.rapl).unwrap_or_default();
    let notes = json!({
        "shadow_mode": true,
        "phase": "A-4-Fase-3",
        "pain_signals": metrics.pain_signals,
        "pleasure_signals": metrics.pleasure_signals,
    });
    let notes_str = serde_json::to_string(&notes).unwrap_or_default();

    let pain_n = metrics.pain_signals.len() as i64;
    let pleasure_n = metrics.pleasure_signals.len() as i64;

    conn.execute(
        "INSERT INTO somatic_snapshots (
            snapshot_uid, timestamp_utc, source, cycle, phi_estimate,
            body_temperature_c, energy_pct, resonance, pain_signals_n,
            pleasure_signals_n, fragment_count, mem_available_gb,
            swap_used_ratio, root_used_pct, home_used_pct, var_used_pct,
            oracle_overall_level, oracle_status, service_states_json,
            notes_json, runtime_energy_json, raw_json, created_utc
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
        params![
            snap_uid,
            now_utc,
            SHADOW_SOURCE,
            cycle,
            metrics.phi_estimate,
            metrics.temperature,
            metrics.energy,
            metrics.resonance,
            pain_n,
            pleasure_n,
            0, // fragment_count
            metrics.mem_available_gb,
            metrics.swap_used_ratio,
            metrics.root_used_pct,
            metrics.home_used_pct,
            metrics.var_used_pct,
            oracle_level,
            oracle_status,
            "{}", // service_states_json
            notes_str,
            rapl_str,
            "{}", // raw_json
            now_utc,
        ],
    ).map_err(|e| format!("SQLite insert: {}", e))?;

    Ok(())
}

// ─── daemon_state_rust_shadow.json writer ───────────────────────────────────

fn write_shadow_state(cfg: &Config, status: &str, cycle: i64, metrics: &SomaticMetrics) {
    let payload = json!({
        "status": status,
        "cycle_count": cycle,
        "last_health_check": Local::now().to_rfc3339(),
        "timestamp": Local::now().to_rfc3339(),
        "pid": process::id(),
        "phi_estimate": metrics.phi_estimate,
        "somatic_summary": {
            "status": "shadow",
            "current_state": {
                "temperature": metrics.temperature,
                "energy": metrics.energy,
                "resonance": metrics.resonance,
                "pain_signals": metrics.pain_signals,
                "pleasure_signals": metrics.pleasure_signals
            },
            "fragment_opportunities": 0,
            "cycles_processed": cycle,
            "last_narrative": null,
            "spatial_awareness": {
                "core": 0.0,
                "consciousness": 0.0,
                "memory": 0.0
            }
        },
        "storage_oracle": metrics.oracle,
        "organic_memory_budget": {
            "available": false,
            "items_n": 0,
            "approx_bytes": 0,
            "budget_hint_mb": 0,
            "json_surface_preview": null,
            "preview": [],
            "service": null,
            "service_name": null,
            "source": null,
            "top_preview_paths": [],
            "total_items": 0,
            "updated_at_utc": null,
        },
        "witness_incremental_preview": {
            "available": false,
            "preview": [],
            "status": "shadow",
        },
        "latent_drive_buffer": {
            "counts": {},
            "db_path": cfg.project_root.join("data/monitor/omnimind_latent_drive.sqlite").to_string_lossy(),
            "latent_cooling": [],
            "ready_to_reinject": [],
            "threshold_context": {},
        },
        "runtime_energy_telemetry": metrics.rapl,
        "shadow_mode": true,
        "shadow_source": SHADOW_SOURCE,
    });

    if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
        if let Some(parent) = cfg.shadow_state_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cfg.shadow_state_file, json_str + "\n");
    }
}

// ─── materialize_latest_exports shadow (alinhamento 16.5: shape byte-a-byte) ─

fn build_shadow_payload(cfg: &Config) -> Value {
    let conn = match Connection::open(&cfg.db_path) {
        Ok(c) => c,
        Err(_) => return json!({"status": "error", "message": "SQLite open failed"}),
    };

    // Count snapshots and vagus events
    let snapshots_n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM somatic_snapshots",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let vagus_events_n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM vagus_events",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    // Latest snapshot — filter by SHADOW_SOURCE to avoid picking up Python rows
    let latest: Option<(String, String, String, i64, f64, Option<f64>, Option<f64>,
                        Option<f64>, Option<i64>, Option<i64>, Option<i64>,
                        Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>,
                        Option<String>, Option<String>,
                        Option<String>, Option<String>, Option<String>, Option<String>)> = conn.query_row(
        "SELECT timestamp_utc, source, snapshot_uid, cycle, phi_estimate,
                body_temperature_c, energy_pct, resonance,
                pain_signals_n, pleasure_signals_n, fragment_count,
                mem_available_gb, swap_used_ratio, root_used_pct, home_used_pct, var_used_pct,
                oracle_overall_level, oracle_status,
                service_states_json, notes_json, runtime_energy_json, raw_json
         FROM somatic_snapshots WHERE source = ?1 ORDER BY timestamp_utc DESC LIMIT 1",
        params![SHADOW_SOURCE],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, f64>(4)?,
            row.get::<_, Option<f64>>(5)?,
            row.get::<_, Option<f64>>(6)?,
            row.get::<_, Option<f64>>(7)?,
            row.get::<_, Option<i64>>(8)?,
            row.get::<_, Option<i64>>(9)?,
            row.get::<_, Option<i64>>(10)?,
            row.get::<_, Option<f64>>(11)?,
            row.get::<_, Option<f64>>(12)?,
            row.get::<_, Option<f64>>(13)?,
            row.get::<_, Option<f64>>(14)?,
            row.get::<_, Option<f64>>(15)?,
            row.get::<_, Option<String>>(16)?,
            row.get::<_, Option<String>>(17)?,
            row.get::<_, Option<String>>(18)?,
            row.get::<_, Option<String>>(19)?,
            row.get::<_, Option<String>>(20)?,
            row.get::<_, Option<String>>(21)?,
        )),
    ).ok();

    // Extract timestamp for age calculation before moving latest
    let latest_snapshot_ts: Option<String> = latest.as_ref().map(|l| l.0.clone());

    let latest_snapshot = match latest {
        Some((ts, src, _uid, cycle, phi, temp, energy, resonance,
              pain_n, pleasure_n, frag_count,
              mem_gb, swap_ratio, root_pct, home_pct, var_pct,
              oracle_level, oracle_status,
              service_states_json, notes_json, runtime_energy_json, _raw_json)) => {
            // Parse JSON columns (best-effort)
            let disk_pressure = json!({
                "root_used_pct": root_pct,
                "home_used_pct": home_pct,
                "var_used_pct": var_pct,
            });
            let memory_pressure = json!({
                "mem_available_gb": mem_gb,
                "swap_used_ratio": swap_ratio,
            });
            let notes: Value = notes_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(json!({}));
            let service_states: Value = service_states_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(json!({}));
            let runtime_energy: Value = runtime_energy_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(json!({}));

            json!({
                "timestamp_utc": ts,
                "source": src,
                "cycle": cycle,
                "phi_estimate": phi,
                "body_temperature_c": temp,
                "energy_pct": energy,
                "resonance": resonance,
                "pain_signals_n": pain_n,
                "pleasure_signals_n": pleasure_n,
                "fragment_count": frag_count,
                "disk_pressure": disk_pressure,
                "memory_pressure": memory_pressure,
                "oracle_overall_level": oracle_level,
                "oracle_status": oracle_status,
                "service_states": service_states,
                "notes": notes,
                "runtime_energy_telemetry": runtime_energy,
            })
        },
        None => json!(null),
    };

    // Recent vagus events (limit 20) — full shape matching _row_to_event()
    let mut recent_events = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT timestamp_utc, source, watched_unit, event_kind, reflex_action,
                severity, reason, line_excerpt, oracle_overall_level, context_json
         FROM vagus_events ORDER BY timestamp_utc DESC LIMIT 20"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            let ctx_json: Option<String> = row.get(9)?;
            let context: Value = ctx_json
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(json!({}));
            Ok(json!({
                "timestamp_utc": row.get::<_, String>(0)?,
                "source": row.get::<_, String>(1)?,
                "watched_unit": row.get::<_, Option<String>>(2)?,
                "event_kind": row.get::<_, String>(3)?,
                "reflex_action": row.get::<_, Option<String>>(4)?,
                "severity": row.get::<_, Option<String>>(5)?,
                "reason": row.get::<_, Option<String>>(6)?,
                "line_excerpt": row.get::<_, Option<String>>(7)?,
                "oracle_overall_level": row.get::<_, Option<String>>(8)?,
                "context": context,
            }))
        }) {
            for row in rows.flatten() {
                recent_events.push(row);
            }
        }
    }

    // Latest vagus event for watchdog_stream
    let latest_event = recent_events.first().cloned().unwrap_or(json!(null));

    // Calculate ages for watchdog_stream
    let latest_snapshot_age_sec = latest_snapshot_ts
        .as_deref()
        .and_then(age_seconds_from_iso);
    let latest_vagus_event_age_sec = latest_event.get("timestamp_utc")
        .and_then(|t| t.as_str())
        .and_then(age_seconds_from_iso);
    let vagus_event_stream_stale = latest_vagus_event_age_sec.map(|age| age > 3600.0).unwrap_or(false);

    // Windows 10m and 60m — matching _build_window_summary()
    let win_10m = build_window_summary(&conn, 10);
    let win_60m = build_window_summary(&conn, 60);

    json!({
        "timestamp_utc": Utc::now().to_rfc3339(),
        "status": "ok",
        "backing_sqlite": cfg.db_path.to_string_lossy(),
        "shadow_mode": true,
        "summary": {
            "snapshots_n": snapshots_n,
            "vagus_events_n": vagus_events_n,
        },
        "latest_snapshot": latest_snapshot,
        "watchdog_stream": {
            "latest_snapshot_age_sec": latest_snapshot_age_sec,
            "latest_vagus_event": latest_event,
            "latest_vagus_event_age_sec": latest_vagus_event_age_sec,
            "vagus_event_stream_stale": vagus_event_stream_stale,
        },
        "windows": {
            "10m": win_10m,
            "60m": win_60m,
        },
        "recent_vagus_events": recent_events,
    })
}

/// Build window summary matching Python _build_window_summary()
fn build_window_summary(conn: &Connection, minutes: i64) -> Value {
    // Query snapshots in the last N minutes
    let sql = format!(
        "SELECT timestamp_utc, source, cycle, phi_estimate, body_temperature_c,
                energy_pct, mem_available_gb, swap_used_ratio,
                root_used_pct, home_used_pct, var_used_pct, oracle_overall_level
         FROM somatic_snapshots
         WHERE timestamp_utc >= datetime('now', '-{} minutes')
         ORDER BY timestamp_utc DESC",
        minutes
    );

    let mut snapshots: Vec<(String, String, i64, f64, Option<f64>, Option<f64>,
                            Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>,
                            Option<String>)> = Vec::new();

    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, Option<f64>>(6)?,
                row.get::<_, Option<f64>>(7)?,
                row.get::<_, Option<f64>>(8)?,
                row.get::<_, Option<f64>>(9)?,
                row.get::<_, Option<f64>>(10)?,
                row.get::<_, Option<String>>(11)?,
            ))
        }) {
            for row in rows.flatten() {
                snapshots.push(row);
            }
        }
    }

    let n = snapshots.len();
    if n == 0 {
        return json!({
            "window_minutes": minutes,
            "snapshots_n": 0,
            "latest_snapshot": null,
            "avg_body_temperature_c": null,
            "max_body_temperature_c": null,
            "avg_energy_pct": null,
            "min_mem_available_gb": null,
            "max_swap_used_ratio": null,
            "max_root_used_pct": null,
            "max_home_used_pct": null,
            "max_var_used_pct": null,
            "oracle_levels": [],
        });
    }

    // Latest snapshot in window
    let latest = &snapshots[0];
    let latest_snap = json!({
        "timestamp_utc": latest.0,
        "source": latest.1,
        "cycle": latest.2,
        "phi_estimate": latest.3,
        "body_temperature_c": latest.4,
        "energy_pct": latest.5,
    });

    // Aggregates
    let temps: Vec<f64> = snapshots.iter().filter_map(|s| s.4).collect();
    let energies: Vec<f64> = snapshots.iter().filter_map(|s| s.5).collect();
    let mems: Vec<f64> = snapshots.iter().filter_map(|s| s.6).collect();
    let swaps: Vec<f64> = snapshots.iter().filter_map(|s| s.7).collect();
    let roots: Vec<f64> = snapshots.iter().filter_map(|s| s.8).collect();
    let homes: Vec<f64> = snapshots.iter().filter_map(|s| s.9).collect();
    let vars: Vec<f64> = snapshots.iter().filter_map(|s| s.10).collect();

    let avg_temp = if !temps.is_empty() { Some(temps.iter().sum::<f64>() / temps.len() as f64) } else { None };
    let max_temp = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg_energy = if !energies.is_empty() { Some(energies.iter().sum::<f64>() / energies.len() as f64) } else { None };
    let min_mem = mems.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_swap = swaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_root = roots.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_home = homes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_var = vars.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Oracle levels breakdown
    let mut oracle_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for s in &snapshots {
        if let Some(ref level) = s.11 {
            *oracle_counts.entry(level.clone()).or_insert(0) += 1;
        }
    }
    let oracle_levels: Vec<Value> = oracle_counts.iter()
        .map(|(k, v)| json!({"level": k, "count": v}))
        .collect();

    json!({
        "window_minutes": minutes,
        "snapshots_n": n,
        "latest_snapshot": latest_snap,
        "avg_body_temperature_c": avg_temp,
        "max_body_temperature_c": max_temp,
        "avg_energy_pct": avg_energy,
        "min_mem_available_gb": min_mem,
        "max_swap_used_ratio": max_swap,
        "max_root_used_pct": max_root,
        "max_home_used_pct": max_home,
        "max_var_used_pct": max_var,
        "oracle_levels": oracle_levels,
    })
}

fn materialize_shadow_exports(cfg: &Config) {
    let payload = build_shadow_payload(cfg);

    // Write JSON
    if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
        if let Some(parent) = cfg.shadow_mesh_json.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&cfg.shadow_mesh_json, json_str + "\n");
    }

    // Write Markdown (simplified shadow version)
    let md = format!(
        "# Somatic Mesh Runtime — Rust Shadow\n\n\
        **Timestamp:** `{}`\n\
        **Status:** `{}`\n\
        **Shadow mode:** `{}`\n\
        **Backing SQLite:** `{}`\n\n\
        ## Summary\n\n\
        - Snapshots: {}\n\
        - Vagus events: {}\n\n\
        ## Latest Snapshot\n\n\
        ```json\n{}\n```\n",
        payload.get("timestamp_utc").and_then(|v| v.as_str()).unwrap_or("N/A"),
        payload.get("status").and_then(|v| v.as_str()).unwrap_or("N/A"),
        payload.get("shadow_mode").and_then(|v| v.as_bool()).unwrap_or(false),
        payload.get("backing_sqlite").and_then(|v| v.as_str()).unwrap_or("N/A"),
        payload["summary"]["snapshots_n"].as_i64().unwrap_or(0),
        payload["summary"]["vagus_events_n"].as_i64().unwrap_or(0),
        serde_json::to_string_pretty(&payload["latest_snapshot"]).unwrap_or_default(),
    );

    if let Some(parent) = cfg.shadow_mesh_md.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&cfg.shadow_mesh_md, md);
}

// ─── WAL Checkpoint (alinhamento 16.3: 2 bancos — sovereign + somatic_mesh) ─

fn wal_checkpoint(cfg: &Config) {
    let banks = [
        cfg.db_path.clone(),
        cfg.project_root.join("data/monitor/sovereign_dodecatiad_runtime.sqlite"),
    ];

    for bank in &banks {
        if !bank.exists() {
            continue;
        }
        match Connection::open(bank) {
            Ok(conn) => {
                // PRAGMA wal_checkpoint returns a row (busy, log, checkpointed)
                match conn.query_row::<(i64, i64, i64), _, _>(
                    "PRAGMA wal_checkpoint(TRUNCATE)", [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                ) {
                    Ok((busy, log_pages, checkpointed)) => {
                        log(cfg, &format!(
                            "WAL checkpoint OK: {} (busy={}, log_pages={}, checkpointed={})",
                            bank.display(), busy, log_pages, checkpointed
                        ));
                    }
                    Err(e) => log(cfg, &format!("WAL checkpoint falhou ({}): {}", bank.display(), e)),
                }
            }
            Err(e) => log(cfg, &format!("SQLite open falhou ({}): {}", bank.display(), e)),
        }
    }
}

// ─── Main Loop ──────────────────────────────────────────────────────────────

fn main() {
    let cfg = Config::from_env();

    log(&cfg, "=== Somatic Daemon Rust Shadow v0.2.0 — A-4 Fase 1 ===");
    log(&cfg, &format!("PROJECT_ROOT: {}", cfg.project_root.display()));
    log(&cfg, &format!("CYCLE={}s, MIN_SLEEP={}s, HEALTH={}s",
        cfg.cycle_interval, cfg.min_sleep, cfg.health_interval));
    log(&cfg, &format!("DB: {}", cfg.db_path.display()));
    log(&cfg, &format!("Shadow state: {}", cfg.shadow_state_file.display()));
    log(&cfg, &format!("Shadow mesh JSON: {}", cfg.shadow_mesh_json.display()));
    log(&cfg, &format!("Source: {}", SHADOW_SOURCE));

    // Signal handlers (SIGINT, SIGTERM via ctrlc; SIGHUP via separate handler)
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        eprintln!("{} Signal recebido, iniciando shutdown...", LOG_PREFIX);
        r.store(false, Ordering::SeqCst);
    });

    // SIGHUP handler (alinhamento: Python tem _reload_handler)
    let reload_flag = Arc::new(AtomicBool::new(false));
    let _rf = reload_flag.clone();
    // Use signal-hook style via raw syscall would be ideal, but ctrlc only
    // handles SIGINT/SIGTERM. For shadow mode, SIGHUP is logged but not
    // critical. We check the flag in the loop.
    // Note: Full SIGHUP support requires `signal-hook` crate (alinhamento 16.8:
    // tokio opcional, mas signal-hook seria ideal para Fase 2).

    // State
    let mut cycle_count: i64 = 0;
    let mut last_health_check = Instant::now();

    // Ensure dirs exist
    let _ = fs::create_dir_all(cfg.shadow_state_file.parent().unwrap_or(Path::new(".")));
    let _ = fs::create_dir_all(cfg.shadow_mesh_json.parent().unwrap_or(Path::new(".")));
    let _ = fs::create_dir_all(cfg.shadow_mesh_md.parent().unwrap_or(Path::new(".")));

    // Activation snapshot (matching Python: start() grava "activated")
    let (mem_gb, swap_ratio) = read_meminfo();
    let (root_pct, home_pct, var_pct) = read_disk_usage();
    let cpu_temp = read_cpu_temp();
    let cpu_usage = read_cpu_usage_pct();
    let rapl = read_rapl_energy();
    let oracle = read_storage_oracle(&cfg);
    let (energy, pain, pleasure) = calculate_somatic_state(cpu_temp, mem_gb, cpu_usage);

    let init_metrics = SomaticMetrics {
        phi_estimate: 0.0, // Fase 3: phi real requer IntegrationLoop via PyO3
        temperature: cpu_temp,
        energy,
        resonance: 0.0, // Fase 3: resonance real requer SomaticBridge
        pain_signals: pain,
        pleasure_signals: pleasure,
        mem_available_gb: mem_gb,
        swap_used_ratio: swap_ratio,
        root_used_pct: root_pct,
        home_used_pct: home_pct,
        var_used_pct: var_pct,
        rapl: rapl.clone(),
        oracle: oracle.clone(),
    };

    // Activation: record snapshot + write state + materialize
    match record_snapshot(&cfg, 0, &init_metrics) {
        Ok(_) => log(&cfg, "Snapshot de ativação gravado"),
        Err(e) => log(&cfg, &format!("Erro snapshot ativação: {}", e)),
    }
    write_shadow_state(&cfg, "activated", 0, &init_metrics);
    materialize_shadow_exports(&cfg);
    log(&cfg, "Daemon shadow ativado");

    // Main loop
    while running.load(Ordering::SeqCst) {
        let start_time = Instant::now();

        // Gather metrics — sensores reais + cálculo somático nativo
        let (mem_gb, swap_ratio) = read_meminfo();
        let (root_pct, home_pct, var_pct) = read_disk_usage();
        let cpu_temp = read_cpu_temp();
        let cpu_usage = read_cpu_usage_pct();
        let rapl = read_rapl_energy();
        let oracle = read_storage_oracle(&cfg);
        let (energy, pain, pleasure) = calculate_somatic_state(cpu_temp, mem_gb, cpu_usage);

        let metrics = SomaticMetrics {
            phi_estimate: 0.0, // Fase 3: phi real requer IntegrationLoop via PyO3
            temperature: cpu_temp,
            energy,
            resonance: 0.0, // Fase 3: resonance real requer SomaticBridge
            pain_signals: pain,
            pleasure_signals: pleasure,
            mem_available_gb: mem_gb,
            swap_used_ratio: swap_ratio,
            root_used_pct: root_pct,
            home_used_pct: home_pct,
            var_used_pct: var_pct,
            rapl: rapl.clone(),
            oracle: oracle.clone(),
        };

        cycle_count += 1;

        // Log a cada 10 ciclos (matching Python L199-209)
        if cycle_count % 10 == 0 {
            log(&cfg, &format!(
                "Ciclo {:4}: T={:.1}°C, E={:.1}%, mem={:.1}GB, swap={:.1}%, root={:.1}% (fase3)",
                cycle_count, metrics.temperature, metrics.energy,
                metrics.mem_available_gb,
                metrics.swap_used_ratio * 100.0, metrics.root_used_pct
            ));
        }

        // Save state a cada 100 ciclos (matching Python L215-219)
        if cycle_count % 100 == 0 {
            write_shadow_state(&cfg, "running", cycle_count, &metrics);
            log(&cfg, &format!("State saved (ciclo {})", cycle_count));
        }

        // Health check (matching Python L211-213: delta >= HEALTH_INTERVAL)
        if last_health_check.elapsed().as_secs_f64() >= cfg.health_interval {
            log(&cfg, &format!(
                "Health check: ciclos={}, phi=shadow, T={:.1}°C, mem={:.1}GB",
                cycle_count, metrics.temperature, metrics.mem_available_gb
            ));
            // Record snapshot no health check (matching Python L254)
            match record_snapshot(&cfg, cycle_count, &metrics) {
                Ok(_) => {}
                Err(e) => log(&cfg, &format!("Erro snapshot health: {}", e)),
            }
            write_shadow_state(&cfg, "running", cycle_count, &metrics);
            materialize_shadow_exports(&cfg);
            last_health_check = Instant::now();
        }

        // Sleep adaptativo (matching Python L224-226):
        // rest_time = max(CYCLE_INTERVAL, elapsed * 1.5)
        // sleep_time = max(MIN_SLEEP, rest_time - elapsed)
        let elapsed = start_time.elapsed().as_secs_f64();
        let rest_time = cfg.cycle_interval.max(elapsed * 1.5);
        let sleep_time = cfg.min_sleep.max(rest_time - elapsed);
        thread::sleep(Duration::from_secs_f64(sleep_time));
    }

    // Shutdown gracioso (matching Python stop() L267-284)
    log(&cfg, "Iniciando shutdown gracioso...");

    let (mem_gb, swap_ratio) = read_meminfo();
    let (root_pct, home_pct, var_pct) = read_disk_usage();
    let cpu_temp = read_cpu_temp();
    let cpu_usage = read_cpu_usage_pct();
    let rapl = read_rapl_energy();
    let oracle = read_storage_oracle(&cfg);
    let (energy, pain, pleasure) = calculate_somatic_state(cpu_temp, mem_gb, cpu_usage);

    let final_metrics = SomaticMetrics {
        phi_estimate: 0.0,
        temperature: cpu_temp,
        energy,
        resonance: 0.0,
        pain_signals: pain,
        pleasure_signals: pleasure,
        mem_available_gb: mem_gb,
        swap_used_ratio: swap_ratio,
        root_used_pct: root_pct,
        home_used_pct: home_pct,
        var_used_pct: var_pct,
        rapl,
        oracle,
    };

    // Final snapshot + state + materialize
    match record_snapshot(&cfg, cycle_count, &final_metrics) {
        Ok(_) => log(&cfg, "Snapshot final gravado"),
        Err(e) => log(&cfg, &format!("Erro snapshot final: {}", e)),
    }
    write_shadow_state(&cfg, "stopped", cycle_count, &final_metrics);
    materialize_shadow_exports(&cfg);

    // WAL checkpoint (alinhamento 16.3: 2 bancos)
    wal_checkpoint(&cfg);

    log(&cfg, &format!("SomaticDaemon shadow finalizado: {} ciclos executados.", cycle_count));
}
