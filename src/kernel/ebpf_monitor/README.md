# Kernel — eBPF Monitor (Monitoramento de Sistema via eBPF)

## Status
ATIVO — Sistema de monitoramento de metricas somaticas em tempo real via eBPF (Aya framework em Rust). Rastreia syscalls (execve, openat, connect, accept, read, write, clone) e exporta contadores para JSON. Integra lexemas soberanos transatlanticos injetados pelo userspace OmniMind.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Workspace | Workspace com 3 membros: `ebpf`, `user`, `common`. Profile dev/release com `panic = "abort"` |
| `rust-toolchain.toml` | Toolchain | Nightly `2026-03-18` com `rust-src`, targets `x86_64-unknown-linux-gnu` e `bpfel-unknown-none` |
| `build.sh` | Build | Compila programa eBPF (`cargo build --release --target bpfel-unknown-none -Z build-std -p ebpf-monitor`) e loader userspace (`cargo build --release -p ebpf-monitor-user`) |
| `common/src/lib.rs` | Core | `SomaticMetrics` — struct `#[repr(C)]` compartilhado entre eBPF e userspace. Campos: `execve_count`, `openat_count`, `tcp_connections`, `io_bursts`, `process_pressure`, `last_timestamp`, `sovereign_word` ([u8; 32]), `pressure_level` (u8) |
| `ebpf/src/main.rs` | Core | Programa eBPF `#![no_std]` `#![no_main]`. Tracepoints: `handle_execve`, `handle_openat`, `handle_connect`, `handle_accept`, `handle_read`, `handle_write`, `handle_clone`. Map `METRICS: HashMap<u32, SomaticMetrics>` com 1 entrada. Funcao `update_metrics(id)` incrementa contadores |
| `user/src/main.rs` | Core | Loader userspace assincrono (tokio). `attach_tracepoint` com multiplos candidatos por evento. Loop de exportacao (1s): le metricas do map, injeta `sovereign_word` lido de `/run/omnimind/ebpf_metrics.json`, escreve JSON em `/run/omnimind/ebpf_metrics.json` |

## Arquitetura
- **common**: biblioteca `no_std` com tipos compartilhados. `SomaticMetrics` e `#[repr(C)]` para compatibilidade binaria entre kernel eBPF e userspace. Feature `user` ativa `aya::Pod` e `serde::Serialize`.
- **ebpf**: programa kernel que anexa a 7 tracepoints de syscalls. Cada tracepoint incrementa o contador correspondente no map `METRICS`. Inicializacao lazy (cria entrada se nao existe).
- **user**: loader tokio que carrega o bytecode eBPF compilado, anexa tracepoints e exporta metricas a cada segundo. Le lexema soberano do bridge OmniMind (`langue_sovereign.sovereign_name`) e injeta de volta no map eBPF — "o kernel agora fala com nome proprio".
- **sovereign_word**: lexema transatlantico de 32 bytes UTF-8. O eBPF nao gera o lexema (sem alloc), mas o le e reenvia. Exemplos: "KU-LUMU-execve-tata" (execve em pressao), "MA-LOZI-mem-katu" (memoria saudavel). `pressure_level`: 0=normal, 1=pressure, 2=critical.

## Dependencias
- `aya` / `aya-ebpf` / `aya-log` (git: https://github.com/aya-rs/aya)
- `tokio` (macros, rt, rt-multi-thread, net, signal, time)
- `anyhow`, `env_logger`, `log`, `rlimit`, `libc`, `serde`, `serde_json`
- Toolchain: Rust nightly `2026-03-18`, target `bpfel-unknown-none`

## Notas
- Requer privilegios root para carregar programas eBPF e criar `/run/omnimind/`. Fallback para `/tmp/omnimind/` se nao for root.
- `rlimit::setrlimit(MEMLOCK, INFINITY, INFINITY)` e chamado no inicio para permitir locked memory necessario para maps eBPF.
- Tracepoints `connect`, `accept`, `read`, `write`, `clone` sao opcionais (`required=false`) — falham silenciosamente se o syscall nao existir no kernel (ex: `clone3`, `accept4`).
- Saida JSON em `/run/omnimind/ebpf_metrics.json` com `schema_version: 1`, `source: "aya-ebpf-monitor"`, `window_hint_ms: 1000`.
