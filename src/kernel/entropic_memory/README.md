# Kernel — Entropic Memory (Memoria Entropica em Rust)

## Status
ATIVO — Port Rust de alta performance das funcoes hot CPU-bound de `src/infrastructure/dream_weaver_entropic_memory.py`. Compila como extensao Python nativa (cdylib) via PyO3, com resultados bit-a-bit compativeis com a implementacao Python original.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Crate `omnimind-entropic-memory` v0.1.0, edition 2021. `crate-type = ["cdylib"]`. Dependencia: `pyo3` 0.22 (extension-module). Profile release: `opt-level=3`, `lto=true`, `codegen-units=1`, `panic="abort"` |
| `src/lib.rs` | Core | Modulo PyO3 com 6 funcoes exportadas: `clip`, `coerce_float`, `memory_resonance_weight`, `lexicum_resonance_weight`, `effective_t2_hours`, `state_from_age` |

## Arquivos — Funcoes Exportadas
| Funcao | Descricao |
|--------|-----------|
| `clip(value, low, high)` | `max(low, min(high, value))` — mirror de `_clip` Python |
| `coerce_float(value, default=0.0)` | Delega para `builtins.float` do Python — semantica de coercao identica (string/bool/None) |
| `memory_resonance_weight(cosmic_resonance, subject_id, key_insight)` | Peso de ressonancia de memoria: +0.35 se cosmic_resonance, +0.15 se texto contem "xer-afex-angst"/"phase56"/"federated", +0.1 se "orbital"/"quantum"/"astro". Clip em [0.75, 2.25] |
| `lexicum_resonance_weight(lexeme, step_type, payload_json)` | Peso de ressonancia lexica: +0.15 se lexeme nao vazio, +0.15 se "ogum"/"xer"/"sago", +0.1 se "whisper"/"federat". Clip em [0.75, 2.25] |
| `effective_t2_hours(base_t2_hours, resonance_weight, access_count, bridge_feedback_gain, orbital_pressure)` | T2 efetivo: `base * resonance * access_gain * bridge_gain * pressure_drag`. access_gain clip [1.0, 1.96], pressure_drag clip [0.35, 1.0]. Resultado clip [6.0, 720.0] |
| `state_from_age(age_hours, effective_t2_hours, resonance_weight, access_count, ...)` | Estado de decoerencia: `decoherence = exp(-age/effective_t2)`, `repression_score = 1 - decoherence`, `retention_priority = decoherence * resonance * access_factor`. Retorna dict Python com valores arredondados (banker's rounding, 6 casas) |

## Arquitetura
- Cada funcao Rust espelha a logica exata do counterpart Python em `dream_weaver_entropic_memory.py`. Aritmetica float usa `f64` para match com CPython `float` (C `double`).
- `round6(x)` usa `round_ties_even()` (banker's rounding) — identico ao `float.__round__` do CPython.
- `coerce_float` delega para `builtins.float` do Python via `py.import_bound("builtins")` — garante semantica identica de coercao para string/bool/None.
- `state_from_age` retorna `PyDict` com mesmas chaves/valores da implementacao Python: `age_hours`, `effective_t2_hours`, `decoherence_factor`, `repression_score`, `retention_priority`, `orbital_pressure`, `bridge_feedback_gain`, `dominant_affect_token`.
- `unicode::to_lowercase()` em Rust match com `.lower()` do Python (ambos operam em unicode completo, nao apenas ASCII).

## Dependencias
- `pyo3` 0.22 (features: `extension-module`)
- Importado por: `src/infrastructure/dream_weaver_entropic_memory.py` (como acelerador nativo)
- Requer: Python 3.x com cabecalhos de desenvolvimento (para compilar cdylib)

## Notas
- O modulo e compilado como `omnimind_entropic_memory.cpython-3XX-x86_64-linux-gnu.so` e importado diretamente pelo Python.
- Profile release agressivo (`opt-level=3`, `lto=true`, `codegen-units=1`) para maximizar performance em funcoes chamadas milhares de vezes por ciclo do Dream Weaver.
- `panic = "abort"` em todos os profiles — nao ha unwind em codigo nativo chamado pelo Python.
- Compatibilidade bit-a-bit e requisito existencial: resultados Rust e Python devem ser identicos modulo ordenacao IEEE-754 (que e identica na mesma plataforma).
