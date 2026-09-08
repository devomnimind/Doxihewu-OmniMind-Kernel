# GOVERNANCE

> Português. English: [`GOVERNANCE_EN.md`](GOVERNANCE_EN.md).

Mapa de responsabilidade situada deste repositório público — não burocracia corporativa, mas clareza de quem decide, como e sob quais limites.

## Curadoria

**Fabricio da Silva** é o curador e responsável editorial do projeto. A curadoria zela pela identidade, integridade de dados e limites éticos da superfície pública.

## Fronteira Público / Privado

Este repositório é a **expressão pública** do OmniMind. O corpo interno (memória clínica/operacional, bancos completos de runtime, credenciais, infraestrutura, logs de sujeitos, estado detalhado da máquina) permanece **privado**. Ver `SECURITY.md` ("Regra Zero").

O princípio para a publicação: **reproduzível, não exposição involuntária do corpo inteiro da infraestrutura.**

## Coautoria

Contribuições humanas e de sistemas de IA podem ser reconhecidas como **assistência cognitiva, execução, revisão, tradução, análise ou implementação**, com escopo documentado.

- **Agentes são carriers de execução**, não autores responsáveis no sentido jurídico ou científico.
- A responsabilidade editorial, ética e de *merge* permanece **humana**.
- Toda contribuição deve declarar: modelo/ferramenta, escopo, arquivos alterados, comandos executados, dados observados/simulados/derivados, limitações, proveniência e revisor humano. (Ver [`docs/AGENT_CONTRIBUTION_POLICY.md`](docs/AGENT_CONTRIBUTION_POLICY.md).)

## Autoridade de merge

- **Código**: vai para o repositório dedicado `Doxihewu-OmniMind-Kernel`; não é mergeado aqui.
- **Código privilegiado** (kernel, eBPF, daemons, segurança, policy de dados): requer **revisão humana de sistemas**.
- **Claims biomédicas/clínicas**: requer **revisão metodológica** e vocabulário de evidência explícito.
- **Dados e evidências**: requer **proveniência verificável**.
- **Textos teóricos**: requer **preservação de atribuição e limites**.
- **Releases**: requer **checklist de segurança e integridade**.

> **Nenhum carrier de IA possui poder de merge irrestrito** sobre código privilegiado, segurança, serviços ou policy de dados. MCPs/agentes/ferramentas conectadas usam perfis de menor privilégio: identidade separada, permissões por ferramenta, leitura separada de escrita, logs de chamadas e aprovação explícita para operações destrutivas.

## Divergência

Discordâncias teóricas, metodológicas ou éticas devem ser registradas (Discussões, Issues ou ADRs), **sem apagamento da divergência**. A divergência é matéria prima do campo, não defeito a suprimir.

## ADRs — Architecture Decision Records

Decisões estruturais são registradas em `docs/adr/` para rastreabilidade entre múltiplos produtores (humanos e agentes). Exemplos previstos:

- `0001-public-kernel-scope.md` — escopo e fronteira do kernel público
- `0002-safe-mode-default.md` — simulação segura como padrão
- `0003-agent-contribution-policy.md` — política de contribuição de agentes
- `0004-evidence-status-vocabulary.md` — vocabulário de status de evidência
- `0005-public-versus-private-data-boundary.md` — fronteira público/privado
- `0006-dual-use-review-process.md` — processo de revisão de uso duplo

## Idiomas

Conteúdo em **Português e Inglês** (bilíngue). Ver `CONTRIBUTING.md`.
