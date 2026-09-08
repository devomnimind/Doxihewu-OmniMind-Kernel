# Layered Transition — Motor de Transicao em Camadas (Python -> Rust)

## Status
ATIVO — Motor de computacao pesada em Rust que migra kernels matematicos do `integration_loop.py` (ciclo de consciencia) para CPU nativo, seguindo a filosofia "Orquestracao em Python + Execucao Pesada em Rust". Inclui agregacao de embeddings, norma L2, lineage sync, e gravacao de snapshots somaticos em SQLite.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `engine/Cargo.toml` | Build | Configuracao do crate `layered-transition-engine` (cdylib + rlib, pyo3 0.20) |
| `engine/src/lib.rs` | Core | Implementacao Rust dos kernels matematicos + gravacao SQLite + export Markdown |
| `layered_transition_plan.md` | Doc | Plano de transicao em camadas (arquitetura 3 camadas, mapa de migracao por blocos) |
| `test_layered_transition.py` | Test | Benchmarks Python vs Rust (lineage sync, norma L2, agregacao de embeddings) |
| `layered_transition_engine.so` | Binario | Biblioteca compilada (symlink para target/release) |

## Componentes Principais

### Kernels Matematicos (migrados de integration_loop.py)
- `compute_lineage_sync(cycle_count)` — soma trigonometrica sobre 16 bandas: `sum(sin(cycle/(i+1)))/16`
- `compute_vector_norm(vector)` — norma Euclidiana L2 de vetor (substitui `np.linalg.norm`)
- `aggregate_embeddings(embeddings, noise_scale)` — media de embeddings + ruido pseudo-aleatorio + normalizacao L2 (substitui `_compute_output`)

### Gravacao de Snapshots Somaticos em SQLite
- `record_somatic_snapshot_rust(db_path, source, latest_json, latest_md, cycle, phi_estimate, somatic_summary_json, oracle_payload_json, extra_json)` — insere snapshot na tabela `somatic_snapshots`, materializa exports JSON+Markdown
- `record_vagus_event_rust(db_path, source, event_kind, latest_json, latest_md, ...)` — insere evento na tabela `vagus_events`, materializa exports
- `materialize_latest_exports_rust(db_path, latest_json, latest_md)` — reconstroi payload latest a partir do SQLite e escreve JSON+Markdown

### Funcoes Internas de Apoio
- `connect_db(db_path)` — abre SQLite com WAL + synchronous=NORMAL
- `build_window_summary(conn, minutes)` — agregacoes temporais (AVG/MAX/MIN) sobre janela
- `build_latest_payload_rust(conn, db_path)` — payload completo com latest snapshot, watchdog stream, janelas 10m/60m, eventos vagus recentes
- `build_markdown_rust(payload)` — renderiza payload como Markdown legivel
- `extract_somatic_current`, `extract_fragment_count`, `extract_oracle_snapshot` — extratores JSON mapeados do Python

## Arquitetura
- **Camada 1 (Python)**: Orquestracao e controle de ciclo — mantida em `integration_loop.py`
- **Camada 2 (Rust)**: Computacao pesada em CPU — este crate
- **Camada 3 (PyTorch/GPU)**: Inferencia profunda — mantida em Python
- Import defensivo no Python: `try: from layered_transition_engine import ...` com fallback transparente para implementacao Python pura
- SQLite com WAL para baixa latencia de I/O no loop de consciencia
- Compilado como `cdylib` para import direto no Python

## Dependencias
- `pyo3` 0.20 (extension-module) — bindings Python
- `rusqlite` 0.29 (bundled) — gravacao SQLite de snapshots e eventos
- `serde_json` 1.0 — parse/serializacao de payloads JSON
- `chrono` 0.4 — timestamps UTC ISO-8601
- `uuid` 1.4 (v4) — UIDs de snapshots e eventos

## Notas
- O plano de transicao (`layered_transition_plan.md`) detalha a arquitetura em 3 camadas (Doxihewu Layering) e o mapa de migracao por blocos
- Benchmark mostra speedup significativo vs NumPy em lineage sync, norma L2 e agregacao de embeddings
- O `.so` compilado e copiado para o diretorio raiz do modulo para import direto pelo Python
