# CaseGraphen Schemas

This directory contains the first file-based JSON contracts for the
`tools/casegraphen` CLI.

- `case.graph.schema.json` validates `highergraphen.case.graph.v1` inputs.
- `coverage.policy.schema.json` validates deterministic coverage policy inputs.
- `projection.schema.json` validates projection definitions.
- `case.report.schema.json` validates the shared report envelope used by
  `highergraphen.case.*.report.v1` commands.
- `workflow.graph.schema.json` validates
  `highergraphen.case.workflow.graph.v1` inputs for workflow work items,
  readiness rules, evidence records, transitions, projections, and
  correspondence.
- `workflow.report.schema.json` validates
  `highergraphen.case.workflow.report.v1` outputs for readiness, obstructions,
  completion candidates, evidence-boundary findings, projection loss,
  correspondence, and evolution.
- `native.case.space.schema.json` validates
  `highergraphen.case.space.v1` native case-space contracts for cells,
  relations, morphism-log entries, projections, revisions, reviews, and
  close-check skeletons.
- `native.case.report.schema.json` validates
  `highergraphen.case.native.report.v1` package-level native report envelopes.

The matching `*.example.json` files are used by package tests and can be passed
directly to the `casegraphen` CLI.
