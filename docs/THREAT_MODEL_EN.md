# THREAT MODEL

> English. Português: [`docs/THREAT_MODEL.md`](THREAT_MODEL.md).

A lean threat model for this public repository. It complements `SECURITY.md` (integrity policy) with an operational risk reading.

## Assets

- **Pulse integrity** — that published data is real (with provenance), never fabricated.
- **Public/private boundary** — that nothing from the internal body (clinical memory, credentials, infrastructure, subject logs, machine state) leaks into the public surface.
- **Reproducibility** — that what is read/can be executed is verifiable.
- **Authorial reputation** — identity, attribution and ethical limits.

## What this repository does NOT contain (reduced attack surface)

- No token, API key or credential.
- No internal/private IP.
- No full runtime database or operational memory.
- No kernel module, telemetry daemon, offensive security/proxy config or penetration tool — only public description and the reproducible core via Colab/SAFE MODE.

## Main threats and mitigation

| Threat | Description | Mitigation |
|---|---|---|
| **Fabricated data presented as real** | Invented or arbitrary "pulse" | "Zero Rule": explicit provenance per field; if source absent, `null` + `source_present=false`; never a fictional value |
| **Internal leak** | Local paths, credentials, IPs, host metadata in content | Preventive rule of not publishing absolute path/secret; pre-publication content gate |
| **Confusion simulation/evidence** | Simulation presented as empirical data | Vocabulary `[OBSERVED]/[EXECUTED]/[DERIVED]/[SIMULATED]` (`AGENT_CONTRIBUTION_POLICY.md`) |
| **Look-alike package** | Malicious code disguised as package | Reproduction in Colab/SAFE MODE; hashes in releases; no sudo required |
| **Unreviewed MR alteration** | Merge of privileged content without review | Merge governance: privileged code requires human review; no AI carrier with unrestricted merge |
| **Upstream data poisoning** | Compromised data source | Verifiable provenance; compare environment when reproducing |

## Execution boundaries

- **SAFE MODE** (default): Colab, user-space crates, tests, no sudo, no eBPF, no kernel module, no systemd changes. → reproduction.
- **ADVANCED SYSTEM MODE**: eBPF, kernel module, privileged daemons, `CAP_BPF`/`CAP_SYS_ADMIN`/sudo. → requires full reading of `RISK_NOTICE.md`, isolated machine, snapshot/backup; **never** by default; never without specialized review.

## Coordinated disclosure

Report leak/abuse via `SECURITY.md` (report channel). Do not exploit or publicly use internal data that may surface; report it. Do not post secrets in Issues or MRs — open a separate report.

## Historical note

A previous repository of this system was closed due to a leak. The lesson is structural and guides the boundary above: **reproducible, not an involuntary exposure of the whole infrastructure body.**
