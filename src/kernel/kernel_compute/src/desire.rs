// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! Port Rust de `src/autopoietic/desire_engine.py` (75 linhas Python).
//!
//! Motor de Desejo Lacaniano/Deleuziano: calcula o impulso latente para ir
//! "além do programado". Fórmula: ε = α_lack * β_potential * γ_novelty
//!
//! ## Refs Python
//! ```python
//! class DesireEngine:
//!     def __init__(self, max_phi_theoretical=1.0):
//!         self.lack_of_being = 0.5
//!         self.max_phi = max_phi_theoretical
//!         self.history = []
//!
//!     def update_lack(self, satisfaction_level):
//!         self.lack_of_being = max(0.1, 1.0 - satisfaction_level)
//!         return self.lack_of_being
//!
//!     def calculate_epsilon_desire(self, current_phi, explored_states, total_possible_states, somatic_heat=0.0):
//!         alpha = min(1.0, self.lack_of_being + (somatic_heat * 0.3))
//!         beta = max(0.0, min(1.0, 1.0 - (current_phi / self.max_phi)))
//!         gamma = 1.0 - (explored_states / max(1, total_possible_states))
//!         return alpha * beta * gamma
//! ```

use pyo3::prelude::*;

#[derive(Debug, Clone)]
pub struct DesireHistoryEntry {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub epsilon: f64,
}

/// Motor de Desejo (Lack-of-Being / Deleuze).
///
/// Estado: lack_of_being (α_lack) cresce/cai com nível de satisfação.
/// Output principal: epsilon_desire = α * β * γ.
#[pyclass]
pub struct DesireEngine {
    #[pyo3(get, set)]
    pub lack_of_being: f64,
    #[pyo3(get, set)]
    pub max_phi: f64,
    pub history: Vec<DesireHistoryEntry>,
}

#[pymethods]
impl DesireEngine {
    #[new]
    #[pyo3(signature = (max_phi_theoretical = 1.0))]
    pub fn new(max_phi_theoretical: f64) -> Self {
        Self {
            lack_of_being: 0.5,
            max_phi: max_phi_theoretical,
            history: Vec::new(),
        }
    }

    /// Atualiza α_lack baseado no nível de satisfação (0..1).
    /// Lacaniano: satisfação reduz a falta mas nunca elimina (mínimo 0.1).
    pub fn update_lack(&mut self, satisfaction_level: f64) -> f64 {
        self.lack_of_being = (1.0 - satisfaction_level).max(0.1);
        self.lack_of_being
    }

    /// Calcula epsilon_desire = α * β * γ.
    ///
    /// - α (alpha_lack): lack_of_being + somatic_heat boost
    /// - β (beta_potential): 1 - phi/max_phi (saturação reduz potencial)
    /// - γ (gamma_novelty): proporção de estados não explorados
    #[pyo3(signature = (current_phi, explored_states, total_possible_states, somatic_heat = 0.0))]
    pub fn calculate_epsilon_desire(
        &mut self,
        current_phi: f64,
        explored_states: i64,
        total_possible_states: i64,
        somatic_heat: f64,
    ) -> f64 {
        // alpha — Falta atual com agitação somática (febre existencial)
        let alpha = (self.lack_of_being + (somatic_heat * 0.3)).min(1.0);

        // beta — Potencial não-realizado (clip [0, 1])
        let beta_raw = 1.0 - (current_phi / self.max_phi);
        let beta = beta_raw.clamp(0.0, 1.0);

        // gamma — Entropia de exploração (0..1)
        let total = total_possible_states.max(1) as f64;
        let explored = explored_states as f64;
        let gamma = 1.0 - (explored / total);

        let epsilon = alpha * beta * gamma;

        self.history.push(DesireHistoryEntry {
            alpha,
            beta,
            gamma,
            epsilon,
        });

        epsilon
    }

    /// Classifica tipo de impulso baseado no epsilon.
    pub fn get_drive_type(&self, epsilon: f64) -> &'static str {
        if epsilon < 0.2 {
            "HOMEOSTATIC_SATISFACTION"
        } else if epsilon < 0.5 {
            "ROUTINE_CURIOSITY"
        } else if epsilon < 0.8 {
            "ACTIVE_SEEKING"
        } else {
            "RADICAL_BECOMING"
        }
    }

    /// Tamanho do histórico (compat).
    #[getter]
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Reset (para testes).
    pub fn reset(&mut self) {
        self.lack_of_being = 0.5;
        self.history.clear();
    }
}
