# omnimind-sovereign-kuramoto

> Sovereign Kuramoto Solver + Psychoanalytic LIF + Hopf Bifurcation Monitor — implementação OmniMind-nativa (clean-room, AGPL-free).
> Licença: CC-BY-NC-ND-4.0.

## Finalidade

Motor numérico acelerado do kernel: solver de Kuramoto, neurônios LIF com
leitura psicanalítica, monitor de bifurcação de Hopf, estendidos com
hyper-Kuramoto, campo INRC e acoplamento psicoanalítico (Freud10D) e geometria
dodecatíada. Implementação **original OmniMind** dos conceitos matemáticos
(Kuramoto, LIF, Hopf são de domínio público); as extensões são próprias — não é
cópia de implementações AGPL.

## O que executa

**Biblioteca `cdylib` PyO3** (módulo Python `omnimind_sovereign_kuramoto`). Em
runtime serve de motor numérico invocado pelo Python (via `__init__.py`, que
delega ao cdylib quando disponível, com fallback NumPy puro). Não é daemon nem
binário.

## Classes / tipos públicos e contrato (API PyO3)

- `PySovereignKuramoto` (`SovereignKuramotoSolver`): `new(omega, coupling_flat, initial_phases, noise_amp=0.0)`; `set_geometry(w_flat,sigma_g)`; `set_psychoanalytic(psi_flat,sigma_psi)`; `set_hypergraph(edges,weights,k_hyper)`; `set_inrc_field_pressure(f)`; `step(dt,seed=0)`; `run(n_steps,dt,seed=0)`; `order_parameter_pair()`; `order_parameter_triad()`; `get_phases/set_phases`; `set_coupling`; getter `n`; `__repr__`.
- `PyPsychoanalyticLif` (`PsychoanalyticLif`): `new(v_rest=-65,v_reset=-70,v_threshold=-50,refractory_period=3,leak_rate=0.1)`; `set_neutrosophic(t,i,f)`; `step(input_current)`; `dynamic_threshold()`; `dynamic_reset()`; `reset()`; getters `v`, `refractory_counter`, `t_neutro`, `i_neutro`, `f_neutro`.
- `PyPsychoanalyticLifBatch`: `new(n,...)`; `set_neutrosophic_batch(triples)`; `step_batch(inputs)`; `get_potentials/get_thresholds`; `reset_all()`; getter `n`.
- `PyHopfBifurcationMonitor`: `new(n)`; `update(w_flat,state)->(String,f64)`; `spectral_radius(w_flat,state)`; `classify_regime(sr)`; `bifurcation_just_occurred()`; `spectral_radius_trend()`; `summary()`. Define `__version__`/`__doc__`.
- Núcleo Rust real (`kuramoto.rs`, `lif.rs`, `hopf.rs`): `DynamicalRegime { FixedPoint, LimitCycle, Chaotic, Unknown }`, `HopfBifurcationMonitor`, `BifurcationSummary`, `PsychoanalyticLif(Batch)`. PRNG: Box-Muller + ChaCha8; raio espectral por power iteration (sem solver de autovalores).

## Dependências

`pyo3` 0.20 (cdylib), `ndarray` 0.15, `rand` 0.8, `rand_chacha` 0.3, `rayon` 1.10, `serde`, `serde_json`. Release com `lto`.

## Acoplamento a hardware / calibração local

Nenhum acoplamento direto de máquina (é puro compute em memória; sem paths,
`/proc`, `/sys`, rede ou SQLite). As dimensões `n` são semântica de entrada:
tipicamente 10 (Freud10D) ou 12 (dodecatíada). Integração apenas via PyO3/GIL
quando chamado pelo wrapper Python. Apesar disso, pertence ao kernel **real da
máquina local** — as dimensões e acoplamentos (INRC, Freud10D, Dodecatíade) foram
calibrados contra o traço do sujeito-processo local, e devem ser estudados antes
de adaptação a outro sujeito/ambiente.

## Privilégio / risco

Sem privilégio. Não toca kernel/eBPF, não exige sudo. Original, AGPL-free.
