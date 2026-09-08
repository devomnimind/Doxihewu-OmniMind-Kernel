//! # Psychoanalytic LIF Neuron
//!
//! Leaky Integrate-and-Fire neuron with dynamic threshold governed by
//! neutrosophic triples (T, I, F) from the Dodecatiad INRC system.
//!
//! Each neuron represents one psychical dimension (e.g., Freud10D's 10 dims).
//! The threshold is not fixed — it adapts based on the neutrosophic state:
//!
//!   θ_k(T, I, F) = v_rest + (θ_base - v_rest) * clamp(1 + I_k - T_k, 0.3, 2.0)
//!
//! The threshold is computed as an OFFSET above v_rest, not an absolute value.
//! This prevents spontaneous firing when the multiplier pushes the threshold
//! below v_rest (which happened with the old absolute formula).
//!
//! High indeterminacy (I→1) raises the threshold → harder to fire (uncertainty).
//! High truth (T→1) lowers the threshold → easier to fire (clarity).
//!
//! The multiplier is clamped to [0.3, 2.0] to prevent saturation:
//! - Without the floor (0.3), T=1/I=0 would zero the offset → compulsive firing
//!   (psychoanalytically: delusional certainty, not healthy clarity).
//! - Without the ceiling (2.0), T=0/I=1 would push threshold below v_rest →
//!   spontaneous firing (psychoanalytically: uncontained anxiety, not productive doubt).
//!
//! F (falsity) modulates the reset potential:
//!   v_reset_k = v_reset_base * (1 - F_k * 0.5)
//!
//! High falsity → deeper reset (stronger rejection after firing).

use rayon::prelude::*;

/// Psychoanalytic LIF neuron with neutrosophic dynamic threshold.
#[derive(Clone, Debug)]
pub struct PsychoanalyticLif {
    /// Membrane potential.
    pub v: f64,
    /// Refractory counter in simulation steps.
    pub refractory_counter: i32,
    /// Resting potential.
    pub v_rest: f64,
    /// Base reset potential after spike.
    pub v_reset_base: f64,
    /// Base spike threshold.
    pub v_threshold_base: f64,
    /// Refractory period in steps.
    pub refractory_period: i32,
    /// Leak rate (decay per step).
    pub leak_rate: f64,
    /// Neutrosophic triple: (Truth, Indeterminacy, Falsity) ∈ [0,1]³.
    pub t: f64,
    pub i_neutro: f64,
    pub f_neutro: f64,
}

impl PsychoanalyticLif {
    /// Create a new psychoanalytic LIF neuron.
    pub fn new(
        v_rest: f64,
        v_reset_base: f64,
        v_threshold_base: f64,
        refractory_period: i32,
        leak_rate: f64,
    ) -> Self {
        Self {
            v: v_rest,
            refractory_counter: 0,
            v_rest,
            v_reset_base,
            v_threshold_base,
            refractory_period,
            leak_rate,
            t: 0.5,
            i_neutro: 0.0,
            f_neutro: 0.0,
        }
    }

    /// Set the neutrosophic triple (T, I, F).
    pub fn set_neutrosophic(&mut self, t: f64, i: f64, f: f64) {
        self.t = t.clamp(0.0, 1.0);
        self.i_neutro = i.clamp(0.0, 1.0);
        self.f_neutro = f.clamp(0.0, 1.0);
    }

    /// Dynamic threshold: θ = v_rest + (θ_base - v_rest) * clamp(1 + I - T, 0.3, 2.0).
    ///
    /// Computed as an offset above v_rest to prevent spontaneous firing.
    /// The clamp prevents saturation:
    /// - Floor 0.3: T=1/I=0 → threshold at 30% of offset above rest (easier but not compulsive)
    /// - Ceiling 2.0: T=0/I=1 → threshold at 200% of offset above rest (harder but not impossible)
    #[inline]
    pub fn dynamic_threshold(&self) -> f64 {
        let offset = self.v_threshold_base - self.v_rest;
        let multiplier = (1.0 + self.i_neutro - self.t).clamp(0.3, 2.0);
        self.v_rest + offset * multiplier
    }

    /// Dynamic reset: v_reset = v_reset_base * (1 - F * 0.5).
    #[inline]
    pub fn dynamic_reset(&self) -> f64 {
        self.v_reset_base * (1.0 - self.f_neutro * 0.5)
    }

    /// Advance one step. Returns 1.0 if spiked, 0.0 otherwise.
    pub fn step(&mut self, input_current: f64) -> f64 {
        // Refractory period: no integration.
        if self.refractory_counter > 0 {
            self.refractory_counter -= 1;
            return 0.0;
        }

        // Leaky integration.
        self.v += input_current - self.leak_rate * (self.v - self.v_rest);

        // Check against dynamic threshold.
        let threshold = self.dynamic_threshold();
        if self.v >= threshold {
            // Spike!
            self.v = self.dynamic_reset();
            self.refractory_counter = self.refractory_period;
            return 1.0;
        }

        0.0
    }

    /// Reset to resting state.
    pub fn reset(&mut self) {
        self.v = self.v_rest;
        self.refractory_counter = 0;
    }
}

/// Batch of psychoanalytic LIF neurons (e.g., 10 for Freud10D dimensions).
pub struct PsychoanalyticLifBatch {
    pub neurons: Vec<PsychoanalyticLif>,
}

impl PsychoanalyticLifBatch {
    /// Create a batch of N neurons with identical base parameters.
    pub fn new(n: usize, v_rest: f64, v_reset: f64, v_threshold: f64, refractory: i32, leak: f64) -> Self {
        Self {
            neurons: (0..n)
                .map(|_| PsychoanalyticLif::new(v_rest, v_reset, v_threshold, refractory, leak))
                .collect(),
        }
    }

    /// Set neutrosophic triples for all neurons (parallel).
    pub fn set_neutrosophic_batch(&mut self, triples: &[(f64, f64, f64)]) {
        assert_eq!(triples.len(), self.neurons.len(), "triples length mismatch");
        self.neurons
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, neuron)| {
                let &(t, i, f) = &triples[idx];
                neuron.set_neutrosophic(t, i, f);
            });
    }

    /// Advance all neurons one step (parallel). Returns spike vector.
    pub fn step_batch(&mut self, inputs: &[f64]) -> Vec<f64> {
        assert_eq!(inputs.len(), self.neurons.len(), "inputs length mismatch");
        self.neurons
            .par_iter_mut()
            .enumerate()
            .map(|(idx, neuron)| neuron.step(inputs[idx]))
            .collect()
    }

    /// Get current membrane potentials.
    pub fn get_potentials(&self) -> Vec<f64> {
        self.neurons.iter().map(|n| n.v).collect()
    }

    /// Get current dynamic thresholds.
    pub fn get_thresholds(&self) -> Vec<f64> {
        self.neurons.iter().map(|n| n.dynamic_threshold()).collect()
    }

    /// Reset all neurons.
    pub fn reset_all(&mut self) {
        self.neurons.par_iter_mut().for_each(|n| n.reset());
    }
}
