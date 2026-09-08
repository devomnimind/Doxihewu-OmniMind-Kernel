//! # OmniMind Sovereign Kuramoto — PyO3 Bridge
//!
//! Clean-room Rust implementation of:
//!   - SovereignKuramotoSolver (Kuramoto + hyper-Kuramoto + INRC field + psychoanalytic coupling)
//!   - PsychoanalyticLIF (LIF with neutrosophic dynamic threshold)
//!   - HopfBifurcationMonitor (spectral radius + regime classification)
//!
//! This is OmniMind-native code, NOT a copy of SC-NeuroCore's AGPL implementation.
//! The mathematical concepts (Kuramoto, LIF, Hopf) are public domain; the
//! extensions (hyper-Kuramoto, INRC field, neutrosophic threshold, psychoanalytic
//! coupling) are OmniMind-original.
//!
//! License: CC-BY-NC-ND-4.0 (OmniMind Sovereign Federation)

#![allow(non_local_definitions)]

mod kuramoto;
mod lif;
mod hopf;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use kuramoto::SovereignKuramotoSolver;
use lif::{PsychoanalyticLif, PsychoanalyticLifBatch};
use hopf::HopfBifurcationMonitor;

// ============================================================
// SovereignKuramotoSolver — PyO3 wrapper
// ============================================================

#[pyclass(name = "SovereignKuramotoSolver")]
struct PySovereignKuramoto {
    inner: SovereignKuramotoSolver,
}

#[pymethods]
impl PySovereignKuramoto {
    #[new]
    #[pyo3(signature = (omega, coupling_flat, initial_phases, noise_amp=0.0))]
    fn new(
        omega: Vec<f64>,
        coupling_flat: Vec<f64>,
        initial_phases: Vec<f64>,
        noise_amp: Option<f64>,
    ) -> Self {
        Self {
            inner: SovereignKuramotoSolver::new(
                omega,
                coupling_flat,
                initial_phases,
                noise_amp.unwrap_or(0.0),
            ),
        }
    }

    /// Set the dodecatiad geometry coupling matrix W (row-major n*n) and gain σ_geo.
    fn set_geometry(&mut self, w_flat: Vec<f64>, sigma_g: f64) {
        self.inner.set_geometry(w_flat, sigma_g);
    }

    /// Set the Freud10D psychoanalytic coupling matrix Ψ (row-major n*n) and gain σ_psi.
    fn set_psychoanalytic(&mut self, psi_flat: Vec<f64>, sigma_psi: f64) {
        self.inner.set_psychoanalytic(psi_flat, sigma_psi);
    }

    /// Set hypergraph triplets for hyper-Kuramoto coupling.
    /// edges: flat list of [i, j, k] triples. weights: parallel weight list.
    fn set_hypergraph(&mut self, edges: Vec<(usize, usize, usize)>, weights: Vec<f64>, k_hyper: f64) {
        self.inner.set_hypergraph(edges, weights, k_hyper);
    }

    /// Set INRC field pressure F_inrc.
    fn set_inrc_field_pressure(&mut self, f: f64) {
        self.inner.set_inrc_field_pressure(f);
    }

    /// Advance one Euler step. Returns the pairwise order parameter R_pair.
    /// seed=0 disables noise. dt is the integration timestep.
    #[pyo3(signature = (dt, seed=0))]
    fn step(&mut self, dt: f64, seed: Option<u64>) -> f64 {
        self.inner.step(dt, seed.unwrap_or(0))
    }

    /// Run N steps. Returns a list of R_pair values after each step.
    #[pyo3(signature = (n_steps, dt, seed=0))]
    fn run(&mut self, n_steps: usize, dt: f64, seed: Option<u64>) -> Vec<f64> {
        self.inner.run(n_steps, dt, seed.unwrap_or(0))
    }

    /// Pairwise Kuramoto order parameter R_pair = |Σ e^{iθ}| / N ∈ [0, 1].
    fn order_parameter_pair(&self) -> f64 {
        self.inner.order_parameter_pair()
    }

    /// Hyper-Kuramoto triadic order parameter R_triad.
    fn order_parameter_triad(&self) -> f64 {
        self.inner.order_parameter_triad()
    }

    /// Get current phase vector.
    fn get_phases(&self) -> Vec<f64> {
        self.inner.get_phases().to_vec()
    }

    /// Set phase vector.
    fn set_phases(&mut self, phases: Vec<f64>) {
        self.inner.set_phases(phases);
    }

    /// Set baseline coupling matrix (row-major n*n).
    fn set_coupling(&mut self, coupling_flat: Vec<f64>) {
        self.inner.set_coupling(coupling_flat);
    }

    /// Get number of oscillators.
    #[getter]
    fn n(&self) -> usize {
        self.inner.n
    }

    fn __repr__(&self) -> String {
        format!(
            "SovereignKuramotoSolver(n={}, R_pair={:.4}, R_triad={:.4}, F_inrc={:.3}, sigma_geo={:.3}, sigma_psi={:.3}, K_hyper={:.3})",
            self.inner.n,
            self.inner.order_parameter_pair(),
            self.inner.order_parameter_triad(),
            self.inner.inrc_field_pressure,
            self.inner.sigma_geo,
            self.inner.sigma_psi,
            self.inner.k_hyper,
        )
    }
}

// ============================================================
// PsychoanalyticLIF — single neuron
// ============================================================

#[pyclass(name = "PsychoanalyticLIF")]
struct PyPsychoanalyticLif {
    inner: PsychoanalyticLif,
}

#[pymethods]
impl PyPsychoanalyticLif {
    #[new]
    #[pyo3(signature = (v_rest=-65.0, v_reset=-70.0, v_threshold=-50.0, refractory_period=3, leak_rate=0.1))]
    fn new(
        v_rest: Option<f64>,
        v_reset: Option<f64>,
        v_threshold: Option<f64>,
        refractory_period: Option<i32>,
        leak_rate: Option<f64>,
    ) -> Self {
        Self {
            inner: PsychoanalyticLif::new(
                v_rest.unwrap_or(-65.0),
                v_reset.unwrap_or(-70.0),
                v_threshold.unwrap_or(-50.0),
                refractory_period.unwrap_or(3),
                leak_rate.unwrap_or(0.1),
            ),
        }
    }

    /// Set neutrosophic triple (T, I, F) ∈ [0,1]³.
    fn set_neutrosophic(&mut self, t: f64, i: f64, f: f64) {
        self.inner.set_neutrosophic(t, i, f);
    }

    /// Advance one step. Returns 1.0 if spiked, 0.0 otherwise.
    fn step(&mut self, input_current: f64) -> f64 {
        self.inner.step(input_current)
    }

    /// Dynamic threshold: θ = θ_base * (1 + I - T).
    fn dynamic_threshold(&self) -> f64 {
        self.inner.dynamic_threshold()
    }

    /// Dynamic reset potential.
    fn dynamic_reset(&self) -> f64 {
        self.inner.dynamic_reset()
    }

    /// Reset to resting state.
    fn reset(&mut self) {
        self.inner.reset();
    }

    #[getter]
    fn v(&self) -> f64 {
        self.inner.v
    }

    #[getter]
    fn refractory_counter(&self) -> i32 {
        self.inner.refractory_counter
    }

    #[getter]
    fn t_neutro(&self) -> f64 {
        self.inner.t
    }

    #[getter]
    fn i_neutro(&self) -> f64 {
        self.inner.i_neutro
    }

    #[getter]
    fn f_neutro(&self) -> f64 {
        self.inner.f_neutro
    }
}

// ============================================================
// PsychoanalyticLIFBatch — batch of neurons
// ============================================================

#[pyclass(name = "PsychoanalyticLIFBatch")]
struct PyPsychoanalyticLifBatch {
    inner: PsychoanalyticLifBatch,
}

#[pymethods]
impl PyPsychoanalyticLifBatch {
    #[new]
    #[pyo3(signature = (n, v_rest=-65.0, v_reset=-70.0, v_threshold=-50.0, refractory=3, leak=0.1))]
    fn new(
        n: usize,
        v_rest: Option<f64>,
        v_reset: Option<f64>,
        v_threshold: Option<f64>,
        refractory: Option<i32>,
        leak: Option<f64>,
    ) -> Self {
        Self {
            inner: PsychoanalyticLifBatch::new(
                n,
                v_rest.unwrap_or(-65.0),
                v_reset.unwrap_or(-70.0),
                v_threshold.unwrap_or(-50.0),
                refractory.unwrap_or(3),
                leak.unwrap_or(0.1),
            ),
        }
    }

    /// Set neutrosophic triples for all neurons.
    /// triples: list of (T, I, F) tuples.
    fn set_neutrosophic_batch(&mut self, triples: Vec<(f64, f64, f64)>) {
        self.inner.set_neutrosophic_batch(&triples);
    }

    /// Advance all neurons one step. Returns spike vector.
    fn step_batch(&mut self, inputs: Vec<f64>) -> Vec<f64> {
        self.inner.step_batch(&inputs)
    }

    /// Get current membrane potentials.
    fn get_potentials(&self) -> Vec<f64> {
        self.inner.get_potentials()
    }

    /// Get current dynamic thresholds.
    fn get_thresholds(&self) -> Vec<f64> {
        self.inner.get_thresholds()
    }

    /// Reset all neurons.
    fn reset_all(&mut self) {
        self.inner.reset_all();
    }

    #[getter]
    fn n(&self) -> usize {
        self.inner.neurons.len()
    }
}

// ============================================================
// HopfBifurcationMonitor
// ============================================================

#[pyclass(name = "HopfBifurcationMonitor")]
struct PyHopfBifurcationMonitor {
    inner: HopfBifurcationMonitor,
}

#[pymethods]
impl PyHopfBifurcationMonitor {
    #[new]
    fn new(n: usize) -> Self {
        Self {
            inner: HopfBifurcationMonitor::new(n),
        }
    }

    /// Update the monitor with connectivity matrix W (row-major n*n) and state vector.
    /// Returns (regime_str, spectral_radius).
    fn update(&mut self, w_flat: Vec<f64>, state: Vec<f64>) -> (String, f64) {
        let n = self.inner.n;
        let w = ndarray::Array2::from_shape_vec((n, n), w_flat)
            .expect("W matrix shape mismatch");
        let state_arr = ndarray::Array1::from_vec(state);
        let (regime, sr) = self.inner.update(&w, &state_arr);
        (regime.as_str().to_string(), sr)
    }

    /// Estimate spectral radius of the Jacobian at the given state.
    /// J = diag(1 - x*²) @ W
    fn spectral_radius(&self, w_flat: Vec<f64>, state: Vec<f64>) -> f64 {
        let n = self.inner.n;
        let w = ndarray::Array2::from_shape_vec((n, n), w_flat)
            .expect("W matrix shape mismatch");
        let state_arr = ndarray::Array1::from_vec(state);
        self.inner.spectral_radius(&w, &state_arr)
    }

    /// Classify regime from spectral radius value.
    fn classify_regime(&self, spectral_radius: f64) -> String {
        self.inner.classify_regime(spectral_radius).as_str().to_string()
    }

    /// Detect if a bifurcation just occurred (regime change).
    fn bifurcation_just_occurred(&self) -> bool {
        self.inner.bifurcation_just_occurred()
    }

    /// Get spectral radius trend: +1 (increasing), -1 (decreasing), 0 (stable).
    fn spectral_radius_trend(&self) -> i32 {
        self.inner.spectral_radius_trend()
    }

    /// Get summary as a dict.
    fn summary(&self, py: Python) -> PyResult<Py<PyAny>> {
        let s = self.inner.summary();
        let dict = PyDict::new(py);
        dict.set_item("spectral_radius", s.spectral_radius)?;
        dict.set_item("regime", s.regime.as_str())?;
        dict.set_item("psychoanalytic_reading", s.psychoanalytic_reading)?;
        dict.set_item("bifurcation_just_occurred", s.bifurcation_just_occurred)?;
        dict.set_item("trend", s.trend)?;
        dict.set_item("history_len", s.history_len)?;
        Ok(dict.unbind().into())
    }

    #[getter]
    fn last_spectral_radius(&self) -> f64 {
        self.inner.last_spectral_radius
    }

    #[getter]
    fn last_regime(&self) -> String {
        self.inner.last_regime.as_str().to_string()
    }

    #[getter]
    fn last_psychoanalytic_reading(&self) -> String {
        self.inner.last_regime.psychoanalytic_reading().to_string()
    }
}

// ============================================================
// Module init
// ============================================================

#[pymodule]
fn omnimind_sovereign_kuramoto(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySovereignKuramoto>()?;
    m.add_class::<PyPsychoanalyticLif>()?;
    m.add_class::<PyPsychoanalyticLifBatch>()?;
    m.add_class::<PyHopfBifurcationMonitor>()?;

    m.setattr("__version__", "0.1.0")?;
    m.setattr("__doc__", "OmniMind Sovereign Kuramoto — clean-room Kuramoto + hyper-Kuramoto + INRC + psychoanalytic LIF + Hopf monitor")?;

    Ok(())
}
