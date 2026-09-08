# AGENT CONTRIBUTION POLICY

> English. Português: [`docs/AGENT_CONTRIBUTION_POLICY.md`](AGENT_CONTRIBUTION_POLICY.md).

This project has a real technical co-authorship practice with AI systems (Devin, DeepSeek/Kalungai, and other carriers — see `README.md`). This is a strength, provided governance treats agents as **execution carriers**, not responsible authors in the legal or scientific sense.

## Principle

**Agents may propose, implement, test, document and review changes. Editorial, ethical and merge responsibility remains human.**

This policy speaks directly to the **TATA** protocol (`kernel_base/DOXIHEWU_TATA_PROTOCOL.md`) and to the evidence-status vocabulary used to prevent partial implementation from being presented as verified conclusion.

## What every MR produced with agent assistance must declare

- **Carrier** — model/tool used (e.g.: `devin-glm5.2`, `deepseek-...`, `claude-...`)
- **Scope** — what the agent acted on and what it did **not**
- **Changed files**
- **Commands actually executed** (not assumed)
- **Tests actually executed** (not merely imagined)
- **Data** — observed, simulated or derived
- **Known limitations**
- **Provenance** — evidence of origin
- **Responsible human reviewer**

## Claims forbidden without proof

Without reproducible evidence, **do not** use these terms:

- `Validated`
- `Calibrated`
- `Reproduced`
- `Safe`
- `No regression`
- `Clinically relevant`
- `Experimentally confirmed`

## Evidence-status vocabulary (mandatory)

When describing a result, label its status. It applies to **humans and agents**:

| Label | Meaning |
|---|---|
| `[OBSERVED]` | Something seen/measured directly |
| `[EXECUTED]` | A command/routine was actually run |
| `[DERIVED]` | Result computed from other sources |
| `[SIMULATED]` | Produced by simulation (never as empirical evidence) |
| `[PREDICTED]` | Anticipated before observation |
| `[PROPOSED]` | Suggestion, not yet tested |
| `[UNVERIFIED]` | Not verified |
| `[BLOCKED]` | Prevented from moving forward |

## MR template (summary)

Reinforce with the full template in `.gitlab/merge_request_templates/`:

```markdown
## Contribution type
- [ ] Code (kernel repo) | [ ] Documentation | [ ] Reproduction
- [ ] Research | [ ] Security | [ ] Visualization
- [ ] Generated or assisted by agent

## Scope
What this MR changes and what it does not.

## Evidence
| Claim | Status | Artifact | Command | Result |
|---|---|---|---|---|
| ... | [EXECUTED] | ... | ... | ... |

## Data and provenance
- Observed data: ...
- Derived data: ...
- Simulated data: ...
- Unverified data: ...

## Risks and limits
- ...

## Checklist
- [ ] No secret or token
- [ ] Does not modify a kernel module without specialized review
- [ ] Does not call simulation observed evidence
- [ ] Includes documentation update (PT + EN)
- [ ] Includes a verifiable next step
```

## Operational limits

- **No AI carrier** has unrestricted merge power (see `GOVERNANCE.md`).
- Prefer what is **reproducible** over what is merely **asserted**.
- When presenting AI-assisted implementation, never present partial work as a verified conclusion without the labels above.

## Public / Private

Never publish, in an MR, content that violates the public/private boundary (`SECURITY.md`): tokens, credentials, internal IPs, local absolute paths, subject logs, internal infrastructure.
