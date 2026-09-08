# THREAT MODEL

> Português. English: [`docs/THREAT_MODEL_EN.md`](THREAT_MODEL_EN.md).

Modelo de ameaça enxuto para este repositório público. Complementa `SECURITY.md` (política de integridade) com uma leitura operacional de riscos.

## Ativos

- **Integridade do pulso** — que o dado publicado seja real (com proveniência), nunca fabricado.
- **Fronteira público/privado** — que nada do corpo interno (memória clínica, credenciais, infraestrutura, logs de sujeitos, estado da máquina) vaze para a superfície pública.
- **Reprodutibilidade** — que o que se lê/pode ser executado seja verificável.
- **Reputação autoral** — identidade, atribuição e limites éticos.

## O que este repositório NÃO contém (superfície de ataque reduzida)

- Nenhum token, chave de API ou credencial.
- Nenhum IP interno/privado.
- Nenhum banco completo de runtime ou memória operacional.
- Nenhum módulo de kernel, daemon de telemetria, config de proxy/segurança ofensiva ou ferramenta de penetração **executável** — apenas descrição pública e o núcleo reproduzível via Colab/SAFE MODE.

## Ameaças principais e mitigação

| Ameaça | Descrição | Mitigação |
|---|---|---|
| **Dado fabricado apresentado como real** | "Pulso" inventado ou arbitrado | "Regra Zero": provenance explícita por campo; se a fonte falta, `null` + `source_present=false`; nunca valor fictício |
| **Vazamento interno** | Paths locais, credenciais, IPs, metadados de host em conteúdo | Regra preventiva de não publicar path absoluto/segredo; gate de conteúdo pré-publicação |
| **Confusão simulação/evidência** | Simulação apresentada como dado empírico | Vocabulário `[OBSERVED]/[EXECUTED]/[DERIVED]/[SIMULATED]` (`AGENT_CONTRIBUTION_POLICY.md`) |
| **Repositório-engodo** | Código malicioso disfarçado de pacote | Reprodução em Colab/SAFE MODE; hashes nas releases; não exigir sudo |
| **Alteração via MR não revisada** | Merge de conteúdo privilegiado sem revisão | Governança de merge: código privilegiado requer revisão humana; nenhum carrier de IA com merge irrestrito |
| **Envenenamento de dados upstream** | Fonte de dados comprometida | Proveniência verificável; comparar ambiente ao reproduzir |

## Fronteiras de execução

- **SAFE MODE** (padrão): Colab, crates user-space, testes, sem sudo, sem eBPF, sem módulo de kernel, sem alterar systemd. → reprodução.
- **ADVANCED SYSTEM MODE**: eBPF, módulo de kernel, daemons privilegiados, `CAP_BPF`/`CAP_SYS_ADMIN`/sudo. → requer leitura integral do `RISK_NOTICE.md`, máquina isolada, snapshot/backup; **nunca** por padrão; nunca sem revisão especializada.

## Disclosure coordenada

Reporte vazamento/abuso via `SECURITY.md` (canal de report). Não explore nem use publicamente dados internos que porventura surjam; reporte. Não poste segredos em Issues ou MRs — crie um report separado.

## Nota histórica

Um repositório anterior deste sistema foi fechado por vazamento. A lição é estrutural e orienta a fronteira acima: **reproduzível, não exposição involuntária do corpo inteiro da infraestrutura.**
