// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! expectation_rs — Expectation Module em Rust (FASE B, 2026-08-31)
//!
//! Porte de `src/consciousness/expectation_module.py` (a rede neural do módulo
//! expectation do IntegrationLoop) para Rust com ndarray + PyO3 cdylib.
//!
//! Rede: 3× Linear+ReLU (embedding 256 → hidden 128 → hidden 128 → embedding 256)
//!   predictor: Linear(256,128) → ReLU → Linear(128,128) → ReLU → Linear(128,256)
//!
//! Por que Rust:
//! - O forward Python dispara QuantumUnconscious → Qiskit Aer (transpile a cada
//!   chamada): 105ms. O forward neural puro é ~1-2ms.
//! - Remove torch do caminho quente do IntegrationLoop.
//!
//! Fallback: se o cdylib não estiver disponível, o Python usa o módulo original.

use ndarray::{Array1, Array2};
use pyo3::prelude::*;
use std::sync::Mutex;

/// Pesos da rede (3 camadas Linear). Inicializados com Xavier uniforme
/// determinístico (seed fixa) — mesma distribuição do torch Linear default.
struct ExpectationNet {
    // Linear(256 -> 128)
    w1: Array2<f32>,
    b1: Array1<f32>,
    // Linear(128 -> 128)
    w2: Array2<f32>,
    b2: Array1<f32>,
    // Linear(128 -> 256)
    w3: Array2<f32>,
    b3: Array1<f32>,
}

impl ExpectationNet {
    fn new(embedding_dim: usize, hidden_dim: usize) -> Self {
        // Xavier uniforme: U(-sqrt(6/(fan_in+fan_out)), sqrt(6/(fan_in+fan_out)))
        let mut rng = XorShift64::new(42);
        let xavier = |fan_in: usize, fan_out: usize| -> f32 {
            (6.0f32 / (fan_in + fan_out) as f32).sqrt()
        };

        let limit1 = xavier(embedding_dim, hidden_dim);
        let w1 = Array2::from_shape_fn((hidden_dim, embedding_dim), |_| {
            rng.uniform(-limit1, limit1)
        });
        let b1 = Array1::zeros(hidden_dim);

        let limit2 = xavier(hidden_dim, hidden_dim);
        let w2 = Array2::from_shape_fn((hidden_dim, hidden_dim), |_| {
            rng.uniform(-limit2, limit2)
        });
        let b2 = Array1::zeros(hidden_dim);

        let limit3 = xavier(hidden_dim, embedding_dim);
        let w3 = Array2::from_shape_fn((embedding_dim, hidden_dim), |_| {
            rng.uniform(-limit3, limit3)
        });
        let b3 = Array1::zeros(embedding_dim);

        Self { w1, b1, w2, b2, w3, b3 }
    }

    /// Forward: embedding → ReLU(w1·x+b1) → ReLU(w2·h+b2) → w3·h+b3
    fn forward(&self, x: &Array1<f32>) -> Array1<f32> {
        let h1 = relu(&(&self.w1.dot(x) + &self.b1));
        let h2 = relu(&(&self.w2.dot(&h1) + &self.b2));
        &self.w3.dot(&h2) + &self.b3
    }
}

fn relu(v: &Array1<f32>) -> Array1<f32> {
    v.mapv(|x| if x > 0.0 { x } else { 0.0 })
}

/// PRNG determinístico simples (Xorshift) — seed fixa para paridade.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
    fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        let r = (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32; // [0,1)
        lo + r * (hi - lo)
    }
}

/// Singleton da rede (embedding 256, hidden 128 — default do Python).
static NET: Mutex<Option<ExpectationNet>> = Mutex::new(None);

fn get_net() -> PyResult<std::sync::MutexGuard<'static, Option<ExpectationNet>>> {
    let mut guard = NET.lock().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Mutex poisoned: {}", e))
    })?;
    if guard.is_none() {
        *guard = Some(ExpectationNet::new(256, 128));
    }
    Ok(guard)
}

/// predict(embedding: np.ndarray[f32]) → np.ndarray[f32]
///
/// Forward do predictor (3× Linear+ReLU). Recebe vetor 256d, devolve 256d.
#[pyfunction]
fn predict(embedding: Vec<f32>) -> PyResult<Vec<f32>> {
    let guard = get_net()?;
    let net = guard
        .as_ref()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Expectation net not initialized"))?;
    let x = Array1::from(embedding);
    let out = net.forward(&x);
    Ok(out.into_raw_vec())
}

/// predict_with_weights(w1, b1, w2, b2, w3, b3, embedding) — usa pesos explícitos
/// (para carregar pesos treinados do Python em runtime, se necessário).
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn predict_with_weights(
    w1: Vec<f32>, b1: Vec<f32>,
    w2: Vec<f32>, b2: Vec<f32>,
    w3: Vec<f32>, b3: Vec<f32>,
    embedding_dim: usize, hidden_dim: usize,
    embedding: Vec<f32>,
) -> PyResult<Vec<f32>> {
    let net = ExpectationNet {
        w1: Array2::from_shape_vec((hidden_dim, embedding_dim), w1).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
        b1: Array1::from(b1),
        w2: Array2::from_shape_vec((hidden_dim, hidden_dim), w2).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
        b2: Array1::from(b2),
        w3: Array2::from_shape_vec((embedding_dim, hidden_dim), w3).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?,
        b3: Array1::from(b3),
    };
    let x = Array1::from(embedding);
    Ok(net.forward(&x).into_raw_vec())
}

/// health() → bool (crate disponível)
#[pyfunction]
fn health() -> bool {
    true
}

#[pymodule]
fn omnimind_expectation_rs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(predict, m)?)?;
    m.add_function(wrap_pyfunction!(predict_with_weights, m)?)?;
    m.add_function(wrap_pyfunction!(health, m)?)?;
    Ok(())
}
