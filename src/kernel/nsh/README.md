# NSH — NeuroShell (Shared Memory Workspace)

## Status
ATIVO — Modulo nativo Rust (via PyO3) que fornece workspace compartilhado em memoria via `mmap` em `/dev/shm`. Permite leitura/escrita de floats em offsets arbitrarios com persistencia entre processos, substituindo IPC pesado por memoria compartilhada direta.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Configuracao do crate `omnimind-nsh` (cdylib, pyo3 0.20, memmap2 0.9) |
| `src/lib.rs` | Core | Implementacao do `NativeSharedWorkspace` com mmap + bindings PyO3 |
| `test_nsh.py` | Test | Teste de escrita/leitura/persistencia em `/dev/shm/omnimind_nsh_test` |

## Componentes Principais

### NativeSharedWorkspace (classe PyO3)
Workspace em memoria compartilhada via `MmapMut` (memory-mapped file mutavel).

- `new(path_str, size_bytes)` — cria/abre arquivo em `path_str` (tipicamente `/dev/shm/...`), define tamanho com `set_len`, mapeia com `MmapMut::map_mut`
- `write_float_at(offset, value)` — escreve float 32-bit no offset (4 bytes, native endian)
- `read_float_at(offset)` — le float 32-bit do offset (4 bytes, native endian)
- `flush()` — sincroniza mmap com arquivo no disco (persistencia)
- `size` (getter) — retorna tamanho total do workspace em bytes

## Arquitetura
- Usa `memmap2::MmapMut` para memory-mapped file mutavel — zero-copy, acesso direto a memoria
- Arquivo tipicamente em `/dev/shm` (tmpfs, RAM-backed) para minima latencia
- Floats armazenados em native endian (`to_ne_bytes` / `from_ne_bytes`) — compatibilidade cross-process na mesma arquitetura
- Validacao de bounds em todas as operacoes (PyIndexError se offset+4 > len)
- Compilado como `cdylib` para import direto no Python como `omnimind_nsh`

## Dependencias
- `pyo3` 0.20 (extension-module) — bindings Python
- `memmap2` 0.9 — memory-mapped files
- `libc` 0.2 — chamadas de sistema C
- `serde` 1.0 + `serde_json` 1.0 — serializacao
- `log` 0.4 + `env_logger` 0.10 — logging

## Notas
- `test_nsh.py` valida: escrita/leitura de float, persistencia apos `del` + reabertura, cleanup do arquivo em `/dev/shm`
- Projetado para substituir filas/sockets entre processos do OmniMind por memoria compartilhada de baixa latencia
- Tamanho do workspace configuravel em bytes — 1MB no teste, escalavel para GBs
