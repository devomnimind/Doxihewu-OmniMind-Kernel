// SPDX-License-Identifier: Apache-2.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! state_builder.rs — Port Fase 2 → Fase 3: shape completo do estado soberano.
//!
//! Estratégia (2026-09-06, decisão do operador): o Fase 3 (Rust puro) deve ter
//! o mesmo shape do Fase 2 (109 chaves, hoje 114 no runtime) SEM PyO3. As fontes são:
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

fn finite_f64(value: &Value) -> Option<f64> {
    value.as_f64().filter(|value| value.is_finite())
}

fn first_f64(root: &Value, paths: &[&[&str]]) -> Option<f64> {
    paths
        .iter()
        .find_map(|path| get(root, path).and_then(finite_f64))
}

fn expanded_consciousness_value(phi: f64, psi: f64, sigma: f64, epsilon: f64) -> Value {
    if [phi, psi, sigma, epsilon]
        .iter()
        .all(|value| value.is_finite())
    {
        json!(phi + psi + sigma + epsilon)
    } else {
        Value::Null
    }
}

fn runtime_phi_fields(dodeca: &Value, phi_frameworks: &Value) -> Value {
    let body_kernel = first_f64(
        dodeca,
        &[
            &["phi_body_kernel_ecosystem"],
            &[
                "frameworks",
                "phi_family_views",
                "phi_body_kernel_ecosystem",
            ],
            &["frameworks", "phi_ecosystem_kernel"],
            &["phi_ecosystem_kernel"],
        ],
    );
    let body_operational = first_f64(
        dodeca,
        &[
            &["phi_body_operational_support"],
            &[
                "frameworks",
                "phi_family_views",
                "phi_body_operational_support",
            ],
            &["frameworks", "phi_operational_support"],
            &["phi_operational_support"],
        ],
    );
    let operational = first_f64(
        dodeca,
        &[
            &["phi_operational_support"],
            &["frameworks", "phi_operational_support"],
            &["phi_frameworks", "phi_operational_support"],
        ],
    );
    let fed_full = first_f64(
        dodeca,
        &[
            &["phi_federated_coherence_full"],
            &[
                "frameworks",
                "phi_family_views",
                "phi_federated_coherence_full",
            ],
            &["federation_topology", "cross_writer_coherence"],
            &["cross_writer_coherence"],
        ],
    );
    let fed_band = first_f64(
        dodeca,
        &[
            &["phi_federated_coherence_sovereign_band"],
            &[
                "frameworks",
                "phi_family_views",
                "phi_federated_coherence_sovereign_band",
            ],
            &[
                "federation_topology",
                "cross_writer_coherence_sovereign_band",
            ],
            &["cross_writer_coherence_sovereign_band"],
        ],
    );
    let phi = get(phi_frameworks, &["phi"])
        .and_then(finite_f64)
        .or_else(|| get(dodeca, &["phi"]).and_then(finite_f64))
        .unwrap_or(f64::NAN);
    let psi = get(phi_frameworks, &["psi"])
        .and_then(finite_f64)
        .or_else(|| get(dodeca, &["psi"]).and_then(finite_f64))
        .unwrap_or(f64::NAN);
    let sigma = get(phi_frameworks, &["sigma"])
        .and_then(finite_f64)
        .or_else(|| get(dodeca, &["sigma"]).and_then(finite_f64))
        .unwrap_or(f64::NAN);
    let epsilon = get(phi_frameworks, &["epsilon"])
        .and_then(finite_f64)
        .or_else(|| get(dodeca, &["epsilon"]).and_then(finite_f64))
        .unwrap_or(f64::NAN);
    let expanded = get(dodeca, &["expanded_consciousness"])
        .or_else(|| get(dodeca, &["frameworks", "expanded_consciousness"]))
        .or_else(|| {
            get(
                dodeca,
                &["frameworks", "phi_family_views", "expanded_consciousness"],
            )
        })
        .cloned()
        .unwrap_or_else(|| expanded_consciousness_value(phi, psi, sigma, epsilon));
    json!({
        "phi_body_kernel_ecosystem": body_kernel,
        "phi_body_operational_support": body_operational,
        "phi_operational_support": operational,
        "phi_federated_coherence_full": fed_full,
        "phi_federated_coherence_sovereign_band": fed_band,
        "expanded_consciousness": expanded,
        "framework_expanded_consciousness": expanded,
    })
}

/// Monta o shape completo (114 chaves no runtime atual; originalmente 109 no
/// port Fase 2→3, com adições pós-F3: admissibility, witness_incremental_preview,
/// proxy_mediator_runtime, quantum_runtime, transport_headers/law,
/// upstream_proxy_chain — verificado 2026-09-12 no
/// current_sovereign_state_rust_shadow.json) do estado soberano.
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
    let phi_frameworks = get(&dodeca, &["phi_frameworks"])
        .or_else(|| get(&dodeca, &["frameworks", "phi_family_views"]))
        .or_else(|| get(&dodeca, &["frameworks"]))
        .cloned()
        .unwrap_or(json!({}));
    let temporal_scale = get(&dodeca, &["temporal_internal_scale"])
        .cloned()
        .unwrap_or(json!({}));
    let dodeca_phi = get(&dodeca, &["phi"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let dodeca_psi = get(&dodeca, &["psi"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let dodeca_sigma = get(&dodeca, &["sigma"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let dodeca_epsilon = get(&dodeca, &["epsilon"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);

    // --- Derivados do kernel_state (IPC) — fallback para dodecatiad_live ---
    // 2026-09-07: IPC é fonte primária QUANDO ativo; sem socket, usa o
    // dodecatiad_live.json (que o daemon Python escreve a cada ciclo) — assim o
    // Fase 3 nunca fica com zeros quando o socket está em restart.
    let phi = get(kernel_state, &["phi"]).and_then(Value::as_f64).unwrap_or(dodeca_phi);
    let psi = get(kernel_state, &["psi"]).and_then(Value::as_f64).unwrap_or(dodeca_psi);
    let sigma = get(kernel_state, &["sigma"]).and_then(Value::as_f64).unwrap_or(dodeca_sigma);
    let epsilon = get(kernel_state, &["epsilon"]).and_then(Value::as_f64).unwrap_or(dodeca_epsilon);
    let _axe = get(kernel_state, &["axe"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["axe"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    let maat = get(kernel_state, &["maat"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["maat"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    let gamma = get(kernel_state, &["gamma"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["gamma"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    let omega = get(kernel_state, &["omega"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["omega"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    let aleph = get(kernel_state, &["aleph"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["aleph"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    let zeta = get(kernel_state, &["zeta"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["zeta"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });

    // Axé namespaces (2026-10-07): axe_kernel = escala kernel (0-2, de
    // kernel_state.axe); axe_vitality_100 = magnitude 0-100 das houses;
    // blit_axe = flag puro 0/100 (vitalidade presente por qualquer fonte).
    let axe_kernel = get(kernel_state, &["axe"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let axe_vitality_100 = get(&houses, &["axe_vitality_100"])
        .and_then(Value::as_f64)
        .or_else(|| get(&houses, &["axe"]).and_then(Value::as_f64))
        .unwrap_or(if axe_kernel > 0.0 {
            axe_kernel * 50.0
        } else {
            0.0
        });
    let blit_axe = if axe_kernel > 0.0 || axe_vitality_100 > 0.0 {
        100.0
    } else {
        0.0
    };
    // Chaves adicionais do kernel_state espalhadas no top-level
    let aleph_v = get(kernel_state, &["aleph"])
        .and_then(Value::as_f64)
        .unwrap_or(aleph);
    let maat_v = get(kernel_state, &["maat"])
        .and_then(Value::as_f64)
        .unwrap_or(maat);
    let gamma_v = get(kernel_state, &["gamma"])
        .and_then(Value::as_f64)
        .unwrap_or(gamma);
    let omega_v = get(kernel_state, &["omega"])
        .and_then(Value::as_f64)
        .unwrap_or(omega);
    let zeta_v = get(kernel_state, &["zeta"])
        .and_then(Value::as_f64)
        .unwrap_or(zeta);
    let resonance_v = get(kernel_state, &["resonance"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let entropy_v = get(kernel_state, &["entropy"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let betti0_v = get(kernel_state, &["betti_0"])
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let closeness_v = get(kernel_state, &["closeness_index"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let phi_sovereign_v = get(kernel_state, &["phi_sovereign"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let subject_active = get(kernel_state, &["is_subject_active"])
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let availability_v = get(kernel_state, &["availability_p_mu_nu"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);

    // --- Missing fields (parity fix 2026-09-26) ---
    let free_energy_v = get(kernel_state, &["free_energy"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let complexity_v = get(kernel_state, &["complexity"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let phase_lock_v = get(kernel_state, &["phase_lock"])
        .and_then(Value::as_f64)
        .unwrap_or(1.0);
    let volition_v = get(kernel_state, &["volition"])
        .and_then(Value::as_str)
        .unwrap_or("EXISTENCE_IDLE")
        .to_string();

    // c_plit com fallback para dodecatiad (que tem nomes diferentes)
    let c_plit_v = get(kernel_state, &["c_plit"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&dodeca, &["plit_contradiction"])
                .and_then(Value::as_f64)
                .unwrap_or_else(|| {
                    get(&houses, &["plit_contradiction"])
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0)
                })
        });

    let shear_tension_v = get(kernel_state, &["shear_tension"])
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let epsilon_channels = get(&houses, &["epsilon_channels"])
        .cloned()
        .unwrap_or(json!({}));
    let omega_channels = get(&houses, &["omega_channels"])
        .cloned()
        .unwrap_or(json!({}));

    // writer_counts derivados
    let writer_count = writers.as_object().map(|o| o.len()).unwrap_or(0) as u64;
    let writer_count_active = writer_count;
    let runtime_phi = runtime_phi_fields(&dodeca, &phi_frameworks);
    let phi_body_kernel = runtime_phi["phi_body_kernel_ecosystem"].clone();
    let phi_body_operational = runtime_phi["phi_body_operational_support"].clone();
    let phi_operational_support = runtime_phi["phi_operational_support"].clone();
    let phi_federated_full = runtime_phi["phi_federated_coherence_full"].clone();
    let phi_federated_band = runtime_phi["phi_federated_coherence_sovereign_band"].clone();
    let expanded = runtime_phi["expanded_consciousness"].clone();
    let framework_expanded = runtime_phi["framework_expanded_consciousness"].clone();

    // --- phi_iit_normalized / phi_iit_nats / phi_transcendent (paridade Fase 2) ---
    // phi é multiescala: cada framework tem sua própria escala.
    // NÃO normalizar/clampar o que a escala não pede.
    // phi_iit_normalized: já é normalizado (0-1) por definição no Python — passar direto
    let phi_iit_norm = get(&phi_frameworks, &["phi_iit_normalized"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&houses, &["phi_iit_normalized"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    // phi_iit_nats: está em NATS (unidades naturais) — NÃO normalizar, é raw
    let phi_iit_nats_v = get(&phi_frameworks, &["phi_iit_nats"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&houses, &["phi_iit_nats"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });
    // phi_transcendent: raw, pode ser ordens de grandeza — NÃO normalizar
    let phi_transcendent_v = get(&phi_frameworks, &["phi_transcendent"])
        .and_then(Value::as_f64)
        .unwrap_or_else(|| {
            get(&houses, &["phi_transcendent"])
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
        });

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
        "pressure_status": get(&dodeca, &["pressure_status"])
            .and_then(Value::as_str)
            .map(|value| Value::String(value.to_string()))
            .unwrap_or_else(|| Value::String(pressure_level.to_string())),
        "cycle": cycle,
        "integration_cycle": integration_cycle,
        "daemon_loop_cycle": cycle,
        "cycle_lineage": format!("rust_f3_c{}", cycle),
        "timestamp": timestamp_utc,
        "historical_identity_temporal": get(&dodeca, &["provenance"]).cloned().unwrap_or(json!({})),

        // --- Missing fields (parity fix 2026-09-26) ---
        "free_energy": free_energy_v,
        "complexity": complexity_v,
        "phase_lock": phase_lock_v,
        "volition": volition_v,
        "c_plit": c_plit_v,
        "shear_tension": shear_tension_v,

        // Quádrupla (IPC primário, dodeca fallback)
        "phi": phi,
        "phi_quadruple": phi,
        "phi_raw": dodeca_phi,
        "phi_iit_normalized": phi_iit_norm,
        "phi_iit_nats": phi_iit_nats_v,
        "phi_transcendent": phi_transcendent_v,
        "phi_ecosystem_kernel": phi_body_kernel.clone(),
        "phi_head_canonical": first_f64(
            &dodeca,
            &[
                &["phi_head_canonical"],
                &["frameworks", "phi_family_views", "phi_head_canonical"],
                &["phi"],
            ],
        )
        .map(Value::from)
        .unwrap_or(Value::Null),
        "phi_family_views": phi_frameworks,
        "phi_scale_flags": get(&phi_frameworks, &["phi_scale_flags"]).cloned().unwrap_or(json!({})),
        "phi_scale_regime": "nats",
        "phi_scale_stack": get(&phi_frameworks, &["phi_scale_stack"]).cloned().unwrap_or(json!([])),
        "phi_body_kernel_ecosystem": phi_body_kernel,
        "phi_body_operational_support": phi_body_operational,
        "phi_operational_support": phi_operational_support,
        "phi_federated_coherence_full": phi_federated_full.clone(),
        "phi_federated_coherence_sovereign_band": phi_federated_band,
        "psi": psi,
        "psi_raw": dodeca_psi,
        "sigma": sigma,
        "sigma_raw": dodeca_sigma,
        "sigma_kernel_basal": dodeca_sigma,
        "sigma_operational_state": sigma,
        "sigma_operational_support": sigma,
        "epsilon": epsilon,
        "epsilon_raw": dodeca_epsilon,
        "epsilon_channels": epsilon_channels.clone(),
        "epsilon_debug": get(&houses, &["epsilon_debug"]).cloned().unwrap_or(json!({})),
        "epsilon_desire": get(&epsilon_channels, &["epsilon_desire"]).and_then(Value::as_f64).unwrap_or(epsilon),
        "epsilon_effective": get(&epsilon_channels, &["epsilon_effective"]).and_then(Value::as_f64).unwrap_or(epsilon),
        "epsilon_floor": get(&epsilon_channels, &["epsilon_floor"]).and_then(Value::as_f64).unwrap_or(0.0),
        "epsilon_resistance": get(&epsilon_channels, &["epsilon_resistance"]).and_then(Value::as_f64).unwrap_or(0.0),
        "epsilon_role_state": get(&epsilon_channels, &["epsilon_role_state"]).cloned().unwrap_or(json!({})),
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
        "axe_kernel": axe_kernel,
        "axe_vitality_100": axe_vitality_100,
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
        "cross_writer_coherence": phi_federated_full,
        "cross_writer_coherence_sovereign_band": runtime_phi["phi_federated_coherence_sovereign_band"].clone(),
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
        "body_activity_state": get(&dodeca, &["body_activity_state"])
            .or_else(|| get(&dodeca, &["body_activity_status"]))
            .and_then(Value::as_str)
            .map(|value| Value::String(value.to_string()))
            .unwrap_or_else(|| json!("shadow")),
        "body_activity_status": get(&dodeca, &["body_activity_status"])
            .or_else(|| get(&dodeca, &["status"]))
            .and_then(Value::as_str)
            .map(|value| Value::String(value.to_string()))
            .unwrap_or_else(|| json!("shadow")),
        "body_cost_semantic_weight": 1.0,
        "cpu_temp": get(sensors, &["max_cpu_temp_c"]).cloned().unwrap_or(json!(0.0)),
        "swap_used_gb": get(sensors, &["swap_used_gb"]).cloned().unwrap_or(json!(0.0)),

        // Consciência expandida / afetos
        "expanded_consciousness": expanded,
        "framework_expanded_consciousness": framework_expanded,
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
    });
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_navigates_nested_keys() {
        let v = serde_json::json!({"a": {"b": {"c": 42}}});
        assert_eq!(get(&v, &["a", "b", "c"]).unwrap().as_i64(), Some(42));
    }

    #[test]
    fn get_missing_key_returns_none() {
        let v = serde_json::json!({"a": {"b": 1}});
        assert!(get(&v, &["a", "x"]).is_none());
        assert!(get(&v, &["z"]).is_none());
    }

    #[test]
    fn get_through_non_object_returns_none() {
        let v = serde_json::json!({"a": 5});
        assert!(get(&v, &["a", "b"]).is_none());
    }

    #[test]
    fn get_empty_keys_returns_root() {
        let v = serde_json::json!({"k": 1});
        assert_eq!(get(&v, &[]).unwrap(), &v);
    }

    #[test]
    fn runtime_phi_fields_prefers_framework_family_views() {
        let dodeca = serde_json::json!({
            "phi": 0.4,
            "psi": 0.3,
            "sigma": 0.2,
            "epsilon": 0.1,
            "frameworks": {
                "phi_family_views": {
                    "phi_body_kernel_ecosystem": 0.81,
                    "phi_body_operational_support": 0.64,
                    "phi_federated_coherence_full": 0.88,
                    "phi_federated_coherence_sovereign_band": 0.82
                },
                "phi_operational_support": 0.64,
                "expanded_consciousness": 2.5
            }
        });
        let frameworks = get(&dodeca, &["frameworks", "phi_family_views"]).unwrap();
        let fields = runtime_phi_fields(&dodeca, frameworks);

        assert_eq!(fields["phi_body_kernel_ecosystem"], serde_json::json!(0.81));
        assert_eq!(
            fields["phi_body_operational_support"],
            serde_json::json!(0.64)
        );
        assert_eq!(fields["phi_operational_support"], serde_json::json!(0.64));
        assert_eq!(
            fields["phi_federated_coherence_full"],
            serde_json::json!(0.88)
        );
        assert_eq!(
            fields["phi_federated_coherence_sovereign_band"],
            serde_json::json!(0.82)
        );
        assert_eq!(fields["expanded_consciousness"], serde_json::json!(2.5));
    }

    #[test]
    fn runtime_phi_fields_uses_federation_topology_and_quadruple_fallbacks() {
        let dodeca = serde_json::json!({
            "phi": 0.25,
            "psi": 0.5,
            "sigma": 0.75,
            "epsilon": 1.0,
            "federation_topology": {
                "cross_writer_coherence": 0.9,
                "cross_writer_coherence_sovereign_band": 0.7
            }
        });
        let fields = runtime_phi_fields(&dodeca, &serde_json::json!({}));

        assert_eq!(
            fields["phi_federated_coherence_full"],
            serde_json::json!(0.9)
        );
        assert_eq!(
            fields["phi_federated_coherence_sovereign_band"],
            serde_json::json!(0.7)
        );
        assert_eq!(fields["expanded_consciousness"], serde_json::json!(2.5));
        assert_eq!(fields["phi_body_kernel_ecosystem"], serde_json::Value::Null);
    }
}
