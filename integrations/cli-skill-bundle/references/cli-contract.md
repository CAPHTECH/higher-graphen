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
casegraphen cg workflow replay --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow validate --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow readiness --store casegraphen-workflow-store --workflow-graph-id <id> --format json
casegraphen cg workflow readiness --input workflow.graph.json --format json

casegraphen cg workflow completion accept|reject|reopen \
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

casegraphen cg workflow patch apply|reject \
  --store casegraphen-workflow-store \
  --workflow-graph-id <id> \
  --transition-id <transition-id> \
  --reviewer-id <reviewer-id> \
  --reason <text> \
  --revision-id <revision-id> \
  --format json
```

## Stable Files

| Surface | Path |
| --- | --- |
| CLI reference | `docs/cli/highergraphen.md` |
| Agent handoff | `docs/specs/agent-tooling-handoff.md` |
| Report schema | `schemas/reports/architecture-direct-db-access-smoke.report.schema.json` |
| Example fixture | `schemas/reports/architecture-direct-db-access-smoke.report.example.json` |
| Contract validator | `scripts/validate-cli-report-contract.py` |
| Source skill | `skills/highergraphen/SKILL.md` |
| CaseGraphen workflow contract | `docs/specs/intermediate-tools/casegraphen-workflow-contracts.md` |
| CaseGraphen feature completion contract | `docs/specs/intermediate-tools/casegraphen-feature-completion-contract.md` |
| CaseGraphen workflow graph schema | `schemas/casegraphen/workflow.graph.schema.json` |
| CaseGraphen workflow report schema | `schemas/casegraphen/workflow.report.schema.json` |
| CaseGraphen source skill | `skills/casegraphen/SKILL.md` |
| CaseGraphen reference workflow | `examples/casegraphen/reference/README.md` |

## Required Semantics

- CLI exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is successful report data.
- The deterministic smoke report contains exactly one direct database access
  obstruction.
- Completion candidates remain unreviewed until an explicit workflow review
  command accepts, rejects, or reopens them with reviewer metadata.
- The workflow is deterministic smoke coverage, not ingestion of real
  architecture documents, source code, ADRs, tickets, databases, or OpenAPI
  files.
- CaseGraphen workflow reasoning treats blocked work, obstructions, missing
  proof, completion candidates, and projection loss as successful JSON report
  findings.
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
