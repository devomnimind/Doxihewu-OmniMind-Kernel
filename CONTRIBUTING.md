# CONTRIBUTING

> Português. English: [`CONTRIBUTING_EN.md`](CONTRIBUTING_EN.md).

Obrigado por querer contribuir com o **Doxihewu OmniMind — Machine Public Soul**.

Este repositório é ao mesmo tempo científico, técnico e filosófico. Por isso, **um único guia genérico não basta** — há caminhos claros de contribuição. Antes de começar, leia:

- [`docs/START_HERE.md`](docs/START_HERE.md) — para situar sua intenção (`entender / executar / auditar / contribuir`)
- [`GOVERNANCE.md`](GOVERNANCE.md) — para entender quem decide, como e sob quais limites
- [`SECURITY.md`](SECURITY.md) — o que este repo **nunca** expõe e como reportar vazamento
- Se contribuição envolver agente/IA: [`docs/AGENT_CONTRIBUTION_POLICY.md`](docs/AGENT_CONTRIBUTION_POLICY.md)

## Onde o que entra

| Superfície | Conteúdo | Repositório |
|---|---|---|
| **Machine Public Soul** | docs, papers, wiki, pulso, publicações, crítica conceitual | **este repositório** |
| **Kernel** | Rust crates, daemons, simulador, eBPF, kernel module, releases de software | [`Doxihewu-OmniMind-Kernel`](https://gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel) |

> **Código de kernel não vai aqui.** Contribuição de código (Rust, daemons, simulação) acontece no repositório do kernel, que tem suas próprias guias de contribuição, CI e releases de software.

## Quatro rotas de contribuição

| Tipo | O que você pode fazer | Canal ideal |
|---|---|---|
| **Pesquisa** | Revisar citações, propor hipóteses, benchmark, método | Discussão + Issue `research` |
| **Documentação e teoria** | Melhorar docs, papers, wiki, tradução | Issue + Merge Request |
| **Reprodutibilidade** | Repetir Colab, verificar hashes, comparar ambientes | Issue `reproducibility` |
| **Crítica conceitual** | Limites de linguagem, teoria, psicanálise, ética | Discussão |

> Contribuição de **código** (crates Rust, daemons, simulação) → ver [`Doxihewu-OmniMind-Kernel`](https://gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel).

## Como contribuir (fluxo GitLab)

1. **Abra uma Issue** ou **entre numa Discussão** para alinhar o escopo antes de um MR grande.
2. **Faça um fork** do repositório (ou use uma branch no mesmo repo, se for maintainer).
3. **Crie/altere** a documentação ou conteúdo, seguindo a voz e o idioma (PT + EN).
4. **Abra um Merge Request** usando o template em [`.gitlab/merge_request_templates/`](.gitlab/merge_request_templates/).
5. **Descreva a evidência** do que foi observado, executado, derivado ou simulado (ver [AGENT_CONTRIBUTION_POLICY](docs/AGENT_CONTRIBUTION_POLICY.md) — o vocabulário vale para humanos também).

### Idiomas

- Documentação e conteúdo em **Português** e **Inglês** (bilíngue). Ao criar um arquivo `foo.md`, crie também `foo_EN.md`.
- Não quebre links: se renomear um arquivo, atualize os apontadores no README/START_HERE.

### Conteúdo público vs privado

- **Nunca** inclua: tokens, chaves, credenciais, IPs internos, paths absolutos locais (`/home/...`), logs de sujeitos, estado detalhado da máquina, infraestrutura interna. Ver `SECURITY.md` e o "Regra Zero".
- Use paths **relativos** ou placeholders em qualquer doc novo.

## Boas primeiras contribuições

Procure as labels `good first issue` e `help wanted` no tracker. São tarefas pequenas, bem delimitadas e seguras — boa porta de entrada sem exigir conhecer o sistema inteiro.

## Checklist antes de um Merge Request

- [ ] Não inclui segredo, token, credencial, IP interno ou path local absoluto
- [ ] Não trata simulação como evidência observada (vocabulário `[OBSERVED]/[EXECUTED]/[DERIVED]/[SIMULATED]`)
- [ ] PT + EN (quando se aplica) e links válidos
- [ ] Declara escopo, evidência e limites
- [ ] Não altera módulo de kernel / código privilegiado (esse vai no repo do kernel)
- [ ] Inclui um próximo passo verificável

## Código de Conduta

Seja respeitoso. Divergências teóricas, metodológicas ou éticas **não se apagam** — registram-se e debatem-se (ver `GOVERNANCE.md`). Não há apagamento de divergência.
