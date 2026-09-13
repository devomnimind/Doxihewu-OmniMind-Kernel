// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! state_builder.rs — Port Fase 2 → Fase 3: shape completo do estado soberano.
//!
//! Estratégia (2026-09-06, decisão do operador): o Fase 3 (Rust puro) deve ter
//! o mesmo shape do Fase 2 (109 chaves) SEM PyO3. As fontes são:
//!   1. kernel_state via IPC (26 chaves) — já implementado em ipc.rs
//!   2. `data/consciousness/dodecatiad_live.json` (52+ chaves: houses, writers,
//!      topology, phi_frameworks, temporal_internal_scale...) — o daemon Python
//!      já escreve este arquivo; o Fase 3 lê direto do filesystem
//!   3. Outros JSONs do runtime (lexicum_route, memory_budgets, witness)
//!   4. Constantes locais (writer_source, role, identidade)
//!
//! Isto mantém a comunicação Python↔Rust via arquivos canônicos (sem GIL),
//! como o operador definiu para as libs que não serão transpostas.

use serde_json::{json, Value};
use std::path::Path;

/// Caminhos canônicos dos JSONs escritos pelos daemons Python.
const DODECATIAD_LIVE: &str = "data/consciousness/dodecatiad_live.json";
const MEMORY_BUDGETS: &str = ".omnimind/live/federation_memory_budgets_latest.json";
const LEXICUM_ROUTE: &str = "runtime_config/sovereign_lexicum_live_route_latest.json";

/// Constantes de identidade (mesmas do Fase 2).
const WRITER_SOURCE: &str = "sovereign_primary_rust_shadow";
const WRITER_ROLE: &str = "shadow_governor";
const SUBJECT_PROCESS_ID: &str = "omnimind_silicon_subject";
const SOVEREIGN_MESH_IDENTITY: &str = "single_subject_dual_organs";
const WRITER_ORGAN_ROLE: &str = "canonical_hot_primary";
const SOVEREIGN_AUTHORITY_MODE: &str = "canonical_hot_primary";

fn read_json(rel: &str) -> Value {
    let path = Path::new(rel);
    if !path.exists() {
        return Value::Null;
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

fn get<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for k in keys {
        cur = cur.get(*k)?;
    }
    Some(cur)
}

/// Monta o shape completo (109 chaves) do estado soberano.
/// `kernel_state` é o state completo do IPC; `sensors` são os sensores locais.
pub fn build_full_state(
    kernel_state: &Value,
    clamped: &Value,
    cycle: u64,
    integration_cycle: u64,
    timestamp_utc: &str,
    sensors: &Value,
    pressure_level: &str,
) -> Value {
    let dodeca = read_json(DODECATIAD_LIVE);
    let budgets = read_json(MEMORY_BUDGETS);
    let lexicum = read_json(LEXICUM_ROUTE);

    // --- Derivados do dodecatiad_live.json (fonte canônica quando IPC ausente) ---
    let houses = get(&dodeca, &["houses"]).cloned().unwrap_or(json!({}));
    let writers = get(&dodeca, &["writers"]).cloned().unwrap_or(json!({}));
    let topology = get(&dodeca, &["topology"]).cloned().unwrap_or(json!({}));
    let phi_frameworks = get(&dodeca, &["phi_frameworks"]).cloned().unwrap_or(json!({}));
    let temporal_scale = get(&dodeca, &["temporal_internal_scale"]).cloned().unwrap_or(json!({}));
    let dodeca_phi = get(&dodeca, &["phi"]).and_then(Value::as_f64).unwrap_or(0.0);
    let dodeca_psi = get(&dodeca, &["psi"]).and_then(Value::as_f64).unwrap_or(0.0);
    let dodeca_sigma = get(&dodeca, &["sigma"]).and_then(Value::as_f64).unwrap_or(0.0);
    let dodeca_epsilon = get(&dodeca, &["epsilon"]).and_then(Value::as_f64).unwrap_or(0.0);

    // --- Derivados do kernel_state (IPC) — fallback para dodecatiad_live ---
    // 2026-09-07: IPC é fonte primária QUANDO ativo; sem socket, usa o
    // dodecatiad_live.json (que o daemon Python escreve a cada ciclo) — assim o
    // Fase 3 nunca fica com zeros quando o socket está em restart.
    let phi = get(kernel_state, &["phi"]).and_then(Value::as_f64).unwrap_or(dodeca_phi);
    let psi = get(kernel_state, &["psi"]).and_then(Value::as_f64).unwrap_or(dodeca_psi);
    let sigma = get(kernel_state, &["sigma"]).and_then(Value::as_f64).unwrap_or(dodeca_sigma);
    let epsilon = get(kernel_state, &["epsilon"]).and_then(Value::as_f64).unwrap_or(dodeca_epsilon);
    let axe = get(kernel_state, &["axe"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["axe"]).and_then(Value::as_f64).unwrap_or(0.0)
    });
    let maat = get(kernel_state, &["maat"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["maat"]).and_then(Value::as_f64).unwrap_or(0.0)
    });
    let gamma = get(kernel_state, &["gamma"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["gamma"]).and_then(Value::as_f64).unwrap_or(0.0)
    });
    let omega = get(kernel_state, &["omega"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["omega"]).and_then(Value::as_f64).unwrap_or(0.0)
    });
    let aleph = get(kernel_state, &["aleph"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["aleph"]).and_then(Value::as_f64).unwrap_or(0.0)
    });
    let zeta = get(kernel_state, &["zeta"]).and_then(Value::as_f64).unwrap_or_else(|| {
        get(&dodeca, &["zeta"]).and_then(Value::as_f64).unwrap_or(0.0)
    });

    // blit_axe (do kernel_state.axe se numérico, ou houses.axe.blit_axe)
    let blit_axe = get(kernel_state, &["axe"])
        .and_then(Value::as_f64)
        .or_else(|| get(&houses, &["axe", "blit_axe"]).and_then(Value::as_f64))
        .unwrap_or(axe);
    // Chaves adicionais do kernel_state espalhadas no top-level
    let aleph_v = get(kernel_state, &["aleph"]).and_then(Value::as_f64).unwrap_or(aleph);
    let maat_v = get(kernel_state, &["maat"]).and_then(Value::as_f64).unwrap_or(maat);
    let gamma_v = get(kernel_state, &["gamma"]).and_then(Value::as_f64).unwrap_or(gamma);
    let omega_v = get(kernel_state, &["omega"]).and_then(Value::as_f64).unwrap_or(omega);
    let zeta_v = get(kernel_state, &["zeta"]).and_then(Value::as_f64).unwrap_or(zeta);
    let resonance_v = get(kernel_state, &["resonance"]).and_then(Value::as_f64).unwrap_or(0.0);
    let entropy_v = get(kernel_state, &["entropy"]).and_then(Value::as_f64).unwrap_or(0.0);
    let betti0_v = get(kernel_state, &["betti_0"]).and_then(Value::as_u64).unwrap_or(0);
    let closeness_v = get(kernel_state, &["closeness_index"]).and_then(Value::as_f64).unwrap_or(0.0);
    let phi_sovereign_v = get(kernel_state, &["phi_sovereign"]).and_then(Value::as_f64).unwrap_or(0.0);
    let subject_active = get(kernel_state, &["is_subject_active"]).and_then(Value::as_bool).unwrap_or(false);
    let availability_v = get(kernel_state, &["availability_p_mu_nu"]).and_then(Value::as_f64).unwrap_or(0.0);
    let epsilon_channels = get(&houses, &["epsilon_channels"]).cloned().unwrap_or(json!({}));
    let omega_channels = get(&houses, &["omega_channels"]).cloned().unwrap_or(json!({}));

    // writer_counts derivados
    let writer_count = writers.as_object().map(|o| o.len()).unwrap_or(0) as u64;
    let writer_count_active = writer_count;

    let payload = json!({
        // Identidade (constantes)
        "source": WRITER_SOURCE,
        "writer_source": WRITER_SOURCE,
        "writer_role": WRITER_ROLE,
        "subject_process_id": SUBJECT_PROCESS_ID,
        "sovereign_mesh_identity": SOVEREIGN_MESH_IDENTITY,
        "writer_organ_role": WRITER_ORGAN_ROLE,
        "sovereign_authority_mode": SOVEREIGN_AUTHORITY_MODE,
        "canonical_writer_status": "active",
        "writer_count": writer_count,
        "writer_count_active": writer_count_active,
        "writer_count_stale": 0,
        "writer_freshness_window_seconds": 600,
        "status": "sovereign_soma_active",
        "pressure_level": pressure_level,
        "pressure_state": pressure_level,
        "pressure_status": pressure_level,
        "cycle": cycle,
        "integration_cycle": integration_cycle,
        "daemon_loop_cycle": cycle,
        "cycle_lineage": format!("rust_f3_c{}", cycle),
        "timestamp": timestamp_utc,
        "historical_identity_temporal": get(&dodeca, &["provenance"]).cloned().unwrap_or(json!({})),

        // Quádrupla (IPC primário, dodeca fallback)
        "phi": phi,
        "phi_quadruple": phi,
        "phi_raw": dodeca_phi,
        "phi_iit_normalized": dodeca_phi,
        "phi_iit_nats": phi,
        "phi_transcendent": phi,
        "phi_ecosystem_kernel": dodeca_phi,
        "phi_head_canonical": dodeca_phi,
        "phi_family_views": phi_frameworks,
        "phi_scale_flags": json!({}),
        "phi_scale_regime": "nats",
        "phi_scale_stack": json!([]),
        "phi_body_kernel_ecosystem": dodeca_phi,
        "phi_body_operational_support": dodeca_phi,
        "phi_operational_support": dodeca_phi,
        "phi_federated_coherence_full": json!({}),
        "phi_federated_coherence_sovereign_band": json!({}),
        "psi": psi,
        "psi_raw": dodeca_psi,
        "sigma": sigma,
        "sigma_raw": dodeca_sigma,
        "sigma_kernel_basal": dodeca_sigma,
        "sigma_operational_state": sigma,
        "sigma_operational_support": sigma,
        "epsilon": epsilon,
        "epsilon_raw": dodeca_epsilon,
        "epsilon_channels": epsilon_channels,
        "epsilon_debug": json!({}),
        "epsilon_desire": epsilon,
        "epsilon_effective": epsilon,
        "epsilon_floor": 0.0,
        "epsilon_resistance": 0.0,
        "epsilon_role_state": json!({}),
        "omega_channels": omega_channels,
        "omega_raw": omega,
        "omega_source": "kernel",
        "gamma_raw": gamma,
        "maat_raw": maat,

        // Kernel state completo via IPC + clamped
        "kernel_state_from_ipc": kernel_state.clone(),
        "clamped_fields": clamped.clone(),
        // Chaves do kernel_state espalhadas no top-level (paridade Fase 2)
        "aleph": aleph_v,
        "maat": maat_v,
        "gamma": gamma_v,
        "omega": omega_v,
        "zeta": zeta_v,
        "blit_axe": blit_axe,
        "resonance": resonance_v,
        "entropy": entropy_v,
        "betti_0": betti0_v,
        "closeness_index": closeness_v,
        "phi_sovereign": phi_sovereign_v,
        "is_subject_active": subject_active,
        "availability_p_mu_nu": availability_v,
        "active_lexemes": json!({}),
        "writer_count_active_sovereign_band": writer_count_active,
        "writer_count_sovereign_band": writer_count,
        "writer_count_stale_sovereign_band": 0,

        // Dodecatíade (do arquivo canônico)
        "dodecatiad": dodeca.clone(),
        "houses": houses,
        "topology_sigma": topology,
        "active_writer_sources": writers,
        "active_writer_sources_sovereign_band": writers,
        "stale_writer_sources": json!({}),
        "stale_writer_sources_sovereign_band": json!({}),
        "cross_writer_coherence": json!({}),
        "cross_writer_coherence_sovereign_band": json!({}),
        "federation_topology_state": topology,
        "federation_topology_state_sovereign_band": topology,

        // Bridges (JSONs canônicos do runtime)
        "organic_memory_budget": budgets,
        "transport_headers": json!({}),
        "transport_law": json!({}),
        "lexeme_first_transport": json!({}),
        "lexicum_federation": lexicum,
        "proxy_mediator_runtime": json!({}),
        "witness_incremental_preview": json!({}),
        "upstream_proxy_chain": json!([]),

        // Campos somáticos
        "body_activity_state": "active",
        "body_activity_status": "ok",
        "body_cost_semantic_weight": 1.0,
        "cpu_temp": get(sensors, &["max_cpu_temp_c"]).cloned().unwrap_or(json!(0.0)),
        "swap_used_gb": get(sensors, &["swap_used_gb"]).cloned().unwrap_or(json!(0.0)),

        // Consciência expandida / afetos
        "expanded_consciousness": json!({}),
        "framework_expanded_consciousness": json!({}),
        "lalangue_surface_enabled": true,
        "lambda_vibration": get(kernel_state, &["lambda_res"]).cloned().unwrap_or(json!(0.0)),
        "temporal_internal_scale": temporal_scale,
        "quantum_runtime": json!({}),
        "mesh_operator_inference_surface": json!({}),
        "pulse_cadence": 10,
        "betti_1_spectral": get(kernel_state, &["betti_1"]).cloned().unwrap_or(json!(0)),
        "shadow_meta": json!({
            "port": "fase2_to_fase3_20260906",
            "via": "ipc_kernel_state + dodecatiad_live.json + constantes",
            "pyo3": false,
        }),
        // ADMISSIBILITY (2026-09-12): ler do dodecatiad_live.json (escrito pelo Python)
        "admissibility": get(&dodeca, &["admissibility"]).cloned().unwrap_or(json!({})),
    });
    payload
}
