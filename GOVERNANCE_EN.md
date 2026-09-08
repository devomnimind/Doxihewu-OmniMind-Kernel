# GOVERNANCE

> English. Português: [`GOVERNANCE.md`](GOVERNANCE.md).

A situated responsibility map for this public repository — not corporate bureaucracy, but clarity of who decides, how, and under what limits.

## Curation

**Fabricio da Silva** is the curator and editorial lead of the project. Curation safeguards the identity, data integrity and ethical limits of the public surface.

## Public / Private boundary

This repository is the **public expression** of OmniMind. The internal body (clinical/operational memory, full runtime databases, credentials, infrastructure, subject logs, detailed machine state) remains **private**. See `SECURITY.md` ("Zero Rule").

Publication principle: **reproducible, not an involuntary exposure of the whole infrastructure body.**

## Co-authorship

Human and AI-system contributions can be recognized as **cognitive assistance, execution, review, translation, analysis or implementation**, with documented scope.

- **Agents are execution carriers**, not responsible authors in the legal or scientific sense.
- Editorial, ethical and *merge* responsibility remains **human**.
- Every contribution must declare: model/tool, scope, changed files, executed commands, observed/simulated/derived data, limitations, provenance, and the responsible human reviewer. (See [`docs/AGENT_CONTRIBUTION_POLICY.md`](docs/AGENT_CONTRIBUTION_POLICY.md).)

## Merge authority

- **Code**: goes to the dedicated `Doxihewu-OmniMind-Kernel` repository; not merged here.
- **Privileged code** (kernel, eBPF, daemons, security, data policy): requires **human systems review**.
- **Biomedical/clinical claims**: require **methodological review** and explicit evidence vocabulary.
- **Data and evidence**: require **verifiable provenance**.
- **Theoretical texts**: require **preservation of attribution and limits**.
- **Releases**: require a **security and integrity checklist**.

> **No AI carrier has unrestricted merge power** over privileged code, security, services or data policy. Connected MCPs/agents/tools use least-privilege profiles: separate identity, per-tool permissions, read separated from write, call logs, and explicit approval for destructive operations.

## Divergence

Theoretical, methodological or ethical disagreements should be recorded (Discussions, Issues or ADRs), **without erasing the divergence**. Divergence is raw material of the field, not a defect to suppress.

## ADRs — Architecture Decision Records

Structural decisions are recorded in `docs/adr/` for traceability across multiple producers (human and agents). Planned examples:

- `0001-public-kernel-scope.md` — public kernel scope and boundary
- `0002-safe-mode-default.md` — safe simulation as default
- `0003-agent-contribution-policy.md` — agent contribution policy
- `0004-evidence-status-vocabulary.md` — evidence status vocabulary
- `0005-public-versus-private-data-boundary.md` — public/private boundary
- `0006-dual-use-review-process.md` — dual-use review process

## Languages

Content in **Portuguese and English** (bilingual). See `CONTRIBUTING.md`.
