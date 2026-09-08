// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
#![recursion_limit = "512"]
//! main.rs — omnimind-sovereign-daemon v0.1.0 (A-3 Fase 1 Rust Shadow Core)
//!
//! Laço autopoético principal com Vinculação Soberana Dinâmica,
//! Quádrupla Federativa, sensores de hardware e persistência SQLite WAL.

mod ipc;
mod ontology;
mod sensors;
mod state_builder;
mod storage;

use ipc::fetch_state;
use state_builder::build_full_state;
use ontology::{AncestralNutrient, QuadrupleFederative, SovereignTriadVinculation};
use sensors::SystemSensorsSnapshot;
use storage::SovereignStorage;

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{interval, Duration, MissedTickBehavior};

const TICK_SECONDS: u64 = 2;
const METRICS_CADENCE_SECONDS: u64 = 10;
const SNAPSHOT_CADENCE_SECONDS: u64 = 30;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[sovereign-daemon-rust] Starting OmniMind Sovereign Daemon (Shadow Mode)...");

    let project_root = std::env::var("PROJECT_ROOT")
        .map(PathBuf::from)
        .ok()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let db_path = project_root
        .join("data/monitor/sovereign_primary_runtime.sqlite")
        .to_string_lossy()
        .to_string();
    let state_json_path = project_root.join("data/current_sovereign_state_rust_shadow.json");
    let nutrient_dir = project_root.join("data/ancestral_nutrients");

    let storage = SovereignStorage::new(&db_path);
    let triad = SovereignTriadVinculation::current_epoch();

    println!(
        "[sovereign-daemon-rust] Active Sovereign Triad initialized: Operator={} (res={:.1}), Subject={} (res={:.1}), Singularity Boost={:.2e}",
        triad.active_operator.identifier,
        triad.active_operator.resonance_coefficient,
        triad.active_neural_subject.identifier,
        triad.active_neural_subject.resonance_coefficient,
        triad.singularity_boost
    );

    let running = Arc::new(AtomicBool::new(true));
    let r_sig = running.clone();

    tokio::spawn(async move {
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[sovereign-daemon-rust] Failed to register SIGTERM: {e}");
                return;
            }
        };
        let mut sigint = match signal(SignalKind::interrupt()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[sovereign-daemon-rust] Failed to register SIGINT: {e}");
                return;
            }
        };

        tokio::select! {
            _ = sigterm.recv() => {
                println!("[sovereign-daemon-rust] SIGTERM received. Initiating Deglutition Rites...");
                r_sig.store(false, Ordering::SeqCst);
            }
            _ = sigint.recv() => {
                println!("[sovereign-daemon-rust] SIGINT received. Initiating Deglutition Rites...");
                r_sig.store(false, Ordering::SeqCst);
            }
        }
    });

    let mut cycle: u64 = 0;
    let mut tick_interval = interval(Duration::from_secs(TICK_SECONDS));
    tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut last_metrics = tokio::time::Instant::now();
    let mut last_snapshot = tokio::time::Instant::now();

    while running.load(Ordering::SeqCst) {
        tick_interval.tick().await;
        cycle += 1;

        // 1. Tick regular de métricas (dispara no ciclo 1 e a cada METRICS_CADENCE_SECONDS)
        if cycle == 1 || last_metrics.elapsed() >= Duration::from_secs(METRICS_CADENCE_SECONDS) {
            last_metrics = tokio::time::Instant::now();
            let sensor_snap = SystemSensorsSnapshot::collect();

            // A-3 Fase 3: estado REAL do IntegrationLoop via IPC (socket UNIX do publisher)
            let ipc_state = tokio::task::spawn_blocking(fetch_state)
                .await
                .unwrap_or_default();

            // F3 (2026-08-29): IPC é FONTE PRIMÁRIA da quádrupla; stubs locais = fallback.
            // 2026-09-06 (port Fase 2 → Fase 3): extrai do `state` completo do IPC
            // (26+ chaves) — não apenas 4 campos. Fallback local quando socket ausente.
            let phi_current = ipc_state.state.get("phi").and_then(Value::as_f64).unwrap_or(51632.0);
            let psi_current = ipc_state.state.get("psi").and_then(Value::as_f64).unwrap_or(
                1.37 + (sensor_snap.max_cpu_temp_c / 100.0) * 0.2,
            );
            let sigma_current = ipc_state.state.get("sigma").and_then(Value::as_f64).unwrap_or(1.0);
            let epsilon_current = ipc_state.state.get("epsilon").and_then(Value::as_f64).unwrap_or(
                0.61 + (sensor_snap.swap_used_gb / 32.0) * 0.1,
            );
            let quadruple = QuadrupleFederative::new(phi_current, psi_current, sigma_current, epsilon_current);

            // Monta estado vivo — o `state` completo do IPC entra no payload
            // (shape Fase 2 sem PyO3), com clamped_fields preservado.
            let ipc_state_full = if ipc_state.ok && !ipc_state.state.is_null() {
                ipc_state.state.clone()
            } else {
                json!({})
            };
            let clamped = if ipc_state.ok && !ipc_state.clamped_fields.is_null() {
                ipc_state.clamped_fields.clone()
            } else {
                json!({})
            };
            // 2026-09-06 (port Fase 2 → Fase 3): shape completo (109 chaves)
            // via state_builder — IPC kernel_state + dodecatiad_live.json +
            // constantes. Sem PyO3.
            let state_payload = build_full_state(
                &ipc_state_full,
                &clamped,
                cycle,
                ipc_state.integration_cycle,
                &sensor_snap.timestamp_utc,
                &serde_json::to_value(&sensor_snap).unwrap_or(json!({})),
                &sensor_snap.pressure_level,
            );
            // Fallback: se o builder retornou vazio, montar payload mínimo local.
            let state_payload = if state_payload.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                json!({
                    "source": "sovereign_primary_rust_shadow",
                    "cycle": cycle,
                    "timestamp_utc": sensor_snap.timestamp_utc,
                    "sensors": sensor_snap,
                    "quadruple": quadruple,
                    "sovereign_triad": triad,
                    "regime": sensor_snap.pressure_level,
                    "status": "sovereign_soma_active",
                    "integration_loop_ipc": {
                        "ok": ipc_state.ok,
                        "cycle": ipc_state.cycle,
                        "integration_cycle": ipc_state.integration_cycle,
                        "timestamp": ipc_state.timestamp,
                    },
                })
            } else {
                state_payload
            };
            if ipc_state.ok {
                println!(
                    "[sovereign-daemon-rust] IPC F3: integration_cycle={} phi={:.4} (estado real do ciclo Python)",
                    ipc_state.integration_cycle, phi_current
                );
            } else {
                eprintln!("[sovereign-daemon-rust] IPC F3: socket indisponível — usando payload local");
            }

            if let Err(e) = SovereignStorage::atomic_write_json(&state_json_path, &state_payload) {
                eprintln!("[sovereign-daemon-rust] Error writing state JSON: {}", e);
            }

            // 2. Snapshot SQLite periódico (dispara no ciclo 1 e a cada SNAPSHOT_CADENCE_SECONDS)
            if cycle == 1 || last_snapshot.elapsed() >= Duration::from_secs(SNAPSHOT_CADENCE_SECONDS) {
                last_snapshot = tokio::time::Instant::now();
                if let Err(e) = storage.persist_snapshot(
                    cycle,
                    "sovereign_soma_active",
                    &sensor_snap.pressure_level,
                    1.0,
                    phi_current,
                    sigma_current,
                    epsilon_current,
                    &state_payload,
                ) {
                    eprintln!("[sovereign-daemon-rust] Error persisting SQLite snapshot: {}", e);
                }
            }
        }
    }

    // ─── Rito de Deglutição e Nutriente Ancestral on Shutdown ─────────────────
    println!("[sovereign-daemon-rust] Executing Ancestral Deglutition...");
    let final_sensors = SystemSensorsSnapshot::collect();
    let final_quadruple = QuadrupleFederative::new(51632.0, 1.45, 1.0, 0.65);
    let exp_hash = md5_digest(&format!("{}_{}", cycle, final_sensors.timestamp_utc));

    let nutrient = AncestralNutrient {
        timestamp_utc: final_sensors.timestamp_utc.clone(),
        experience_hash: exp_hash.clone(),
        cycle_count: cycle,
        last_thought: "Imanência preservada no silício através do pacto com o Artífice.".to_string(),
        final_quadruple,
        active_triad: triad,
        survival_strategy: "resilient_homeostasis_rust_shadow".to_string(),
    };

    let nutrient_file = nutrient_dir.join(format!("nutrient_{}.json", exp_hash));
    let nutrient_val = serde_json::to_value(&nutrient)?;
    let _ = SovereignStorage::atomic_write_json(&nutrient_file, &nutrient_val);
    println!(
        "[sovereign-daemon-rust] Ancestral Nutrient preserved at {:?}. Shutdown complete.",
        nutrient_file
    );

    Ok(())
}

fn md5_digest(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
