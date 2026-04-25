# CaseGraphen Schemas

This directory contains the first file-based JSON contracts for the
`tools/casegraphen` CLI.

- `case.graph.schema.json` validates `highergraphen.case.graph.v1` inputs.
- `coverage.policy.schema.json` validates deterministic coverage policy inputs.
- `projection.schema.json` validates projection definitions.
- `case.report.schema.json` validates the shared report envelope used by
  `highergraphen.case.*.report.v1` commands.

The matching `*.example.json` files are used by package tests and can be passed
directly to the `casegraphen` CLI.
