# HigherGraphen Structure Patterns

Use these patterns to build high-order structure before judging sufficiency.

## CLI Workflow Surface

Cells:

- `command:<tool>:<domain>:<action>`
- `parser:<tool>:<domain>`
- `runner:<domain>:<action>`
- `adapter:<domain>:<source>`
- `schema:<domain>:input`
- `schema:<domain>:report`
- `test:<scope>:<case>`

Incidences:

- command supports parser
- command supports runner
- adapter emits schema
- runner consumes input schema
- runner emits report schema
- test verifies command, adapter, or runner

Obligation examples:

- `requirement:morphism:<domain>:command-to-runner`
- `requirement:morphism:<domain>:adapter-to-input-schema`
- `requirement:law:<domain>:output-file-suppresses-stdout`

## Runtime Workflow Surface

Cells:

- `runner:<domain>:<action>`
- `registry:<domain>:workflow-module`
- `export:<domain>:runtime-api`
- `contract:<domain>:runtime-report-shapes`
- `projection:<domain>:human-view`
- `projection:<domain>:ai-view`
- `projection:<domain>:audit-trace`

Incidences:

- registry contains runner
- export supports runner
- runner supports runtime report shapes
- projection preserves runtime report shapes
- test verifies runner, projection, or law

## Schema And Fixture Surface

Cells:

- `schema:<domain>:input`
- `schema:<domain>:report`
- `fixture:<domain>:input-example`
- `fixture:<domain>:report-example`
- `validator:<domain>:json-contracts`

Incidences:

- runtime shape supports schema
- fixture supports schema
- validator verifies fixture-to-schema morphism
- report fixture supports report schema

## Review Boundary Surface

Cells:

- `fact:<domain>:accepted-input`
- `candidate:<domain>:completion`
- `obstruction:<domain>:detected`
- `review:<domain>:accept`
- `review:<domain>:reject`

Incidences:

- input fact lifts to accepted cell
- obstruction derives from invariant
- candidate proposes completion for obstruction
- review decision accepts or rejects candidate

## Test-Gap Surface

Cells:

- changed behavior symbol
- branch or boundary condition
- requirement
- test
- coverage
- detector policy
- morphism requirement

Incidences:

- symbol implements requirement
- symbol has branch
- test verifies requirement
- test covers symbol or branch
- coverage supports branch
- detector policy accepts test kind

Prefer high-order obligations for HigherGraphen-owned surfaces:

- command-runner morphism
- runtime export-runner morphism
- workflow registry-runner morphism
- runtime shape-schema morphism
- fixture-schema morphism
- projection-runtime shape morphism

Use file-level obligations only when no meaningful high-order cell can be
extracted.
