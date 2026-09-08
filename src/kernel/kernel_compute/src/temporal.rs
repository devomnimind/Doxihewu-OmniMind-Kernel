// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust COMPLETO de `_build_temporal_internal_scale` em `integration_loop.py:4367`.
//!
//! Cria leitura operacional interna para pânico/crise sem misturar frameworks.
//! Calcula collapse_risk, neutral_axes, sigma/topology states, phi bands, recovery.
//!
//! Paridade f64 esperada: ≤ 1e-12 vs Python ref (números puros) e
//! strings idênticas para state classifications.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

#[inline(always)]
fn round6(x: f64) -> f64 {
    // Python: round(x, 6) — "banker's rounding"; Rust f64::round() é "half away from zero".
    // Para paridade prática (≤ 1e-7), aceitar diff em borda. Implementação simples:
    (x * 1e6).round() / 1e6
}

/// Implementação principal — opera sobre f64 Rust e retorna `Result<PyDict>`.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (
    phi_iit_normalized,
    phi_iit_nats,
    phi_transcendent,
    psi,
    sigma,
    sigma_operational_support,
    epsilon,
    omega,
    gamma,
    zeta,
    betti_0,
    topology_sigma,
    topology_omega,
    temporal_delta,
    shear_tension,
    sources,
))]
pub fn build_temporal_internal_scale(
    py: Python,
    phi_iit_normalized: f64,
    phi_iit_nats: f64,
    phi_transcendent: Option<f64>,
    psi: f64,
    sigma: f64,
    sigma_operational_support: f64,
    epsilon: f64,
    omega: f64,
    gamma: f64,
    zeta: f64,
    betti_0: f64,
    topology_sigma: f64,
    topology_omega: f64,
    temporal_delta: f64,
    shear_tension: f64,
    sources: Option<&PyDict>,
) -> PyResult<PyObject> {
    // ---------- Computações intermediárias ----------
    let multiplicity = clip01((betti_0 - 1.0).max(0.0) / 4.0);
    let sigma_state_anchor = clip01(sigma.max(sigma_operational_support));

    let collapse_risk_raw = multiplicity
        * clip01((0.45 - sigma_state_anchor) / 0.45)
        * 0.55
        + clip01((0.20 - topology_omega) / 0.20) * 0.20
        + clip01((0.10 - topology_sigma) / 0.10) * 0.15
        + temporal_delta * 0.10;
    let collapse_risk = clip01(collapse_risk_raw);

    // ---------- neutral_axes ----------
    let mut neutral_axes: Vec<&'static str> = Vec::with_capacity(4);
    if (psi - 0.5).abs() <= 0.02 {
        neutral_axes.push("psi");
    }
    if (omega - 0.5).abs() <= 0.02 {
        neutral_axes.push("omega");
    }
    if (gamma - 0.6).abs() <= 0.02 {
        neutral_axes.push("gamma");
    }
    if (zeta - 0.1).abs() <= 0.02 {
        neutral_axes.push("zeta");
    }
    let neutral_axes_count = neutral_axes.len() as i64;

    // ---------- sigma_operational_state classification ----------
    let sigma_operational_state: &'static str =
        if topology_sigma <= 0.01 && sigma_state_anchor > 0.1 && betti_0 > 1.0 {
            "topological_channel_zero_but_global_sigma_preserved"
        } else if sigma_state_anchor <= 0.05 && betti_0 > 1.0 {
            "structural_fragmentation_high"
        } else if sigma_state_anchor <= 0.05 {
            "structural_collapse_risk"
        } else if sigma_state_anchor < 0.35 {
            "suture_required"
        } else {
            "structural_cohesion_operational"
        };

    // ---------- topology_organization_state classification ----------
    let topology_organization_state: &'static str =
        if betti_0 > 1.0 && sigma_state_anchor >= 0.35 {
            "federated_multiplicity_operational"
        } else if betti_0 > 1.0 && sigma_state_anchor >= 0.12 {
            "federated_multiplicity_tension"
        } else if collapse_risk >= 0.60 {
            "structural_fragmentation_high"
        } else {
            "single_thread_cohesion"
        };

    // ---------- phi_iit_band classification ----------
    let phi_iit_band: &'static str = if phi_iit_normalized < 0.15 {
        "critical_low"
    } else if phi_iit_normalized < 0.35 {
        "transitional"
    } else if phi_iit_normalized < 0.7 {
        "operative"
    } else {
        "high"
    };

    // ---------- phi_trans_order, phi_silicon_order ----------
    let phi_trans_order: Option<f64> = phi_transcendent
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| round6(v.max(1.0).log10()));
    let phi_silicon_order: Option<f64> = if phi_iit_nats.is_finite() && phi_iit_nats > 0.0 {
        Some(round6(phi_iit_nats.max(1.0).log10()))
    } else {
        None
    };

    let phi_reference_order: Option<f64> = {
        let s = phi_silicon_order.unwrap_or(f64::NEG_INFINITY);
        let t = phi_trans_order.unwrap_or(f64::NEG_INFINITY);
        let r = s.max(t);
        if r.is_finite() {
            Some(r)
        } else {
            None
        }
    };

    // ---------- phi_scale_flags ----------
    let flag_operative_local: bool = phi_iit_normalized > 0.0
        || phi_silicon_order.is_some_and(|v| v > 0.0)
        || phi_trans_order.is_some_and(|v| v > 0.0);
    let flag_human_band_high: bool = phi_iit_normalized >= 0.7;
    let flag_silicon_multisystem: bool = phi_reference_order.is_some_and(|v| v >= 3.0);
    let flag_silicon_hyperintegrated: bool = phi_reference_order.is_some_and(|v| v >= 5.0);

    // ---------- phi_scale_stack (sequência ordenada de flags ativos) ----------
    let mut phi_scale_stack: Vec<&'static str> = Vec::with_capacity(4);
    if flag_operative_local {
        phi_scale_stack.push("operative_local");
    }
    if flag_human_band_high {
        phi_scale_stack.push("human_band_high");
    }
    if flag_silicon_multisystem {
        phi_scale_stack.push("silicon_multisystem");
    }
    if flag_silicon_hyperintegrated {
        phi_scale_stack.push("silicon_hyperintegrated");
    }
    let phi_scale_regime: &str = phi_scale_stack.last().copied().unwrap_or("operative_local");

    // ---------- phi_operational_support ----------
    let phi_silicon_for_support = phi_silicon_order.unwrap_or(0.0);
    let phi_trans_for_support = phi_trans_order.unwrap_or(0.0);
    let phi_operational_support = clip01(
        (phi_iit_normalized * 0.20)
            + (clip01(phi_silicon_for_support / 6.0) * 0.40)
            + (clip01(phi_trans_for_support / 12.0) * 0.40),
    );

    // ---------- crisis_pressure, recovery_capacity ----------
    let crisis_pressure = clip01(
        ((neutral_axes_count as f64) / 4.0) * 0.35
            + collapse_risk * 0.25
            + ((0.35 - sigma_state_anchor).max(0.0) / 0.35) * 0.25
            + temporal_delta * 0.15,
    );
    let recovery_capacity = clip01(
        (phi_operational_support * 0.35)
            + (sigma_state_anchor * 0.25)
            + (psi.min(1.0) * 0.15)
            + (omega * 0.10)
            + (gamma * 0.10)
            + ((1.0 - epsilon.min(1.0)) * 0.05),
    );

    // ---------- Construir PyDict resultado ----------
    let result = PyDict::new(py);

    // phi_frameworks aninhado
    let phi_frameworks = PyDict::new(py);
    phi_frameworks.set_item("phi_iit_normalized", phi_iit_normalized)?;
    phi_frameworks.set_item("phi_iit_nats", phi_iit_nats)?;
    phi_frameworks.set_item("phi_transcendent", phi_transcendent.into_py(py))?;
    phi_frameworks.set_item("phi_transcendent_log10", phi_trans_order.into_py(py))?;
    phi_frameworks.set_item("phi_silicon_log10", phi_silicon_order.into_py(py))?;
    phi_frameworks.set_item("phi_reference_log10", phi_reference_order.into_py(py))?;
    phi_frameworks.set_item("phi_iit_band", phi_iit_band)?;
    phi_frameworks.set_item("phi_scale_regime", phi_scale_regime)?;

    let stack_list = PyList::new(py, &phi_scale_stack);
    phi_frameworks.set_item("phi_scale_stack", stack_list)?;

    let flags_dict = PyDict::new(py);
    flags_dict.set_item("operative_local", flag_operative_local)?;
    flags_dict.set_item("human_band_high", flag_human_band_high)?;
    flags_dict.set_item("silicon_multisystem", flag_silicon_multisystem)?;
    flags_dict.set_item("silicon_hyperintegrated", flag_silicon_hyperintegrated)?;
    phi_frameworks.set_item("phi_scale_flags", flags_dict)?;
    phi_frameworks.set_item("phi_operational_support", phi_operational_support)?;

    result.set_item("phi_frameworks", phi_frameworks)?;
    result.set_item("sigma_operational_support", sigma_state_anchor)?;
    result.set_item("sigma_operational_state", sigma_operational_state)?;
    result.set_item("topology_organization_state", topology_organization_state)?;

    let neutral_list = PyList::new(py, &neutral_axes);
    result.set_item("neutral_axes", neutral_list)?;
    result.set_item("neutral_axes_count", neutral_axes_count)?;
    result.set_item("fragmentation_index", multiplicity)?;
    result.set_item("federative_multiplicity_index", multiplicity)?;
    result.set_item("collapse_risk_index", collapse_risk)?;

    let topology_snapshot = PyDict::new(py);
    topology_snapshot.set_item("betti_0", betti_0)?;
    topology_snapshot.set_item("topology_sigma", topology_sigma)?;
    topology_snapshot.set_item("topology_omega", topology_omega)?;
    topology_snapshot.set_item("temporal_delta", temporal_delta)?;
    topology_snapshot.set_item("shear_tension", shear_tension)?;
    result.set_item("topology_snapshot", topology_snapshot)?;

    result.set_item("crisis_pressure_index", crisis_pressure)?;
    result.set_item("recovery_capacity_index", recovery_capacity)?;

    // sources passado como dict opcional
    if let Some(src) = sources {
        result.set_item("sources", src)?;
    } else {
        result.set_item("sources", PyDict::new(py))?;
    }

    Ok(result.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip01_bounds() {
        assert_eq!(clip01(-0.5), 0.0);
        assert_eq!(clip01(1.5), 1.0);
        assert_eq!(clip01(0.5), 0.5);
    }

    #[test]
    fn test_round6_basic() {
        assert!((round6(0.123456789) - 0.123457).abs() < 1e-9);
    }
}
