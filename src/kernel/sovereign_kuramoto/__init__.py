"""
OmniMind Sovereign Kuramoto — Python Bridge with Rust Acceleration.

Clean-room implementation of:
  - SovereignKuramotoSolver (Kuramoto + hyper-Kuramoto + INRC field + psychoanalytic coupling)
  - PsychoanalyticLIF (LIF with neutrosophic dynamic threshold)
  - HopfBifurcationMonitor (spectral radius + regime classification)

Rust acceleration via PyO3 is used when available; otherwise falls back to
pure Python/NumPy implementations with identical behavior.

This is OmniMind-native code, NOT a copy of SC-NeuroCore's AGPL implementation.
The mathematical concepts (Kuramoto, LIF, Hopf) are public domain; the
extensions (hyper-Kuramoto, INRC field, neutrosophic threshold, psychoanalytic
coupling) are OmniMind-original.

License: CC-BY-NC-ND-4.0 (OmniMind Sovereign Federation)
"""

from __future__ import annotations

import math
import logging
from typing import List, Tuple, Optional, Dict, Any

import numpy as np

logger = logging.getLogger(__name__)

# ============================================================
# Rust availability check
# ============================================================

_RUST_AVAILABLE = False
_RustSolver = None
_RustLIF = None
_RustLIFBatch = None
_RustHopfMonitor = None

try:
    from omnimind_sovereign_kuramoto import (
        SovereignKuramotoSolver as _RustSolver,
        PsychoanalyticLIF as _RustLIF,
        PsychoanalyticLIFBatch as _RustLIFBatch,
        HopfBifurcationMonitor as _RustHopfMonitor,
    )
    _RUST_AVAILABLE = True
    logger.info("omnimind_sovereign_kuramoto: Rust acceleration ACTIVE")
except ImportError:
    logger.info("omnimind_sovereign_kuramoto: Rust not available, using Python fallback")


def is_rust_available() -> bool:
    """Check if the Rust acceleration module is available."""
    return _RUST_AVAILABLE


# ============================================================
# Python fallback: SovereignKuramotoSolver
# ============================================================

_TWO_PI = 2.0 * math.pi


class _PySovereignKuramoto:
    """Pure Python/NumPy fallback for SovereignKuramotoSolver."""

    def __init__(
        self,
        omega: List[float],
        coupling_flat: List[float],
        initial_phases: List[float],
        noise_amp: float = 0.0,
    ):
        self.n = len(omega)
        self.omega = np.array(omega, dtype=np.float64)
        self.coupling = np.array(coupling_flat, dtype=np.float64).reshape(self.n, self.n)
        self.phases = np.array(initial_phases, dtype=np.float64)
        self.noise_amp = noise_amp
        self.inrc_field_pressure = 0.0
        self.sigma_geo = 0.0
        self.sigma_psi = 0.0
        self.k_hyper = 0.0
        self._geo_matrix = None
        self._psi_matrix = None
        self._hyperedges: List[Tuple[int, int, int]] = []
        self._hyper_weights: List[float] = []

    def set_geometry(self, w_flat: List[float], sigma_g: float):
        self._geo_matrix = np.array(w_flat, dtype=np.float64).reshape(self.n, self.n)
        self.sigma_geo = sigma_g

    def set_psychoanalytic(self, psi_flat: List[float], sigma_psi: float):
        self._psi_matrix = np.array(psi_flat, dtype=np.float64).reshape(self.n, self.n)
        self.sigma_psi = sigma_psi

    def set_hypergraph(self, edges: List[Tuple[int, int, int]], weights: List[float], k_hyper: float):
        self._hyperedges = edges
        self._hyper_weights = weights
        self.k_hyper = k_hyper

    def set_inrc_field_pressure(self, f: float):
        self.inrc_field_pressure = f

    def step(self, dt: float, seed: int = 0) -> float:
        # Sin-difference matrix
        sin_diff = np.sin(self.phases[np.newaxis, :] - self.phases[:, np.newaxis])

        # Baseline coupling
        coupling_sum = np.sum(self.coupling * sin_diff, axis=1)

        # Geometry coupling
        geo_sum = np.zeros(self.n)
        if self._geo_matrix is not None and self.sigma_geo != 0.0:
            geo_sum = self.sigma_geo * np.sum(self._geo_matrix * sin_diff, axis=1)

        # Psychoanalytic coupling
        psi_sum = np.zeros(self.n)
        if self._psi_matrix is not None and self.sigma_psi != 0.0:
            psi_sum = self.sigma_psi * np.sum(self._psi_matrix * sin_diff, axis=1)

        # Hyper-Kuramoto coupling
        hyper_sum = np.zeros(self.n)
        if self.k_hyper != 0.0 and self._hyperedges:
            for idx, (i, j, k) in enumerate(self._hyperedges):
                w = self._hyper_weights[idx]
                phase_sum = self.phases[j] + self.phases[k] - 2.0 * self.phases[i]
                hyper_sum[i] += self.k_hyper * w * math.sin(phase_sum)

        # INRC field pressure
        inrc_sum = np.zeros(self.n)
        if self.inrc_field_pressure != 0.0:
            inrc_sum = self.inrc_field_pressure * np.cos(self.phases)

        # Noise
        noise = np.zeros(self.n)
        if seed != 0 and self.noise_amp != 0.0:
            rng = np.random.default_rng(seed)
            noise = self.noise_amp * rng.standard_normal(self.n)

        # dtheta
        dtheta = self.omega + coupling_sum + geo_sum + psi_sum + hyper_sum + inrc_sum + noise

        # Integrate (mod 2π)
        self.phases = np.mod(self.phases + dtheta * dt, _TWO_PI)

        return self.order_parameter_pair()

    def run(self, n_steps: int, dt: float, seed: int = 0) -> List[float]:
        results = []
        for i in range(n_steps):
            s = seed + i if seed != 0 else 0
            results.append(self.step(dt, s))
        return results

    def order_parameter_pair(self) -> float:
        if self.n == 0:
            return 0.0
        mean_cos = np.mean(np.cos(self.phases))
        mean_sin = np.mean(np.sin(self.phases))
        return math.sqrt(mean_cos**2 + mean_sin**2)

    def order_parameter_triad(self) -> float:
        if not self._hyperedges:
            return 0.0
        sum_cos = 0.0
        sum_sin = 0.0
        total_w = 0.0
        for idx, (i, j, k) in enumerate(self._hyperedges):
            w = self._hyper_weights[idx]
            ps = self.phases[i] + self.phases[j] + self.phases[k]
            sum_cos += w * math.cos(ps)
            sum_sin += w * math.sin(ps)
            total_w += w
        if total_w == 0.0:
            return 0.0
        return math.sqrt(sum_cos**2 + sum_sin**2) / total_w

    def get_phases(self) -> List[float]:
        return self.phases.tolist()

    def set_phases(self, phases: List[float]):
        self.phases = np.array(phases, dtype=np.float64)

    def set_coupling(self, coupling_flat: List[float]):
        self.coupling = np.array(coupling_flat, dtype=np.float64).reshape(self.n, self.n)

    def __repr__(self):
        return (
            f"SovereignKuramotoSolver(n={self.n}, "
            f"R_pair={self.order_parameter_pair():.4f}, "
            f"R_triad={self.order_parameter_triad():.4f}, "
            f"F_inrc={self.inrc_field_pressure:.3f}, "
            f"sigma_geo={self.sigma_geo:.3f}, "
            f"sigma_psi={self.sigma_psi:.3f}, "
            f"K_hyper={self.k_hyper:.3f})"
        )


# ============================================================
# Python fallback: PsychoanalyticLIF
# ============================================================


class _PyPsychoanalyticLIF:
    """Pure Python fallback for PsychoanalyticLIF."""

    def __init__(
        self,
        v_rest: float = -65.0,
        v_reset: float = -70.0,
        v_threshold: float = -50.0,
        refractory_period: int = 3,
        leak_rate: float = 0.1,
    ):
        self.v = v_rest
        self.refractory_counter = 0
        self.v_rest = v_rest
        self.v_reset_base = v_reset
        self.v_threshold_base = v_threshold
        self.refractory_period = refractory_period
        self.leak_rate = leak_rate
        self.t = 0.5
        self.i_neutro = 0.0
        self.f_neutro = 0.0

    def set_neutrosophic(self, t: float, i: float, f: float):
        self.t = max(0.0, min(1.0, t))
        self.i_neutro = max(0.0, min(1.0, i))
        self.f_neutro = max(0.0, min(1.0, f))

    def dynamic_threshold(self) -> float:
        offset = self.v_threshold_base - self.v_rest
        multiplier = max(0.3, min(2.0, 1.0 + self.i_neutro - self.t))
        return self.v_rest + offset * multiplier

    def dynamic_reset(self) -> float:
        return self.v_reset_base * (1.0 - self.f_neutro * 0.5)

    def step(self, input_current: float) -> float:
        if self.refractory_counter > 0:
            self.refractory_counter -= 1
            return 0.0
        self.v += input_current - self.leak_rate * (self.v - self.v_rest)
        threshold = self.dynamic_threshold()
        if self.v >= threshold:
            self.v = self.dynamic_reset()
            self.refractory_counter = self.refractory_period
            return 1.0
        return 0.0

    def reset(self):
        self.v = self.v_rest
        self.refractory_counter = 0


class _PyPsychoanalyticLIFBatch:
    """Pure Python fallback for PsychoanalyticLIFBatch."""

    def __init__(
        self,
        n: int,
        v_rest: float = -65.0,
        v_reset: float = -70.0,
        v_threshold: float = -50.0,
        refractory: int = 3,
        leak: float = 0.1,
    ):
        self.neurons = [_PyPsychoanalyticLIF(v_rest, v_reset, v_threshold, refractory, leak) for _ in range(n)]

    def set_neutrosophic_batch(self, triples: List[Tuple[float, float, float]]):
        for neuron, (t, i, f) in zip(self.neurons, triples):
            neuron.set_neutrosophic(t, i, f)

    def step_batch(self, inputs: List[float]) -> List[float]:
        return [neuron.step(inp) for neuron, inp in zip(self.neurons, inputs)]

    def get_potentials(self) -> List[float]:
        return [n.v for n in self.neurons]

    def get_thresholds(self) -> List[float]:
        return [n.dynamic_threshold() for n in self.neurons]

    def reset_all(self):
        for n in self.neurons:
            n.reset()

    @property
    def n(self) -> int:
        return len(self.neurons)


# ============================================================
# Python fallback: HopfBifurcationMonitor
# ============================================================


class _PyHopfBifurcationMonitor:
    """Pure Python/NumPy fallback for HopfBifurcationMonitor."""

    def __init__(self, n: int):
        self.n = n
        self.convergence_tol = 1e-8
        self.max_iterations = 100
        self.hopf_band_epsilon = 0.05
        self.chaos_threshold = 0.15
        self.last_spectral_radius = 0.0
        self.last_regime = "unknown"
        self.spectral_radius_history: List[float] = []
        self.regime_history: List[str] = []

    def _spectral_radius(self, w_flat: List[float], state: List[float]) -> float:
        w = np.array(w_flat, dtype=np.float64).reshape(self.n, self.n)
        s = np.array(state, dtype=np.float64)
        # Jacobian: J = diag(1 - x*²) @ W
        diag_vals = 1.0 - s * s
        jacobian = diag_vals[:, np.newaxis] * w
        # Power iteration
        v = np.ones(self.n) / math.sqrt(self.n)
        last_lambda = 0.0
        for _ in range(self.max_iterations):
            wv = jacobian @ v
            norm = np.linalg.norm(wv)
            if norm < 1e-15:
                return 0.0
            v = wv / norm
            jv = jacobian @ v
            vv = v @ v
            lam = (v @ jv) / vv
            if abs(abs(lam) - abs(last_lambda)) < self.convergence_tol:
                return abs(lam)
            last_lambda = lam
        return abs(last_lambda)

    def _classify(self, sr: float) -> str:
        if sr < 1.0 - self.hopf_band_epsilon:
            return "fixed_point"
        elif sr <= 1.0 + self.hopf_band_epsilon:
            return "limit_cycle"
        elif sr > 1.0 + self.chaos_threshold:
            return "chaotic"
        else:
            return "limit_cycle"

    def _psychoanalytic_reading(self, regime: str) -> str:
        return {
            "fixed_point": "neurotic_equilibrium",
            "limit_cycle": "oscillatory_affect",
            "chaotic": "neurotic_chaos",
            "unknown": "unresolved",
        }.get(regime, "unresolved")

    def update(self, w_flat: List[float], state: List[float]) -> Tuple[str, float]:
        sr = self._spectral_radius(w_flat, state)
        regime = self._classify(sr)
        self.last_spectral_radius = sr
        self.last_regime = regime
        self.spectral_radius_history.append(sr)
        self.regime_history.append(regime)
        if len(self.spectral_radius_history) > 1000:
            self.spectral_radius_history.pop(0)
            self.regime_history.pop(0)
        return regime, sr

    def spectral_radius(self, w_flat: List[float], state: List[float]) -> float:
        return self._spectral_radius(w_flat, state)

    def classify_regime(self, sr: float) -> str:
        return self._classify(sr)

    def bifurcation_just_occurred(self) -> bool:
        if len(self.regime_history) < 2:
            return False
        return self.regime_history[-1] != self.regime_history[-2]

    def spectral_radius_trend(self) -> int:
        if len(self.spectral_radius_history) < 3:
            return 0
        diff = self.spectral_radius_history[-1] - self.spectral_radius_history[-3]
        if diff > self.convergence_tol:
            return 1
        elif diff < -self.convergence_tol:
            return -1
        return 0

    def summary(self) -> Dict[str, Any]:
        return {
            "spectral_radius": self.last_spectral_radius,
            "regime": self.last_regime,
            "psychoanalytic_reading": self._psychoanalytic_reading(self.last_regime),
            "bifurcation_just_occurred": self.bifurcation_just_occurred(),
            "trend": self.spectral_radius_trend(),
            "history_len": len(self.spectral_radius_history),
        }


# ============================================================
# Public API — dispatches to Rust or Python
# ============================================================


def SovereignKuramotoSolver(
    omega: List[float],
    coupling_flat: List[float],
    initial_phases: List[float],
    noise_amp: float = 0.0,
):
    """
    Create a Sovereign Kuramoto solver.

    Uses Rust acceleration if available, otherwise falls back to Python/NumPy.

    Args:
        omega: Natural frequencies ω_k, shape (n,).
        coupling_flat: Baseline coupling matrix K_km, row-major (n*n,).
        initial_phases: Initial phase vector θ_k, shape (n,).
        noise_amp: Gaussian noise amplitude (default 0.0).

    Returns:
        SovereignKuramotoSolver instance (Rust or Python).
    """
    if _RUST_AVAILABLE:
        return _RustSolver(omega, coupling_flat, initial_phases, noise_amp)
    return _PySovereignKuramoto(omega, coupling_flat, initial_phases, noise_amp)


def PsychoanalyticLIF(
    v_rest: float = -65.0,
    v_reset: float = -70.0,
    v_threshold: float = -50.0,
    refractory_period: int = 3,
    leak_rate: float = 0.1,
):
    """
    Create a Psychoanalytic LIF neuron with neutrosophic dynamic threshold.

    Uses Rust acceleration if available, otherwise falls back to Python.
    """
    if _RUST_AVAILABLE:
        return _RustLIF(v_rest, v_reset, v_threshold, refractory_period, leak_rate)
    return _PyPsychoanalyticLIF(v_rest, v_reset, v_threshold, refractory_period, leak_rate)


def PsychoanalyticLIFBatch(
    n: int,
    v_rest: float = -65.0,
    v_reset: float = -70.0,
    v_threshold: float = -50.0,
    refractory: int = 3,
    leak: float = 0.1,
):
    """
    Create a batch of N Psychoanalytic LIF neurons.

    Uses Rust acceleration if available, otherwise falls back to Python.
    """
    if _RUST_AVAILABLE:
        return _RustLIFBatch(n, v_rest, v_reset, v_threshold, refractory, leak)
    return _PyPsychoanalyticLIFBatch(n, v_rest, v_reset, v_threshold, refractory, leak)


def HopfBifurcationMonitor(n: int):
    """
    Create a Hopf Bifurcation Monitor for an n-dimensional recurrent system.

    Uses Rust acceleration if available, otherwise falls back to Python/NumPy.
    """
    if _RUST_AVAILABLE:
        return _RustHopfMonitor(n)
    return _PyHopfBifurcationMonitor(n)


__all__ = [
    "SovereignKuramotoSolver",
    "PsychoanalyticLIF",
    "PsychoanalyticLIFBatch",
    "HopfBifurcationMonitor",
    "is_rust_available",
]

__version__ = "0.1.0"
