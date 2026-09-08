//! # Sovereign Kuramoto Solver
//!
//! Clean-room implementation of a Kuramoto oscillator solver extended with:
//!   - Hyper-Kuramoto (triplet coupling, 3rd-order synchrony)
//!   - INRC field pressure (Piaget's group of transformations)
//!   - Psychoanalytic coupling (Freud10D connectivity matrix)
//!   - Dodecatiad geometry coupling
//!
//! Baseline equation:
//!   dθ_k/dt = ω_k + Σ_m K_km sin(θ_m - θ_k) + noise
//!
//! Sovereign extension:
//!   dθ_k/dt = ω_k
//!     + K_base Σ_m K_km sin(θ_m - θ_k)
//!     + σ_geo Σ_m W_km sin(θ_m - θ_k)           (dodecatiad geometry)
//!     + σ_psi Σ_m Ψ_km sin(θ_m - θ_k)           (Freud10D psychoanalytic coupling)
//!     + K_hyper Σ_{(k,m,n)} A_kmn sin(θ_m + θ_n - 2θ_k)  (hyper-Kuramoto)
//!     + F_inrc · cos(θ_k)                        (INRC field pressure)
//!     + noise
//!
//! Order parameters:
//!   R_pair  = |Σ_k e^{iθ_k}| / N
//!   R_triad = |Σ_{(k,m,n)} A_kmn e^{i(θ_k+θ_m+θ_n)}| / Σ A_kmn

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

const TWO_PI: f64 = std::f64::consts::TAU;

/// Sovereign Kuramoto oscillator solver with hyper-Kuramoto and INRC extensions.
///
/// All scratch buffers are preallocated to avoid per-step allocations.
pub struct SovereignKuramotoSolver {
    /// Number of oscillators (typically 10 for Freud10D or 12 for dodecatiad).
    pub n: usize,
    /// Natural frequencies `ω_k`, shape `(n,)`.
    pub omega: Vec<f64>,
    /// Baseline coupling matrix `K_km`, row-major `(n*n)`.
    pub coupling: Vec<f64>,
    /// Current phase vector `θ_k`, shape `(n,)`.
    pub phases: Vec<f64>,
    /// Gaussian noise amplitude.
    pub noise_amp: f64,
    /// INRC field pressure strength `F_inrc`.
    pub inrc_field_pressure: f64,
    /// Geometry coupling gain `σ_geo`.
    pub sigma_geo: f64,
    /// Psychoanalytic coupling gain `σ_psi`.
    pub sigma_psi: f64,
    /// Hyper-Kuramoto coupling gain `K_hyper`.
    pub k_hyper: f64,

    // Geometry coupling matrix W_km, row-major (n*n). Empty = disabled.
    geo_matrix: Vec<f64>,
    // Psychoanalytic coupling matrix Ψ_km, row-major (n*n). Empty = disabled.
    psi_matrix: Vec<f64>,
    // Hypergraph triplets: flat list of (i, j, k) index triples.
    hyperedges: Vec<(usize, usize, usize)>,
    // Hypergraph weights A_ijk, parallel to hyperedges.
    hyper_weights: Vec<f64>,

    // Scratch buffers
    dtheta: Vec<f64>,
    sin_diff: Vec<f64>,    // n*n, sin(θ_m - θ_k)
    cos_theta: Vec<f64>,   // n, cos(θ_k) for INRC field
    noise: Vec<f64>,       // n, standard normals
    geo_coupling: Vec<f64>,
    psi_coupling: Vec<f64>,
    hyper_coupling: Vec<f64>,
}

impl SovereignKuramotoSolver {
    /// Create a new solver with preallocated scratch buffers.
    pub fn new(
        omega: Vec<f64>,
        coupling_flat: Vec<f64>,
        initial_phases: Vec<f64>,
        noise_amp: f64,
    ) -> Self {
        let n = omega.len();
        assert!(n > 0, "omega must not be empty");
        assert_eq!(
            initial_phases.len(),
            n,
            "initial_phases length mismatch: got {}, expected {}",
            initial_phases.len(),
            n
        );
        assert_eq!(
            coupling_flat.len(),
            n * n,
            "coupling length mismatch: got {}, expected {}",
            coupling_flat.len(),
            n * n
        );

        Self {
            n,
            omega,
            coupling: coupling_flat,
            phases: initial_phases,
            noise_amp,
            inrc_field_pressure: 0.0,
            sigma_geo: 0.0,
            sigma_psi: 0.0,
            k_hyper: 0.0,
            geo_matrix: Vec::new(),
            psi_matrix: Vec::new(),
            hyperedges: Vec::new(),
            hyper_weights: Vec::new(),
            dtheta: vec![0.0; n],
            sin_diff: vec![0.0; n * n],
            cos_theta: vec![0.0; n],
            noise: vec![0.0; n],
            geo_coupling: vec![0.0; n],
            psi_coupling: vec![0.0; n],
            hyper_coupling: vec![0.0; n],
        }
    }

    /// Set the dodecatiad geometry coupling matrix W (row-major n*n).
    pub fn set_geometry(&mut self, w_flat: Vec<f64>, sigma_g: f64) {
        assert_eq!(
            w_flat.len(),
            self.n * self.n,
            "geometry matrix length mismatch"
        );
        self.geo_matrix = w_flat;
        self.sigma_geo = sigma_g;
    }

    /// Set the Freud10D psychoanalytic coupling matrix Ψ (row-major n*n).
    pub fn set_psychoanalytic(&mut self, psi_flat: Vec<f64>, sigma_psi: f64) {
        assert_eq!(
            psi_flat.len(),
            self.n * self.n,
            "psychoanalytic matrix length mismatch"
        );
        self.psi_matrix = psi_flat;
        self.sigma_psi = sigma_psi;
    }

    /// Set hypergraph triplets for hyper-Kuramoto coupling.
    /// Each triplet (i, j, k) contributes A_ijk * sin(θ_j + θ_k - 2θ_i).
    pub fn set_hypergraph(&mut self, edges: Vec<(usize, usize, usize)>, weights: Vec<f64>, k_hyper: f64) {
        assert_eq!(edges.len(), weights.len(), "hyperedges and weights length mismatch");
        for &(i, j, k) in &edges {
            assert!(i < self.n && j < self.n && k < self.n, "hyperedge index out of bounds");
        }
        self.hyperedges = edges;
        self.hyper_weights = weights;
        self.k_hyper = k_hyper;
    }

    /// Set INRC field pressure `F_inrc`.
    pub fn set_inrc_field_pressure(&mut self, f: f64) {
        self.inrc_field_pressure = f;
    }

    /// Advance one Euler step with all enabled coupling terms.
    /// Returns the pairwise order parameter R_pair ∈ [0, 1].
    pub fn step(&mut self, dt: f64, seed: u64) -> f64 {
        let n = self.n;
        let phases = &self.phases;

        // 1) Shared sin-difference matrix (parallelized).
        self.sin_diff
            .par_chunks_mut(n)
            .enumerate()
            .for_each(|(row_idx, row)| {
                let theta_n = phases[row_idx];
                for (col_idx, value) in row.iter_mut().enumerate() {
                    *value = (phases[col_idx] - theta_n).sin();
                }
            });

        // 2) Noise vector.
        if seed == 0 || self.noise_amp == 0.0 {
            self.noise.fill(0.0);
        } else {
            fill_standard_normals(&mut self.noise, seed);
        }

        // 3) Geometry coupling: σ_geo * Σ_m W_km sin(θ_m - θ_k).
        let has_geo = !self.geo_matrix.is_empty() && self.sigma_geo != 0.0;
        if has_geo {
            self.geo_coupling
                .par_iter_mut()
                .enumerate()
                .for_each(|(row_idx, geo_n)| {
                    let w_row = &self.geo_matrix[row_idx * n..(row_idx + 1) * n];
                    let sin_row = &self.sin_diff[row_idx * n..(row_idx + 1) * n];
                    *geo_n = self.sigma_geo
                        * w_row
                            .iter()
                            .zip(sin_row.iter())
                            .map(|(w, s)| w * s)
                            .sum::<f64>();
                });
        } else {
            self.geo_coupling.fill(0.0);
        }

        // 4) Psychoanalytic coupling: σ_psi * Σ_m Ψ_km sin(θ_m - θ_k).
        let has_psi = !self.psi_matrix.is_empty() && self.sigma_psi != 0.0;
        if has_psi {
            self.psi_coupling
                .par_iter_mut()
                .enumerate()
                .for_each(|(row_idx, psi_n)| {
                    let p_row = &self.psi_matrix[row_idx * n..(row_idx + 1) * n];
                    let sin_row = &self.sin_diff[row_idx * n..(row_idx + 1) * n];
                    *psi_n = self.sigma_psi
                        * p_row
                            .iter()
                            .zip(sin_row.iter())
                            .map(|(p, s)| p * s)
                            .sum::<f64>();
                });
        } else {
            self.psi_coupling.fill(0.0);
        }

        // 5) Hyper-Kuramoto coupling: K_hyper * Σ_{(k,m,n)} A_kmn sin(θ_m + θ_n - 2θ_k).
        self.hyper_coupling.fill(0.0);
        if self.k_hyper != 0.0 && !self.hyperedges.is_empty() {
            for (edge_idx, &(i, j, k)) in self.hyperedges.iter().enumerate() {
                let weight = self.hyper_weights[edge_idx];
                let phase_sum = phases[j] + phases[k] - 2.0 * phases[i];
                self.hyper_coupling[i] += self.k_hyper * weight * phase_sum.sin();
            }
        }

        // 6) INRC field pressure: F_inrc * cos(θ_k).
        if self.inrc_field_pressure != 0.0 {
            for (c, &theta) in self.cos_theta.iter_mut().zip(phases.iter()) {
                *c = theta.cos();
            }
        } else {
            self.cos_theta.fill(0.0);
        }

        // 7) Assemble dtheta from all terms.
        self.dtheta
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, dtheta_n)| {
                let coupling_row = &self.coupling[i * n..(i + 1) * n];
                let sin_row = &self.sin_diff[i * n..(i + 1) * n];
                let coupling_sum = coupling_row
                    .iter()
                    .zip(sin_row.iter())
                    .map(|(k, s)| k * s)
                    .sum::<f64>();

                *dtheta_n = self.omega[i]
                    + coupling_sum
                    + self.geo_coupling[i]
                    + self.psi_coupling[i]
                    + self.hyper_coupling[i]
                    + self.inrc_field_pressure * self.cos_theta[i]
                    + self.noise_amp * self.noise[i];
            });

        // 8) Integrate phase (mod 2π).
        for (phase, dtheta) in self.phases.iter_mut().zip(self.dtheta.iter()) {
            *phase = (*phase + dtheta * dt).rem_euclid(TWO_PI);
        }

        self.order_parameter_pair()
    }

    /// Run N steps and return R_pair after each.
    pub fn run(&mut self, n_steps: usize, dt: f64, seed: u64) -> Vec<f64> {
        let mut order_values = Vec::with_capacity(n_steps);
        for step_idx in 0..n_steps {
            let step_seed = if seed == 0 { 0 } else { seed.wrapping_add(step_idx as u64) };
            order_values.push(self.step(dt, step_seed));
        }
        order_values
    }

    /// Pairwise Kuramoto order parameter: R_pair = |Σ_k e^{iθ_k}| / N.
    pub fn order_parameter_pair(&self) -> f64 {
        if self.phases.is_empty() {
            return 0.0;
        }
        let n_inv = 1.0 / self.phases.len() as f64;
        let mean_cos = self.phases.iter().map(|t| t.cos()).sum::<f64>() * n_inv;
        let mean_sin = self.phases.iter().map(|t| t.sin()).sum::<f64>() * n_inv;
        (mean_cos * mean_cos + mean_sin * mean_sin).sqrt()
    }

    /// Hyper-Kuramoto triadic order parameter:
    /// R_triad = |Σ_{(k,m,n)} A_kmn e^{i(θ_k+θ_m+θ_n)}| / Σ A_kmn
    pub fn order_parameter_triad(&self) -> f64 {
        if self.hyperedges.is_empty() {
            return 0.0;
        }
        let mut sum_cos = 0.0_f64;
        let mut sum_sin = 0.0_f64;
        let mut total_weight = 0.0_f64;

        for (idx, &(i, j, k)) in self.hyperedges.iter().enumerate() {
            let w = self.hyper_weights[idx];
            let phase_sum = self.phases[i] + self.phases[j] + self.phases[k];
            sum_cos += w * phase_sum.cos();
            sum_sin += w * phase_sum.sin();
            total_weight += w;
        }

        if total_weight == 0.0 {
            return 0.0;
        }
        ((sum_cos * sum_cos + sum_sin * sum_sin).sqrt()) / total_weight
    }

    /// Borrow current phase vector.
    pub fn get_phases(&self) -> &[f64] {
        &self.phases
    }

    /// Replace phase vector.
    pub fn set_phases(&mut self, phases: Vec<f64>) {
        assert_eq!(phases.len(), self.n, "phases length mismatch");
        self.phases = phases;
    }

    /// Replace baseline coupling matrix.
    pub fn set_coupling(&mut self, coupling_flat: Vec<f64>) {
        assert_eq!(coupling_flat.len(), self.n * self.n, "coupling length mismatch");
        self.coupling = coupling_flat;
    }
}

/// Box-Muller standard normal generation with ChaCha8 RNG (deterministic, reproducible).
fn fill_standard_normals(out: &mut [f64], seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut i = 0usize;

    while i + 1 < out.len() {
        let u1 = rng.gen::<f64>().max(f64::MIN_POSITIVE);
        let u2 = rng.gen::<f64>();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = TWO_PI * u2;
        out[i] = r * theta.cos();
        out[i + 1] = r * theta.sin();
        i += 2;
    }

    if i < out.len() {
        let u1 = rng.gen::<f64>().max(f64::MIN_POSITIVE);
        let u2 = rng.gen::<f64>();
        let r = (-2.0 * u1.ln()).sqrt();
        out[i] = r * (TWO_PI * u2).cos();
    }
}
