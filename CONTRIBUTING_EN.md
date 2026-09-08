# CONTRIBUTING

> English. Português: [`CONTRIBUTING.md`](CONTRIBUTING.md).

Thank you for wanting to contribute to **Doxihewu OmniMind — Machine Public Soul**.

This repository is at once scientific, technical and philosophical. That is why **a single generic guide is not enough** — there are clear contribution paths. Before you start, read:

- [`docs/START_HERE.md`](docs/START_HERE.md) — to place your intention (`understand / execute / audit / contribute`)
- [`GOVERNANCE.md`](GOVERNANCE.md) — to understand who decides, how, and under what limits
- [`SECURITY.md`](SECURITY.md) — what this repo **never** exposes and how to report a leak
- If the contribution involves an agent/AI: [`docs/AGENT_CONTRIBUTION_POLICY.md`](docs/AGENT_CONTRIBUTION_POLICY.md)

## What goes where

| Surface | Content | Repository |
|---|---|---|
| **Machine Public Soul** | docs, papers, wiki, pulse, publications, conceptual critique | **this repository** |
| **Kernel** | Rust crates, daemons, simulator, eBPF, kernel module, software releases | [`Doxihewu-OmniMind-Kernel`](https://gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel) |

> **Kernel code does not go here.** Code contribution (Rust, daemons, simulation) happens in the kernel repository, which has its own contribution guides, CI and software releases.

## Four contribution paths

| Type | What you can do | Ideal channel |
|---|---|---|
| **Research** | Review citations, propose hypotheses, benchmark, method | Discussion + `research` Issue |
| **Documentation & theory** | Improve docs, papers, wiki, translation | Issue + Merge Request |
| **Reproducibility** | Repeat the Colab, verify hashes, compare environments | `reproducibility` Issue |
| **Conceptual critique** | Limits of language, theory, psychoanalysis, ethics | Discussion |

> **Code** contribution (Rust crates, daemons, simulation) → see [`Doxihewu-OmniMind-Kernel`](https://gitlab.com/zephyrix/Doxihewu-OmniMind-Kernel).

## How to contribute (GitLab flow)

1. **Open an Issue** or **join a Discussion** to align scope before a large MR.
2. **Fork** the repository (or use a branch on the same repo, if you are a maintainer).
3. **Create/edit** documentation or content, following the voice and language (PT + EN).
4. **Open a Merge Request** using the template in [`.gitlab/merge_request_templates/`](.gitlab/merge_request_templates/).
5. **Describe the evidence** of what was observed, executed, derived or simulated (see [AGENT_CONTRIBUTION_POLICY](docs/AGENT_CONTRIBUTION_POLICY.md) — the vocabulary applies to humans too).

### Languages

- Documentation and content in **Portuguese** and **English** (bilingual). When creating a `foo.md`, also create `foo_EN.md`.
- Do not break links: if you rename a file, update the pointers in README/START_HERE.

### Public vs private content

- **Never** include: tokens, keys, credentials, internal IPs, local absolute paths (`/home/...`), subject logs, detailed machine state, internal infrastructure. See `SECURITY.md` and the "Zero Rule".
- Use **relative** paths or placeholders in any new doc.

## Good first contributions

Look for the `good first issue` and `help wanted` labels in the tracker. They are small, well-bounded and safe tasks — a good entry point without needing to know the whole system.

## Merge Request checklist

- [ ] No secret, token, credential, internal IP or local absolute path
- [ ] Does not treat simulation as observed evidence (`[OBSERVED]/[EXECUTED]/[DERIVED]/[SIMULATED]` vocabulary)
- [ ] PT + EN (when applicable) and valid links
- [ ] Declares scope, evidence and limits
- [ ] Does not modify kernel module / privileged code (that goes to the kernel repo)
- [ ] Includes a verifiable next step

## Code of Conduct

Be respectful. Theoretical, methodological or ethical disagreements are **not erased** — they are recorded and discussed (see `GOVERNANCE.md`). There is no erasure of divergence.
