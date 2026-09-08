// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
#![allow(clippy::too_many_arguments)]
use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use pyo3::wrap_pyfunction;
use rusqlite::{params, Connection, Row};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;
use std::path::Path;

/// Calculates lineage synchronization using trigonometric sum.
/// Replaces: np.sum([np.sin(self.cycle_count / (i + 1)) for i in range(16)]) / 16.0
#[pyfunction]
fn compute_lineage_sync(cycle_count: i32) -> f64 {
    let mut sum = 0.0;
    for i in 0..16 {
        sum += ((cycle_count as f64) / ((i + 1) as f64)).sin();
    }
    sum / 16.0
}

/// Computes the L2 Euclidean norm of a vector.
/// Replaces: np.linalg.norm(vector)
#[pyfunction]
fn compute_vector_norm(vector: Vec<f64>) -> f64 {
    let sum_sq: f64 = vector.iter().map(|&x| x * x).sum();
    sum_sq.sqrt()
}

/// Aggregates a list of input embeddings by taking their mean,
/// applying scaling, and returning the L2 normalized result.
/// Replaces: np.mean(stacked, axis=0) + normalization steps in _compute_output
#[pyfunction]
fn aggregate_embeddings(embeddings: Vec<Vec<f64>>, noise_scale: f64) -> PyResult<Vec<f64>> {
    if embeddings.is_empty() {
        return Ok(Vec::new());
    }

    let dim = embeddings[0].len();
    let count = embeddings.len() as f64;

    // Check dimensionality parity
    for emb in &embeddings {
        if emb.len() != dim {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "All embeddings must have the same dimension",
            ));
        }
    }

    let mut mean_vector = vec![0.0; dim];
    for emb in &embeddings {
        for (i, &val) in emb.iter().enumerate() {
            mean_vector[i] += val;
        }
    }

    for val in &mut mean_vector {
        *val /= count;
    }

    // Apply simple deterministic pseudorandom noise based on vector values
    // to mimic Python's np.random.randn without importing heavy rand crates.
    let mut hash_seed = 0.0;
    for &val in &mean_vector {
        hash_seed += val.abs();
    }

    for (i, val) in mean_vector.iter_mut().enumerate() {
        let pseudo_random = ((i as f64 + hash_seed).sin() * 10000.0).fract();
        *val += pseudo_random * noise_scale;
    }

    // L2 Normalize
    let norm = {
        let sum_sq: f64 = mean_vector.iter().map(|&x| x * x).sum();
        sum_sq.sqrt()
    };

    if norm > 1e-12 {
        for val in &mut mean_vector {
            *val /= norm;
        }
    }

    Ok(mean_vector)
}

// -------------------------------------------------------------
// Database telemetry & snapshot recording in Rust
// -------------------------------------------------------------

fn ensure_parent(path_str: &str) -> std::io::Result<()> {
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn connect_db(db_path: &str) -> rusqlite::Result<Connection> {
    let path = Path::new(db_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(db_path)?;
    // PRAGMA journal_mode=WAL retorna uma linha — usar query_row, não execute
    let _mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
    conn.execute("PRAGMA synchronous=NORMAL", [])?;
    Ok(conn)
}

fn safe_float(v: Option<&Value>) -> Option<f64> {
    v.and_then(|val| val.as_f64().or_else(|| val.as_str().and_then(|s| s.parse::<f64>().ok())))
}

fn safe_int(v: Option<&Value>) -> Option<i64> {
    v.and_then(|val| val.as_i64().or_else(|| val.as_str().and_then(|s| s.parse::<i64>().ok())))
}

fn get_utc_iso() -> String {
    Utc::now().to_rfc3339()
}

fn age_seconds_from_iso(iso_str: &str) -> Option<f64> {
    if iso_str.trim().is_empty() {
        return None;
    }
    let clean = iso_str.replace("Z", "+00:00");
    if let Ok(dt) = DateTime::parse_from_rfc3339(&clean) {
        let seconds = (Utc::now().timestamp_millis() - dt.with_timezone(&Utc).timestamp_millis()) as f64 / 1000.0;
        Some((seconds.max(0.0) * 1000.0).round() / 1000.0)
    } else {
        None
    }
}

// Extractors mapped exactly from Python
fn extract_somatic_current(summary: &Value) -> Value {
    if let Some(current) = summary.get("current_state") {
        if current.is_object() {
            return current.clone();
        }
    }
    if summary.is_object() {
        summary.clone()
    } else {
        json!({})
    }
}

fn extract_fragment_count(summary: &Value) -> Option<i64> {
    for key in &["fragment_opportunities", "fragment_count"] {
        if let Some(val) = summary.get(key) {
            if let Some(arr) = val.as_array() {
                return Some(arr.len() as i64);
            }
            if let Some(n) = safe_int(Some(val)) {
                return Some(n);
            }
        }
    }
    if let Some(current) = summary.get("current_state") {
        if let Some(fc) = current.get("fragment_count") {
            return safe_int(Some(fc));
        }
    }
    None
}

fn extract_oracle_snapshot(oracle: &Value) -> Value {
    let disk_pressure = oracle.get("disk_pressure").cloned().unwrap_or_else(|| json!({}));
    let memory_pressure = oracle.get("memory_pressure").cloned().unwrap_or_else(|| json!({}));
    
    let mem_available_gb = safe_float(memory_pressure.get("mem_available_gb"));
    let swap_used_ratio = safe_float(memory_pressure.get("swap_used_ratio"));
    
    let root_used_pct = safe_float(disk_pressure.get("root").and_then(|r| r.get("used_pct")));
    let home_used_pct = safe_float(disk_pressure.get("home").and_then(|h| h.get("used_pct")));
    let var_used_pct = safe_float(disk_pressure.get("var").and_then(|v| v.get("used_pct")));
    
    json!({
        "overall_level": oracle.get("overall_level"),
        "status": oracle.get("status"),
        "mem_available_gb": mem_available_gb,
        "swap_used_ratio": swap_used_ratio,
        "root_used_pct": root_used_pct,
        "home_used_pct": home_used_pct,
        "var_used_pct": var_used_pct,
        "services": oracle.get("services").cloned().unwrap_or_else(|| json!({}))
    })
}

// Build window summary exactly matching SQLite operations in python
fn build_window_summary(conn: &Connection, minutes: i64) -> rusqlite::Result<Value> {
    let window_start = (Utc::now() - chrono::Duration::minutes(minutes)).to_rfc3339();
    
    let mut stmt_count = conn.prepare("SELECT COUNT(*) AS n FROM somatic_snapshots WHERE timestamp_utc >= ?")?;
    let snapshots_n: i64 = stmt_count.query_row([&window_start], |r| r.get(0))?;
    
    let mut stmt_latest = conn.prepare(
        "SELECT * FROM somatic_snapshots WHERE timestamp_utc >= ? ORDER BY timestamp_utc DESC LIMIT 1"
    )?;
    let latest_snapshot = stmt_latest.query_row([&window_start], row_to_snapshot).ok();
    
    let mut stmt_metrics = conn.prepare(
        "SELECT
            AVG(body_temperature_c) AS avg_temp,
            MAX(body_temperature_c) AS max_temp,
            AVG(energy_pct) AS avg_energy,
            MIN(mem_available_gb) AS min_mem_available_gb,
            MAX(swap_used_ratio) AS max_swap_used_ratio,
            MAX(root_used_pct) AS max_root_used_pct,
            MAX(home_used_pct) AS max_home_used_pct,
            MAX(var_used_pct) AS max_var_used_pct
        FROM somatic_snapshots
        WHERE timestamp_utc >= ?"
    )?;
    
    let metrics: Value = stmt_metrics.query_row([&window_start], |r| {
        Ok(json!({
            "avg_temp": r.get::<_, Option<f64>>("avg_temp")?,
            "max_temp": r.get::<_, Option<f64>>("max_temp")?,
            "avg_energy": r.get::<_, Option<f64>>("avg_energy")?,
            "min_mem_available_gb": r.get::<_, Option<f64>>("min_mem_available_gb")?,
            "max_swap_used_ratio": r.get::<_, Option<f64>>("max_swap_used_ratio")?,
            "max_root_used_pct": r.get::<_, Option<f64>>("max_root_used_pct")?,
            "max_home_used_pct": r.get::<_, Option<f64>>("max_home_used_pct")?,
            "max_var_used_pct": r.get::<_, Option<f64>>("max_var_used_pct")?
        }))
    }).unwrap_or_else(|_| json!({}));
    
    let mut stmt_levels = conn.prepare(
        "SELECT oracle_overall_level, COUNT(*) AS n
         FROM somatic_snapshots
         WHERE timestamp_utc >= ?
         GROUP BY oracle_overall_level
         ORDER BY n DESC, oracle_overall_level"
    )?;
    
    let mut rows_levels = stmt_levels.query([&window_start])?;
    let mut oracle_levels = Vec::new();
    while let Some(r) = rows_levels.next()? {
        oracle_levels.push(json!({
            "level": r.get::<_, Option<String>>(0)?,
            "count": r.get::<_, i64>(1)?
        }));
    }
    
    Ok(json!({
        "window_minutes": minutes,
        "snapshots_n": snapshots_n,
        "latest_snapshot": latest_snapshot,
        "avg_body_temperature_c": metrics.get("avg_temp"),
        "max_body_temperature_c": metrics.get("max_temp"),
        "avg_energy_pct": metrics.get("avg_energy"),
        "min_mem_available_gb": metrics.get("min_mem_available_gb"),
        "max_swap_used_ratio": metrics.get("max_swap_used_ratio"),
        "max_root_used_pct": metrics.get("max_root_used_pct"),
        "max_home_used_pct": metrics.get("max_home_used_pct"),
        "max_var_used_pct": metrics.get("max_var_used_pct"),
        "oracle_levels": oracle_levels
    }))
}

fn row_to_snapshot(row: &Row) -> rusqlite::Result<Value> {
    let service_states: Value = serde_json::from_str(&row.get::<_, String>("service_states_json").unwrap_or_default()).unwrap_or(Value::Null);
    let notes: Value = serde_json::from_str(&row.get::<_, String>("notes_json").unwrap_or_default()).unwrap_or(Value::Null);
    let runtime_energy: Value = serde_json::from_str(&row.get::<_, String>("runtime_energy_json").unwrap_or_default()).unwrap_or(Value::Null);
    
    Ok(json!({
        "timestamp_utc": row.get::<_, String>("timestamp_utc")?,
        "source": row.get::<_, String>("source")?,
        "cycle": row.get::<_, Option<i64>>("cycle")?,
        "phi_estimate": row.get::<_, Option<f64>>("phi_estimate")?,
        "body_temperature_c": row.get::<_, Option<f64>>("body_temperature_c")?,
        "energy_pct": row.get::<_, Option<f64>>("energy_pct")?,
        "resonance": row.get::<_, Option<f64>>("resonance")?,
        "pain_signals_n": row.get::<_, Option<i64>>("pain_signals_n")?,
        "pleasure_signals_n": row.get::<_, Option<i64>>("pleasure_signals_n")?,
        "fragment_count": row.get::<_, Option<i64>>("fragment_count")?,
        "oracle_overall_level": row.get::<_, Option<String>>("oracle_overall_level")?,
        "oracle_status": row.get::<_, Option<String>>("oracle_status")?,
        "disk_pressure": {
            "root_used_pct": row.get::<_, Option<f64>>("root_used_pct")?,
            "home_used_pct": row.get::<_, Option<f64>>("home_used_pct")?,
            "var_used_pct": row.get::<_, Option<f64>>("var_used_pct")?,
        },
        "memory_pressure": {
            "mem_available_gb": row.get::<_, Option<f64>>("mem_available_gb")?,
            "swap_used_ratio": row.get::<_, Option<f64>>("swap_used_ratio")?,
        },
        "service_states": service_states,
        "notes": notes,
        "runtime_energy_telemetry": runtime_energy
    }))
}

fn row_to_event(row: &Row) -> rusqlite::Result<Value> {
    let context: Value = serde_json::from_str(&row.get::<_, String>("context_json").unwrap_or_default()).unwrap_or(Value::Null);
    
    Ok(json!({
        "timestamp_utc": row.get::<_, String>("timestamp_utc")?,
        "source": row.get::<_, String>("source")?,
        "watched_unit": row.get::<_, Option<String>>("watched_unit")?,
        "event_kind": row.get::<_, String>("event_kind")?,
        "reflex_action": row.get::<_, Option<String>>("reflex_action")?,
        "severity": row.get::<_, Option<String>>("severity")?,
        "reason": row.get::<_, Option<String>>("reason")?,
        "line_excerpt": row.get::<_, Option<String>>("line_excerpt")?,
        "oracle_overall_level": row.get::<_, Option<String>>("oracle_overall_level")?,
        "context": context
    }))
}

fn build_latest_payload_rust(conn: &Connection, db_path: &str) -> rusqlite::Result<Value> {
    let latest_snapshot_row = conn.query_row(
        "SELECT * FROM somatic_snapshots ORDER BY timestamp_utc DESC LIMIT 1",
        [],
        row_to_snapshot
    ).ok();
    
    let mut stmt_events = conn.prepare("SELECT * FROM vagus_events ORDER BY timestamp_utc DESC LIMIT 20")?;
    let mut rows_events = stmt_events.query([])?;
    let mut recent_vagus_events = Vec::new();
    while let Some(r) = rows_events.next()? {
        recent_vagus_events.push(row_to_event(r)?);
    }
    
    let summary: Value = conn.query_row(
        "SELECT
            (SELECT COUNT(*) FROM somatic_snapshots) AS snapshots_n,
            (SELECT COUNT(*) FROM vagus_events) AS vagus_events_n",
        [],
        |r| {
            Ok(json!({
                "snapshots_n": r.get::<_, i64>(0)?,
                "vagus_events_n": r.get::<_, i64>(1)?
            }))
        }
    ).unwrap_or_else(|_| json!({"snapshots_n": 0, "vagus_events_n": 0}));
    
    let latest_event = recent_vagus_events.first().cloned();
    let latest_snapshot_age_sec = latest_snapshot_row.as_ref()
        .and_then(|snap| snap.get("timestamp_utc").and_then(|t| t.as_str()))
        .and_then(age_seconds_from_iso);
        
    let latest_vagus_event_age_sec = latest_event.as_ref()
        .and_then(|ev| ev.get("timestamp_utc").and_then(|t| t.as_str()))
        .and_then(age_seconds_from_iso);
        
    let vagus_event_stream_stale = latest_vagus_event_age_sec.map(|age| age > 3600.0).unwrap_or(false);
    
    let payload = json!({
        "timestamp_utc": get_utc_iso(),
        "status": "ok",
        "backing_sqlite": db_path,
        "summary": summary,
        "latest_snapshot": latest_snapshot_row,
        "watchdog_stream": {
            "latest_snapshot_age_sec": latest_snapshot_age_sec,
            "latest_vagus_event": latest_event,
            "latest_vagus_event_age_sec": latest_vagus_event_age_sec,
            "vagus_event_stream_stale": vagus_event_stream_stale
        },
        "windows": {
            "10m": build_window_summary(conn, 10)?,
            "60m": build_window_summary(conn, 60)?
        },
        "recent_vagus_events": recent_vagus_events
    });
    
    Ok(payload)
}

fn build_markdown_rust(payload: &Value) -> String {
    let summary = payload.get("summary").cloned().unwrap_or_else(|| json!({}));
    let latest = payload.get("latest_snapshot").cloned().unwrap_or_else(|| json!({}));
    let watchdog = payload.get("watchdog_stream").cloned().unwrap_or_else(|| json!({}));
    let latest_event = watchdog.get("latest_vagus_event").cloned().unwrap_or_else(|| json!({}));
    let windows = payload.get("windows").cloned().unwrap_or_else(|| json!({}));
    
    let mut lines = vec![
        "# Somatic Mesh Runtime".to_string(),
        "".to_string(),
        format!("- timestamp_utc: `{}`", payload.get("timestamp_utc").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- backing_sqlite: `{}`", payload.get("backing_sqlite").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- snapshots_n: `{}`", summary.get("snapshots_n").and_then(|v| v.as_i64()).unwrap_or(0)),
        format!("- vagus_events_n: `{}`", summary.get("vagus_events_n").and_then(|v| v.as_i64()).unwrap_or(0)),
        format!("- latest_snapshot_age_sec: `{:?}`", watchdog.get("latest_snapshot_age_sec")),
        format!("- latest_vagus_event_age_sec: `{:?}`", watchdog.get("latest_vagus_event_age_sec")),
        format!("- vagus_event_stream_stale: `{}`", watchdog.get("vagus_event_stream_stale").and_then(|v| v.as_bool()).unwrap_or(false)),
        "".to_string(),
        "## Latest Snapshot".to_string(),
        "".to_string(),
        format!("- source: `{}`", latest.get("source").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- cycle: `{:?}`", latest.get("cycle")),
        format!("- phi_estimate: `{:?}`", latest.get("phi_estimate")),
        format!("- body_temperature_c: `{:?}`", latest.get("body_temperature_c")),
        format!("- energy_pct: `{:?}`", latest.get("energy_pct")),
        format!("- oracle_overall_level: `{}`", latest.get("oracle_overall_level").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- mem_available_gb: `{:?}`", latest.get("memory_pressure").and_then(|m| m.get("mem_available_gb"))),
        format!("- swap_used_ratio: `{:?}`", latest.get("memory_pressure").and_then(|m| m.get("swap_used_ratio"))),
        "".to_string(),
        "## Watchdog Stream".to_string(),
        "".to_string(),
        format!("- latest_vagus_event_kind: `{}`", latest_event.get("event_kind").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- latest_vagus_event_severity: `{}`", latest_event.get("severity").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- latest_vagus_event_reason: `{}`", latest_event.get("reason").and_then(|v| v.as_str()).unwrap_or("")),
        format!("- latest_vagus_event_timestamp_utc: `{}`", latest_event.get("timestamp_utc").and_then(|v| v.as_str()).unwrap_or("")),
        "".to_string(),
        "## Windows".to_string(),
        "".to_string(),
    ];
    
    for key in &["10m", "60m"] {
        let win = windows.get(*key).cloned().unwrap_or_else(|| json!({}));
        lines.push(format!("### {}", key));
        lines.push("".to_string());
        lines.push(format!("- snapshots_n: `{}`", win.get("snapshots_n").and_then(|v| v.as_i64()).unwrap_or(0)));
        lines.push(format!("- avg_body_temperature_c: `{:?}`", win.get("avg_body_temperature_c")));
        lines.push(format!("- max_body_temperature_c: `{:?}`", win.get("max_body_temperature_c")));
        lines.push(format!("- min_mem_available_gb: `{:?}`", win.get("min_mem_available_gb")));
        lines.push(format!("- max_swap_used_ratio: `{:?}`", win.get("max_swap_used_ratio")));
        lines.push(format!("- max_home_used_pct: `{:?}`", win.get("max_home_used_pct")));
        lines.push(format!("- max_var_used_pct: `{:?}`", win.get("max_var_used_pct")));
        lines.push("".to_string());
    }
    
    lines.push("## Recent Vagus Events".to_string());
    lines.push("".to_string());
    if let Some(events) = payload.get("recent_vagus_events").and_then(|e| e.as_array()) {
        if events.is_empty() {
            lines.push("- none".to_string());
        } else {
            for item in events.iter().take(8) {
                lines.push(format!(
                    "- `{}` `{}` `{}` unit=`{}` reason=`{}`",
                    item.get("timestamp_utc").and_then(|v| v.as_str()).unwrap_or(""),
                    item.get("event_kind").and_then(|v| v.as_str()).unwrap_or(""),
                    item.get("reflex_action").and_then(|v| v.as_str()).unwrap_or(""),
                    item.get("watched_unit").and_then(|v| v.as_str()).unwrap_or(""),
                    item.get("reason").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
        }
    } else {
        lines.push("- none".to_string());
    }
    lines.push("".to_string());
    lines.join("\n")
}

fn write_json_file(path_str: &str, payload: &Value) -> std::io::Result<()> {
    ensure_parent(path_str)?;
    let content = serde_json::to_string_pretty(payload).unwrap_or_default() + "\n";
    fs::write(path_str, content)?;
    Ok(())
}

fn write_md_file(path_str: &str, content: &str) -> std::io::Result<()> {
    ensure_parent(path_str)?;
    fs::write(path_str, content)?;
    Ok(())
}

fn materialize_latest(payload: &Value, latest_json: &str, latest_md: &str) -> Result<(), PyErr> {
    write_json_file(latest_json, payload).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let md_content = build_markdown_rust(payload);
    write_md_file(latest_md, &md_content).map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(())
}

#[pyfunction]
fn record_somatic_snapshot_rust(
    db_path: String,
    source: String,
    latest_json: String,
    latest_md: String,
    cycle: Option<i64>,
    phi_estimate: Option<f64>,
    somatic_summary_json: Option<String>,
    oracle_payload_json: Option<String>,
    extra_json: Option<String>,
) -> PyResult<String> {
    let somatic_summary: Value = somatic_summary_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let oracle_payload: Value = oracle_payload_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let extra: Value = extra_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    
    let current = extract_somatic_current(&somatic_summary);
    let fragment_count = extract_fragment_count(&somatic_summary);
    let oracle = extract_oracle_snapshot(&oracle_payload);
    
    let service_states = extra.get("service_states").cloned();
    let runtime_energy = extra.get("runtime_energy_telemetry").cloned();
    
    let raw_payload = json!({
        "source": source,
        "cycle": cycle,
        "phi_estimate": phi_estimate,
        "somatic_summary": somatic_summary,
        "oracle_payload": oracle_payload,
        "extra": extra
    });
    
    let snapshot_uid = Uuid::new_v4().simple().to_string();
    let timestamp_utc = get_utc_iso();
    
    let body_temperature_c = safe_float(current.get("temperature").or_else(|| current.get("body_temperature")));
    let energy_pct = safe_float(current.get("energy").or_else(|| current.get("energy_level")));
    let resonance = safe_float(current.get("resonance").or_else(|| current.get("resonance_field")));
        
    let pain_signals_n = current.get("pain_signals")
        .and_then(|v| v.as_array().map(|a| a.len() as i64))
        .or_else(|| safe_int(current.get("pain_signals_n")));
        
    let pleasure_signals_n = current.get("pleasure_signals")
        .and_then(|v| v.as_array().map(|a| a.len() as i64))
        .or_else(|| safe_int(current.get("pleasure_signals_n")));
        
    let service_states_json = service_states.map(|s| serde_json::to_string(&s).unwrap_or_default());
    let notes_json = Some(serde_json::to_string(&extra).unwrap_or_default());
    let runtime_energy_json = runtime_energy.map(|r| serde_json::to_string(&r).unwrap_or_default());
    let raw_json = serde_json::to_string(&raw_payload).unwrap_or_default();
    
    let conn = connect_db(&db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    
    conn.execute(
        "INSERT INTO somatic_snapshots (
            snapshot_uid, timestamp_utc, source, cycle, phi_estimate,
            body_temperature_c, energy_pct, resonance,
            pain_signals_n, pleasure_signals_n, fragment_count,
            mem_available_gb, swap_used_ratio,
            root_used_pct, home_used_pct, var_used_pct,
            oracle_overall_level, oracle_status,
            service_states_json, notes_json, runtime_energy_json,
            raw_json, created_utc
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
        params![
            snapshot_uid,
            timestamp_utc,
            source,
            cycle,
            phi_estimate,
            body_temperature_c,
            energy_pct,
            resonance,
            pain_signals_n,
            pleasure_signals_n,
            fragment_count,
            safe_float(oracle.get("mem_available_gb")),
            safe_float(oracle.get("swap_used_ratio")),
            safe_float(oracle.get("root_used_pct")),
            safe_float(oracle.get("home_used_pct")),
            safe_float(oracle.get("var_used_pct")),
            oracle.get("overall_level").and_then(|v| v.as_str()),
            oracle.get("status").and_then(|v| v.as_str()),
            service_states_json,
            notes_json,
            runtime_energy_json,
            raw_json,
            timestamp_utc
        ]
    ).map_err(|e| PyIOError::new_err(e.to_string()))?;
    
    let payload = build_latest_payload_rust(&conn, &db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    materialize_latest(&payload, &latest_json, &latest_md)?;
    
    Ok(serde_json::to_string(&payload).unwrap_or_default())
}

#[pyfunction]
fn record_vagus_event_rust(
    db_path: String,
    source: String,
    event_kind: String,
    latest_json: String,
    latest_md: String,
    reflex_action: Option<String>,
    watched_unit: Option<String>,
    severity: Option<String>,
    reason: Option<String>,
    line_excerpt: Option<String>,
    oracle_payload_json: Option<String>,
    context_json: Option<String>,
) -> PyResult<String> {
    let oracle_payload: Value = oracle_payload_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let context_val: Value = context_json.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let oracle = extract_oracle_snapshot(&oracle_payload);
    
    let event_uid = Uuid::new_v4().simple().to_string();
    let timestamp_utc = get_utc_iso();
    let ctx_str = serde_json::to_string(&context_val).unwrap_or_default();
    
    let conn = connect_db(&db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    
    conn.execute(
        "INSERT INTO vagus_events (
            event_uid, timestamp_utc, source, watched_unit, event_kind,
            reflex_action, severity, reason, line_excerpt,
            oracle_overall_level, context_json, created_utc
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            event_uid,
            timestamp_utc,
            source,
            watched_unit,
            event_kind,
            reflex_action,
            severity,
            reason,
            line_excerpt,
            oracle.get("overall_level").and_then(|v| v.as_str()),
            ctx_str,
            timestamp_utc
        ]
    ).map_err(|e| PyIOError::new_err(e.to_string()))?;
    
    let payload = build_latest_payload_rust(&conn, &db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    materialize_latest(&payload, &latest_json, &latest_md)?;
    
    Ok(serde_json::to_string(&payload).unwrap_or_default())
}

#[pyfunction]
fn materialize_latest_exports_rust(
    db_path: String,
    latest_json: String,
    latest_md: String,
) -> PyResult<String> {
    let conn = connect_db(&db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let payload = build_latest_payload_rust(&conn, &db_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    materialize_latest(&payload, &latest_json, &latest_md)?;
    Ok(serde_json::to_string(&payload).unwrap_or_default())
}

#[pymodule]
fn layered_transition_engine(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(compute_lineage_sync, m)?)?;
    m.add_function(wrap_pyfunction!(compute_vector_norm, m)?)?;
    m.add_function(wrap_pyfunction!(aggregate_embeddings, m)?)?;
    m.add_function(wrap_pyfunction!(record_somatic_snapshot_rust, m)?)?;
    m.add_function(wrap_pyfunction!(record_vagus_event_rust, m)?)?;
    m.add_function(wrap_pyfunction!(materialize_latest_exports_rust, m)?)?;
    Ok(())
}
