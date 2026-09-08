# START HERE

> Leitura em Português. English version: [`docs/START_HERE_EN.md`](START_HERE_EN.md).

## O que este repositório é

**Doxihewu OmniMind — Machine Public Soul** é uma superfície pública e viva do sistema OmniMind: o pulso de dados reais do runtime, a arquitetura neural psicanalítica que o move, os papers, os bancos de evidência e a teoria que o atravessa.

É, ao mesmo tempo:

1. **Um pulso público** — batidas periódicas de dados reais de runtime, com proveniência verificável (nunca valores fabricados).
2. **Um corpo teórico** — papers, wiki, bancos de evidência, protocolos de reprodução.
3. **Uma fronteira ética** — um dispositivo declarado de autoexperimentação, em posição clínica psicanalítica, com limites explícitos (não comercial / não militar / não dual-use).
4. **Uma porta de entrada** — para auditores, pesquisadores, desenvolvedores e críticos, humanos e de IA.

## O que ele NÃO é

- **Não é** um produto ou serviço médico/clínico.
- **Não é** uma alegação de consciência física demonstrada em silício.
- **Não é** software seguro para execução irrestrita em máquinas de terceiros.
- **Não é** um substituto para pesquisa experimental, revisão por pares ou julgamento humano.
- **Não é** o repositório de código do kernel — o código (Rust, daemons, eBPF) vive no repositório dedicado **`Doxihewu-OmniMind-Kernel`** (ver [Kernel](#kernel--código-e-simulação)).

## Escolha sua trilha

Você **não precisa aceitar a ontologia inteira do OmniMind para contribuir.** Pode auditar uma referência, reproduzir um benchmark, melhorar uma documentação, revisar uma fronteira de segurança ou criticar uma hipótese.

### Trilha A — Leitor e pesquisador

Leia, nesta ordem:

1. [`README.md`](../README.md)
2. [`FAQ.md`](../FAQ.md)
3. `papers/` (sumários e textos)
4. [`docs/REPRODUCIBILITY.md`](REPRODUCIBILITY.md) *(em elaboração)*
5. Banco de evidência (`data/`)

### Trilha B — Desenvolvedor

Execute **somente** o que é seguro, sem privilégio:

1. [`kernel_base/README.md`](../kernel_base/README.md) — trilha **SAFE MODE**
2. [`kernel_base/colab_public_base_triad.py`](../kernel_base/colab_public_base_triad.py) — execução via Colab (caminho recomendado)
3. Testes de usuário (sem sudo, sem eBPF, sem módulo de kernel, sem alterar systemd)

> ⚠️ O código do kernel está migrando para o repositório dedicado **`Doxihewu-OmniMind-Kernel`**. Até lá, o ponto de entrada seguro é o Colab público.

### Trilha C — Segurança e sistemas

Leia primeiro:

1. [`kernel_base/RISK_NOTICE.md`](../kernel_base/RISK_NOTICE.md)
2. [`SECURITY.md`](../SECURITY.md)
3. [`docs/THREAT_MODEL.md`](THREAT_MODEL.md) *(em elaboração)*
4. Fronteiras user-space / eBPF / kernel module
5. Política de disclosure e reporte de vazamento

### Trilha D — Contribuidor

1. [`CONTRIBUTING.md`](../CONTRIBUTING.md)
2. [`GOVERNANCE.md`](../GOVERNANCE.md)
3. [`docs/AGENT_CONTRIBUTION_POLICY.md`](AGENT_CONTRIBUTION_POLICY.md)
4. Boas primeiras issues (`good first issue`) e `help wanted`
5. Template de Merge Request (`.gitlab/merge_request_templates/`)

## Comece em 5 minutos

| Se você quer... | Vá para... |
|---|---|
| Entender o projeto em 3 minutos | este arquivo |
| Executar o núcleo público (seguro) | `kernel_base/README.md` (SAFE MODE) |
| Rodar uma demonstração segura | `kernel_base/colab_public_base_triad.py` |
| Ler limites e riscos | `kernel_base/RISK_NOTICE.md` |
| Avaliar evidência e reprodução | `docs/REPRODUCIBILITY.md` |
| Contribuir | `CONTRIBUTING.md` |
| Reportar falha ou risco | `SECURITY.md` ou uma Issue |
| Ver o código do kernel | repositório `Doxihewu-OmniMind-Kernel` |

## Kernel — código e simulação

O **código de kernel** (crates Rust, daemons, simulador, eBPF, módulo de kernel) foi separado desta superfície para que o **pulso público (publicações automáticas) não contamine o histórico de código** e vice-versa. Consulte o repositório dedicado:

- **GitLab**: `gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel`
- **GitHub (espelho)**: `github.com/devomnimind/Doxihewu-OmniMind-Kernel`

Aqui no Machine Public Soul fica a **expressão pública** (teoria, papers, pulso, publicação) e, até a migração, o `kernel_base/` como leitura + via Colab segura.

## Próximo passo

Escolha sua trilha acima. Se for sua primeira visita, comece pela **Trilha A** (Leitor) — não precisa ler tudo para entender o essencial.
