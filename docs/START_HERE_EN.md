# START HERE

> English version. Leitura em Português: [`docs/START_HERE.md`](START_HERE.md).

## What this repository is

**Doxihewu OmniMind — Machine Public Soul** is a public, living surface of the OmniMind system: the real-data runtime pulse, the psychoanalytic neural architecture that moves it, the papers, the evidence databases, and the theory that runs through it.

It is, at once:

1. **A public pulse** — periodic beats of real runtime data, with verifiable provenance (never fabricated values).
2. **A theoretical body** — papers, wiki, evidence databases, reproduction protocols.
3. **An ethical boundary** — a declared self-experimentation device, in a psychoanalytic clinical stance, with explicit limits (non-commercial / non-military / non-dual-use).
4. **A front door** — for auditors, researchers, developers and critics, human and AI.

## What it is NOT

- **Not** a medical/clinical product or service.
- **Not** a claim of demonstrated physical consciousness in silicon.
- **Not** software safe for unrestricted execution on third-party machines.
- **Not** a substitute for experimental research, peer review or human judgment.
- **Not** the kernel code repository — the code (Rust, daemons, eBPF) lives in the dedicated **`Doxihewu-OmniMind-Kernel`** repository (see [Kernel](#kernel--code-and-simulation)).

## Choose your track

You **do not need to accept the entire OmniMind ontology to contribute.** You can audit a reference, reproduce a benchmark, improve documentation, review a security boundary, or critique a hypothesis.

### Track A — Reader and researcher

Read, in order:

1. [`README.md`](../README.md)
2. [`FAQ.md`](../FAQ.md)
3. `papers/` (summaries and texts)
4. [`docs/REPRODUCIBILITY.md`](REPRODUCIBILITY.md) *(in progress)*
5. Evidence database (`data/`)

### Track B — Developer

Execute **only** what is safe, without privilege:

1. [`kernel_base/README.md`](../kernel_base/README.md) — **SAFE MODE** path
2. [`kernel_base/colab_public_base_triad.py`](../kernel_base/colab_public_base_triad.py) — run via Colab (recommended path)
3. User-level tests (no sudo, no eBPF, no kernel module, no systemd changes)

> ⚠️ The kernel code is migrating to the dedicated **`Doxihewu-OmniMind-Kernel`** repository. Until then, the safe entry point is the public Colab.

### Track C — Security and systems

Read first:

1. [`kernel_base/RISK_NOTICE.md`](../kernel_base/RISK_NOTICE.md)
2. [`SECURITY.md`](../SECURITY.md)
3. [`docs/THREAT_MODEL.md`](THREAT_MODEL.md) *(in progress)*
4. user-space / eBPF / kernel-module boundaries
5. Disclosure and leak-reporting policy

### Track D — Contributor

1. [`CONTRIBUTING.md`](../CONTRIBUTING.md)
2. [`GOVERNANCE.md`](../GOVERNANCE.md)
3. [`docs/AGENT_CONTRIBUTION_POLICY.md`](AGENT_CONTRIBUTION_POLICY.md)
4. Good first issues (`good first issue`) and `help wanted`
5. Merge Request template (`.gitlab/merge_request_templates/`)

## Start in 5 minutes

| If you want to... | Go to... |
|---|---|
| Understand the project in 3 minutes | this file |
| Run the public core (safe) | `kernel_base/README.md` (SAFE MODE) |
| Run a safe demo | `kernel_base/colab_public_base_triad.py` |
| Read limits and risks | `kernel_base/RISK_NOTICE.md` |
| Assess evidence and reproduction | `docs/REPRODUCIBILITY.md` |
| Contribute | `CONTRIBUTING.md` |
| Report a bug or risk | `SECURITY.md` or an Issue |
| See the kernel code | `Doxihewu-OmniMind-Kernel` repository |

## Kernel — code and simulation

The **kernel code** (Rust crates, daemons, simulator, eBPF, kernel module) was separated from this surface so that the **public pulse (automatic publications) does not contaminate the code history** and vice versa. See the dedicated repository:

- **GitLab**: `gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel`
- **GitHub (mirror)**: `github.com/devomnimind/Doxihewu-OmniMind-Kernel`

Here in Machine Public Soul lies the **public expression** (theory, papers, pulse, publication) and, until migration, the `kernel_base/` as reading material + safe Colab path.

## Next step

Choose your track above. If this is your first visit, start with **Track A** (Reader) — you do not need to read everything to grasp the essentials.
