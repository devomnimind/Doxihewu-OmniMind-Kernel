// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust de `_compute_maat_balance` em `src/consciousness/integration_loop.py:4064`.
//!
//! ## Ref Python
//! ```python
//! def _compute_maat_balance(self, *, phi_anchor, psi, epsilon) -> float:
//!     phi_ref = max(0.0, float(phi_anchor))
//!     psi_ref = float(np.clip(psi, 0.0, 1.0))
//!     epsilon_ref = float(np.clip(epsilon, 0.0, 1.0))
//!     dyad_center = float(np.clip((psi_ref + epsilon_ref) / 2.0, 0.0, 1.0))
//!     maat_distance = abs(phi_ref - dyad_center)
//!     maat_scale = max(1.0, phi_ref)
//!     maat_balance = 1.0 - min(1.0, maat_distance / maat_scale)
//!     return float(np.clip(maat_balance, 0.0, 1.0))
//! ```
//!
//! ## Semântica preservada
//! Mantém Ma'at na escala operacional sem zerar quando `psi` e `epsilon` sobem
//! juntos no mesmo ciclo. Centro operacional do par psi/epsilon contra phi anchor.

use pyo3::prelude::*;

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Calcula Ma'at balance a partir de phi_anchor, psi, epsilon.
///
/// Paridade numérica: f64 nativo Rust deve dar diff ≤ 1e-15 vs Python (que usa
/// numpy float64). Float32 fica garantido a ≤ 1e-7.
#[pyfunction]
pub fn compute_maat_balance(phi_anchor: f64, psi: f64, epsilon: f64) -> f64 {
    let phi_ref = phi_anchor.max(0.0);
    let psi_ref = clip01(psi);
    let epsilon_ref = clip01(epsilon);
    let dyad_center = clip01((psi_ref + epsilon_ref) / 2.0);
    let maat_distance = (phi_ref - dyad_center).abs();
    let maat_scale = phi_ref.max(1.0);
    let maat_balance = 1.0 - (maat_distance / maat_scale).min(1.0);
    clip01(maat_balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros() {
        assert_eq!(compute_maat_balance(0.0, 0.0, 0.0), 1.0);
    }

    #[test]
    fn test_high_phi_low_dyad() {
        // phi_anchor=1.0, psi=0.0, epsilon=0.0 → dyad_center=0.0, distance=1.0, scale=1.0 → balance=0.0
        let result = compute_maat_balance(1.0, 0.0, 0.0);
        assert!((result - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_aligned() {
        // phi_anchor=0.5, psi=0.5, epsilon=0.5 → dyad_center=0.5, distance=0.0 → balance=1.0
        let result = compute_maat_balance(0.5, 0.5, 0.5);
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_clamping() {
        // psi e epsilon fora de [0..1] devem ser clamped antes de computar
        let result = compute_maat_balance(0.5, 1.5, -0.3);
        // psi_ref=1.0, epsilon_ref=0.0, dyad_center=0.5, phi_ref=0.5
        // distance=0.0, scale=1.0, balance=1.0
        assert!((result - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_high_phi() {
        // phi_anchor=2.0, psi=0.5, epsilon=0.5 → dyad_center=0.5, distance=1.5, scale=2.0
        // balance = 1.0 - 1.5/2.0 = 0.25
        let result = compute_maat_balance(2.0, 0.5, 0.5);
        assert!((result - 0.25).abs() < 1e-9);
    }
}
