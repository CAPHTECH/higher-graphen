# CaseGraphen Schemas

This directory contains the first file-based JSON contracts for the
`tools/casegraphen` CLI.

- `case.graph.schema.json` validates `highergraphen.case.graph.v1` inputs.
- `coverage.policy.schema.json` validates deterministic coverage policy inputs.
- `projection.schema.json` validates projection definitions.
- `case.report.schema.json` validates the shared report envelope used by
  `highergraphen.case.*.report.v1` commands. The result payload is
  intentionally broad for operation compatibility, with a conservative
  `result.higher_order` fragment for opt-in topology persistence diagnostics.
- `workflow.graph.schema.json` validates
  `highergraphen.case.workflow.graph.v1` inputs for workflow work items,
  readiness rules, evidence records, transitions, projections, and
  correspondence.
- `workflow.report.schema.json` validates
  `highergraphen.case.workflow.report.v1` outputs for readiness, obstructions,
  completion candidates, evidence-boundary findings, projection loss,
  correspondence, and evolution.
- `workflow.operation.report.schema.json` validates focused workflow and
  workflow-store operation reports such as
  `highergraphen.case.workflow.validate.report.v1`,
  `highergraphen.case.workflow.topology.report.v1`, and
  `highergraphen.case.workflow.patch_apply.report.v1`. The result payload is
  intentionally broad, with a conservative `result.higher_order` fragment for
  opt-in topology persistence diagnostics.
- `native.case.space.schema.json` validates
  `highergraphen.case.space.v1` native case-space contracts for cells,
  relations, morphism-log entries, projections, revisions, reviews, and
  close-check skeletons.
- `native.case.report.schema.json` validates
  `highergraphen.case.native.report.v1` package-level native report envelopes.
- `native-cli.report.schema.json` validates generated repo-owned native CLI
  operation reports with `highergraphen.case.native_cli.report.v1`. The result
  payload is intentionally broad, with a conservative
  `result.topology.higher_order` fragment for native case history topology;
  higher-order reports may include `filtration_source` and `stage_sources`
  when workflow history or native morphism-log ordering is available.
- `report-schema-aliases.json` records operation-specific report IDs that are
  intentionally validated by shared report-envelope schemas.

The matching `*.example.json` files are used by package tests. Input fixtures
such as `case.graph.example.json`, `workflow.graph.example.json`,
`projection.example.json`, `coverage.policy.example.json`, and
`native.case.space.example.json` can be passed directly to the relevant
`casegraphen` CLI commands. Report fixtures such as
`workflow.report.example.json` and `native.case.report.example.json` are output
contract examples, not CLI inputs.
