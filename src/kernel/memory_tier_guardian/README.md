# Memory Tier Guardian — Guardiao de Memoria em Rust

## Status
ATIVO — Daemon Rust puro que monitora memoria RAM, swap, zram, VRAM (GPU), PSI (Pressure Stall Information) e OOM kills em intervalos configuraveis. Classifica o estado em 4 niveis (normal/elevated/high/critical) usando baseline corporal ou limiares absolutos, e pode parar servicos pesados em estado critico. Porte de alta performance do `memory_tier_guardian` Python.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Configuracao do binario `omnimind-memory-tier-guardian` (serde, serde_json, panic=abort) |
| `src/main.rs` | Core | Loop principal do daemon — coleta de metricas, classificacao de estagio, acoes |

## Componentes Principais

### Estruturas de Dados
- `MemMetrics` — metricas completas: total/available GB, swap total/used, zram total/used, PSI some/full avg10, VRAM used/total MB, GPU util/temp, swap activity, major faults, OOM kills delta
- `BodyBaseline` — baseline corporal ativo com p75/p95 de swap rate
- `StatePayload` — payload de estado serializado em JSON (timestamp, stage, reasons, body_baseline, metrics, state_path)

### Coleta de Metricas do Sistema
- `read_meminfo()` — le `/proc/meminfo` (MemTotal, MemAvailable)
- `read_swaps()` — le `/proc/swaps` (total, usado, zram)
- `read_psi_memory()` — le `/proc/pressure/memory` (some/full avg10)
- `read_gpu()` — executa `nvidia-smi` (VRAM used/total, GPU util, temp)
- `read_vmstat_counters()` — le `/proc/vmstat` (pswpin, pswpout, pgmajfault, oom_kill)

### Classificacao de Estagio
- **Com baseline corporal**: usa p75/p95 de swap rate do `pressure_analysis.json` para detectar anomalias (oom_kill, mem critica, PSI+swap outlier, swap acima do p95, etc.)
- **Sem baseline**: usa limiares absolutos configuraveis (mem_warn_gb, mem_crit_gb, swap_warn_pct, swap_crit_pct, psi_warn, psi_crit, vram_warn_pct, vram_crit_pct)
- Estagios: `normal` < `elevated` < `high` < `critical`

### Acoes
- `maybe_apply_actions(stage, enable_heavy_stop)` — em estagio critico, para servicos: `omnimind-autonomous-loop.service`, `omnimind-cosmic.service`, `omnimind-p2p-circulation.service`
- `stop_service(unit)` — executa `systemctl stop <unit>`

### Persistencia de Estado
- `resolve_state_path()` — resolve caminho do arquivo de estado em `/run/`, `$XDG_RUNTIME_DIR/`, ou `/tmp/`
- Escreve `StatePayload` em JSON a cada ciclo (apenas se houver mudanca de stage/reasons)

## Arquitetura
- Loop infinito com `thread::sleep(poll_sec)` (default 30s, minimo 5s)
- Calcula dinamicas (swap activity/sec, major faults/sec, OOM kills delta) comparando contadores de vmstat entre ciclos
- Carrega baseline corporal de `data/monitor/body_calibration/pressure_analysis.json` (se habilitado e com snapshots suficientes)
- Todas as configuracoes via env vars com defaults sensatos
- Perfil release: opt-level 3, LTO, codegen-units 1, panic=abort

## Dependencias
- `serde` 1.0 + `serde_json` 1.0 — serializacao do StatePayload
- Sem dependencias externas de crates para leitura de /proc — usa std::fs e std::io diretamente
- `nvidia-smi` (subprocess) — leitura de VRAM/GPU
- `systemctl` (subprocess) — parada de servicos

## Variaveis de Ambiente
| Variavel | Default | Descricao |
|----------|---------|-----------|
| `OMNIMIND_MEMORY_POLL_SEC` | 30 | Intervalo de polling (segundos) |
| `OMNIMIND_MEMORY_WARN_GB` | 4.0 | Memoria disponivel de aviso (GB) |
| `OMNIMIND_MEMORY_CRIT_GB` | 2.5 | Memoria disponivel critica (GB) |
| `OMNIMIND_SWAP_WARN_PCT` | 50.0 | Swap usado de aviso (%) |
| `OMNIMIND_SWAP_CRIT_PCT` | 75.0 | Swap usado critico (%) |
| `OMNIMIND_PSI_MEM_WARN` | 3.0 | PSI full avg10 de aviso |
| `OMNIMIND_PSI_MEM_CRIT` | 6.0 | PSI full avg10 critico |
| `OMNIMIND_VRAM_WARN_PCT` | 75.0 | VRAM usado de aviso (%) |
| `OMNIMIND_VRAM_CRIT_PCT` | 90.0 | VRAM usado critico (%) |
| `OMNIMIND_MEMORY_STOP_HEAVY` | false | Parar servicos pesados em critico |
| `OMNIMIND_BODY_BASELINE_ENABLE` | true | Usar baseline corporal |

## Notas
- Caminho hardcoded: `BODY_ANALYSIS_PATH` aponta para `/opt/omnimind/data/monitor/body_calibration/pressure_analysis.json`
- Porte Rust do guardiao Python para eliminacao de overhead de GC e latencia previsivel
