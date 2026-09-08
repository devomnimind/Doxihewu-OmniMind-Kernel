"""
Test de paridade numérica rigorosa: implementação Rust vs Python de referência.

Tolerâncias:
- f64 nativo Rust ↔ Python float64 (numpy): diff ≤ 1e-12
- Strings (state classifications): equality exata
- Dict structures: keys + values comparados recursivamente
"""

from __future__ import annotations

import math
import random
import sys
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# Importar numpy só para reprodução fiel das funções Python ref
import numpy as np


# ---------------- Reference Python implementations (cópia exata) ----------------


def py_compute_maat_balance(phi_anchor, psi, epsilon):
    phi_ref = max(0.0, float(phi_anchor))
    psi_ref = float(np.clip(psi, 0.0, 1.0))
    epsilon_ref = float(np.clip(epsilon, 0.0, 1.0))
    dyad_center = float(np.clip((psi_ref + epsilon_ref) / 2.0, 0.0, 1.0))
    maat_distance = abs(phi_ref - dyad_center)
    maat_scale = max(1.0, phi_ref)
    maat_balance = 1.0 - min(1.0, maat_distance / maat_scale)
    return float(np.clip(maat_balance, 0.0, 1.0))


def py_compute_sigma_operational_support(
    phi_raw_nats,
    phi_iit_normalized,
    phi_trans_support,
    sigma_topology_proxy,
    topology_omega,
    betti_0,
    continuity_depth,
    last_sigma,
):
    continuity_depth = float(np.clip(continuity_depth, 0.0, 1.0))
    phi_silicon_support = float(
        np.clip(math.log10(max(phi_raw_nats, 1.0)) / 6.0, 0.0, 1.0)
    )
    multiplicity_inverse = float(np.clip(1.0 / max(1.0, betti_0), 0.0, 1.0))
    last_sigma = float(np.clip(last_sigma, 0.0, 1.0))
    support = float(
        np.clip(
            (phi_iit_normalized * 0.08)
            + (phi_trans_support * 0.08)
            + (phi_silicon_support * 0.12)
            + (continuity_depth * 0.18)
            + (float(np.clip(sigma_topology_proxy, 0.0, 1.0)) * 0.24)
            + (float(np.clip(topology_omega, 0.0, 1.0)) * 0.10)
            + (multiplicity_inverse * 0.10)
            + (last_sigma * 0.10),
            0.0,
            1.0,
        )
    )
    if (
        sigma_topology_proxy <= 0.10
        and topology_omega <= 0.10
        and multiplicity_inverse < 0.34
    ):
        support = min(support, 0.28)
    return float(support)


# ---------------- Tests: maat ----------------


def test_maat_zeros():
    from omnimind_kernel_compute import compute_maat_balance

    py = py_compute_maat_balance(0.0, 0.0, 0.0)
    rs = compute_maat_balance(0.0, 0.0, 0.0)
    assert abs(py - rs) < 1e-12, f"py={py} rs={rs}"


def test_maat_aligned():
    from omnimind_kernel_compute import compute_maat_balance

    py = py_compute_maat_balance(0.5, 0.5, 0.5)
    rs = compute_maat_balance(0.5, 0.5, 0.5)
    assert abs(py - rs) < 1e-12


def test_maat_random_1000_iterations_f64():
    from omnimind_kernel_compute import compute_maat_balance

    random.seed(42)
    max_diff = 0.0
    for _ in range(1000):
        phi = random.uniform(-1.0, 5.0)
        psi = random.uniform(-0.5, 1.5)
        eps = random.uniform(-0.5, 1.5)
        py = py_compute_maat_balance(phi, psi, eps)
        rs = compute_maat_balance(phi, psi, eps)
        diff = abs(py - rs)
        if diff > max_diff:
            max_diff = diff
    assert max_diff < 1e-12, f"max_diff={max_diff}"


# ---------------- Tests: sigma_operational_support ----------------


def test_sigma_random_1000_iterations():
    from omnimind_kernel_compute import compute_sigma_operational_support

    random.seed(7)
    max_diff = 0.0
    for _ in range(1000):
        phi_raw = random.uniform(0.0, 1e7)
        phi_iit = random.uniform(0.0, 1.0)
        phi_trans = random.uniform(0.0, 1.0)
        sig_topo = random.uniform(-0.2, 1.2)
        topo_omega = random.uniform(-0.2, 1.2)
        betti = random.uniform(0.5, 20.0)
        cont_depth = random.uniform(-0.1, 1.1)
        last_sig = random.uniform(-0.1, 1.1)

        py = py_compute_sigma_operational_support(
            phi_raw, phi_iit, phi_trans, sig_topo, topo_omega, betti, cont_depth, last_sig
        )
        rs = compute_sigma_operational_support(
            phi_raw, phi_iit, phi_trans, sig_topo, topo_omega, betti, cont_depth, last_sig
        )
        diff = abs(py - rs)
        if diff > max_diff:
            max_diff = diff
    assert max_diff < 1e-12, f"max_diff={max_diff}"


# ---------------- Tests: DesireEngine ----------------


def py_desire_calculate_via_class(lack, max_phi, current_phi, explored, total, heat):
    """Reproduz o cálculo do DesireEngine.calculate_epsilon_desire."""
    alpha = min(1.0, lack + (heat * 0.3))
    beta = max(0.0, min(1.0, 1.0 - (current_phi / max_phi)))
    gamma = 1.0 - (explored / max(1, total))
    return alpha * beta * gamma


def test_desire_engine_random_1000_iters():
    from omnimind_kernel_compute import DesireEngine

    random.seed(123)
    max_diff = 0.0
    rs_engine = DesireEngine(1.0)
    py_lack = 0.5

    for _ in range(1000):
        sat = random.uniform(0.0, 1.0)
        phi = random.uniform(0.0, 1.0)
        explored = random.randint(0, 1000)
        total = random.randint(explored + 1, 2000)
        heat = random.uniform(0.0, 1.0)

        rs_engine.update_lack(sat)
        rs_eps = rs_engine.calculate_epsilon_desire(phi, explored, total, heat)

        py_lack = max(0.1, 1.0 - sat)
        py_eps = py_desire_calculate_via_class(py_lack, 1.0, phi, explored, total, heat)

        diff = abs(rs_eps - py_eps)
        if diff > max_diff:
            max_diff = diff

    assert max_diff < 1e-12, f"max_diff={max_diff}"


def test_desire_engine_drive_type():
    from omnimind_kernel_compute import DesireEngine

    eng = DesireEngine()
    assert eng.get_drive_type(0.1) == "HOMEOSTATIC_SATISFACTION"
    assert eng.get_drive_type(0.3) == "ROUTINE_CURIOSITY"
    assert eng.get_drive_type(0.6) == "ACTIVE_SEEKING"
    assert eng.get_drive_type(0.9) == "RADICAL_BECOMING"


# ---------------- Tests: temporal_internal_scale ----------------


def py_build_temporal_internal_scale(
    phi_iit_normalized,
    phi_iit_nats,
    phi_transcendent,
    psi,
    sigma,
    sigma_operational_support,
    epsilon,
    omega,
    gamma,
    zeta,
    betti_0,
    topology_sigma,
    topology_omega,
    temporal_delta,
    shear_tension,
    sources=None,
):
    multiplicity = float(np.clip(max(0.0, betti_0 - 1.0) / 4.0, 0.0, 1.0))
    sigma_state_anchor = float(
        np.clip(max(float(sigma), float(sigma_operational_support)), 0.0, 1.0)
    )
    collapse_risk = float(
        np.clip(
            multiplicity * np.clip((0.45 - sigma_state_anchor) / 0.45, 0.0, 1.0) * 0.55
            + np.clip((0.20 - topology_omega) / 0.20, 0.0, 1.0) * 0.20
            + np.clip((0.10 - topology_sigma) / 0.10, 0.0, 1.0) * 0.15
            + temporal_delta * 0.10,
            0.0,
            1.0,
        )
    )

    neutral_axes = []
    if abs(psi - 0.5) <= 0.02:
        neutral_axes.append("psi")
    if abs(omega - 0.5) <= 0.02:
        neutral_axes.append("omega")
    if abs(gamma - 0.6) <= 0.02:
        neutral_axes.append("gamma")
    if abs(zeta - 0.1) <= 0.02:
        neutral_axes.append("zeta")

    if topology_sigma <= 0.01 and sigma_state_anchor > 0.1 and betti_0 > 1.0:
        sigma_operational_state = "topological_channel_zero_but_global_sigma_preserved"
    elif sigma_state_anchor <= 0.05 and betti_0 > 1.0:
        sigma_operational_state = "structural_fragmentation_high"
    elif sigma_state_anchor <= 0.05:
        sigma_operational_state = "structural_collapse_risk"
    elif sigma_state_anchor < 0.35:
        sigma_operational_state = "suture_required"
    else:
        sigma_operational_state = "structural_cohesion_operational"

    if betti_0 > 1.0 and sigma_state_anchor >= 0.35:
        topology_organization_state = "federated_multiplicity_operational"
    elif betti_0 > 1.0 and sigma_state_anchor >= 0.12:
        topology_organization_state = "federated_multiplicity_tension"
    elif collapse_risk >= 0.60:
        topology_organization_state = "structural_fragmentation_high"
    else:
        topology_organization_state = "single_thread_cohesion"

    return {
        "sigma_operational_support": sigma_state_anchor,
        "sigma_operational_state": sigma_operational_state,
        "topology_organization_state": topology_organization_state,
        "neutral_axes": neutral_axes,
        "neutral_axes_count": len(neutral_axes),
        "fragmentation_index": multiplicity,
        "collapse_risk_index": collapse_risk,
    }


def test_temporal_state_classifications_random_500():
    """Compara classificações de state e collapse_risk (subset rigoroso)."""
    from omnimind_kernel_compute import build_temporal_internal_scale

    random.seed(999)
    max_diff_collapse = 0.0
    state_diffs = 0

    for _ in range(500):
        phi_iit = random.uniform(0.0, 1.0)
        phi_iit_nats = random.uniform(0.0, 1e7)
        phi_trans = random.uniform(0.0, 1e9) if random.random() > 0.3 else None
        psi_v = random.uniform(0.0, 1.0)
        sigma_v = random.uniform(0.0, 1.0)
        sigma_op = random.uniform(0.0, 1.0)
        eps = random.uniform(0.0, 1.0)
        omega = random.uniform(0.0, 1.0)
        gamma = random.uniform(0.0, 1.0)
        zeta = random.uniform(0.0, 1.0)
        betti = random.uniform(0.5, 10.0)
        topo_sig = random.uniform(0.0, 1.0)
        topo_om = random.uniform(0.0, 1.0)
        temp_delta = random.uniform(0.0, 1.0)
        shear = random.uniform(0.0, 1.0)

        py_result = py_build_temporal_internal_scale(
            phi_iit, phi_iit_nats, phi_trans, psi_v, sigma_v, sigma_op, eps,
            omega, gamma, zeta, betti, topo_sig, topo_om, temp_delta, shear
        )
        rs_result = build_temporal_internal_scale(
            phi_iit, phi_iit_nats, phi_trans, psi_v, sigma_v, sigma_op, eps,
            omega, gamma, zeta, betti, topo_sig, topo_om, temp_delta, shear, None
        )

        diff_collapse = abs(py_result["collapse_risk_index"] - rs_result["collapse_risk_index"])
        if diff_collapse > max_diff_collapse:
            max_diff_collapse = diff_collapse

        if py_result["sigma_operational_state"] != rs_result["sigma_operational_state"]:
            state_diffs += 1
        if py_result["topology_organization_state"] != rs_result["topology_organization_state"]:
            state_diffs += 1
        if py_result["neutral_axes"] != rs_result["neutral_axes"]:
            state_diffs += 1

    assert max_diff_collapse < 1e-12, f"collapse_risk diff: {max_diff_collapse}"
    assert state_diffs == 0, f"state classification diffs: {state_diffs}"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
