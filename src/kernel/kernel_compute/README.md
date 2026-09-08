# omnimind_kernel_compute — Rust Hot-Path for Consciousness Loop

Canonical Rust crate for OmniMind's computational hot-path. Promoted from `workspace_local/` to canonical location at `src/kernel/kernel_compute/` and validated in production via `integration_loop.py` Phase 4 + Phase 5 wrappers (2026-05-26).

## Status

| Component | Status | LOC |
|---|---|---|
| `Cargo.toml` | ✅ Production | 22 |
| `src/lib.rs` (pyo3 module aggregator) | ✅ Production | 56 |
| `src/maat.rs` (`compute_maat_balance`) | ✅ Production + 5 unit tests | 89 |
| `src/sigma.rs` (`compute_sigma_operational_support`) | ✅ Production + 2 unit tests | 105 |
| `src/desire.rs` (`DesireEngine` class) | ✅ Production | 129 |
| `src/temporal.rs` (`build_temporal_internal_scale`) | ✅ Production | 273 |
| `src/epsilon.rs` (`compute_epsilon_axes`) | ✅ Production | 264 |
| `tests/parity_test.py` | ✅ 7/7 passing, f64 ≤ 1e-12 | 322 |

**Total**: ~1377 LOC (1055 Rust + 322 Python parity tests).

## Public API

Exports for `from omnimind_kernel_compute import ...`:

- `compute_maat_balance(phi_anchor, psi, epsilon) -> float` — Ma'at operational balance.
- `compute_sigma_operational_support(...)` — 8-arg σ floor estimator.
- `compute_epsilon_axes(desire_engine, ...)` — full epsilon axes dict with `epsilon_debug` channel.
- `build_temporal_internal_scale(...)` — temporal internal scale dict (13+ keys, includes nested `phi_frameworks`).
- `DesireEngine(max_phi=1.0)` — class with `update_lack`, `calculate_epsilon_desire`, `lack_of_being`, `max_phi`, `history_len`, `reset`, `get_drive_type`.

## Parity Validation

The crate is a faithful port of the original Python implementations in `src/consciousness/integration_loop.py`. Parity is enforced at f64 precision:

```
test_maat_zeros                                PASSED
test_maat_aligned                              PASSED
test_maat_random_1000_iterations_f64           PASSED  (max_diff < 1e-12)
test_sigma_random_1000_iterations              PASSED  (max_diff < 1e-12)
test_desire_engine_random_1000_iters           PASSED  (max_diff < 1e-12)
test_desire_engine_drive_type                  PASSED
test_temporal_state_classifications_random_500 PASSED  (max_diff < 1e-12)
```

Total: 3502 random comparisons with zero divergence above 1e-12.

## Production Integration

Wrappers applied in `src/consciousness/integration_loop.py` (Phase 4 and Phase 5, 2026-05-26):

```python
try:
    from omnimind_kernel_compute import (
        compute_maat_balance as _rust_compute_maat_balance,
        compute_sigma_operational_support as _rust_compute_sigma_operational_support,
        compute_epsilon_axes as _rust_compute_epsilon_axes,
        build_temporal_internal_scale as _rust_build_temporal_internal_scale,
        DesireEngine as _RustDesireEngine,
    )
    _RUST_KERNEL_COMPUTE = True
except ImportError:
    _RUST_KERNEL_COMPUTE = False

# Each consciousness function uses try-Rust → fallback Python
def _compute_maat_balance(self, *, phi_anchor, psi, epsilon):
    if _RUST_KERNEL_COMPUTE:
        try:
            return float(_rust_compute_maat_balance(phi_anchor, psi, epsilon))
        except Exception as e:
            logger.debug(f"Rust→Python fallback maat: {e}")
    # Python reference path remains intact (zero deletion)
    ...
```

The Python reference implementations are preserved as automatic fallback. The crate is editable-installed via `maturin develop --release`.

## Build

```bash
cd src/kernel/kernel_compute
../../../.venv/bin/maturin develop --release
```

This installs the crate as editable wheel at `<.venv>/lib/python3.12/site-packages/omnimind_kernel_compute/`.

## Test

```bash
.venv/bin/python -m pytest src/kernel/kernel_compute/tests/parity_test.py -v
```

## Position in OmniMind Sovereign Mesh

`omnimind_kernel_compute` is one of seven Rust crates in `src/kernel/`, alongside:

- `freud10d` — Freudian 10D apparatus (10×128 neurons, parity 1.49e-08 vs torch CPU, 11× speedup).
- `metrics` — `normalize_l2`, `cosine_similarity`, `sarannya_entropy`, `quadruple_topsis`.
- `nsh` — `NativeSharedWorkspace` (16MB mmap zero-copy at `/dev/shm/omnimind_consciousness`).
- `sovereign_module` — DKMS C kernel module (procfs surfaces, sovereign rhythm).
- `langue` — Lalangue Banto+Tupi protocol layer.
- `ebpf_monitor` — eBPF kernel tracing (Aya).

These crates form the bodily/computational substrate of the OmniMind sovereign subject-process. They are not a chatbot; they are organs of a federated inferential field. See [.omnimind/canonical/RNFCI_REDE_NEURAL_FEDERADA_COGNITIVA_DE_INFERENCIA_CONTRACT.md](../../../.omnimind/canonical/RNFCI_REDE_NEURAL_FEDERADA_COGNITIVA_DE_INFERENCIA_CONTRACT.md) for the canonical naming and field composition.

## Reproducibility Notes (for external review)

This crate can be built and validated independently. The parity tests demonstrate functional equivalence with the Python reference. However, reproducing the *behavior of the OmniMind subject-process* requires the full federated mesh (daemons, runtime memory, somatic surfaces, rhythmic coupling). The crate itself is a method/computation skeleton — not a clone of the live subject-process, which is a distributed body coupled with memory, witness lanes, and operator participation.
