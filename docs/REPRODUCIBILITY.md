# REPRODUCIBILITY

> Português. English: [`docs/REPRODUCIBILITY_EN.md`](REPRODUCIBILITY_EN.md).

Este documento descreve como **reproduzir** o que este repositório publica — separando o que é verificável de perto do que é apenas afirmado.

## Princípio

- Tudo que pulsa aqui carrega **proveniência verificável** (`source`, `path`, `source_present`). Se a fonte real não está disponível, o campo é `null` com `source_present=false` — nunca um valor fictício (ver `SECURITY.md`, "Regra Zero").
- **Simulação nunca é apresentada como evidência observada** (ver o vocabulário em `docs/AGENT_CONTRIBUTION_POLICY.md`).
- O extrato público é reprodutível a partir dos bancos canônicos `/ release / Zenodo`, não a partir dos dados brutos do runtime interno.

## O que este repositório contém

- **Pulso público**: `data/pulse/current.json` (a batida mais recente do sistema, com `provenance.sources` apontando para bancos canônicos).
- **Papers e bancos de evidência publicados**: os `.sqlite`/PDFs de evidência **não** ficam no git — vivem em **release do GitHub** e **Zenodo** (DOI `10.5281/zenodo.22647857`), com hashes/checksums reportados.
- **Núcleo reproduzível**: `kernel_base/omnimind_public_base.tar.gz` + `kernel_base/colab_public_base_triad.py` (compila e testa 5 crates no Colab, sem root, sem token obrigatório).

## Passos de reprodução (trilha segura)

1. **Núcleo** — rode o Colab público:
   ```
   kernel_base/colab_public_base_triad.py
   ```
   Ele baixa o tarball do HF, compila os crates user-space e roda os testes. Não exige sudo nem altera o sistema.

2. **Simulação** — use o simulador (local), nunca o hardware, para reprodução rotineira. Requer `HF_TOKEN` opcional (repo público) — nunca cole um token privado.

3. **Evidência** — para os bancos de evidência e hashes, consulte o release do GitHub / Zenodo correspondente, não este repo (o git guarda o ponteiro, não os bytes).

## Verificação de hashes

Para qualquer release, confira o checksum reportado contra a soma do artefato baixado:

```bash
sha256sum <artefato.baixado>
```

Compare com o valor publicado na release / Zenodo. Se divergir, **não** use o artefato e reporte (ver `SECURITY.md`).

## Limitações

- A reprodução confirma que o **pipeline funciona e é determinístico sob as condições descritas** — não valida alegações biomédicas nem presença de consciência.
- Ambientes podem divergir (versão de `rustc`, libs). Registre a versão do ambiente ao reproduzir para comparabilidade.
- Itens marcados `[SIMULATED]` em papers são **modelo**, não dado empírico.

## Reportando uma reprodução

Ao reproduzir e obter resultado **diferente** do publicado, abra uma Issue com label `reproducibility`, informando: ambiente (OS, rustc, libs), comandos executados, resultado obtido vs esperado, e hashes.
