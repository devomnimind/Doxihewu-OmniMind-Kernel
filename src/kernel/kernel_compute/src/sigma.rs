// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust de `_compute_sigma_operational_support` em `integration_loop.py:4323`.
//!
//! ## Ref Python (44 linhas)
//! ```python
//! def _compute_sigma_operational_support(self, *, phi_raw_nats, phi_iit_normalized,
//!                                          phi_trans_support, sigma_topology_proxy,
//!                                          topology_omega, betti_0) -> float:
//!     continuity_depth = float(np.clip(self._continuity_depth_proxy(), 0.0, 1.0))
//!     phi_silicon_support = float(np.clip(math.log10(max(phi_raw_nats, 1.0)) / 6.0, 0.0, 1.0))
//!     multiplicity_inverse = float(np.clip(1.0 / max(1.0, betti_0), 0.0, 1.0))
//!     last_sigma = float(np.clip(getattr(self.last_triad, "sigma", 0.0) or 0.0, 0.0, 1.0))
//!     support = float(np.clip(
//!         (phi_iit_normalized * 0.08) + (phi_trans_support * 0.08)
//!         + (phi_silicon_support * 0.12) + (continuity_depth * 0.18)
//!         + (np.clip(sigma_topology_proxy, 0.0, 1.0) * 0.24)
//!         + (np.clip(topology_omega, 0.0, 1.0) * 0.10)
//!         + (multiplicity_inverse * 0.10) + (last_sigma * 0.10),
//!         0.0, 1.0,
//!     ))
//!     if sigma_topology_proxy <= 0.10 and topology_omega <= 0.10 and multiplicity_inverse < 0.34:
//!         support = min(support, 0.28)
//!     return support
//! ```

use pyo3::prelude::*;

#[inline(always)]
fn clip01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Estima piso operacional de σ.
///
/// `continuity_depth` e `last_sigma` precisam ser passados explicitamente
/// (são state interno do daemon).
#[pyfunction]
#[pyo3(signature = (
    phi_raw_nats,
    phi_iit_normalized,
    phi_trans_support,
    sigma_topology_proxy,
    topology_omega,
    betti_0,
    continuity_depth,
    last_sigma,
))]
pub fn compute_sigma_operational_support(
    phi_raw_nats: f64,
    phi_iit_normalized: f64,
    phi_trans_support: f64,
    sigma_topology_proxy: f64,
    topology_omega: f64,
    betti_0: f64,
    continuity_depth: f64,
    last_sigma: f64,
) -> f64 {
    let continuity_depth_clamped = clip01(continuity_depth);
    // FIX ESCALA (2026-08-28): divisor era 6.0, deve ser 100.0 para match Python.
    // Python (integration_loop.py): np.clip(math.log10(max(phi_raw_nats, 1.0)) / 100.0, 0.0, 1.0)
    // Com 6.0: phi_raw_nats=1e6 → 1.0 (clipado); com 100.0: → 0.06 (correto)
    let phi_silicon_support = clip01(phi_raw_nats.max(1.0).log10() / 100.0);
    let multiplicity_inverse = clip01(1.0 / betti_0.max(1.0));
    let last_sigma_clamped = clip01(last_sigma);
    let sigma_topology_clamped = clip01(sigma_topology_proxy);
    let topology_omega_clamped = clip01(topology_omega);

    let support = clip01(
        (phi_iit_normalized * 0.08)
            + (phi_trans_support * 0.08)
            + (phi_silicon_support * 0.12)
            + (continuity_depth_clamped * 0.18)
            + (sigma_topology_clamped * 0.24)
            + (topology_omega_clamped * 0.10)
            + (multiplicity_inverse * 0.10)
            + (last_sigma_clamped * 0.10),
    );

    // Penalização: topology zero-zone limita support a 0.28
    if sigma_topology_proxy <= 0.10
        && topology_omega <= 0.10
        && multiplicity_inverse < 0.34
    {
        support.min(0.28)
    } else {
        support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeros() {
        let result = compute_sigma_operational_support(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        // multiplicity_inverse = 1.0 (passa do gate de 0.34, não penaliza)
        // support = 0.10 (multiplicity_inverse * 0.10)
        assert!((result - 0.10).abs() < 1e-9);
    }

    #[test]
    fn test_topology_zero_penalty() {
        // sigma_topology=0.0, topology_omega=0.0, betti_0=10 → multiplicity_inverse=0.1 < 0.34 → penalty
        let result = compute_sigma_operational_support(1e6, 1.0, 1.0, 0.0, 0.0, 10.0, 1.0, 1.0);
        assert!(result <= 0.28 + 1e-9);
    }
}
