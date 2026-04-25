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

## Stable Files

| Surface | Path |
| --- | --- |
| CLI reference | `docs/cli/highergraphen.md` |
| Agent handoff | `docs/specs/agent-tooling-handoff.md` |
| Report schema | `schemas/reports/architecture-direct-db-access-smoke.report.schema.json` |
| Example fixture | `schemas/reports/architecture-direct-db-access-smoke.report.example.json` |
| Contract validator | `scripts/validate-cli-report-contract.py` |
| Source skill | `skills/highergraphen/SKILL.md` |

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

