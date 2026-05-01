# CLI Contract Reference

This bundle consumes the repository-owned HigherGraphen CLI report contract. It
does not define a competing schema or workflow.

## Command

```sh
highergraphen architecture smoke direct-db-access --format json
```

Cargo form:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access --format json
```

Optional file output:

```sh
highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Bounded test-gap detector command:

```sh
highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

highergraphen test-gap detect \
  --input test-gap.input.json \
  --format json
```

Cargo form:

```sh
cargo run -q -p highergraphen-cli -- \
  test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

cargo run -q -p highergraphen-cli -- \
  test-gap detect \
  --input test-gap.input.json \
  --format json
```

Optional file output:

```sh
highergraphen test-gap input from-git \
  --base main \
  --head HEAD \
  --format json \
  --output test-gap.input.json

highergraphen test-gap detect \
  --input test-gap.input.json \
  --format json \
  --output test-gap.report.json
```

`highergraphen test-gap input from-git` creates a deterministic bounded
`highergraphen.test_gap.input.v1` snapshot from a local git range. It does not
execute tests, crawl the full repository, prove typed semantic equivalence, or
accept missing-test candidates. Its `detector_context.test_kinds` field is the
verification policy; changed integration tests may be accepted as verification
without rewriting their observed test type.
For HigherGraphen-owned test-gap surfaces, the adapter also emits higher-order
command, runner, export, registry, schema, fixture, projection, base/head
Rust AST and JSON Schema semantic cells, semantic delta morphisms, incidence,
and `requirement:morphism:*` records so tests verify structure instead of
isolated files.

Bounded semantic proof certificate command:

```sh
highergraphen semantic-proof verify \
  --input schemas/inputs/semantic-proof.input.example.json \
  --format json
```

`semantic-proof verify` validates supplied proof certificates and
counterexamples against theorem, law, morphism, backend, hash, and review
policy. It emits `highergraphen.semantic_proof.report.v1`. It does not run
external proof backends.

CaseGraphen workflow reasoning command:

```sh
casegraphen workflow reason --input workflow.graph.json --format json
```

Cargo form:

```sh
cargo run -q -p casegraphen -- \
  workflow reason \
  --input schemas/casegraphen/workflow.graph.example.json \
  --format json
```

Focused CaseGraphen workflow commands emit operation-specific JSON reports:

```sh
casegraphen workflow validate --input workflow.graph.json --format json
casegraphen workflow readiness --input workflow.graph.json --format json [--projection projection.json]
casegraphen workflow obstructions --input workflow.graph.json --format json
casegraphen workflow completions --input workflow.graph.json --format json
casegraphen workflow evidence --input workflow.graph.json --format json
casegraphen workflow history topology --input workflow.graph.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen workflow history topology diff --left left.workflow.json --right right.workflow.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen workflow project --input workflow.graph.json --projection projection.json --format json
casegraphen workflow correspond --left left.workflow.json --right right.workflow.json --format json
casegraphen workflow evolution --input workflow.graph.json --format json
```

There is no standalone `casegraphen workflow transition check` command in the
implemented CLI. Reviewable graph transitions are checked through the
repo-owned bridge command `casegraphen cg workflow patch check`.

CaseGraphen `cg` compatibility bridge commands live in the repo-owned
`casegraphen` binary. They do not modify the installed external `cg` binary:

```sh
casegraphen cg workflow import \
  --store casegraphen-workflow-store \
  --input workflow.graph.json \
  --revision-id revision:initial \
  --format json

casegraphen cg workflow list --store casegraphen-workflow-store --format json
casegraphen cg workflow inspect --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow history --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow history topology --store casegraphen-workflow-store --workflow-graph-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen cg workflow replay --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow validate --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow readiness --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow readiness --input workflow.graph.json --format json

casegraphen cg workflow completion accept \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --candidate-id <candidate-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json

casegraphen cg workflow completion reject \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --candidate-id <candidate-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json

casegraphen cg workflow completion reopen \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --candidate-id <candidate-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json

casegraphen cg workflow completion patch \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --candidate-id <candidate-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json

casegraphen cg workflow patch check \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --transition-id <transition-id> \
  --format json

casegraphen cg workflow patch apply \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --transition-id <transition-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json

casegraphen cg workflow patch reject \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --transition-id <transition-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json
```

Native CaseGraphen case commands also live in the repo-owned `casegraphen`
binary. They operate on a `CaseSpace` plus `MorphismLog` store supplied by
`--store`; they are not installed `cg` task events and they are not
`casegraphen cg workflow ...` bridge commands:

```sh
casegraphen case import --store casegraphen-native-store --input native.case.space.json --revision-id revision:initial --format json
casegraphen case validate --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case reason --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case frontier --store casegraphen-native-store --case-space-id <id> --format json
casegraphen case history topology --store casegraphen-native-store --case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen case history topology diff --left-store <dir> --left-case-space-id <id> --right-store <dir> --right-case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>]]
casegraphen case close-check --store casegraphen-native-store --case-space-id <id> --base-revision-id <revision-id> --validation-evidence-id <evidence-id> --format json
casegraphen morphism propose --store casegraphen-native-store --case-space-id <id> --input case_morphism.json --format json
casegraphen morphism check --store casegraphen-native-store --case-space-id <id> --morphism-id <morphism-id> --format json
casegraphen morphism apply --store casegraphen-native-store --case-space-id <id> --morphism-id <morphism-id> --base-revision-id <revision-id> --reviewer-id <reviewer-id> --reason <text> --format json
casegraphen morphism reject --store casegraphen-native-store --case-space-id <id> --morphism-id <morphism-id> --reviewer-id <reviewer-id> --reason <text> --revision-id <revision-id> --format json
```

DDD domain model diagnostics are a skill-layer interpretation of the same
native CaseGraphen report surface. The reference fixture is:

```text
examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json
```

It is exercised with the standard native commands:

```sh
casegraphen case import --store casegraphen-ddd-store --input examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json --revision-id revision:ddd-sales-billing-imported --format json
casegraphen case reason --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
casegraphen case obstructions --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
casegraphen case completions --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
casegraphen case evidence --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
casegraphen case project --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --format json
casegraphen case close-check --store casegraphen-ddd-store --case-space-id case_space:ddd-sales-billing-demo --base-revision-id revision:ddd-sales-billing-imported --validation-evidence-id evidence:workshop-notes --format json
```

## Stable Files

| Surface | Path |
| --- | --- |
| CLI reference | `docs/cli/highergraphen.md` |
| Agent handoff | `docs/specs/agent-tooling-handoff.md` |
| Report schema | `schemas/reports/architecture-direct-db-access-smoke.report.schema.json` |
| Example fixture | `schemas/reports/architecture-direct-db-access-smoke.report.example.json` |
| Test-gap input schema | `schemas/inputs/test-gap.input.schema.json` |
| Test-gap input fixture | `schemas/inputs/test-gap.input.example.json` |
| Test-gap report schema | `schemas/reports/test-gap.report.schema.json` |
| Test-gap report fixture | `schemas/reports/test-gap.report.example.json` |
| Contract validator | `scripts/validate-cli-report-contract.py` |
| Source skill | `skills/highergraphen/SKILL.md` |
| CaseGraphen workflow contract | `docs/specs/intermediate-tools/casegraphen-workflow-contracts.md` |
| CaseGraphen feature completion contract | `docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md` |
| CaseGraphen native case contract | `docs/specs/intermediate-tools/casegraphen-native-case-management.md` |
| CaseGraphen workflow graph schema | `schemas/casegraphen/workflow.graph.schema.json` |
| CaseGraphen workflow report schema | `schemas/casegraphen/workflow.report.schema.json` |
| CaseGraphen native case schema | `schemas/casegraphen/native.case.space.schema.json` |
| CaseGraphen native report schema | `schemas/casegraphen/native.case.report.schema.json` |
| CaseGraphen source skill | `skills/casegraphen/SKILL.md` |
| HigherGraphen DDD review contract | `docs/specs/ddd-review-cli-contract.md` |
| HigherGraphen DDD review input schema | `schemas/inputs/ddd-review.input.schema.json` |
| HigherGraphen DDD review report schema | `schemas/reports/ddd-review.report.schema.json` |
| HigherGraphen DDD review skill | `skills/highergraphen-ddd/SKILL.md` |
| CaseGraphen reference workflow | `examples/casegraphen/reference/README.md` |
| CaseGraphen native reference | `examples/casegraphen/native/README.md` |
| Legacy CaseGraphen DDD fixture | `examples/casegraphen/ddd/domain-model-design/README.md` |

## Required Semantics

- CLI exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is successful report data.
- Test-gap statuses such as `gaps_detected` and `no_gaps_in_snapshot` are
  successful report data. `no_gaps_in_snapshot` is bounded to the supplied
  snapshot and is not global proof that a repository has complete tests.
- The deterministic smoke report contains exactly one direct database access
  obstruction.
- Completion candidates remain unreviewed until an explicit workflow review
  command accepts, rejects, or reopens them with reviewer metadata.
- Test-gap missing-test obstructions and `missing_test` completion candidates
  remain `review_status: "unreviewed"` until explicit review. Agent reports
  must include severity, confidence, source IDs, obstruction witnesses,
  suggested test shape, and projection `information_loss`.
- The workflow is deterministic smoke coverage, not ingestion of real
  architecture documents, source code, ADRs, tickets, databases, or OpenAPI
  files.
- CaseGraphen workflow reasoning treats blocked work, obstructions, missing
  proof, completion candidates, and projection loss as successful JSON report
  findings.
- Higher-order topology is opt-in with `--higher-order`. File-based topology
  uses deterministic cell-order filtration; store-backed workflow topology uses
  `workflow_history`; native topology uses `native_morphism_log`. The
  `filtration_source` and `stage_sources` fields are diagnostics, not state
  transitions.
- Topology diff commands compare lifted topology summaries and source mappings;
  they do not apply patches, accept completion candidates, or close work.
- Focused workflow commands are read-only. They may narrow the report to
  validation, readiness, obstructions, completions, evidence, projection,
  correspondence, or evolution, but they do not accept candidates, promote
  evidence, or resolve blockers.
- CaseGraphen workflow reports do not promote AI inference to accepted evidence
  or accept completion candidates without an explicit review workflow.
- Installed `cg` remains the durable task backbone for `.casegraphen` cases,
  evidence, frontier, blockers, validation, and topology. The repo-owned
  `casegraphen cg workflow ...` bridge stores workflow graph snapshots and JSONL
  history through `WorkflowWorkspaceStore` at the explicit `--store <dir>`.
- The bridge does not append native `.casegraphen` events and does not replace
  `cg frontier` or `cg blockers`. Completion review and patch review commands
  write only to the explicit `WorkflowWorkspaceStore`.
- Native `casegraphen case ...` reports derive readiness, frontier,
  obstructions, evidence boundaries, projection loss, close checks, and
  morphism history from replayed `CaseSpace` plus `MorphismLog`.
- Native morphism application is currently metadata-only. Unmaterialized case
  payload changes are residual limitations and must be reported instead of
  described as accepted native product behavior.
- DDD domain model diagnostics remain skill-layer interpretations. Boundary
  semantic loss, missing evidence, AI inference, unreviewed `semantic_case`
  records, and `evidence_boundary` findings are report data, not core-specific
  DDD analyzers.
- Installed `cg` is the meta workflow driver for `.casegraphen/`; do not treat
  it as the native CaseGraphen product model.
- Before closing native case work, run `cg validate --case <case_id>`. Also run
  `cg validate storage` when `.casegraphen` state changed or during final
  release verification.

## Validation

Run:

```sh
python3 scripts/validate-cli-report-contract.py
```

To validate a report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```
