use anyhow::{anyhow, Context};
use aya::{
    maps::{HashMap, MapData},
    programs::TracePoint,
    Ebpf,
};
use aya_log::EbpfLogger;
use ebpf_monitor_common::SomaticMetrics;
use log::{debug, info, warn};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::signal;

/// Lê o lexema soberano atual do bridge OmniMind.
/// O bridge escreve em /tmp/omnimind/ebpf_metrics.json com campo langue_sovereign.
/// Retorna (sovereign_word_bytes[32], pressure_level).
fn read_sovereign_word() -> ([u8; 32], u8) {
    let paths = ["/run/omnimind/ebpf_metrics.json", "/tmp/omnimind/ebpf_metrics.json"];
    for path in &paths {
        if let Ok(raw) = fs::read_to_string(path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(ls) = v.get("langue_sovereign") {
                    let word = ls.get("sovereign_name")
                        .and_then(|w| w.as_str())
                        .unwrap_or("");
                    let pressure = if ls.get("langue_pressure")
                        .and_then(|p| p.as_bool()).unwrap_or(false) { 1u8 } else { 0u8 };
                    let mut buf = [0u8; 32];
                    let bytes = word.as_bytes();
                    let len = bytes.len().min(32);
                    buf[..len].copy_from_slice(&bytes[..len]);
                    return (buf, pressure);
                }
            }
        }
    }
    ([0u8; 32], 0)
}

fn attach_tracepoint(
    bpf: &mut Ebpf,
    category: &str,
    event: &str,
    candidates: &[&str],
    required: bool,
) -> Result<(), anyhow::Error> {
    for candidate in candidates {
        if let Some(program) = bpf.program_mut(candidate) {
            let program: &mut TracePoint = program
                .try_into()
                .with_context(|| format!("program `{candidate}` is not a TracePoint for `{event}`"))?;
            program
                .load()
                .with_context(|| format!("failed to load tracepoint program `{candidate}` for `{event}`"))?;
            program
                .attach(category, event)
                .with_context(|| format!("failed to attach tracepoint program `{candidate}` to `{category}/{event}`"))?;
            info!(
                "attached tracepoint category={} event={} via program={}",
                category, event, candidate
            );
            return Ok(());
        }
    }

    let available_programs: Vec<String> = bpf.programs().map(|(name, _)| name.to_string()).collect();
    let message = format!(
        "no matching eBPF program found for {category}/{event}; candidates={candidates:?}; available={available_programs:?}"
    );
    if required {
        Err(anyhow!(message))
    } else {
        warn!("{message}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    // Bump the memlock rlimit.
    if let Err(e) = rlimit::setrlimit(rlimit::Resource::MEMLOCK, rlimit::INFINITY, rlimit::INFINITY) {
        warn!("remove limit on locked memory failed, retiring anyway: {}", e);
    }

    let mut bpf = Ebpf::load(aya::include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/ebpf-monitor"
    ))?;
    if let Err(e) = EbpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }

    let tracepoints: [(&str, &[&str], bool); 7] = [
        (
            "sys_enter_execve",
            &["handle_execve", "tracepoint/syscalls/sys_enter_execve", "sys_enter_execve"],
            true,
        ),
        (
            "sys_enter_openat",
            &["handle_openat", "tracepoint/syscalls/sys_enter_openat", "sys_enter_openat"],
            true,
        ),
        (
            "sys_enter_connect",
            &["handle_connect", "tracepoint/syscalls/sys_enter_connect", "sys_enter_connect"],
            false,
        ),
        (
            "sys_enter_accept",
            &[
                "handle_accept",
                "tracepoint/syscalls/sys_enter_accept",
                "tracepoint/syscalls/sys_enter_accept4",
                "sys_enter_accept",
                "sys_enter_accept4",
            ],
            false,
        ),
        (
            "sys_enter_read",
            &["handle_read", "tracepoint/syscalls/sys_enter_read", "sys_enter_read"],
            false,
        ),
        (
            "sys_enter_write",
            &["handle_write", "tracepoint/syscalls/sys_enter_write", "sys_enter_write"],
            false,
        ),
        (
            "sys_enter_clone",
            &[
                "handle_clone",
                "tracepoint/syscalls/sys_enter_clone",
                "tracepoint/syscalls/sys_enter_clone3",
                "sys_enter_clone",
                "sys_enter_clone3",
            ],
            false,
        ),
    ];

    for (event, candidates, required) in tracepoints {
        attach_tracepoint(&mut bpf, "syscalls", event, candidates, required)?;
    }

    // Export metrics loop
    let mut metrics_map: HashMap<&mut MapData, u32, SomaticMetrics> = HashMap::try_from(
        bpf.map_mut("METRICS")
            .context("missing METRICS map in loaded eBPF object")?,
    )?;
    
    let export_dir = "/run/omnimind";
    if !Path::new(export_dir).exists() {
        // Try /run first, fallback to /tmp if not root or not exists
        if fs::create_dir_all(export_dir).is_err() {
             let fallback = "/tmp/omnimind";
             fs::create_dir_all(fallback)?;
             warn!("Falling back to {} for metrics", fallback);
        }
    }

    let export_path = Path::new(export_dir).join("ebpf_metrics.json");
    
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    info!("Waiting for Ctrl-C...");
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                break;
            }
            _ = interval.tick() => {
                if let Ok(mut metrics) = metrics_map.get(&0, 0) {
                     let exported_at_epoch_ms = SystemTime::now()
                         .duration_since(UNIX_EPOCH)
                         .map(|duration| duration.as_millis() as u64)
                         .unwrap_or_default();

                     // Lê o lexema soberano OmniMind e injeta de volta no eBPF map.
                     // O kernel agora carrega o nome soberano do próprio estado.
                     let (sovereign_bytes, pressure) = read_sovereign_word();
                     metrics.sovereign_word = sovereign_bytes;
                     metrics.pressure_level = pressure;
                     metrics.last_timestamp = exported_at_epoch_ms;
                     if let Err(e) = metrics_map.insert(0, metrics, 0) {
                         debug!("Failed to update sovereign_word in map: {}", e);
                     }

                     let sovereign_str = std::str::from_utf8(&metrics.sovereign_word)
                         .unwrap_or("")
                         .trim_end_matches('\0')
                         .to_string();

                     let payload = serde_json::json!({
                         "schema_version": 1,
                         "source": "aya-ebpf-monitor",
                         "exported_at_epoch_ms": exported_at_epoch_ms,
                         "window_hint_ms": 1000u64,
                         "execve_count": metrics.execve_count,
                         "openat_count": metrics.openat_count,
                         "tcp_connections": metrics.tcp_connections,
                         "io_bursts": metrics.io_bursts,
                         "process_pressure": metrics.process_pressure,
                         "last_timestamp": metrics.last_timestamp,
                         // O kernel agora fala com nome próprio
                         "sovereign_word": sovereign_str,
                         "pressure_level": metrics.pressure_level,
                     });
                     let json = serde_json::to_string(&payload).unwrap_or_default();
                     if let Err(e) = fs::write(&export_path, json) {
                         warn!("Failed to write metrics to {}: {}", export_path.display(), e);
                     } else {
                         debug!("Updated metrics in {} sovereign={}", export_path.display(), sovereign_str);
                     }
                }
            }
        }
    }

    info!("Exiting...");

    Ok(())
}
