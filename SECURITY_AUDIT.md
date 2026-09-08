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

All 9 crates using pyo3 have been migrated to **pyo3 0.29.2**:

- `entropic_memory`: pyo3 0.29 (was 0.22)
- `expectation_rs`: pyo3 0.29 (was 0.20)
- `freud10d`: pyo3 0.29 (was 0.20)
- `kernel_compute`: pyo3 0.29 (was 0.20)
- `langue`: pyo3 0.29 (was 0.20)
- `layered_transition/engine`: pyo3 0.29 (was 0.20)
- `metrics`: pyo3 0.29 (was 0.20)
- `nsh`: pyo3 0.29 (was 0.20)
- `sovereign_kuramoto`: pyo3 0.29 (was 0.20)

All 9 crates compile successfully in both `dev` and `release` profiles.

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

All 9 crates using pyo3 have been migrated from 0.20/0.22 to **0.29.2**, which
includes the fixes for all three advisories. The migration required:

- `#[pymodule]` signatures: `m: &PyModule` → `m: &Bound<'_, PyModule>`
- `PyObject` return types → `Py<PyAny>` with `.unbind().into()`
- `&PyAny`/`&PyDict`/`&PyList` arguments → `&Bound<'_, T>`
- `PyList::new()` now returns `PyResult<Bound<PyList>>` (needs `?`)
- `PyDict::new_bound()` / `import_bound()` (0.22) → `PyDict::new()` / `import()` (0.29)
- `#[pyo3(text_signature = ...)]` on structs removed (not valid in 0.29)
- `downcast()` → `cast()` on `Bound` types
- `is_true()` → `is_truthy()`
- `into_py()` removed — use direct values or `.unbind().into()`
- `#[pyclass]` with non-Sync types (rusqlite) → `#[pyclass(unsendable)]`
- Dict nesting: populate children before `set_item` on parent (Bound moves)

## Warning for adaptors and forks

> **This is public code.** Even though the OmniMind origin code does not call the
> vulnerable functions, **you must verify your own adaptations**. If you fork,
> modify, or extend any crate and start using `PyString::from_object`,
> `PyCFunction::new_closure`, or iterate `PyList`/`PyTuple` with `.nth()` /
> `.nth_back()`, **you will be exposed to the vulnerabilities described above.**

### What to check in your adaptation

1. **`PyString::from_object`** — if you create Python strings from Python objects
   with an encoding argument, you are exposed to a buffer overflow (OOB read).
   Mitigation: upgrade to pyo3 `>=0.24.1` or use `PyString::new()` instead.

2. **`PyCFunction::new_closure`** — if you create Python closures from Rust,
   the closure may be called concurrently from multiple Python threads without
   `Sync`. Mitigation: upgrade to pyo3 `>=0.29.0` or ensure your closure is
   `Sync` manually.

3. **`BoundListIterator::nth` / `BoundTupleIterator::nth`** — if you iterate
   Python lists or tuples using `.nth(n)` with a large `n`, you may trigger an
   out-of-bounds read. Mitigation: upgrade to pyo3 `>=0.29.0` or avoid `.nth()`
   on PyO3 iterators (use indexing or `.next()` in a loop instead).

### Upgrade path

Upgrading to pyo3 0.29.x is recommended for all adaptations. This requires a
significant API migration (0.20 → 0.29 introduces the `Bound<'py, T>` smart
pointer pattern). Key migration steps:

- `&PyList` → `&Bound<'_, PyList>` (or `Bound<'_, PyList>`)
- `PyList::new(py, data)` → `PyList::new(py, data)` (API mostly compatible)
- `obj.cast()` → `obj.bind()` in some contexts
- `PyModule::new` → `Bound<'_, PyModule>` variants

See the [PyO3 migration guide](https://pyo3.rs/v0.29/migration) for details.
