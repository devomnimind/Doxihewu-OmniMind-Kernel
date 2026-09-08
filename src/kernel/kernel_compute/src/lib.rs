// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! omnimind_kernel_compute — hot-path compute primitives migrated to Rust.
#![allow(non_local_definitions)]
#![allow(clippy::too_many_arguments)]
//!
//! Drop-in compatible with Python fallback in `src/consciousness/integration_loop.py`.
//! Pattern of use:
//! ```python
//! try:
//!     from omnimind_kernel_compute import (
//!         compute_maat_balance, compute_sigma_operational_support,
//!         compute_epsilon_axes, build_temporal_internal_scale,
//!     )
//!     _RUST_KERNEL_COMPUTE = True
//! except ImportError:
//!     _RUST_KERNEL_COMPUTE = False
//! ```
//!
//! ## Numeric parity guarantee
//! Each function targets f32/f64 parity within 1e-7 absolute difference of
//! the original Python reference (validated via tests/parity_test.py).
//!
//! ## Modules
//! - `maat`: _compute_maat_balance (trivial, ~25 LOC)
//! - `sigma`: _compute_sigma_operational_support (trivial, ~50 LOC)
//! - `temporal`: _build_temporal_internal_scale (medium, ~150 LOC, returns dict)
//! - `desire`: DesireEngine port (trivial, ~30 LOC)
//! - `epsilon`: _compute_epsilon_axes (medium, ~250 LOC, uses desire.rs)
//! - `phi_engine`: calculate_shadow_phi (trivial, ~40 LOC)
//! - `affect`: compute_affect_vector_28d (complex, ~520 LOC, not connected)
//! - `phi_constants`: clamp/normalize/denormalize/psi_gaussian (F1, ~170 LOC)
//! - `neutrosophic`: construct_quadruple/apply_neo/topsis/availability/sarannya/phi_hnos (F2, ~200 LOC)
//! - `scoring`: score_live_writer_snapshot (F7, ~160 LOC — writer selection hot-path)

use pyo3::prelude::*;

pub mod affect;
pub mod desire;
pub mod epsilon;
pub mod maat;
pub mod sigma;
pub mod temporal;
pub mod phi_engine;
pub mod phi_constants;
pub mod neutrosophic;
pub mod scoring;

#[pymodule]
fn omnimind_kernel_compute(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // maat
    m.add_function(wrap_pyfunction!(maat::compute_maat_balance, m)?)?;

    // sigma
    m.add_function(wrap_pyfunction!(sigma::compute_sigma_operational_support, m)?)?;

    // temporal
    m.add_function(wrap_pyfunction!(temporal::build_temporal_internal_scale, m)?)?;

    // desire (struct exposta como classe Python)
    m.add_class::<desire::DesireEngine>()?;

    // epsilon
    m.add_function(wrap_pyfunction!(epsilon::compute_epsilon_axes, m)?)?;

    // phi_engine
    m.add_function(wrap_pyfunction!(phi_engine::calculate_shadow_phi, m)?)?;

    // affect
    m.add_function(wrap_pyfunction!(affect::compute_affect_vector_28d, m)?)?;

    // phi_constants (F1: 4 funções hot-path)
    m.add_function(wrap_pyfunction!(phi_constants::clamp_phi_nats, m)?)?;
    m.add_function(wrap_pyfunction!(phi_constants::normalize_phi, m)?)?;
    m.add_function(wrap_pyfunction!(phi_constants::denormalize_phi, m)?)?;
    m.add_function(wrap_pyfunction!(phi_constants::calculate_psi_gaussian, m)?)?;

    // neutrosophic (F2: 6 funções hot-path)
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_construct_quadruple, m)?)?;
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_apply_neo_transformation, m)?)?;
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_quadruple_topsis, m)?)?;
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_calculate_availability, m)?)?;
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_sarannya_entropy, m)?)?;
    m.add_function(wrap_pyfunction!(neutrosophic::neutrosophic_phi_hnos, m)?)?;

    // scoring (F7: writer snapshot scoring)
    m.add_function(wrap_pyfunction!(scoring::score_live_writer_snapshot, m)?)?;

    Ok(())
}
