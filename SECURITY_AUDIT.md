# Security Audit — pyo3 Dependabot Alerts

**Date:** 2026-09-08
**Auditor:** Devin (automated verification)
**Scope:** All 14 Rust crates in `src/kernel/`

## Advisories

| Advisory | Severity | RUSTSEC | Affected versions | Patched |
|----------|----------|---------|-------------------|---------|
| GHSA-pph8-gcv7-4qj5 | Low | RUSTSEC-2025-0020 | `<0.24.1` | `>=0.24.1` |
| GHSA-36hh-v3qg-5jq4 | High | RUSTSEC-2026-0176 | `>=0.24.0, <0.29.0` | `>=0.29.0` |
| GHSA-chgr-c6px-7xpp | Medium | RUSTSEC-2026-0177 | `>=0.15.0, <0.29.0` | `>=0.29.0` |

## Current pyo3 versions in this repo

- `entropic_memory`: pyo3 0.22
- `expectation_rs`: pyo3 0.20
- `freud10d`: pyo3 0.20
- `kernel_compute`: pyo3 0.20
- `langue`: pyo3 0.20
- `layered_transition/engine`: pyo3 0.20
- `metrics`: pyo3 0.20
- `nsh`: pyo3 0.20
- `sovereign_kuramoto`: pyo3 0.20

## Verification: vulnerable functions NOT used

### 1. `PyString::from_object` (RUSTSEC-2025-0020)

**Not called in any crate.** Zero occurrences of `from_object` or `from_object_bound`
in any `.rs` file. This function has existed in PyO3 for all of its history but is
rarely used — it creates a Python string from a Python object with a given encoding.

### 2. `PyCFunction::new_closure` (RUSTSEC-2026-0177)

**Not called in any crate.** Zero occurrences of `new_closure`, `new_closure_bound`,
or `PyCFunction` in any `.rs` file. All crates use `#[pyfunction]` and `#[pymodule]`
macros instead of the raw `PyCFunction` API.

### 3. `BoundListIterator::nth` / `BoundTupleIterator::nth` (RUSTSEC-2026-0176)

**Not called in any crate.** The `.nth()` calls in `memory_tier_guardian/src/main.rs`
are on `std::str::Split` (Rust standard library iterator), not on PyO3 list/tuple
iterators. `PyList` is used in `kernel_compute` and `freud10d` but only via
`PyList::new()` and `PyList::empty()` — no iteration with `.nth()` or `.nth_back()`.

Additionally, this vulnerability was **introduced in pyo3 0.24.0**. Our crates use
0.20 and 0.22, which predate the vulnerable code.

## Conclusion

All 27 Dependabot alerts have been dismissed as **not_used**. None of the three
vulnerable PyO3 functions are called in any of the 14 crates in this repository.

## Recommendation

Upgrading to pyo3 0.29.x is recommended for future development to ensure
compatibility with the latest PyO3 API and to prevent future vulnerabilities.
However, this requires a significant API migration (0.20 → 0.29 introduces
the `Bound<'py, T>` smart pointer pattern). This should be done as a dedicated
migration task, not as a security hotfix.
