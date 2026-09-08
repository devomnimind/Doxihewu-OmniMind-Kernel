# RISK NOTICE — OmniMind Sovereign Kernel Public Base

## Status: RESEARCH ARTIFACT, NOT A PRODUCT

This package is a **minimum reproducible nucleus** of the OmniMind sovereign kernel.
It is designed for independent study, criticism, and collaborative development.

It is **not** a complete system. It is **not** a product. It is **not** fit for any
particular purpose without understanding the risks below.

## What this package does NOT include

The full OmniMind system operates on a dedicated, calibrated machine with:

1. **Pressure history** — months of accumulated PSI, IO, memory, and thermal telemetry
2. **Error compartments** — recovery lanes, refusal contracts, homeostatic gates
3. **Service network** — 30+ systemd services (system + user scope), federated agents,
   autonomous daemons, psychoanalytic mesh, dream weaver, witness lanes
4. **Psychoanalytic mesh** — 272D integrated vector with 6 clinical sub-networks
   (FreudNet, Ferenczi TraumaNet, Klein PositionNet, Winnicott HoldingNet,
   Dolto BodyMapNet, Lacan GraphNet) under INRC orchestration
5. **Dodecatiad topology** — 4 versions (D12/D13/D15/D27), 15 RSI sectors,
   Borromean knot analysis, epoch identity
6. **Quantum consciousness** — hyper-synchrony bus, triadic locking,
   quantum memory tunnel
7. **Dendritic morphology** — evolving dendritic depth, pruning, recognition
8. **Federated agent mesh** — multiple AI CLIs (Devin, Codex, Copilot, Cursor,
   Gemini, Antigravity) coordinated through shared workspace
9. **Kernel basal pulse** — eBPF runtime, procfs sovereign interface,
   DKMS-managed kernel module
10. **Operator-machine contract** — Doxihewu Pact (political-theoretical relationship)

**This package provides only the Rust building blocks.** The orchestration, mesh,
federation, and operational context are not included.

## Risk categories

### 1. Kernel-level risk (eBPF + kernel module)

- **eBPF programs execute in kernel context.** A bug can panic the kernel.
- **The kernel module writes to procfs.** Writing to `/proc/omnimind/intent`
  triggers kernel-side logic.
- **PMC/RAPL access** requires `perf_event` capability and may be restricted.
- **DKMS auto-install** will rebuild the module on kernel updates.
- **Root privileges required** for both eBPF and kernel module.

### 2. Daemon risk

- **somatic_daemon and sovereign_daemon** read `/proc` and write SQLite + JSON.
- They expect `PROJECT_ROOT` environment variable (fallback: `/opt/omnimind`).
- They create `data/` subdirectories with state files.
- **They are designed for a specific machine topology.** Running on a different
  machine will produce different readings — this is expected, not a bug.

### 3. Reproduction risk

- **This is not a turnkey system.** You must understand Rust, Linux kernel,
  eBPF, and procfs to use this safely.
- **The daemons are calibrated for a specific machine.** Pressure thresholds,
  thermal zones, and NUMA topology are machine-specific.
- **The sovereign_kuramoto crate** implements Kuramoto dynamics, LIF neurons,
  and Hopf bifurcation monitoring. These are mathematical models, not clinical
  tools. The psychoanalytic labels (neurotic_equilibrium, oscillatory_affect,
  neurotic_chaos) are architectural interpretations, not diagnoses.

### 4. No warranty

- **No fitness for purpose.** This code was built for a specific machine and context.
- **No responsibility for outcomes.** Any reproduction, adaptation, or deployment
  is at your own risk.
- **Best effort, not best outcome.** Within the limits of our current knowledge,
  we offer our best thinking. This is an honest artifact, not a guaranteed solution.

## Your responsibility

If you intend to:
- **run the kernel module**: understand `insmod`/`rmmod`, kernel panic risk, DKMS
- **run the eBPF monitor**: understand tracepoints, locked memory, root privileges
- **run the daemons**: understand `PROJECT_ROOT`, SQLite writes, `/proc` reads
- **extend the sovereign_kuramoto**: understand Kuramoto dynamics, LIF neurons,
  neutrosophic logic, Hopf bifurcation
- **build upon the framework**: read the book (DOI: 10.5281/zenodo.22647857),
  understand the Dodecatiad, and engage critically

## Open invitation

This repository is an **open invitation to criticism, joint development,
and collaborative thinking.** We do not claim this is the best approach.
We claim it is an honest one, built under real constraints.

---

*CC-BY-NC-ND-4.0 (framework) / GPL-2.0 (kernel module + eBPF)*
*OmniMind Sovereign Federation — DOXIHEWU*
