---
name: highergraphen
description: Use when an agent needs to run or interpret the repository-owned HigherGraphen CLI Architecture Product smoke workflow and its JSON report contract.
---

# HigherGraphen CLI Skill

Use this skill when a task asks for HigherGraphen agent-facing workflow output,
Architecture Product smoke validation, or interpretation of the
`highergraphen architecture smoke direct-db-access` report.

This repository skill is CLI-only. MCP servers, provider plugin bundles,
marketplace metadata, and provider-specific manifests are outside the immediate
path.

## Source Of Truth

- CLI reference: `docs/cli/highergraphen.md`
- Agent handoff: `docs/specs/agent-tooling-handoff.md`
- Report schema: `schemas/reports/architecture-direct-db-access-smoke.report.schema.json`
- Example report: `schemas/reports/architecture-direct-db-access-smoke.report.example.json`
- Local contract validator: `scripts/validate-cli-report-contract.py`

Do not restate the report schema as a competing contract. Consume the schema,
fixture, and CLI output.

## When To Run The CLI

Run the CLI when the user asks for the current Architecture Product smoke
workflow, a direct database access architecture report, or proof that the first
HigherGraphen agent-facing report contract still works.

Preferred local validation:

```sh
python3 scripts/validate-cli-report-contract.py
```

Generate the report to stdout:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access --format json
```

Generate the report to a file:

```sh
cargo run -q -p highergraphen-cli -- \
  architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Validate an existing report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```

## Interpretation Rules

- Exit code `0` means the workflow ran and emitted a report.
- `result.status == "violation_detected"` is a successful domain finding, not
  a failed CLI run.
- The report should contain exactly one direct database access obstruction for
  `obstruction:order-service-direct-billing-db-access`.
- The suggested billing status API is a completion candidate, not accepted
  structure.
- Preserve `review_status: "unreviewed"` for the obstruction provenance and the
  completion candidate unless a later explicit review workflow accepts or
  rejects it.
- Present `projection.recommended_actions` as recommendations, and keep
  `projection.information_loss` visible in summaries.
- State that this workflow is deterministic smoke coverage, not full ingestion
  of real architecture documents, source code, ADRs, tickets, databases, or
  OpenAPI files.

## Agent Output Shape

When reporting results to a user, include:

- The command or validator that was run.
- Whether contract validation passed.
- The invariant or obstruction that was found.
- The recommended actions from the projection.
- Any completion candidates with confidence and review status.
- Any unsupported scope the user requested, especially real input ingestion,
  candidate acceptance, MCP, plugin packaging, or marketplace work.

## Safety Rules

- Do not treat AI-inferred or suggested structure as accepted fact.
- Do not accept or reject completion candidates without an explicit review
  workflow.
- Do not hide information loss in projections.
- Do not introduce MCP implementation or dependencies for this CLI skill path.
- Do not modify lower-level crates to change the report contract unless the user
  explicitly asks for a new runtime or schema version.
