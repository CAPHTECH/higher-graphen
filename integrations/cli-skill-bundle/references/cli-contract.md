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
| CaseGraphen workflow graph schema | `schemas/casegraphen/workflow.graph.schema.json` |
| CaseGraphen workflow report schema | `schemas/casegraphen/workflow.report.schema.json` |
| CaseGraphen source skill | `skills/casegraphen/SKILL.md` |

## Required Semantics

- CLI exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is successful report data.
- The deterministic smoke report contains exactly one direct database access
  obstruction.
- The billing status API remains an unreviewed completion candidate until a
  later explicit review workflow accepts or rejects it.
- The workflow is deterministic smoke coverage, not ingestion of real
  architecture documents, source code, ADRs, tickets, databases, or OpenAPI
  files.
- CaseGraphen workflow reasoning treats blocked work, obstructions, missing
  proof, completion candidates, and projection loss as successful JSON report
  findings.
- CaseGraphen workflow reports do not promote AI inference to accepted evidence
  or accept completion candidates without an explicit review workflow.

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
