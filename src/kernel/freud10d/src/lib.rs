// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Aparato Psíquico Freudiano 10D — Porte Rust.
//!
//! Espelha `src/consciousness/freudian_10d_apparatus.py` em pyo3+ndarray.
//!
//! 10 dimensões topográficas:
//!   0=Phi(Percepção) 1=Psi(Memória) 2=Omega(Consciência) 3=Theta(Pré-consciente)
//!   4=Upsilon(Inconsciente) 5=Xi(Id) 6=Zeta(Ego) 7=Eta(Superego)
//!   8=Kappa(Transferência) 9=Lambda(Sublimação)
//!
//! Forward pass: 5 iterações de `tanh(W.t() @ state)` em CPU (ndarray).
//! GPU lane fica para versão futura via candle/tch.
#![allow(non_local_definitions)]

use ndarray::{Array1, Array2};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// ---------- Índices das dimensões (alinhados com Dimension10D Python) ----------
const DIM_PHI: usize = 0;
const DIM_PSI: usize = 1;
const DIM_OMEGA: usize = 2;
const DIM_THETA: usize = 3;
const DIM_UPSILON: usize = 4;
const DIM_XI: usize = 5;
const DIM_ZETA: usize = 6;
const DIM_ETA: usize = 7;
const DIM_KAPPA: usize = 8;
const DIM_LAMBDA: usize = 9;
const N_DIMS: usize = 10;
const FORWARD_ITERS: usize = 5;

const DIM_NAMES: [&str; N_DIMS] = [
    "phi", "psi", "omega", "theta", "upsilon",
    "xi", "zeta", "eta", "kappa", "lambda_",
];

/// Inicializa a matriz de conectividade 10×10 idêntica ao Python.
fn initialize_connectivity() -> Array2<f32> {
    let mut w = Array2::<f32>::zeros((N_DIMS, N_DIMS));

    // Fluxo perceptual básico
    w[[DIM_PHI, DIM_PSI]] = 0.8;
    w[[DIM_PSI, DIM_OMEGA]] = 0.7;

    // Topográfico
    w[[DIM_PSI, DIM_THETA]] = 0.6;
    w[[DIM_THETA, DIM_UPSILON]] = 0.3;
    w[[DIM_UPSILON, DIM_PSI]] = 0.2;

    // Estrutural
    w[[DIM_PSI, DIM_XI]] = 0.45;
    w[[DIM_UPSILON, DIM_XI]] = 0.75;
    w[[DIM_THETA, DIM_ZETA]] = 0.55;
    w[[DIM_OMEGA, DIM_ZETA]] = 0.35;
    w[[DIM_XI, DIM_ZETA]] = 0.9;
    w[[DIM_ZETA, DIM_ETA]] = 0.5;
    w[[DIM_ETA, DIM_ZETA]] = 0.6;

    // Processos
    w[[DIM_PSI, DIM_KAPPA]] = 0.4;
    w[[DIM_UPSILON, DIM_KAPPA]] = 0.5;
    w[[DIM_XI, DIM_KAPPA]] = 0.35;
    w[[DIM_XI, DIM_LAMBDA]] = 0.5;
    w[[DIM_ZETA, DIM_LAMBDA]] = 0.45;
    w[[DIM_ETA, DIM_LAMBDA]] = 0.2;
    w[[DIM_LAMBDA, DIM_OMEGA]] = 0.7;

    // Recorrentes
    w[[DIM_OMEGA, DIM_PSI]] = 0.3;
    w[[DIM_ZETA, DIM_PSI]] = 0.2;
    w[[DIM_ETA, DIM_THETA]] = 0.15;
    w[[DIM_KAPPA, DIM_UPSILON]] = 0.4;

    // Feedback perceptual — phi precisa de conexões incoming
    // (sem estas, phi sempre = 0 após tanh(W.t() @ state))
    w[[DIM_PHI, DIM_PHI]] = 0.3;       // auto-recorrência: percepção persiste
    w[[DIM_OMEGA, DIM_PHI]] = 0.2;     // consciência modula percepção
    w[[DIM_PSI, DIM_PHI]] = 0.15;      // memória modula percepção
    w[[DIM_ZETA, DIM_PHI]] = 0.1;      // ego modula percepção

    // Auto-recorrência do superego (estabiliza eta)
    w[[DIM_ETA, DIM_ETA]] = 0.2;

    w
}

/// Aparato Psíquico Freudiano 10D (CPU-bound, ndarray).
#[pyclass]
pub struct Freud10DApparatus {
    #[pyo3(get)]
    n_neurons: usize,
    #[pyo3(get)]
    n_dims: usize,
    w: Array2<f32>,
    state: Array1<f32>,
    tension_history: Vec<f32>,
    pleasure_history: Vec<f32>,
}

#[pymethods]
impl Freud10DApparatus {
    /// Inicializa o aparato. n_neurons_per_dim por padrão = 128 (mantém compat).
    #[new]
    #[pyo3(signature = (n_neurons_per_dim = 128))]
    fn new(n_neurons_per_dim: usize) -> Self {
        Self {
            n_neurons: n_neurons_per_dim,
            n_dims: N_DIMS,
            w: initialize_connectivity(),
            state: Array1::<f32>::zeros(N_DIMS),
            tension_history: Vec::with_capacity(1024),
            pleasure_history: Vec::with_capacity(1024),
        }
    }

    /// Forward pass: aplica perception → 5 iterações tanh(W.t() @ state).
    /// Retorna (consciousness_scalar, state_vector_10d).
    fn forward(&mut self, perception: Vec<f32>) -> PyResult<(f32, Vec<f32>)> {
        // 1. Inicia state com média da perception em DIM_PHI
        let mut state = Array1::<f32>::zeros(N_DIMS);
        if !perception.is_empty() {
            let mean = perception.iter().sum::<f32>() / (perception.len() as f32);
            state[DIM_PHI] = mean;
        }

        // 2. 5 iterações: state = tanh(W.t() @ state)
        let wt = self.w.t().to_owned(); // 10×10 transposta
        for _ in 0..FORWARD_ITERS {
            let next = wt.dot(&state);
            for i in 0..N_DIMS {
                state[i] = next[i].tanh();
            }
        }

        // 3. Atualiza state interno
        self.state.assign(&state);

        // 4. Tensão = sum(|state|)
        let tension: f32 = state.iter().map(|x| x.abs()).sum();
        self.tension_history.push(tension);

        // 5. Prazer = -delta_tension
        let prev_len = self.tension_history.len();
        if prev_len >= 2 {
            let delta = tension - self.tension_history[prev_len - 2];
            self.pleasure_history.push(-delta);
        }

        let consciousness = state[DIM_OMEGA];
        Ok((consciousness, state.to_vec()))
    }

    /// Estado atual como dict {phi, psi, ..., lambda_}
    fn state_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        for (i, name) in DIM_NAMES.iter().enumerate() {
            d.set_item(name, self.state[i] as f64)?;
        }
        Ok(d.unbind().into())
    }

    /// Vetor 10D atual (compat com Psychic10DState.to_vector).
    fn state_vector(&self) -> Vec<f32> {
        self.state.to_vec()
    }

    /// Fluxo topográfico: Consciente ↔ Pré-consciente ↔ Inconsciente.
    fn topographic_flow(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        d.set_item("conscious", self.state[DIM_OMEGA] as f64)?;
        d.set_item("preconscious", self.state[DIM_THETA] as f64)?;
        d.set_item("unconscious", self.state[DIM_UPSILON] as f64)?;
        d.set_item("repression", self.w[[DIM_THETA, DIM_UPSILON]] as f64)?;
        d.set_item("return_repressed", self.w[[DIM_UPSILON, DIM_PSI]] as f64)?;
        Ok(d.unbind().into())
    }

    /// Conflito estrutural: Id ↔ Ego ↔ Superego.
    fn structural_conflict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        let conflict = (self.state[DIM_XI] - self.state[DIM_ETA]).abs();
        d.set_item("id_drive", self.state[DIM_XI] as f64)?;
        d.set_item("ego_mediation", self.state[DIM_ZETA] as f64)?;
        d.set_item("superego_censorship", self.state[DIM_ETA] as f64)?;
        d.set_item("conflict_intensity", conflict as f64)?;
        Ok(d.unbind().into())
    }

    /// Métricas completas equivalentes a get_10d_metrics() do Python.
    fn get_10d_metrics(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);

        // state_10d como list
        let lst = PyList::empty(py);
        for v in self.state.iter() {
            lst.append(*v as f64)?;
        }
        d.set_item("state_10d", lst)?;

        // topographic + structural via funções já existentes
        d.set_item("topographic", self.topographic_flow(py)?)?;
        d.set_item("structural", self.structural_conflict(py)?)?;
        d.set_item("transference", self.state[DIM_KAPPA] as f64)?;
        d.set_item("sublimation", self.state[DIM_LAMBDA] as f64)?;

        let tension_last = self.tension_history.last().copied().unwrap_or(0.0);
        d.set_item("tension", tension_last as f64)?;

        let pleasure_last = self.pleasure_history.last().copied().unwrap_or(0.0);
        d.set_item("pleasure", pleasure_last as f64)?;

        Ok(d.unbind().into())
    }

    /// Tamanho do histórico de tensão (útil para debug/monitor).
    #[getter]
    fn tension_history_len(&self) -> usize {
        self.tension_history.len()
    }

    /// Janela mais recente do histórico de tensão (até `n` últimos).
    fn tension_window(&self, n: usize) -> Vec<f32> {
        let len = self.tension_history.len();
        let start = len.saturating_sub(n);
        self.tension_history[start..].to_vec()
    }

    /// Reset opcional para zerar histórico e estado (útil em testes).
    fn reset(&mut self) {
        self.state = Array1::<f32>::zeros(N_DIMS);
        self.tension_history.clear();
        self.pleasure_history.clear();
    }

    /// Versão do módulo.
    #[staticmethod]
    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    // -----------------------------------------------------------------------
    // INRC Piagetiano — Grupo de Transformações Cognitivas sobre o state 10D
    // -----------------------------------------------------------------------
    //
    // Cada operador atua sobre o state_vector 10D preservando invariantes.
    // Propriedades: I²=I, N²=I, R²=I, C²=I; NR=C, NC=R, RC=N.
    //
    // Mapeamento dimensional:
    //   I (Identidade)    → preserva state atual
    //   N (Negação)       → inverte state (antítese, contra-hipótese)
    //   R (Reciprocidade) → troca perspectiva Ego↔Superego, Id↔Sublimação
    //   C (Compensação)   → ajusta state para preservar invariantes canônicos

    /// I — Identidade: retorna state atual sem modificação.
    fn inrc_identity(&self) -> Vec<f32> {
        self.state.to_vec()
    }

    /// N — Negação: inverte o state (antítese).
    /// state_N[i] = -state[i] (mantém escala tanh, inverte sinal)
    fn inrc_negate(&self) -> Vec<f32> {
        self.state.iter().map(|x| -x).collect()
    }

    /// R — Reciprocidade: troca perspectivas entre pares de dimensões.
    /// Ego(Zeta) ↔ Superego(Eta), Id(Xi) ↔ Sublimação(Lambda), Phi ↔ Omega.
    fn inrc_reciprocate(&self) -> Vec<f32> {
        let mut s = self.state.to_vec();
        s.swap(DIM_ZETA, DIM_ETA);       // Ego ↔ Superego
        s.swap(DIM_XI, DIM_LAMBDA);      // Id ↔ Sublimação
        s.swap(DIM_PHI, DIM_OMEGA);      // Percepção ↔ Consciência
        s
    }

    /// C — Compensação: ajusta state para preservar invariantes.
    /// Invariantes canônicos: tensão_média ≈ 0.5, equilíbrio Ego-Superego.
    /// Aplica correção suave (30%) em direção ao equilíbrio.
    fn inrc_compensate(&self) -> Vec<f32> {
        let mut s = self.state.to_vec();
        // Invariante 1: equilíbrio Ego-Superego (Zeta ≈ Eta)
        let ego_super_mean = (s[DIM_ZETA] + s[DIM_ETA]) / 2.0;
        s[DIM_ZETA] = s[DIM_ZETA] * 0.70 + ego_super_mean * 0.30;
        s[DIM_ETA] = s[DIM_ETA] * 0.70 + ego_super_mean * 0.30;
        // Invariante 2: tensão total moderada (target = 0.5 por dim)
        let tension: f32 = s.iter().map(|x| x.abs()).sum::<f32>() / N_DIMS as f32;
        if tension > 0.7 {
            let scale = 0.7 / tension;
            for v in s.iter_mut() {
                *v *= scale;
            }
        }
        s
    }

    /// Tripleto neutrosófico (T, I, F) para cada dimensão do state 10D.
    /// T = |state[i]| (verdade/ativação), I = 1 - |state[i]| - F (indeterminação),
    /// F = tension_penalty (falsidade/conflito).
    fn neutrosophic_state(&self, py: Python) -> PyResult<Py<PyAny>> {
        let d = PyDict::new(py);
        let tension_penalty = {
            let t: f32 = self.state.iter().map(|x| x.abs()).sum::<f32>() / N_DIMS as f32;
            (t - 0.5).clamp(0.0, 0.5)
        };
        for (i, name) in DIM_NAMES.iter().enumerate() {
            let t_val = self.state[i].abs().min(1.0);
            let f_val = tension_penalty;
            let i_val = (1.0 - t_val - f_val).max(0.0);
            let triple = PyDict::new(py);
            triple.set_item("T", t_val as f64)?;
            triple.set_item("I", i_val as f64)?;
            triple.set_item("F", f_val as f64)?;
            d.set_item(name, triple)?;
        }
        Ok(d.unbind().into())
    }

    /// Recomenda operação INRC baseado no state atual.
    /// Retorna ("I"|"N"|"R"|"C", reason_str).
    fn inrc_recommend(&self) -> (String, String) {
        let mean_abs: f32 = self.state.iter().map(|x| x.abs()).sum::<f32>() / N_DIMS as f32;
        let ego_super_diff = (self.state[DIM_ZETA] - self.state[DIM_ETA]).abs();
        let id_val = self.state[DIM_XI].abs();
        let omega_val = self.state[DIM_OMEGA].abs();

        if ego_super_diff > 0.4 {
            ("C".to_string(), format!("ego_superego_imbalance={:.3}", ego_super_diff))
        } else if id_val > 0.7 && omega_val < 0.3 {
            ("R".to_string(), format!("id_drive_high={:.3}_consciousness_low={:.3}", id_val, omega_val))
        } else if mean_abs < 0.15 {
            ("N".to_string(), format!("stagnant_field_mean_abs={:.3}", mean_abs))
        } else {
            ("I".to_string(), format!("coherent_field_mean_abs={:.3}", mean_abs))
        }
    }
}

/// Função utilitária standalone para extrair valores freudianos compatíveis
/// com `integrate_with_topology` do Python (mapeia 10D → 7 chaves).
#[pyfunction]
fn freudian_values_from_state(state_vec: Vec<f32>, py: Python) -> PyResult<Py<PyAny>> {
    if state_vec.len() < N_DIMS {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("state_vec deve ter pelo menos {} elementos", N_DIMS),
        ));
    }
    let d = PyDict::new(py);
    d.set_item("phi", state_vec[DIM_PHI] as f64)?;
    d.set_item("psi", state_vec[DIM_PSI] as f64)?;
    d.set_item("sigma", state_vec[DIM_ETA] as f64)?; // sigma = superego (lei)
    d.set_item("epsilon", state_vec[DIM_XI] as f64)?; // epsilon = pulsão (id)
    d.set_item("omega", state_vec[DIM_OMEGA] as f64)?;
    d.set_item("lambda_", state_vec[DIM_LAMBDA] as f64)?;
    d.set_item("zeta", state_vec[DIM_ZETA] as f64)?;
    Ok(d.unbind().into())
}

/// Módulo Python `omnimind_freud10d`.
#[pymodule]
fn omnimind_freud10d(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Freud10DApparatus>()?;
    m.add_function(wrap_pyfunction!(freudian_values_from_state, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("N_DIMS", N_DIMS)?;
    m.add("FORWARD_ITERS", FORWARD_ITERS)?;
    Ok(())
}
