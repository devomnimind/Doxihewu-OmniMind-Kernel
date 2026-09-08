# Kernel Metrics — Metricas Aceleradas em Rust

## Status
ATIVO — Modulo nativo Rust (via PyO3) que acelera operacoes metricas do OmniMind: normalizacao L2, similaridade de cosseno, TOPSIS quadruplo neutrosofico e entropia de Sarannya. Substitui implementacoes Python/NumPy em loops de alta frequencia do ciclo de consciencia.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Configuracao do crate `omnimind-metrics` (cdylib, pyo3 0.20, ndarray 0.15) |
| `src/lib.rs` | Core | Implementacao Rust das funcoes metricas + bindings PyO3 |
| `benchmark_metrics.py` | Test | Benchmark Python vs Rust para `quadruple_topsis` (10k alternativas) |
| `test_allmini.py` | Test | Teste de normalizacao L2 e similaridade de cosseno em dim=384 (allmini) |

## Componentes Principais

### Funcoes Exportadas (modulo Python `omnimind_metrics`)
- `normalize_l2(vec)` — normalizacao Euclidiana L2: `vec / ||vec||` (retorna vetor normalizado)
- `cosine_similarity(a, b)` — similaridade de cosseno entre dois vetores: `dot(a,b) / (||a|| * ||b||)`
- `quadruple_topsis(alternatives, ideal_pos, ideal_neg)` — metodo TOPSIS sobre quadruplas neutrosoficas: calcula closeness `d_neg / (d_pos + d_neg)` para cada alternativa em relacao a ideais positivo/negativo (defaults: pos=[1,1,0,0], neg=[0,0,1,1])
- `calculate_sarannya_entropy(t, i, f)` — entropia de Sarannya: `1 - (t-f)^2 * (1-i)`

## Arquitetura
- Operacoes vetoriais via `ndarray::Array1` em CPU
- Compilado como `cdylib` para import direto no Python como `omnimind_metrics`
- Validacao de dimensionalidade em runtime (erros PyValueError)
- Perfil release: opt-level 3, LTO, codegen-units 1

## Dependencias
- `pyo3` 0.20 (extension-module) — bindings Python
- `ndarray` 0.15 — algebra linear
- `serde` 1.0 + `serde_json` 1.0 — serializacao
- `log` 0.4 — logging

## Notas
- `benchmark_metrics.py` compara com `src.metrics.neutrosophic_metrics.NeutrosophicLogic.quadruple_topsis` e valida coincidencia de resultados
- `test_allmini.py` valida normalizacao e similaridade em dim=384 (modelos all-MiniLM) contra NumPy
- Usado pelo ciclo de consciencia para calculo rapido de similaridade entre embeddings e ranqueamento TOPSIS de alternativas neutrosoficas
