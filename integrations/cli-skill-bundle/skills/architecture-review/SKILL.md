---
name: architecture-review
description: Use when an agent needs the current HigherGraphen Architecture Product direct database access smoke review workflow.
---

# Architecture Review Skill

Use this skill when a task asks for the current HigherGraphen architecture
review smoke workflow or for interpretation of the direct database access
architecture report.

This bundled skill is CLI-only. It delegates execution and contract validation
to the `highergraphen` CLI and the repository-owned report validator.

## Source Of Truth

- Bundle CLI contract reference:
  `integrations/cli-skill-bundle/references/cli-contract.md`
- CLI reference: `docs/cli/highergraphen.md`
- Report schema:
  `schemas/reports/architecture-direct-db-access-smoke.report.schema.json`
- Example report:
  `schemas/reports/architecture-direct-db-access-smoke.report.example.json`
- Contract validator: `scripts/validate-cli-report-contract.py`
- Umbrella CLI skill: `skills/highergraphen/SKILL.md`

Do not restate the report schema as a competing contract. Consume the schema,
fixture, validator, and CLI output.

## Run The Review

Preferred validation:

```sh
python3 scripts/validate-cli-report-contract.py
```

Stable CLI command:

```sh
highergraphen architecture smoke direct-db-access --format json
```

Generate the report:

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

## Interpretation Rules

- Treat `result.status == "violation_detected"` as a successful architecture
  review finding, not a failed CLI run.
- Report the direct database access obstruction and its provenance.
- Keep the billing status API as a completion candidate with
  `review_status: "unreviewed"`.
- Do not describe completion candidates as accepted architecture.
- Include `projection.recommended_actions` and
  `projection.information_loss` when summarizing the review.
- State that this is deterministic smoke coverage, not full ingestion of real
  architecture documents, source code, ADRs, tickets, databases, or OpenAPI
  files.

## Safety Rules

- Do not accept or reject completion candidates without an explicit review
  workflow.
- Do not hide projection information loss.
- Do not introduce MCP behavior, provider SDK calls, marketplace assumptions,
  or provider-specific manifests for this bundle path.
