# ADR-0002 — SAFE MODE as Default

## Status

Aceito (2026-09-08).

## Contexto

O `kernel_base` contém componentes de risco variado: crates user-space (seguros) e crates **privilegiados** (`ebpf_monitor` eBPF/root, `sovereign_module` kernel module/DKMS). Um visitante externo precisa saber, à primeira vista, o que é seguro rodar e o que exige isolamento — sem ler a teoria inteira.

## Decisão

A experiência pública é **SAFE MODE por padrão**:

- **SAFE MODE** (padrão): compilar crates user-space, rodar testes, usar o Colab público (`colab_public_base_triad.py`). Sem sudo, sem eBPF, sem módulo de kernel, sem alterar systemd.
- **ADVANCED SYSTEM MODE**: `ebpf_monitor` + `sovereign_module` são **source-only** e privilegiados. Exigem máquina isolada, leitura integral de `RISK_NOTICE.md`/`SECURITY.md`/`THREAT_MODEL.md`, snapshot/backup, e **revisão humana especializada**; nunca por padrão.

O distintivo foi adicionado ao topo de `kernel_base/README.md` (`[!WARNING]`).

## Consequências

- Reduz a barreira de entrada segura (reproduzível sem privilégio).
- Deixa explícita a fronteira onde começa o risco privilegiado.
- Exige disciplina: contribuições que toquem modo avançado passam por revisão humana (ver `GOVERNANCE.md`).

## Próximo passo verificável

Um visitante novo deve ser capaz de, seguindo só o SAFE MODE, compilar e testar os crates user-space sem sudo nem alterações de sistema.
