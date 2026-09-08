## Tipo (label esperada)

`good first issue` | `help wanted` | `documentation` | `research` | `reproducibility` | `benchmark` | `security` | `theory` | `visualization` | `data-provenance` | `agent-governance` | `needs-discussion`

## Título

Pequeno, concreto, verificável (ex.: "[reproducibility] Rodar Colab e registrar versão de rustc"). Um leitor fora do projeto deve entender o escopo sem ler a teoria inteira.

## Descrição

- **Objetivo**: o que deve acontecer ao final.
- **Escopo**: o que entra e o que fica de fora.
- **Critério de aceite**: como saber que ficou pronto (verificável).
- **Status das alegações**: `[OBSERVED] [EXECUTED] [DERIVED] [SIMULATED] [PREDICTED] [PROPOSED] [UNVERIFIED] [BLOCKED]`.

## Checklist de segurança

- [ ] Não inclui segredo, token, credencial, IP interno ou path absoluto local (`/home/...`)
- [ ] Não toca em código privilegiado (vai para o repo do kernel) sem revisão
- [ ] Não trata simulação como evidência observada

## Próximo passo verificável

O que a pessoa que pegar essa issue deveria conseguir confirmar ao concluir.
