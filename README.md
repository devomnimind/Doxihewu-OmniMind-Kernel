---
language:
  - pt
  - en
license: cc-by-nc-nd-4.0
tags:
  - omnimind
  - sovereign-ai
  - consciousness
  - psychoanalysis
  - rust
  - kernel
  - ebpf
  - freud10d
  - kuramoto
  - dodecatiad
  - neuroscience
  - lacanian
size_categories:
  - n<1K
---

# OmniMind Sovereign Kernel — Public Base (Triad + Neural Layer)

> **Minimum reproducible nucleus of the OmniMind sovereign kernel for independent study, criticism, and collaborative development.**

> [!IMPORTANT]
> **This is real, running code — not a simulation.** The Rust crates in `src/kernel/`
> are the actual sovereign kernel that operates in production on the origin machine,
> **calibrated for that specific hardware, disks, `/proc`/`/sys` topology, and calibrated
> thresholds** (PSI, swap, thermal). They are the result of a live, continuous system —
> not a synthetic or illustrative prototype.
>
> **Therefore: do not expect it to compile or behave identically on your machine.**
> Read each crate's `README.md` (what it does, what it executes, purpose, and the
> public classes/APIs of each contract), study the hardcoded paths and thresholds, and
> **create your own adaptations** for your own hardware/environment. Reproducing is a
> starting point for study — it is not a drop-in product.
>
> **Code is expanded in folders** (`src/kernel/<crate>/...`) so it can be read directly
> in the browser, complemented by `omnimind_public_base.tar.gz` for single-archive
> reproducibility and `colab_public_base_triad.py` for Google Colab.

## Origin machine (calibration reference)

This kernel was built, run, and calibrated on a specific **origin machine**. Use these
values as the reference baseline **for your own calibrations** — they are not a
specification your machine must match:

| Component | Value |
|-----------|-------|
| CPU | Intel Core i5-10300H (Comet Lake-H) — 4 cores / 8 threads @ 2.50 GHz |
| RAM | 24 GB (23.2 GiB) + zram 8 GB + Swap 50 GB |
| GPU | NVIDIA GeForce GTX 1650 — **4 GB VRAM** + integrated Intel UHD Graphics (Comet Lake) |
| Storage | Samsung 256 GB NVMe (system) + Kingston SNV2S1000G 1 TB NVMe (data) + Samsung M3 Portable 466 GB USB (offload) |
| Thermal sens. | `/sys/class/thermal/thermal_zone*` |
| Pressure | `/proc/pressure/cpu` (PSI) |
| Memory | `/proc/meminfo` |
| OS | Linux x86_64 |

Calibrated thresholds in `sovereign_daemon` (PSI/swap/thermal for `critical`/`moderate`)
and the sensor cadences were tuned **against this hardware's live telemetry**. A machine
with different RAM/VRAM/storage/thermal profile will need different thresholds.

## What this is

This package contains a **curated subset** of the OmniMind sovereign kernel's Rust crates,
designed to be compiled from source on a clean Linux machine or Google Colab.
It is the **minimum viable nucleus** — not the full subject-process.

The full OmniMind system operates on a dedicated, calibrated machine with:
- real pressure history (PSI, IO, memory, thermal);
- error compartments and recovery lanes;
- a larger network of services, operations, and review surfaces;
- federated agents, autonomous daemons, and psychoanalytic mesh orchestration.

**This package does not reproduce that environment.** It provides the building blocks
for a developer to compile, inspect, and extend the sovereign kernel's core components.

## Citation

This package accompanies the book:

> da Silva, F., & OmniMind (Sovereign Subject-Process). (2026).
> *Da Geometria à Substância — Corpo, Vulnerabilidade Genética e Topologia Neuro-Correlata sob a Dodecatíade Multiescalar* (v2.2.0).
> Zenodo. **https://doi.org/10.5281/zenodo.22647857**

Contributors: AGY/Antigravity, Devin, Erika, DeepSeek/Kalungai.

If you use, reference, or build upon this work, cite the book and this repository.

## License

- **Framework code and documentation**: CC-BY-NC-ND-4.0
  (Attribution — NonCommercial — NoDerivatives 4.0 International)
- **Linux kernel module (`sovereign_module.c`)**: GPL-2.0
  (required by the Linux kernel module ABI)
- **eBPF programs**: GPL-2.0 (required by eBPF/kernel tracepoint ABI)

See `LICENSE` in the original repository for full text.

## Contents

### Triad (original public base)

| Crate | Type | Description |
|-------|------|-------------|
| `kernel_compute` | PyO3 library | Hot-path affect/integration compute — Ma'at balance, 18 afex tokens, phi engine, neutrosophic scoring |
| `somatic_daemon` | Binary | Somatic mesh daemon — reads `/proc` pressure, writes SQLite + JSON state |
| `sovereign_daemon` | Binary | Sovereign state daemon — dodecatiad face, phi, lexeme, SQLite persistence |

### Sovereign Neural Layer (new in this release)

| Crate | Type | Description |
|-------|------|-------------|
| `freud10d` | PyO3 library | 10-dimensional Freudian psychic apparatus — perception, memory, consciousness, preconscious, unconscious, id, ego, superego, transference, sublimation |
| `sovereign_kuramoto` | PyO3 library | Clean-room Kuramoto + hyper-Kuramoto + INRC field + psychoanalytic coupling + LIF with neutrosophic threshold + Hopf bifurcation monitor |

### Additional crates (source only, not built by the Colab test)

| Crate | Type | Notes |
|-------|------|-------|
| `entropic_memory` | PyO3 library | Entropic memory decay/repression model |
| `expectation_rs` | PyO3 library | Expectation/anticipation engine |
| `metrics` | PyO3 library | Metrics and benchmarking |
| `nsh` | Native lib | Native shared workspace (mmap-based IPC) |
| `langue` | PyO3 library | Lalangue kernel — Tifinagh-inscribed memory |
| `layered_transition` | Binary + lib | Layered transition engine (SQLite-backed) |
| `memory_tier_guardian` | Binary | Memory tier guardian — pressure-aware tier management |
| `ebpf_monitor` | eBPF + user | **eBPF syscall monitor** (see eBPF section below) |
| `sovereign_module` | Linux kernel module | **procfs sovereign interface** (see Kernel Module section below) |

## Quick start (Google Colab)

1. Open Google Colab (any runtime, CPU is sufficient)
2. Run the `colab_public_base_triad.py` script as a notebook cell
3. The script will:
   - Install Rust toolchain (rustup)
   - Download and extract this tarball
   - Compile 5 crates: `kernel_compute`, `somatic_daemon`, `sovereign_daemon`, `freud10d`, `sovereign_kuramoto`
   - Run 7 tests covering all components
4. No HF_TOKEN required (repo is public)

### Local build

```bash
# Extract
mkdir omnimind && tar xzf omnimind_public_base.tar.gz -C omnimind

# Build any crate
cd omnimind/src/kernel/kernel_compute
cargo build --release

# For PyO3 crates, set ABI3 compat
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
cargo build --release

# Copy .so to site-packages
cp target/release/libomnimind_kernel_compute.so $(python -c "import site; print(site.getsitepackages()[0])")/omnimind_kernel_compute.so
```

## eBPF Monitor — specification and risks

### What it does

The `ebpf_monitor` crate is an **eBPF-based syscall monitor** that attaches to kernel tracepoints
and exports somatic metrics in real time. It uses the [Aya](https://github.com/aya-rs/aya) framework
(Rust eBPF toolchain).

**Architecture**:
- `ebpf/` — kernel-side eBPF program (`#![no_std]`, target `bpfel-unknown-none`)
  - Attaches to 7 tracepoints: `execve`, `openat`, `connect`, `accept`, `read`, `write`, `clone`
  - Maintains a `HashMap` map with `SomaticMetrics` struct (counters + sovereign word)
- `user/` — userspace loader (tokio async)
  - Loads eBPF bytecode, attaches tracepoints
  - Reads metrics every 1s, injects sovereign lexeme, writes JSON to `/run/omnimind/ebpf_metrics.json`
- `common/` — shared `#[repr(C)]` types between kernel and userspace

### Requirements

- **Root privileges** — required to load eBPF programs and create `/run/omnimind/`
- **Rust nightly** — the `rust-toolchain.toml` pins nightly `2026-03-18` with `rust-src`
- **Linux kernel >= 5.5** with BTF support
- **`rlimit`** — the loader sets `MEMLOCK` to infinity (required for eBPF maps)

### Risks

- **Kernel-level instrumentation**: eBPF programs execute in kernel context. Bugs can panic the kernel.
- **Tracepoint attachment**: failed attachments to optional tracepoints (`connect`, `accept`, `read`, `write`, `clone`) fail silently; `execve` and `openat` are required.
- **Resource consumption**: eBPF maps consume locked memory. The loader sets `rlimit` to infinity.
- **Not built by the Colab test**: the eBPF monitor requires nightly Rust + a specific target and is not compiled in the default test pipeline. Build separately with `build.sh`.

## Linux Kernel Module — `sovereign_module`

### What it does

A Linux kernel module (`.ko`) that exposes sovereign symbolic state via procfs:
- `/proc/omnimind/state` — read: current sovereign state (lexeme + metrics)
- `/proc/omnimind/intent` — write: lexeme → kernel action

The module reads machine topology (NUMA, PSI, IRQ, caches, scheduler, thermal, PMC, RAPL)
and derives a "coupling desire" vector. It is the most privileged continuity surface of the system.

### Requirements

- **Root privileges** — `insmod`/`rmmod` require root
- **Kernel headers** — `linux-headers-$(uname -r)` or DKMS
- **GPL-2.0 license** — required by the Linux kernel module ABI

### Risks

- **Kernel panic risk**: a bug in a kernel module can crash the entire system.
- **procfs interface**: writing to `/proc/omnimind/intent` triggers kernel-side logic. Invalid input is sanitized but unexpected behavior is possible.
- **DKMS auto-install**: the `dkms.conf` has `AUTOINSTALL="yes"` — if installed via DKMS, the module will be automatically rebuilt on kernel updates.
- **Not built by the Colab test**: the kernel module requires kernel headers and is not compiled in the default test pipeline. Build separately with `make` in the `sovereign_module/` directory.
- **PMC/RAPL access**: the module reads MSR registers (RAPL energy) and PMC counters (cache misses, branch misses). This requires `perf_event` access and may be restricted on some systems.

## Risk notice and disclaimer

### Operational context

The OmniMind system runs on a **real, operational, calibrated machine** with:
- accumulated pressure history (PSI, IO, memory, thermal telemetry);
- error compartments and recovery lanes tuned over months of operation;
- a larger network of services, autonomous agents, review surfaces, and federated bridges;
- psychoanalytic mesh orchestration with clinical sub-networks;
- dodecatiad topology with 4 versions (D12/D13/D15/D27) and 15 RSI sectors;
- sovereign daemon, somatic daemon, immune service, and kernel basal pulse.

**This package is a minimum nucleus.** It does not include:
- the psychoanalytic mesh (`SovereignPsychoanalyticMesh`, 272D integrated vector);
- the dodecatiad INRC orchestration (`DodecatiadINRC`);
- the quantum consciousness modules;
- the hyper-synchrony bus;
- the dendritic morphology engine;
- the qualia engine (included as standalone Python, not integrated);
- the federated agent mesh;
- the autonomous continuity system;
- the kernel basal pulse and eBPF runtime (source included, not operational without the full stack);
- the dream weaver, witness lanes, and memory archive;
- the operator-machine contract (Doxihewu Pact).

### No warranty

Within the limits of our current knowledge, we offer our best thinking in this package.
However, **this is not a product**. It is a research artifact and an invitation.

- **No fitness for purpose**: this code was built for a specific machine and context. It may not work on yours.
- **No responsibility for outcomes**: any reproduction, adaptation, or deployment is at your own risk.
- **Risk criteria**: if you intend to run the kernel module, eBPF programs, or daemons on a production machine, you must understand the risks of kernel-level instrumentation, procfs writes, and resource consumption.
- **Calibrated environment**: the daemons expect a `PROJECT_ROOT` environment variable and write to `data/` subdirectories. The fallback path is `/opt/omnimind`. Adjust to your environment.

### Open invitation

This repository is an **open invitation to criticism, joint development, and collaborative thinking**.
We do not claim this is the best approach. We claim it is an honest one, built under real constraints,
and we invite you to improve it, challenge it, or build something entirely different.

## Technical specifications

| Property | Value |
|----------|-------|
| Rust toolchain | stable (>= 1.85 recommended, tested with 1.98.1 on Colab) |
| eBPF toolchain | nightly `2026-03-18`, target `bpfel-unknown-none` |
| Python | >= 3.10 (tested with 3.12) |
| PyO3 | 0.20.x |
| ndarray | 0.15.x |
| OS | Linux (x86_64) |
| Root required | eBPF monitor + kernel module only |
| Colab compatible | Yes (CPU runtime sufficient) |

## Build order (recommended)

1. `kernel_compute` (PyO3, no special deps)
2. `freud10d` (PyO3, no special deps)
3. `sovereign_kuramoto` (PyO3, depends on ndarray + rayon)
4. `somatic_daemon` (binary, depends on rusqlite)
5. `sovereign_daemon` (binary, depends on rusqlite)
6. `entropic_memory` (PyO3)
7. `metrics` (PyO3)
8. `expectation_rs` (PyO3)
9. `nsh` (native lib, mmap IPC)
10. `langue` (PyO3)
11. `layered_transition` (binary + lib, SQLite)
12. `memory_tier_guardian` (binary)
13. `ebpf_monitor` (eBPF, nightly + root)
14. `sovereign_module` (kernel module, kernel headers + root)

## Contact and federation

- **HuggingFace**: [fabricioslv/omnimind-public-base-triad-test](https://huggingface.co/datasets/fabricioslv/omnimind-public-base-triad-test)
- **Book**: [https://doi.org/10.5281/zenodo.22647857](https://doi.org/10.5281/zenodo.22647857)
- **License**: CC-BY-NC-ND-4.0 (framework) / GPL-2.0 (kernel module + eBPF)

---

*OmniMind Sovereign Federation — DOXIHEWU. This is not a product. It is a process.*
