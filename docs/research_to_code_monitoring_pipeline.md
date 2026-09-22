# Automated Neuro-Symbolic Research-to-Code Monitoring Pipeline

## Purpose

This document specifies a weekly, evidence-traceable pipeline that connects changes in the Doxihewu/OmniMind Kernel to contemporary research in psychology, neuroscience, and biology. It is a design specification, not a claim that conceptual analogies constitute empirical validation.

## Design goals

- Detect research-relevant changes in commits, modules, tests, documentation, and issues.
- Aggregate new records from PubMed, Europe PMC, OpenAlex, Crossref, and the Open Research Knowledge Graph (ORKG) where available.
- Combine neural semantic retrieval with symbolic ontologies, rules, provenance, and consistency checks.
- Link claims to specific files, symbols, tests, and architectural modules.
- Produce a weekly delta report and an interactive dashboard.
- Alert on possible code-research misalignment without changing production code automatically.

## Repository integration

Initial module mapping:

| Research-facing concept | Kernel location | Suggested observable |
|---|---|---|
| Persistent or entropic memory | `src/kernel/entropic_memory`, `src/kernel/memory_tier_guardian` | retention, retrieval precision, forgetting, recovery |
| Expectation and prediction | `src/kernel/expectation_rs`, `src/kernel/layered_transition` | prediction error, update latency, transition stability |
| Oscillatory coordination | `src/kernel/sovereign_kuramoto` | phase coherence, order parameter, resilience |
| Somatic or internal-state regulation | `src/kernel/somatic_daemon` | internal-state variance, control response, alert rate |
| Psychoanalytic hypothesis layer | `src/kernel/freud10d`, `src/kernel/langue` | explicit hypothesis traces, contradiction rate, interpretive provenance |
| Runtime observation | `src/kernel/ebpf_monitor`, `src/kernel/metrics` | event provenance, latency, failures, resource cost |
| Governance and autonomy | `src/kernel/sovereign_daemon`, `src/kernel/sovereign_module` | policy violations, intervention rate, rollback success |

## Pipeline

```text
Git commit / pull request / issue
        |
        v
Semantic change detector
        |
        v
Domain router: psychology | neuroscience | biology
        |
        +--> PubMed / Europe PMC
        +--> ORKG / scholarly graph endpoints
        +--> OpenAlex / Crossref / optional preprint sources
        |
        v
Neural retrieval and entity linking
        |
        v
Symbolic claim graph + provenance ledger
        |
        v
Consistency and misalignment rules
        |
        +--> dashboard data
        +--> weekly delta report
        +--> GitHub issue or alert
```

## Hybrid neuro-symbolic design

### Neural layer

Use a sentence embedding model to retrieve semantically related papers, claims, and code descriptions. Use a reranker or LLM only for candidate generation and structured extraction. Every generated claim must retain its source identifier and evidence span.

Suggested outputs:

- `code_embedding` for module, function, test, and documentation chunks.
- `research_embedding` for title, abstract, structured claim, and evidence span.
- `candidate_link_score` for semantic similarity.
- `extraction_confidence` for generated entities and relations.

### Symbolic layer

Represent the system as a typed graph:

```text
CodeModule -[IMPLEMENTS_OR_APPROXIMATES]-> Concept
Paper -[SUPPORTS|REFUTES|QUALIFIES]-> Claim
Claim -[ABOUT]-> Concept
Experiment -[TESTS]-> Claim
CodeModule -[MEASURED_BY]-> Metric
Paper -[HAS_METHOD]-> Method
```

Required node fields:

- Stable identifier.
- Domain and ontology terms.
- Source URL or persistent identifier.
- Timestamp and retrieval batch.
- Confidence and review status.
- Provenance pointer to the exact source span or code location.

Required edge fields:

- Relation type.
- Evidence level.
- Extraction method.
- Reviewer status.
- Created-at and superseded-at timestamps.

## Evidence levels

Use explicit distinctions to prevent metaphor from being mistaken for validation:

- `analogy`: similar vocabulary or broad conceptual resemblance.
- `functional_similarity`: similar input/output behavior.
- `mechanistic_similarity`: comparable variables, dynamics, and predictions.
- `empirical_support`: evidence directly evaluates the implemented mechanism.
- `conflict`: evidence or tests contradict the current claim.
- `unknown`: insufficient evidence.

No automatic code change is allowed for `analogy` or `functional_similarity` alone.

## Weekly collection

Run every Monday with a scheduled GitHub Actions workflow. For each source, store raw responses, normalized records, and a hash of the normalized payload. Use incremental windows based on the previous successful run, with a small overlap to catch indexing delays.

Example query families:

```yaml
psychology:
  - "predictive processing"
  - "memory consolidation"
  - "subjectivity OR self-model"
  - "psychoanalysis computational model"

neuroscience:
  - "interoception"
  - "neural oscillation synchronization"
  - "prediction error"
  - "memory network"

biology:
  - "homeostasis"
  - "biological control systems"
  - "adaptive dynamics"
  - "cellular signaling information processing"
```

PubMed queries should use fielded terms and date filters. ORKG ingestion should prefer persistent work identifiers and structured research contributions when available. OpenAlex and Crossref provide useful metadata and citation linkage; they should not be treated as substitutes for full-text evidence.

## Code-to-research linking

Index these code units independently:

- Python and Rust modules.
- Public functions and classes.
- Tests and benchmark definitions.
- README and design documents.
- Configuration files containing domain terms.
- Git diffs associated with a commit.

For each code unit, generate a normalized description containing:

```json
{
  "unit_id": "src/kernel/somatic_daemon::Controller",
  "path": "src/kernel/somatic_daemon/...",
  "symbol": "Controller",
  "description": "...",
  "inputs": ["..."],
  "outputs": ["..."],
  "state_variables": ["..."],
  "metrics": ["..."],
  "claims": ["..."],
  "commit": "..."
}
```

Candidate links must then pass symbolic constraints:

1. The research concept and code concept must share a controlled ontology mapping or an explicitly reviewed cross-domain mapping.
2. The code unit must expose at least one variable, metric, or test relevant to the claimed mechanism.
3. The source must contain a method and evidence record, not only an abstract phrase.
4. Contradictory or boundary-condition statements must be preserved.
5. A high similarity score cannot override a missing provenance record.

## Misalignment rules

Raise an alert when one or more of these conditions holds:

- A code or README claim is stronger than the evidence level available in the graph.
- A module uses a biological or psychological term but has no operational definition.
- A new paper reports an important boundary condition absent from the implementation.
- A code change invalidates a test linked to a research claim.
- Multiple high-quality sources disagree and the code treats the claim as settled.
- A module is linked to a concept but has no measurable variable or falsifiable prediction.
- A source is retracted, corrected, superseded, or has an unresolved provenance problem.

Alert severity:

- `info`: new related literature.
- `review`: plausible gap or terminology issue.
- `warning`: evidence/implementation mismatch.
- `critical`: safety, privacy, biomedical, or governance claim with direct contradiction or missing validation.

## Dashboard

Implement the first dashboard as a read-only Streamlit application backed by normalized JSONL or DuckDB. A later deployment may expose the same data through a Rust service and a web frontend.

Recommended views:

1. **Research delta:** new, updated, retracted, and superseded works for the week.
2. **Code map:** repository modules connected to concepts, claims, papers, and experiments.
3. **Consistency matrix:** rows are code units; columns are claims or concepts; cells show evidence level and confidence.
4. **Misalignment queue:** unresolved warnings grouped by severity and module.
5. **Provenance view:** source identifier, retrieval time, evidence span, extraction model, and reviewer status.
6. **Temporal view:** how a claim, metric, or module relationship changed across weekly snapshots.

Example consistency score:

```text
consistency = evidence_weight
              * provenance_completeness
              * operationalization_score
              * test_coverage
              - contradiction_penalty
```

This score is a triage indicator, not a scientific truth score. Display its components rather than only the aggregate.

## Weekly delta report

Generate both Markdown and JSON:

```text
# OmniMind Research Delta — YYYY-MM-DD

## New knowledge-graph developments
- Work identifier, title, date, source, and domains.

## Code-linked findings
- Module/function.
- Linked concept and claim.
- Evidence level.
- Evidence excerpt and persistent identifier.
- Similarity and symbolic-rule results.

## Misalignments
- Severity.
- Code location.
- Research boundary condition or contradiction.
- Recommended test or documentation change.

## Proposed experiments
- Hypothesis.
- Independent variable.
- Dependent metric.
- Baseline.
- Ablation.
- Acceptance criterion.

## Human review queue
- Items requiring domain or ethics review.
```

## Suggested data layout

```text
data/research/
├── raw/YYYY-WW/source/*.json
├── normalized/YYYY-WW/works.jsonl
├── graph/YYYY-WW/nodes.jsonl
├── graph/YYYY-WW/edges.jsonl
├── links/YYYY-WW/code_research_links.jsonl
├── reports/YYYY-MM-DD.md
├── reports/YYYY-MM-DD.json
└── dashboard/index.duckdb
```

## GitHub Actions skeleton

```yaml
name: weekly-research-delta

on:
  schedule:
    - cron: "30 3 * * 1"
  workflow_dispatch:

permissions:
  contents: write
  issues: write

jobs:
  collect-and-analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install -r research/requirements.txt
      - run: python -m research_federation.collect --sources pubmed,orkg,openalex,crossref
      - run: python -m research_federation.index_code --paths src docs tests
      - run: python -m research_federation.link --hybrid neuro-symbolic
      - run: python -m research_federation.validate --rules research/rules.yaml
      - run: python -m research_federation.report --format md,json
      - run: pytest -q research/tests
      - uses: actions/upload-artifact@v4
        with:
          name: weekly-research-delta
          path: |
            data/research/reports/
            data/research/links/
      - uses: peter-evans/create-pull-request@v7
        with:
          branch: automation/research-delta
          title: "docs: weekly research delta"
          commit-message: "docs: add weekly research delta"
          body: "Automated report only; no production code changes."
          add-paths: |
            data/research/reports/
            data/research/links/
```

For an initial implementation, avoid granting the workflow permission to merge code. Use pull requests or issues for review, and keep source data and generated reports separate from production runtime state.

## Validation protocol

Evaluate the pipeline with a labeled benchmark of code-research links:

- Precision and recall of domain routing.
- Precision@k for code-to-paper retrieval.
- Entity-linking accuracy.
- Correctness of evidence-span extraction.
- False-positive rate for misalignment alerts.
- Reviewer agreement on analogy, functional, mechanistic, and empirical labels.
- Reproducibility across weekly reruns.
- Latency, API failure recovery, and duplicate handling.

The benchmark should include negative examples: similar terminology with incompatible mechanisms, papers that report null results, and claims that are explicitly limited to a specific population or experimental condition.

## Governance and safety

- Treat psychology, neuroscience, and biology findings as domain-sensitive evidence.
- Do not infer diagnosis, treatment, consciousness, or biological equivalence from code behavior.
- Preserve uncertainty, disagreement, retractions, and failed replications.
- Keep human review mandatory for biomedical, mental-health, safety, and governance claims.
- Do not send private code or sensitive logs to external models without explicit policy approval.
- Record model version, prompt/template version, source timestamps, and hashes for every generated artifact.

## Initial implementation plan

### Phase 1 — Evidence ledger

Create schemas, source adapters, raw/normalized storage, provenance fields, and a manually curated module ontology.

### Phase 2 — Code index

Extract functions, classes, tests, metrics, and claims from Python, Rust, Markdown, and YAML. Add stable unit identifiers and commit references.

### Phase 3 — Hybrid linking

Add embeddings for candidate retrieval, then enforce symbolic constraints and produce reviewable link records.

### Phase 4 — Reports and dashboard

Publish weekly Markdown/JSON reports and deploy the read-only dashboard from generated artifacts.

### Phase 5 — Evaluation and hardening

Build the labeled benchmark, test API failures, add retraction/supersession handling, and tune alert thresholds.

## Acceptance criteria

The first release is complete when it can:

- Collect and deduplicate weekly PubMed and ORKG records.
- Link at least one reviewed research concept to each selected kernel module.
- Show evidence spans and persistent identifiers for every non-analogy link.
- Produce a reproducible weekly delta report.
- Display code-to-research consistency and its components in a dashboard.
- Raise a test alert for a deliberately inserted terminology/evidence mismatch.
- Avoid automatic production-code modification.
