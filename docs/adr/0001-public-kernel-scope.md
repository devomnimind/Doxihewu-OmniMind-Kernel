# ADR-0001 — Public Kernel Scope (repo dedicado)

## Status

Aceito (2026-09-08).

## Contexto

O repositório `Machine Public Soul` mistura dois tipos de tráfego de natureza muito diferente:
- **Pulso público** e documentação — publicações **automáticas** de alta frequência (batidas `pulse:`) do runtime.
- **Código de kernel** (Rust, daemons, eBPF, kernel module) — commits **deliberados**, de baixa frequência, que exigem revisão.

Misturá-los no mesmo stream de `main` suja o histórico e faz com que o pulso automático contamine (e seja contaminado por) push de código.

## Decisão

Separação em **dois repositórios**:

- `Doxihewu-OmniMind-MachinePublicSoul` (GitLab `zephyrix/` + espelho GitHub `devomnimind/`) → superfície **teórica/viva**: docs, papers, pulso, publicações, porta de entrada de participação.
- `Doxihewu-OmniMind-Kernel` (GitLab `zephyrix/` + espelho GitHub `devomnimind/`) → superfície **técnica**: código de kernel (crates Rust, daemons, simulador, eBPF, kernel module), releases de software.

O `kernel_base/` físico migra para o repo do kernel; no Machine Public Soul fica apenas leitura + via Colab (SAFE) até a migração.

## Consequências

- O pulso automático nunca toca o histórico de código do kernel, e vice-versa.
- Cada repo tem CI, labels, contribuição e releases próprios.
- A porta de entrada (`START_HERE`) precisa apontar para ambos.
- Custos: dois repos para gerir; precisa planejar a migração do `kernel_base/`.

## Próximo passo verificável

Criar `zephyrix/Doxihewu-OmniMind-Kernel` + espelho `devomnimind/...`; migrar `kernel_base/` com histórico/atribuição; atualizar apontadores no Machine Public Soul.
