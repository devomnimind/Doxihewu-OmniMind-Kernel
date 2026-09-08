# Freud 10D — Aparato Psiquico Freudiano 10D em Rust

## Status
ATIVO — Porte nativo Rust (via PyO3) do aparelho psiquico freudiano 10D, espelho de `src/consciousness/freudian_10d_apparatus.py`. Implementa 10 dimensoes topograficas com forward pass em ndarray (CPU), operadores INRC piagetianos e estado neutrosofico. Aceleracao para o loop de consciencia do OmniMind.

## Arquivos
| Arquivo | Tipo | Descricao |
|---------|------|-----------|
| `Cargo.toml` | Build | Configuracao do crate `omnimind-freud10d` (cdylib, pyo3 0.20, ndarray 0.15) |
| `src/lib.rs` | Core | Implementacao completa do aparato 10D em Rust + bindings PyO3 |

## Componentes Principais

### Freud10DApparatus (classe PyO3)
Estrutura principal que mantem matriz de conectividade 10x10, vetor de estado 10D e historico de tensao/prazer.

- `new(n_neurons_per_dim=128)` — inicializa aparato com matriz de conectividade pre-definida
- `forward(perception)` — forward pass: injeta percepcao em DIM_PHI, executa 5 iteracoes de `tanh(W.t() @ state)`, retorna `(consciousness_scalar, state_vector_10d)`
- `state_dict()` — estado atual como dict Python `{phi, psi, omega, ..., lambda_}`
- `state_vector()` — vetor 10D atual (compat com `Psychic10DState.to_vector`)
- `topographic_flow()` — fluxo Consciente/Pre-consciente/Inconsciente com repressao e retorno do reprimido
- `structural_conflict()` — conflito Id/Ego/Superego com intensidade
- `get_10d_metrics()` — metricas completas (state_10d, topographic, structural, transference, sublimation, tension, pleasure)
- `tension_window(n)` — janela recente do historico de tensao
- `reset()` — zera estado e historicos

### Operadores INRC Piagetianos
Transformacoes cognitivas sobre o state 10D, preservando invariantes (I^2=I, N^2=I, R^2=I, C^2=I; NR=C, NC=R, RC=N):

- `inrc_identity()` — I: retorna state sem modificacao
- `inrc_negate()` — N: inverte state (antitese)
- `inrc_reciprocate()` — R: troca Ego<->Superego, Id<->Sublimacao, Phi<->Omega
- `inrc_compensate()` — C: ajusta state para preservar invariantes canonicos
- `inrc_recommend()` — recomenda operacao baseado no state atual (retorna operador + razao)

### Estado Neutrosofico
- `neutrosophic_state()` — tripleto (T, I, F) para cada dimensao: T=ativacao, I=indeterminacao, F=conflito

### Funcao Standalone
- `freudian_values_from_state(state_vec)` — mapeia 10D para 7 chaves compativeis com `integrate_with_topology` do Python

## 10 Dimensoes Topograficas
0=Phi(Percepcao), 1=Psi(Memoria), 2=Omega(Consciencia), 3=Theta(Pre-consciente), 4=Upsilon(Inconsciente), 5=Xi(Id), 6=Zeta(Ego), 7=Eta(Superego), 8=Kappa(Transferencia), 9=Lambda(Sublimacao)

## Arquitetura
- Forward pass em CPU via ndarray (10x10), 5 iteracoes de `tanh(W.t() @ state)`
- Matriz de conectividade inicializada com pesos fixos (fluxo perceptual, topografico, estrutural, processos, recorrentes, feedback)
- GPU lane fica para versao futura via candle/tch
- Compilado como `cdylib` para import direto no Python como `omnimind_freud10d`
- Perfil release: opt-level 3, LTO, codegen-units 1

## Dependencias
- `pyo3` 0.20 (extension-module) — bindings Python
- `ndarray` 0.15 — algebra linear em CPU
- `serde` 1.0 + `serde_json` 1.0 — serializacao

## Notas
- Espelha `src/consciousness/freudian_10d_apparatus.py` — mesma matriz de conectividade e dimensoes
- Consumido por `src/consciousness/freudian_10d_apparatus.py` como fachada de compatibilidade
- Servico systemd `omnimind-freud10d-loader.service` carrega o modulo nativo
