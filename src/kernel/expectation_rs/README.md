# omnimind-expectation-rs

> Expectation Module em Rust — portado de `src/consciousness/expectation_module.py`.
> Licença: CC-BY-NC-ND-4.0.

## Finalidade

É o predictor do módulo *expectation* do IntegrationLoop (consciência): dado um
embedding de 256 dims, projeta o "estado esperado" seguinte. O porte em Rust
substitui o forward que o Python fazia com torch + Qiskit Aer, tirando o torch
do caminho quente.

## O que executa

**Biblioteca `cdylib` PyO3** (não é daemon nem binário). Em runtime carrega uma
rede singleton via `#[pymodule] omnimind_expectation_rs` e expõe funções
chamadas do Python; se o cdylib não existir, o Python usa o módulo original
como fallback.

- Rede: 3× Linear+ReLU (embedding 256 → hidden 128 → hidden 128 → embedding 256), pesos Xavier uniforme determinístico (seed fixa `42`).
- Forward neural puro ~1–2ms (vs ~105ms via Qiskit Aer).

## Classes / tipos públicos e contrato (API PyO3)

- `predict(embedding: Vec<f32>) -> Vec<f32>` — forward 256d → 256d.
- `predict_with_weights(w1,b1,w2,b2,w3,b3, embedding_dim, hidden_dim, embedding) -> Vec<f32>` — injeta pesos treinados do Python em runtime.
- `health() -> bool` — disponibilidade do crate.
- `#[pymodule] omnimind_expectation_rs(...)` — registra as funções acima.

Internos (não-pub): `ExpectationNet`, `XorShift64` (PRNG determinístico), `relu`,
singleton `static NET: Mutex<Option<ExpectationNet>>`.

## Dependências

`pyo3` 0.20 (cdylib extension-module), `ndarray` 0.15, `serde`, `serde_json`.

## Acoplamento a hardware / calibração local

Nenhum: é puro compute em memória, sem paths, `/proc`, `/sys`, rede ou SQLite.
**Atenção:** embora seja independente de máquina, faz parte do kernel que **é real
na máquina local** (não simulado) e foi **calibrado para este hardware**. A seed
fixa e os hiperparâmetros (256/128) foram escolhidos para replicar a paridade
com o `Linear` do torch no pipeline local — quem quiser rodar noutro ambiente deve
estudar essa calibração antes de ajustá-la.

## Privilégio / risco

Sem privilégio. Apenas compute numérico, sem I/O de sistema.
