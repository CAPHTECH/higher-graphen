---
name: casegraphen
description: Use when an agent needs to run or interpret the repository-owned CaseGraphen intermediate CLI reports, especially workflow reasoning through `casegraphen workflow reason`.
---

# CaseGraphen CLI Skill

Use this skill when a task asks for CaseGraphen intermediate tool output,
workflow reasoning, readiness, blockers, completion candidates, evidence
boundaries, or interpretation of `casegraphen workflow reason` reports.

This repository skill is CLI-only. MCP servers, provider SDK integrations,
provider marketplace metadata, and the external
`casegraphen reference workspace` repository are outside this surface.

## Source Of Truth

- Workflow contract: `docs/specs/intermediate-tools/casegraphen-workflow-contracts.md`
- Workflow graph schema: `schemas/casegraphen/workflow.graph.schema.json`
- Workflow report schema: `schemas/casegraphen/workflow.report.schema.json`
- Workflow graph example: `schemas/casegraphen/workflow.graph.example.json`
- Workflow report example: `schemas/casegraphen/workflow.report.example.json`
- CLI implementation: `tools/casegraphen/src/cli.rs`

Do not restate the schemas as competing contracts. Consume the schema, fixture,
and CLI output.

## When To Run The CLI

Run the CLI when the user asks for workflow readiness, workflow blockers,
completion candidates, evidence boundary status, or a machine-readable
CaseGraphen workflow reasoning report.

Generate the workflow reasoning report to stdout:

```sh
cargo run -q -p casegraphen -- \
  workflow reason \
  --input schemas/casegraphen/workflow.graph.example.json \
  --format json
```

Generate the report to a file:

```sh
cargo run -q -p casegraphen -- \
  workflow reason \
  --input schemas/casegraphen/workflow.graph.example.json \
  --format json \
  --output casegraphen-workflow.report.json
```

Preferred local validation after CLI or model changes:

```sh
cargo fmt --all --check
cargo test -p casegraphen --test command
cargo test -p casegraphen --lib
cargo check -p casegraphen
```

## Legacy Commands

Existing non-workflow commands keep their compatibility surface:

```sh
casegraphen create --case-graph-id <id> --space-id <id> --store <dir> --format json
casegraphen inspect --input <case.graph.json> --format json
casegraphen list --store <dir> --format json
casegraphen validate --input <case.graph.json> --format json
casegraphen coverage --input <case.graph.json> --coverage <coverage.policy.json> --format json
casegraphen missing --input <case.graph.json> --coverage <coverage.policy.json> --format json
casegraphen conflicts --input <case.graph.json> --format json
casegraphen project --input <case.graph.json> --projection <projection.json> --format json
casegraphen compare --left <case.graph.json> --right <case.graph.json> --format json
```

## Interpretation Rules

- Exit code `0` means the command emitted a structurally valid report.
- `result.status` values such as `blocked`, `obstructions_detected`, or
  `review_required` are successful domain findings, not failed CLI runs.
- Completion candidates are proposed structure. Keep
  `review_status: "unreviewed"` unless an explicit review workflow accepts or
  rejects them.
- AI inference records do not become accepted evidence because they appear in a
  report.
- Source-backed or accepted evidence is the boundary for satisfying evidence
  and proof requirements unless a future contract explicitly says otherwise.
- Keep `projection.information_loss`, source IDs, and audit records visible in
  summaries.
- Do not mutate input workflow graphs when interpreting reports.

## Agent Output Shape

When reporting results to a user, include:

- The command that was run.
- Whether the command and contract validation passed.
- `result.status`, ready item IDs, and not-ready item IDs.
- Blocking obstructions with witness IDs.
- Completion candidates with confidence and review status.
- Evidence boundary findings, especially inference records that remain
  unaccepted.
- Projection loss or omitted IDs when relevant.

## Safety Rules

- Do not promote inferred records to evidence without an explicit review
  transition.
- Do not accept or reject completion candidates without an explicit review
  workflow.
- Do not hide projection loss in human, AI, or audit summaries.
- Do not introduce MCP, provider SDKs, or external repository dependencies for
  this CLI skill path.
- Do not change existing `highergraphen.case.graph.v1` command semantics when
  working on workflow reasoning.
