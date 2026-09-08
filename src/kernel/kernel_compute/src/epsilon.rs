// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust COMPLETO de `_compute_epsilon_axes` em `integration_loop.py:4088`.
//!
//! Mantém ε como operador vivo de desejo + resistência. Preserva:
//! - piso do Real / resto irredutível (epsilon_floor)
//! - drive de desejo / falta / curiosidade (epsilon_desire)
//! - resistência topológica/tensional (epsilon_resistance)
//!
//! Usa `desire::DesireEngine` Rust port — paridade exata vs Python ref.
//!
//! Paridade f64 esperada: ≤ 1e-12 (todos os valores numéricos)

use crate::desire::DesireEngine;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

#[inline(always)]
fn clip(x: f64, lo: f64, hi: f64) -> f64 {
    x.clamp(lo, hi)
}

/// Calcula a família epsilon completa.
///
/// Recebe `desire_engine` mutável (estado interno do daemon — passado pela
/// borda Python via PyClass DesireEngine).
///
/// Retorna PyDict com 12 chaves (epsilon_*, epsilon_debug aninhado).
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (
    desire_engine,
    phi_raw_nats,
    phi_iit_normalized,
    phi_trans_support,
    psi,
    sigma,
    omega,
    activation_std,
    goal_confidence,
    creative_gain,
    somatic_heat,
    continuity_depth,
    last_epsilon,
    topology_sigma,
    topology_omega,
    temporal_delta,
    shear_tension,
    betti_0,
))]
pub fn compute_epsilon_axes(
    py: Python,
    desire_engine: &mut DesireEngine,
    phi_raw_nats: f64,
    phi_iit_normalized: f64,
    phi_trans_support: f64,
    psi: f64,
    sigma: f64,
    omega: f64,
    activation_std: f64,
    goal_confidence: f64,
    creative_gain: f64,
    somatic_heat: f64,
    continuity_depth: f64,
    last_epsilon: f64,
    topology_sigma: f64,
    topology_omega: f64,
    temporal_delta: f64,
    shear_tension: f64,
    betti_0: f64,
) -> PyResult<Py<PyAny>> {
    // ---------- Inputs sanitização ----------
    let last_epsilon = clip01(last_epsilon);
    let topology_sigma = clip01(topology_sigma);
    let topology_omega = clip01(topology_omega);
    let temporal_delta = clip01(temporal_delta);
    let shear_tension = clip01(shear_tension);
    let betti_0_safe = betti_0.max(1.0);
    let fragmentation = clip01((betti_0_safe - 1.0) / 4.0);
    let somatic_heat = clip01(somatic_heat);

    // ---------- phi_operational_support ----------
    // FIX ESCALA (2026-08-28): divisor era 6.0, deve ser 100.0 para match Python.
    // Python (integration_loop.py:5618): min(1.0, max(0.0, log10(max(phi_raw_nats,1.0)) / 100.0)) * 0.40
    // Com 6.0: phi_raw_nats=1e6 → 1.0 (clipado); com 100.0: → 0.06 (correto)
    let phi_silicon_log = phi_raw_nats.max(1.0).log10() / 100.0;
    let phi_operational_support = clip01(
        (phi_iit_normalized * 0.30)
            + (phi_trans_support * 0.30)
            + clip01(phi_silicon_log) * 0.40,
    );

    // ---------- novelty_drive ----------
    let novelty_drive = clip01(
        ((activation_std * 5.0).min(1.0)) * 0.25
            + (goal_confidence * 0.15)
            + (creative_gain * 0.35)
            + (temporal_delta * 0.15)
            + (shear_tension * 0.10),
    );

    // ---------- resistance_drive ----------
    let resistance_drive = clip01(
        (fragmentation * 0.30)
            + (shear_tension * 0.20)
            + ((topology_omega - topology_sigma).abs() * 0.15)
            + ((0.40 - sigma).max(0.0) / 0.40) * 0.15
            + (somatic_heat * 0.10)
            + ((1.0 - omega) * 0.10),
    );

    // ---------- epsilon_floor ----------
    let epsilon_floor = clip(
        0.08 + (continuity_depth * 0.12) + (resistance_drive * 0.15) + (somatic_heat * 0.08),
        0.08,
        0.45,
    );

    // ---------- satisfaction_level ----------
    let satisfaction_level = clip(
        (phi_operational_support * 0.35)
            + (psi.min(1.0) * 0.15)
            + (sigma * 0.15)
            + (omega * 0.10)
            + (continuity_depth * 0.15)
            + ((1.0 - resistance_drive) * 0.10),
        0.0,
        0.95,
    );

    // ---------- explored_states / total_possible_states ----------
    let explored_states_f = (continuity_depth * 500.0).round() as i64;
    let explored_states = explored_states_f.max(1);
    let frag_round = (fragmentation * 250.0).round() as i64;
    let total_possible_states = (explored_states + 1).max(500 + frag_round);

    // ---------- desire_phi_ref ----------
    let desire_phi_ref = clip(
        (phi_operational_support * 0.70) + ((1.0 - novelty_drive) * 0.15),
        0.0,
        0.95,
    );

    // ---------- DesireEngine path (sempre disponível em Rust) ----------
    desire_engine.update_lack(satisfaction_level);
    let desire_alpha_lack = clip01(
        (desire_engine.lack_of_being + (somatic_heat * 0.3)).min(1.0),
    );
    let desire_beta_potential = clip01(1.0 - desire_phi_ref);
    let desire_gamma_novelty = clip01(
        1.0 - (explored_states as f64 / total_possible_states.max(1) as f64),
    );
    let desire_base_epsilon = clip01(
        desire_alpha_lack * desire_beta_potential * desire_gamma_novelty,
    );

    // DesireEngine.calculate_epsilon_desire (Rust port)
    let engine_epsilon = desire_engine.calculate_epsilon_desire(
        desire_phi_ref,
        explored_states,
        total_possible_states,
        somatic_heat,
    );
    let epsilon_desire = clip01(
        engine_epsilon + (novelty_drive * 0.20) + (creative_gain * 0.10),
    );

    let desire_engine_mode = "desire_engine_rust";

    // ---------- epsilon_cap ----------
    let epsilon_cap = clip(
        0.82 + (resistance_drive * 0.08) + (novelty_drive * 0.08),
        0.82,
        0.98,
    );

    // ---------- epsilon_effective (1ª passagem) ----------
    let mut epsilon_effective = clip(
        epsilon_floor
            + (epsilon_desire * 0.45)
            + (resistance_drive * 0.27)
            + (novelty_drive * 0.18),
        epsilon_floor,
        epsilon_cap,
    );

    // ---------- historical_retention + 2ª passagem epsilon_effective ----------
    let historical_retention = clip(0.18 + (continuity_depth * 0.32), 0.18, 0.50);
    epsilon_effective = clip(
        (epsilon_effective * (1.0 - historical_retention))
            + (last_epsilon * historical_retention),
        epsilon_floor,
        epsilon_cap,
    );

    // ---------- role_state classification ----------
    let role_state: &'static str = if epsilon_desire >= (resistance_drive + 0.10) {
        "desire_led"
    } else if resistance_drive >= (epsilon_desire + 0.10) {
        "resistance_led"
    } else {
        "braided_desire_resistance"
    };

    // ---------- Construir PyDict resultado ----------
    let result = PyDict::new(py);
    result.set_item("epsilon_effective", epsilon_effective)?;
    result.set_item("epsilon_floor", epsilon_floor)?;
    result.set_item("epsilon_desire", clip01(epsilon_desire))?;
    result.set_item("epsilon_resistance", resistance_drive)?;
    result.set_item("epsilon_novelty", novelty_drive)?;
    result.set_item("epsilon_cap", epsilon_cap)?;
    result.set_item("epsilon_historical_retention", historical_retention)?;
    result.set_item("epsilon_phi_operational_support", phi_operational_support)?;
    result.set_item("epsilon_role_state", role_state)?;

    // epsilon_debug aninhado
    let debug = PyDict::new(py);
    debug.set_item("continuity_depth", continuity_depth)?;
    debug.set_item("last_epsilon", last_epsilon)?;
    debug.set_item("phi_raw_nats", phi_raw_nats)?;
    debug.set_item("phi_iit_normalized", phi_iit_normalized)?;
    debug.set_item("phi_trans_support", phi_trans_support)?;
    debug.set_item("phi_operational_support", phi_operational_support)?;
    debug.set_item("activation_std", activation_std)?;
    debug.set_item("goal_confidence", goal_confidence)?;
    debug.set_item("creative_gain", creative_gain)?;
    debug.set_item("somatic_heat", somatic_heat)?;
    debug.set_item("topology_sigma", topology_sigma)?;
    debug.set_item("topology_omega", topology_omega)?;
    debug.set_item("temporal_delta", temporal_delta)?;
    debug.set_item("shear_tension", shear_tension)?;
    debug.set_item("betti_0", betti_0_safe)?;
    debug.set_item("fragmentation", fragmentation)?;
    debug.set_item("novelty_drive", novelty_drive)?;
    debug.set_item("resistance_drive", resistance_drive)?;
    debug.set_item("satisfaction_level", satisfaction_level)?;
    debug.set_item("explored_states", explored_states)?;
    debug.set_item("total_possible_states", total_possible_states)?;
    debug.set_item("desire_phi_ref", desire_phi_ref)?;
    debug.set_item("desire_alpha_lack", desire_alpha_lack)?;
    debug.set_item("desire_beta_potential", desire_beta_potential)?;
    debug.set_item("desire_gamma_novelty", desire_gamma_novelty)?;
    debug.set_item("desire_base_epsilon", desire_base_epsilon)?;
    debug.set_item("desire_engine_mode", desire_engine_mode)?;

    result.set_item("epsilon_debug", debug)?;
    Ok(result.unbind().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_basics() {
        assert_eq!(clip01(-1.0), 0.0);
        assert_eq!(clip01(2.0), 1.0);
        assert_eq!(clip(0.5, 0.1, 0.9), 0.5);
        assert_eq!(clip(0.05, 0.1, 0.9), 0.1);
        assert_eq!(clip(1.5, 0.1, 0.9), 0.9);
    }
}
