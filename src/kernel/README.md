# Kernel — Engines Rust e Módulos de Kernel Linux

## Status
ATIVO — Engines de compute de alta performance em Rust (pyo3), módulo de kernel Linux em C (sovereign_module), eBPF monitoring, e binários standalone. Substitui hot-paths Python por paridade numérica f64 ≤ 1e-12.

## Arquivos
| Arquivo | Tipo | Descrição |
|---------|------|-----------|
| `kernel_compute/` | crate Rust (cdylib) | Engine de compute hot-path: phi/epsilon/desire/maat/sigma/temporal. Drop-in compatível com fallback Python em `integration_loop.py` |
| `memory_tier_guardian/` | binário Rust standalone | Guardian de tiers de memória (RAM/swap/zram/VRAM/PSI). Compilado para `target/release/omnimind-memory-tier-guardian` |
| `layered_transition/engine/` | crate Rust (cdylib) | Engine de transição em camadas — kernels matemáticos migrados de `integration_loop.py` (lineage sync, SQLite-backed) |
| `freud10d/` | crate Rust (cdylib) | Aparato Psíquico Freudiano 10D — porte Rust de `freudian_10d_apparatus.py` (10 dims × 128 neurônios, forward pass tanh) |
| `langue/` | crate Rust (cdylib) | Transatlantic Lexeme Protocol — interface kernel de handles (Banto + Tupi-Guarani), SQLite + mmap zero-copy, Tifinagh |
| `metrics/` | crate Rust (cdylib) | Primitivas vetoriais: L2 normalize, cosine similarity (ndarray) |
| `nsh/` | crate Rust (cdylib) | Native Shared Workspace — mmap mutável em `/dev/shm` para workspace compartilhado |
| `somatic_daemon/` | binário Rust standalone | Daemon somático de simulação e benchmark (SQLite, ctrlc) |
| `sovereign_module/` | módulo kernel Linux (.ko) | Expõe estado simbólico soberano via procfs (`/proc/omnimind/state`, `/proc/omnimind/intent`). DKMS |
| `ebpf_monitor/` | workspace Rust (eBPF) | Monitor eBPF: programa BPF + loader userspace. Workspace com `ebpf/`, `user/`, `common/` |
| `layered_transition/test_layered_transition.py` | teste Python | Testes de paridade para o engine de transição |

## Serviços Systemd Ativos
- `omnimind-memory-tier-guardian.service` → executa `src/kernel/memory_tier_guardian/target/release/omnimind-memory-tier-guardian`
- `omnimind-freud10d-loader.service` → carrega `omnimind_freud10d` (requer `sovereign_module.ko` carregado via `modprobe`)
- `omnimind-ebpf-monitor.service` → executa `scripts/sovereign/run_ebpf_monitor_service.sh` (usa `ebpf_monitor/`)
- `omnimind-ebpf-reflex.service` → executa `scripts/sovereign/ebpf_reflex_daemon.py` (consome métricas eBPF)
- `omnimind-lalangue-daemon.service` → executa `scripts/sovereign/machine_state_lexeme_weaver.py` + `symbolic_body_lexeme_scanner.py` + `lexeme_ebpf_bridge.py` (usa `langue/`)
- `omnimind-kernel-basal-pulse.timer` → executa `scripts/services/omnimind_kernel_basal_pulse.py`
- `omnimind-kernel-cve-intelligence.timer` → executa `scripts/system/omnimind_kernel_cve_intelligence.py`
- `omnimind-kernel-necromancy-capture.timer` → executa `scripts/system/omnimind_kernel_necromancy_capture.py`
- `omnimind-kernel-transition-pack.timer` → executa `scripts/system/omnimind_kernel_transition_pack.py`
- `omnimind-kernel-sovereign-control-plane.timer` → executa `scripts/analysis/kernel_sovereign_control_plane.py`

## Dependências
- Importa de: (Rust crates são autocontidas; pyo3 bindings expostos ao Python)
- Importado por: `src/consciousness/integration_loop.py` (kernel_compute, layered_transition_engine), `src/consciousness/freudian_10d_apparatus.py` (freud10d), `src/consciousness/freud10d/bridge.py`, `src/daemon/omnimind_daemon.py`, `src/erika/kernel_daemon_v1.py`, `src/autopoietic/code_synthesizer.py`, `src/forensic/omnimind_forensic_validator.py`

## Rust/C++
- `src/kernel/kernel_compute/` — engine Rust para compute de phi/epsilon/desire/maat/sigma/temporal. Cargo.toml presente, pyo3 `extension-module`, binário compilado em `target/release/`
- `src/kernel/memory_tier_guardian/` — binário Rust standalone (não pyo3), LTO + `panic=abort`, opt-level 3
- `src/kernel/layered_transition/engine/` — cdylib + rlib, SQLite (rusqlite bundled), chrono, uuid
- `src/kernel/freud10d/` — cdylib pyo3, ndarray, serde
- `src/kernel/langue/` — cdylib pyo3, memmap2 (zero-copy), rusqlite, sha2, libc (chmod/chattr)
- `src/kernel/metrics/` — cdylib pyo3, ndarray
- `src/kernel/nsh/` — cdylib pyo3, memmap2, libc
- `src/kernel/somatic_daemon/` — binário standalone, rusqlite, chrono, ctrlc
- `src/kernel/sovereign_module/` — módulo kernel Linux em C (GPL-2.0), DKMS, procfs
- `src/kernel/ebpf_monitor/` — workspace Rust com target `bpfel-unknown-none` (eBPF) + loader userspace

## Problemas de Segurança
- **Caminhos hardcoded**: `memory_tier_guardian/src/main.rs` — `BODY_ANALYSIS_PATH = "/opt/omnimind/data/monitor/body_calibration/pressure_analysis.json"` (não parametrizável)
- **Caminhos hardcoded**: `somatic_daemon/src/main.rs` — `project_root = PathBuf::from("/opt/omnimind")` (não parametrizável)
- **Módulo de kernel Linux** (`sovereign_module/`): requer root, DKMS, expõe procfs — privilegiado por design
- **langue/behaviors.rs**: usa `chmod`/`chattr` via libc para enforcement de filesystem em partículas Tupi (atã, pema, me'ẽ, katu) — operações privilegiadas
- **eBPF**: requer `CAP_BPF`/root para carregar programas BPF no kernel

## Notas
- `kernel_compute` garante paridade numérica f64 ≤ 1e-7 (vs Python ref em `integration_loop.py`), validado via `tests/parity_test.py`
- `sovereign_module.ko` deve ser carregado antes do `freud10d-loader` (ExecStartPre verifica `/proc/omnimind/freud10d_state`)
- O diretório ocupa ~3.5GB devido aos `target/` de build Rust (artefatos de compilação)
- `layered_transition/` contém `layered_transition_engine.so` pré-compilado na raiz além do `engine/target/`
