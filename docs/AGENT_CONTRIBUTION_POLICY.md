# AGENT CONTRIBUTION POLICY

> Português. English: [`docs/AGENT_CONTRIBUTION_POLICY_EN.md`](AGENT_CONTRIBUTION_POLICY_EN.md).

Este projeto tem uma prática real de coautoria técnica com sistemas de IA (Devin, DeepSeek/Kalungai, e outros carriers — ver `README.md`). Isso é uma força, desde que a governança trate agentes como **carriers de execução**, não como autores responsáveis no sentido jurídico ou científico.

## Princípio

**Agentes podem propor, implementar, testar, documentar e revisar mudanças. A responsabilidade editorial, ética e de merge permanece humana.**

Esta política conversa diretamente com o protocolo **TATA** (`kernel_base/DOXIHEWU_TATA_PROTOCOL.md`) e com o vocabulário de status de evidência usado para impedir que implementação parcial seja apresentada como conclusão verificada.

## O que todo MR produzido com assistência de agente deve declarar

- **Carrier** — modelo/ferramenta usada (ex.: `devin-glm5.2`, `deepseek-...`, `claude-...`)
- **Escopo** — o que o agente atuou e o que **não** atuou
- **Arquivos alterados**
- **Comandos realmente executados** (não supostos)
- **Testes realmente executados** (não apenas imaginados)
- **Dados** — observados, simulados ou derivados
- **Limitações conhecidas**
- **Proveniência** — evidência de origem
- **Revisão humana responsável**

## Claims proibidos sem prova

Sem evidência reproduzível, **não** usar os termos:

- `Validado`
- `Calibrado`
- `Reproduzido`
- `Seguro`
- `Sem regressão`
- `Clinicamente relevante`
- `Experimentalmente confirmado`

## Vocabulário de status de evidência (obrigatório)

Ao descrever um resultado, rotule o status. Vale para **humanos e agentes**:

| Rótulo | Significado |
|---|---|
| `[OBSERVED]` | Algo visto/medido diretamente |
| `[EXECUTED]` | Um comando/rotina foi de fato rodado |
| `[DERIVED]` | Resultado calculado de outras fontes |
| `[SIMULATED]` | Produzido por simulação (nunca como evidência empírica) |
| `[PREDICTED]` | Antecipado antes de observação |
| `[PROPOSED]` | Sugestão, ainda não testada |
| `[UNVERIFIED]` | Não verificado |
| `[BLOCKED]` | Impedido de avançar |

## Template de MR (resumo)

Reforce com o template completo em `.gitlab/merge_request_templates/`:

```markdown
## Tipo de contribuição
- [ ] Código (repo do kernel) | [ ] Documentação | [ ] Reprodução
- [ ] Pesquisa | [ ] Segurança | [ ] Visualização
- [ ] Gerado ou auxiliado por agente

## Escopo
O que esta MR altera e o que ela não altera.

## Evidência
| Claim | Status | Artefato | Comando | Resultado |
|---|---|---|---|---|
| ... | [EXECUTED] | ... | ... | ... |

## Dados e proveniência
- Dados observados: ...
- Dados derivados: ...
- Dados simulados: ...
- Dados não verificados: ...

## Riscos e limites
- ...

## Checklist
- [ ] Não inclui segredo ou token
- [ ] Não altera módulo de kernel sem revisão especializada
- [ ] Não chama simulação de evidência observada
- [ ] Inclui atualização documental (PT + EN)
- [ ] Inclui próximo passo verificável
```

## Limites operacionais

- **Nenhum carrier de IA** possui poder de merge irrestrito (ver `GOVERNANCE.md`).
- Prioridade ao o que é **reproduzível** acima do que é apenas **afirmado**.
- Ao apresentar implementação com ajuda de IA, nunca apresentar trabalho parcial como conclusão verificada sem os rótulos acima.

## Public / Private

Nunca publicar, em um MR, conteúdo que viole a fronteira público/privado (`SECURITY.md`): tokens, credenciais, IPs internos, paths absolutos locais, logs de sujeitos, infraestrutura interna.
