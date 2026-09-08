// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust da função hot-path `_score_live_writer_snapshot` do IntegrationLoop.
//!
//! Esta função é chamada para cada writer snapshot (potencialmente múltiplas
//! vezes por ciclo) para pontuar a qualidade do snapshot e selecionar o
//! writer canônico. É compute puro: sem I/O, sem estado, apenas aritmética.
//!
//! Design: a função aceita valores primitivos (f64, bool, &str) extraídos
//! pelo wrapper Python. Isso evita o overhead de 30+ chamadas dict.get()
//! via PyO3, mantendo o dict access em Python (rápido) e a aritmética
//! em Rust (f64 nativo).
//!
//! Paridade f64 esperada: ≤ 1e-15
//!
//! Referência Python: `src/consciousness/integration_loop.py:6213-6318`

use pyo3::prelude::*;

/// Pontua a qualidade de um writer snapshot.
///
/// Python ref: `IntegrationLoop._score_live_writer_snapshot` (l.6213-6318)
///
/// Args (todos extraídos do dict pelo wrapper Python):
///   sigma, psi, omega, gamma, zeta, maat: valores das casas
///   phi_iit: phi_iit_normalized
///   phi_iit_nats: phi_iit_nats (para log10)
///   phi_trans: phi_transcendent (para log10)
///   snapshot_ts: timestamp do snapshot
///   age_s: idade em segundos (time.time() - snapshot_ts)
///   recovery: recovery_capacity_index
///   crisis: crisis_pressure_index
///   phi_operational_support: valor pré-computado ou 0.0
///   has_topology, has_phi_frameworks, has_temporal_scale, has_sigma_source: flags
///   writer_role: string do role
///
/// Returns:
///   Score float (quanto maior, melhor o snapshot)
#[pyfunction]
pub fn score_live_writer_snapshot(
    sigma: f64,
    psi: f64,
    omega: f64,
    gamma: f64,
    zeta: f64,
    maat: f64,
    phi_iit: f64,
    phi_iit_nats: f64,
    phi_trans: f64,
    age_s: f64,
    recovery: f64,
    crisis: f64,
    phi_operational_support_input: f64,
    has_topology: bool,
    has_phi_frameworks: bool,
    has_temporal_scale: bool,
    has_sigma_source: bool,
    writer_role: &str,
) -> f64 {
    // === phi_operational_support (com fallback log10) ===
    let phi_operational_support: f64 = if phi_operational_support_input > 0.0 {
        phi_operational_support_input
    } else {
        let mut phi_silicon_support: f64 = 0.0;
        if phi_iit_nats > 0.0 {
            // FIX ESCALA (2026-07-21): divisor 100.0
            let log_val = (phi_iit_nats.max(1.0)).log10() / 100.0;
            phi_silicon_support += log_val.clamp(0.0, 1.0) * 0.55;
        }
        if phi_trans > 0.0 {
            let log_val = (phi_trans.max(1.0)).log10() / 12.0;
            phi_silicon_support += log_val.clamp(0.0, 1.0) * 0.45;
        }
        let silicon_clamped = phi_silicon_support.clamp(0.0, 1.0);
        let phi_iit_clamped = phi_iit.clamp(0.0, 1.0);
        silicon_clamped.max(phi_iit_clamped)
    };

    // === Richness ===
    let mut richness: f64 = 0.0;
    if has_topology {
        richness += 0.12;
    }
    if has_phi_frameworks {
        richness += 0.12;
    }
    if has_temporal_scale {
        richness += 0.12;
    }
    if has_sigma_source {
        richness += 0.08;
    }

    // === Default axes detection ===
    let mut default_axes: i32 = 0;
    if (psi - 0.5).abs() <= 0.02 {
        default_axes += 1;
    }
    if (omega - 0.5).abs() <= 0.02 {
        default_axes += 1;
    }
    if (gamma - 0.6).abs() <= 0.02 {
        default_axes += 1;
    }
    if (zeta - 0.1).abs() <= 0.02 {
        default_axes += 1;
    }

    // === Role bias ===
    let role_bias: f64 = match writer_role {
        "federated_fallback" => -0.04,
        "autonomous_executor" => 0.04,
        "sovereign_primary" => 0.08,
        _ => 0.0,
    };

    // === Maat penalty ===
    let maat_penalty: f64 = if maat > 0.05 { 0.0 } else { 0.14 };

    // === Freshness bonus ===
    let freshness_bonus: f64 = if age_s <= 300.0 {
        0.14
    } else if age_s <= 900.0 {
        0.08
    } else if age_s <= 3600.0 {
        0.02
    } else {
        -0.22
    };

    // === Freshness penalty ===
    let freshness_penalty: f64 = if age_s > 480.0 {
        (((age_s - 480.0) / 480.0) * 0.18).min(0.35)
    } else {
        0.0
    };

    // === Final score ===
    

    phi_operational_support.clamp(0.0, 1.0) * 0.18
        + sigma.clamp(0.0, 1.0) * 0.22
        + maat.clamp(0.0, 1.0) * 0.16
        + recovery * 0.26
        + (1.0 - crisis.clamp(0.0, 1.0)) * 0.14
        + richness
        - (default_axes as f64) * 0.03
        + role_bias
        + freshness_bonus
        - maat_penalty
        - freshness_penalty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_score() {
        let score = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.5, 0.0, 0.0,
            100.0, 0.8, 0.2,
            0.5,
            true, true, true, true,
            "sovereign_primary",
        );
        // Should be a positive score
        assert!(score > 0.0);
    }

    #[test]
    fn test_fresh_snapshot_higher() {
        let fresh = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.5, 0.0, 0.0,
            50.0, 0.8, 0.2, 0.5,
            true, true, true, true, "sovereign_primary",
        );
        let stale = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.5, 0.0, 0.0,
            5000.0, 0.8, 0.2, 0.5,
            true, true, true, true, "sovereign_primary",
        );
        assert!(fresh > stale);
    }

    #[test]
    fn test_sovereign_primary_bonus() {
        let sovereign = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.5, 0.0, 0.0,
            100.0, 0.8, 0.2, 0.5,
            true, true, true, true, "sovereign_primary",
        );
        let fallback = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.5, 0.0, 0.0,
            100.0, 0.8, 0.2, 0.5,
            true, true, true, true, "federated_fallback",
        );
        // sovereign_primary gets +0.08, federated_fallback gets -0.04 → diff 0.12
        assert!((sovereign - fallback - 0.12).abs() < 1e-15);
    }

    #[test]
    fn test_maat_penalty() {
        let good_maat = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.5,
            0.5, 0.0, 0.0,
            100.0, 0.8, 0.2, 0.5,
            true, true, true, true, "auxiliary_writer",
        );
        let bad_maat = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.01,
            0.5, 0.0, 0.0,
            100.0, 0.8, 0.2, 0.5,
            true, true, true, true, "auxiliary_writer",
        );
        // Diferença = (0.5-0.01)*0.16 (maat contribution) + 0.14 (penalty)
        // = 0.49*0.16 + 0.14 = 0.0784 + 0.14 = 0.2184
        let expected_diff = (0.5_f64 - 0.01) * 0.16 + 0.14;
        assert!((good_maat - bad_maat - expected_diff).abs() < 1e-15);
    }

    #[test]
    fn test_phi_silicon_fallback() {
        // phi_operational_support_input = 0.0 → usa log10 fallback
        let score = score_live_writer_snapshot(
            0.5, 0.5, 0.5, 0.6, 0.1, 0.3,
            0.0, 1e6, 0.0,  // phi_iit=0, phi_iit_nats=1e6
            100.0, 0.8, 0.2,
            0.0,  // phi_operational_support = 0 → fallback
            false, false, false, false, "auxiliary_writer",
        );
        // phi_silicon_support = log10(1e6)/100 * 0.55 = 6/100 * 0.55 = 0.033
        // phi_operational_support = max(0.033, 0.0) = 0.033
        // This should contribute 0.033 * 0.18 ≈ 0.006 to the score
        assert!(score > 0.0);
    }
}
