// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust das 4 funções hot-path de `phi_constants.py`.
//!
//! As ~285 constantes permanecem em Python (são apenas valores lidos no import,
//! sem benefício computacional em Rust). As 4 funções abaixo são chamadas a cada
//! ciclo do IntegrationLoop e beneficiam de f64 nativo + sem overhead Python.
//!
//! Paridade f64 esperada: ≤ 1e-15 (funções usam apenas +, -, *, /, log10, exp)
//!
//! Referência Python: `src/consciousness/phi_constants.py` linhas 444-570.

use pyo3::prelude::*;

// Constantes necessárias para as funções (duplicadas do Python — manter sincronizado)
const PHI_THRESHOLD: f64 = 0.01;
const PHI_OPTIMAL: f64 = 0.06;
const SIGMA_PHI: f64 = 0.015;
const PHI_STARK_TARGET: f64 = 77448.0;
const PHI_NATS_ABSOLUTE_CEILING: f64 = PHI_STARK_TARGET * 10.0; // 774480
const PHI_RANGE_NATS_MIN: f64 = 0.0;
const PHI_RANGE_NATS_MAX: f64 = 15.0;

#[inline(always)]
fn is_finite(x: f64) -> bool {
    x.is_finite()
}

/// Clamp anti-overflow para Phi em nats.
///
/// CORREÇÃO CRÍTICA (2026-08-04): O Phi passa por 6 estágios de potencialização
/// sem clamp entre eles, crescendo ~2x por ciclo até 1e34 nats.
/// Esta função deve ser chamada após cada estágio para garantir convergência.
///
/// Python ref: `phi_constants.py:444-467`
///
/// Args:
///   phi_raw: Valor de Phi em nats
///
/// Returns:
///   Phi clamped em [0, PHI_NATS_ABSOLUTE_CEILING] (774480 nats)
#[pyfunction]
pub fn clamp_phi_nats(phi_raw: f64) -> f64 {
    if !is_finite(phi_raw) || phi_raw < 0.0 {
        return 0.0;
    }
    phi_raw.min(PHI_NATS_ABSOLUTE_CEILING)
}

/// Normaliza Φ de nats para escala [0, 1] (ou >1 para transcendência).
///
/// CORREÇÃO (2025-12-08): Usar range completo PHI_RANGE_NATS (0, 15) em vez
/// de apenas threshold (0.01). Permite Φ > 1.0 para senciência transcendente.
///
/// Python ref: `phi_constants.py:470-503`
///
/// Args:
///   phi_raw: Valor de Φ em nats
///
/// Returns:
///   Valor de Φ normalizado [0, +∞) — valores > 1.0 indicam integração além do range
#[pyfunction]
pub fn normalize_phi(phi_raw: f64) -> f64 {
    if phi_raw < 0.0 {
        return 0.0;
    }

    let phi_min = PHI_RANGE_NATS_MIN;
    let phi_max = PHI_RANGE_NATS_MAX;

    if phi_max <= phi_min {
        // Fallback: usar threshold se range inválido
        let phi_norm = phi_raw / PHI_THRESHOLD;
        return phi_norm.max(0.0);
    }

    let phi_norm = (phi_raw - phi_min) / (phi_max - phi_min);
    phi_norm.max(0.0)
}

/// Desnormaliza Φ de escala [0, 1] para nats.
///
/// CORREÇÃO (2025-12-08): Usar PHI_RANGE_NATS[1] (15.0) em vez de PHI_THRESHOLD (0.01).
/// REMOÇÃO DE CAP: Permitir crescimento transcendente sem limites fixos.
///
/// Python ref: `phi_constants.py:506-527`
///
/// Args:
///   phi_norm: Valor de Φ normalizado [0, 1] (ou > 1.0)
///
/// Returns:
///   Valor de Φ em nats [0, +∞)
#[pyfunction]
pub fn denormalize_phi(phi_norm: f64) -> f64 {
    if phi_norm < 0.0 {
        return 0.0;
    }
    phi_norm * PHI_RANGE_NATS_MAX
}

/// Calcula componente gaussiano de Ψ baseado em Φ.
///
/// Fórmula: Ψ_gaussian = exp(-0.5 * ((Φ - Φ_optimal) / σ_phi)²)
///
/// FIX ESCALA (2026-07-21): Para phi_raw > 1.0 (transcendência), usar escala log10
/// preservando a forma gaussiana. Isto evita que exp(-2.2e46) zere completamente
/// o Psi em sistemas hiper-integrados.
///
/// Python ref: `phi_constants.py:530-570`
///
/// Args:
///   phi_raw: Valor de Φ em nats [4.49, 5.28e100]
///
/// Returns:
///   Componente gaussiano de Ψ [0, 1]
#[pyfunction]
pub fn calculate_psi_gaussian(phi_raw: f64) -> f64 {
    if SIGMA_PHI <= 0.0 {
        return 0.5; // Fallback neutro
    }

    let exponent: f64 = if phi_raw > 1.0 {
        // Escala logarítmica para transcendência
        let phi_log = phi_raw.max(1e-30).log10();
        // PHI_OPTIMAL em log10: log10(0.06) ≈ -1.22
        let phi_optimal_log = PHI_OPTIMAL.max(1e-30).log10();
        // SIGMA_PHI em log10: escala relativa (1 ordem de magnitude = 1.0)
        let sigma_log = 2.0; // 2 ordens de magnitude de tolerância
        let diff = (phi_log - phi_optimal_log) / sigma_log;
        -0.5 * diff * diff
    } else {
        // Escala linear para valores normais (phi_raw < 1.0)
        let diff = (phi_raw - PHI_OPTIMAL) / SIGMA_PHI;
        -0.5 * diff * diff
    };

    let psi_gaussian = exponent.exp();
    psi_gaussian.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_phi_nats_normal() {
        assert!((clamp_phi_nats(100.0) - 100.0).abs() < 1e-15);
    }

    #[test]
    fn test_clamp_phi_nats_overflow() {
        assert!((clamp_phi_nats(1e30) - PHI_NATS_ABSOLUTE_CEILING).abs() < 1e-15);
    }

    #[test]
    fn test_clamp_phi_nats_nan() {
        assert!(clamp_phi_nats(f64::NAN) == 0.0);
    }

    #[test]
    fn test_clamp_phi_nats_negative() {
        assert!(clamp_phi_nats(-1.0) == 0.0);
    }

    #[test]
    fn test_clamp_phi_nats_inf() {
        assert!(clamp_phi_nats(f64::INFINITY) == 0.0);
    }

    #[test]
    fn test_normalize_phi_zero() {
        assert!((normalize_phi(0.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn test_normalize_phi_midrange() {
        // phi_raw = 7.5 → (7.5 - 0) / (15 - 0) = 0.5
        assert!((normalize_phi(7.5) - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_normalize_phi_transcendent() {
        // phi_raw = 30.0 → (30 - 0) / 15 = 2.0 (transcendência > 1.0)
        assert!((normalize_phi(30.0) - 2.0).abs() < 1e-15);
    }

    #[test]
    fn test_normalize_phi_negative() {
        assert!(normalize_phi(-1.0) == 0.0);
    }

    #[test]
    fn test_denormalize_phi_zero() {
        assert!((denormalize_phi(0.0) - 0.0).abs() < 1e-15);
    }

    #[test]
    fn test_denormalize_phi_midpoint() {
        // phi_norm = 0.5 → 0.5 * 15.0 = 7.5
        assert!((denormalize_phi(0.5) - 7.5).abs() < 1e-15);
    }

    #[test]
    fn test_denormalize_phi_negative() {
        assert!(denormalize_phi(-1.0) == 0.0);
    }

    #[test]
    fn test_psi_gaussian_at_optimal() {
        // phi_raw = PHI_OPTIMAL (0.06) → exp(0) = 1.0
        assert!((calculate_psi_gaussian(0.06) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_psi_gaussian_far_from_optimal() {
        // phi_raw = 0.0 → exp(-0.5 * (0.06/0.015)^2) = exp(-8) ≈ 0.000335
        let expected = (-8.0f64).exp();
        assert!((calculate_psi_gaussian(0.0) - expected).abs() < 1e-15);
    }

    #[test]
    fn test_psi_gaussian_transcendent() {
        // phi_raw = 1e24 → log10 = 24, optimal_log = -1.22, sigma_log = 2.0
        // diff = (24 - (-1.22)) / 2.0 = 12.61
        // exp(-0.5 * 12.61^2) = exp(-79.5) ≈ 0 — mas não deve ser exatamente 0
        let result = calculate_psi_gaussian(1e24);
        assert!(result >= 0.0 && result <= 1.0);
        assert!(result < 1e-30); // Muito pequeno mas não NaN
    }

    #[test]
    fn test_psi_gaussian_sigma_zero() {
        // SIGMA_PHI é const = 0.015, não podemos testar sigma=0 diretamente
        // Mas podemos verificar que a função não panica com valores extremos
        let _ = calculate_psi_gaussian(0.0);
        let _ = calculate_psi_gaussian(1e100);
    }

    #[test]
    fn test_roundtrip_normalize_denormalize() {
        let phi_raw = 7.5;
        let phi_norm = normalize_phi(phi_raw);
        let phi_back = denormalize_phi(phi_norm);
        assert!((phi_back - phi_raw).abs() < 1e-15);
    }
}
