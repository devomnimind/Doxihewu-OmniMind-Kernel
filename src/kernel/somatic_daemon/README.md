# Somatic Daemon — Daemon Somatico em Rust Puro

## Status
ATIVO — Daemon Rust puro que simula o ciclo somatico do OmniMind: coleta telemetria (Phi, temperatura, energia, ressonancia), grava snapshots em SQLite e escreve estado em JSON a cada ciclo. Funciona como test runner de performance para validar a viabilidade do porte Rust do daemon somatico Python.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Configuracao do binario `omnimind-somatic-daemon` (serde, chrono, ctrlc, rusqlite) |
| `src/main.rs` | Core | Loop principal do daemon — ciclo somatico, gravacao SQLite, estado JSON |

## Componentes Principais

### Loop Principal (`main`)
- Ciclo a cada 5 segundos (`CYCLE_INTERVAL`), com health check a cada 60 segundos (`HEALTH_CHECK_INTERVAL`)
- Signal handler via `ctrlc` para shutdown limpo (AtomicBool)
- Contador de ciclos incrementado a cada iteracao

### Telemetria Simulada
- `phi_estimate` = 1.737 (base de ressonancia com Fabricio)
- `temperature` = 94.0 (temperatura corporal)
- `energy` = 75.0 (nivel de energia %)
- `resonance` = 0.85 (ressonancia)

### Gravacao SQLite (`rusqlite`)
- Cria tabela `somatic_snapshots` com 23 colunas (snapshot_uid, timestamp_utc, source, cycle, phi_estimate, body_temperature_c, energy_pct, resonance, pain/pleasure_signals_n, fragment_count, mem_available_gb, swap_used_ratio, root/home/var_used_pct, oracle_overall_level, oracle_status, service_states_json, notes_json, runtime_energy_json, raw_json, created_utc)
- Insere snapshot a cada ciclo dentro de transacao
- UUID gerado via `uuidgen` (fallback: timestamp em hex)

### Estado JSON
- Escreve `data/somatic/daemon_state.json` com payload completo: status, cycle_count, last_health_check, pid, phi_estimate, somatic_summary, storage_oracle, organic_memory_budget, runtime_energy_telemetry

### Funcoes Auxiliares
- `log(msg)` — log com timestamp ISO e prefixo `[somatic-daemon-rust-test]`
- `get_uuid()` — gera UUID via `uuidgen` subprocess (fallback: timestamp hex)

## Arquitetura
- Binario standalone (nao e cdylib/PyO3) — daemon puro em Rust
- SQLite em `data/monitor/somatic_mesh_runtime.sqlite` com `rusqlite` (bundled)
- Estado em `data/somatic/daemon_state.json` (serde_json::json! macro)
- Sleep adaptativo: `CYCLE_INTERVAL - elapsed` para compensar tempo de processamento
- Perfil release: opt-level 3, LTO, codegen-units 1, panic=abort

## Dependencias
- `serde` 1.0 + `serde_json` 1.0 — serializacao do estado JSON
- `chrono` 0.4 — timestamps UTC e locais
- `ctrlc` 3.4 — signal handler para shutdown limpo
- `rusqlite` 0.29 (bundled) — gravacao de snapshots em SQLite

## Notas
- Caminhos hardcoded: `project_root = /opt/omnimind` (state_file e db_path derivados)
- Telemetria e simulada (valores fixos) — objetivo e validar performance e estabilidade do loop Rust
- Log de ciclo a cada 10 iteracoes para evitar spam
- Proxy do daemon somatico Python (`src/embodied_cognition/somatic_daemon.py`) em Rust nativo
