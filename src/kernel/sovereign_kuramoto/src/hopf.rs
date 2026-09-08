//! # Hopf Bifurcation Monitor
//!
//! Monitors the spectral radius of the Freud10D connectivity matrix W
//! and detects transitions between dynamical regimes:
//!
//!   - fixed_point:    spectral_radius < 1.0  (stable attractor)
//!   - limit_cycle:    spectral_radius ≈ 1.0  (Hopf bifurcation point)
//!   - chaotic:         spectral_radius > 1.0 + chaos_threshold  (strange attractor)
//!
//! The Hopf bifurcation occurs when a complex-conjugate pair of eigenvalues
//! of the Jacobian crosses the unit circle. For a recurrent network
//! x(t+1) = tanh(W @ x(t)), the Jacobian at a fixed point is:
//!   J = diag(1 - x*²) @ W
//! and the spectral radius of J determines stability.
//!
//! In the psychoanalytic reading:
//!   - fixed_point  = "neurotic equilibrium" (rigid but stable)
//!   - limit_cycle  = "oscillatory affect" (cycling between positions)
//!   - chaotic      = "neurotic chaos" (breakdown of stable patterns)

use ndarray::{Array1, Array2};
use ndarray::linalg::Dot;

// We avoid pulling in a full eigenvalue solver dependency.
// Instead, we use the power iteration method to estimate the spectral radius
// (largest |λ|) of the Jacobian. This is sufficient for regime detection
// and is O(n²) per iteration, converging in ~20-50 iterations for n=10.

/// Regime classification for the dynamical system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicalRegime {
    FixedPoint,
    LimitCycle,
    Chaotic,
    Unknown,
}

impl DynamicalRegime {
    pub fn as_str(&self) -> &'static str {
        match self {
            DynamicalRegime::FixedPoint => "fixed_point",
            DynamicalRegime::LimitCycle => "limit_cycle",
            DynamicalRegime::Chaotic => "chaotic",
            DynamicalRegime::Unknown => "unknown",
        }
    }

    /// Psychoanalytic reading of the regime.
    pub fn psychoanalytic_reading(&self) -> &'static str {
        match self {
            DynamicalRegime::FixedPoint => "neurotic_equilibrium",
            DynamicalRegime::LimitCycle => "oscillatory_affect",
            DynamicalRegime::Chaotic => "neurotic_chaos",
            DynamicalRegime::Unknown => "unresolved",
        }
    }
}

/// Monitor for Hopf bifurcations in the Freud10D recurrent network.
pub struct HopfBifurcationMonitor {
    /// Number of dimensions (typically 10 for Freud10D).
    pub n: usize,
    /// Convergence threshold for power iteration.
    pub convergence_tol: f64,
    /// Maximum power iterations.
    pub max_iterations: usize,
    /// Width of the Hopf bifurcation band (spectral radius within [1-ε, 1+ε]).
    pub hopf_band_epsilon: f64,
    /// Threshold above 1.0+ε beyond which we classify as chaotic.
    pub chaos_threshold: f64,
    /// Last computed spectral radius.
    pub last_spectral_radius: f64,
    /// Last classified regime.
    pub last_regime: DynamicalRegime,
    /// History of spectral radius values for trend detection.
    pub spectral_radius_history: Vec<f64>,
    /// History of regime classifications.
    pub regime_history: Vec<DynamicalRegime>,
}

impl HopfBifurcationMonitor {
    /// Create a new monitor for an n-dimensional system.
    pub fn new(n: usize) -> Self {
        Self {
            n,
            convergence_tol: 1e-8,
            max_iterations: 100,
            hopf_band_epsilon: 0.05,
            chaos_threshold: 0.15,
            last_spectral_radius: 0.0,
            last_regime: DynamicalRegime::Unknown,
            spectral_radius_history: Vec::new(),
            regime_history: Vec::new(),
        }
    }

    /// Estimate the spectral radius of the Jacobian using power iteration.
    ///
    /// For x(t+1) = tanh(W @ x(t)), the Jacobian at fixed point x* is:
    ///   J = diag(1 - x*²) @ W
    ///
    /// We compute |λ_max(J)| via power iteration on J.
    pub fn spectral_radius(&self, w: &Array2<f64>, state: &Array1<f64>) -> f64 {
        // Build Jacobian: J = diag(1 - x*²) @ W
        let mut jacobian = w.clone();
        for i in 0..self.n {
            let diag_val = 1.0 - state[i] * state[i];
            for j in 0..self.n {
                jacobian[(i, j)] *= diag_val;
            }
        }

        // Power iteration to estimate |λ_max|.
        let mut v = Array1::from_elem(self.n, 1.0 / (self.n as f64).sqrt());
        let mut last_lambda = 0.0_f64;

        for _ in 0..self.max_iterations {
            let wv = jacobian.dot(&v);
            let norm = wv.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-15 {
                return 0.0;
            }
            v = wv / norm;
            // Rayleigh quotient approximation.
            let jv = jacobian.dot(&v);
            let vv = v.dot(&v);
            let lambda = v.dot(&jv) / vv;
            if (lambda.abs() - last_lambda.abs()).abs() < self.convergence_tol {
                return lambda.abs();
            }
            last_lambda = lambda;
        }

        last_lambda.abs()
    }

    /// Classify the dynamical regime based on spectral radius.
    pub fn classify_regime(&self, spectral_radius: f64) -> DynamicalRegime {
        if spectral_radius < 1.0 - self.hopf_band_epsilon {
            DynamicalRegime::FixedPoint
        } else if spectral_radius <= 1.0 + self.hopf_band_epsilon {
            DynamicalRegime::LimitCycle
        } else if spectral_radius > 1.0 + self.chaos_threshold {
            DynamicalRegime::Chaotic
        } else {
            // Between Hopf band and chaos threshold — unstable but not yet chaotic.
            DynamicalRegime::LimitCycle
        }
    }

    /// Update the monitor with a new observation.
    /// Returns the detected regime and spectral radius.
    pub fn update(&mut self, w: &Array2<f64>, state: &Array1<f64>) -> (DynamicalRegime, f64) {
        let sr = self.spectral_radius(w, state);
        let regime = self.classify_regime(sr);

        self.last_spectral_radius = sr;
        self.last_regime = regime;
        self.spectral_radius_history.push(sr);
        self.regime_history.push(regime);

        // Keep history bounded.
        if self.spectral_radius_history.len() > 1000 {
            self.spectral_radius_history.remove(0);
            self.regime_history.remove(0);
        }

        (regime, sr)
    }

    /// Detect if a bifurcation just occurred (regime change between last two observations).
    pub fn bifurcation_just_occurred(&self) -> bool {
        let len = self.regime_history.len();
        if len < 2 {
            return false;
        }
        self.regime_history[len - 1] != self.regime_history[len - 2]
    }

    /// Get the trend of spectral radius over recent history.
    /// Returns +1 (increasing), -1 (decreasing), 0 (stable).
    pub fn spectral_radius_trend(&self) -> i32 {
        let len = self.spectral_radius_history.len();
        if len < 3 {
            return 0;
        }
        let recent = &self.spectral_radius_history[len - 3..];
        let diff = recent[2] - recent[0];
        if diff > self.convergence_tol {
            1
        } else if diff < -self.convergence_tol {
            -1
        } else {
            0
        }
    }

    /// Get a summary of the current state.
    pub fn summary(&self) -> BifurcationSummary {
        BifurcationSummary {
            spectral_radius: self.last_spectral_radius,
            regime: self.last_regime,
            psychoanalytic_reading: self.last_regime.psychoanalytic_reading(),
            bifurcation_just_occurred: self.bifurcation_just_occurred(),
            trend: self.spectral_radius_trend(),
            history_len: self.spectral_radius_history.len(),
        }
    }
}

/// Summary of the bifurcation monitor state.
#[derive(Clone, Debug)]
pub struct BifurcationSummary {
    pub spectral_radius: f64,
    pub regime: DynamicalRegime,
    pub psychoanalytic_reading: &'static str,
    pub bifurcation_just_occurred: bool,
    pub trend: i32,
    pub history_len: usize,
}
