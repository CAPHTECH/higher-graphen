# highergraphen CLI

The `highergraphen` command is the operational CLI for HigherGraphen runtime
workflows. It exposes the deterministic Architecture Product direct database
access smoke workflow, the bounded architecture input lift workflow, and the
explicit completion review workflow as stable JSON reports.

For the underlying implementation contract, see
[`runtime-cli-scope.md`](../specs/runtime-cli-scope.md) and
[`runtime-workflow-contract.md`](../specs/runtime-workflow-contract.md). For
agent-specific packaging guidance, see
[`agent-tooling-handoff.md`](../specs/agent-tooling-handoff.md).

## Build or Run Locally

From the repository root, build the CLI with the workspace:

```sh
cargo build -p highergraphen-cli
```

After building, invoke the binary from `target/debug`:

```sh
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

You can also run the package through Cargo:

```sh
cargo run -p highergraphen-cli -- architecture smoke direct-db-access --format json
```

## Commands

```sh
highergraphen architecture smoke direct-db-access --format json [--output <path>]
```

This command runs the Architecture Product direct database access smoke
workflow. The workflow is deterministic in the current implementation and does
not read external architecture files, databases, tickets, ADRs, or source code.

```sh
highergraphen architecture input lift --input <path> --format json [--output <path>]
```

This command reads a bounded architecture JSON v1 document and lifts accepted
component and relation facts into HigherGraphen cells and incidences. Inferred
structures from the input are preserved as unreviewed completion candidates and
are not promoted into accepted cells.

```sh
highergraphen completion review accept \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--reviewed-at <text>] \
  [--output <path>]

highergraphen completion review reject \
  --input <path> \
  --candidate <id> \
  --reviewer <id> \
  --reason <text> \
  --format json \
  [--reviewed-at <text>] \
  [--output <path>]
```

These commands read a workflow report containing `result.completion_candidates`
or a review snapshot containing `source_report` and `completion_candidates`.
They emit a separate completion review report with the source candidate
snapshot, reviewer request, and accepted or rejected outcome. They do not edit
the source report and do not promote the candidate into accepted facts.

## Options

| Option | Required | Description |
| --- | --- | --- |
| `--format json` | Yes | Emits the stable JSON report. No human text format is supported yet. |
| `--input <path>` | For `architecture input lift` | Reads the bounded architecture JSON input document. |
| `--input <path>` | For `completion review` | Reads a report or review snapshot containing completion candidates. |
| `--candidate <id>` | For `completion review` | Selects the candidate to accept or reject. |
| `--reviewer <id>` | For `completion review` | Records the explicit reviewer or workflow identifier. |
| `--reason <text>` | For `completion review` | Records the explicit acceptance or rejection rationale. |
| `--reviewed-at <text>` | No | Adds externally supplied review time metadata to the audit record. |
| `--output <path>` | No | Writes the JSON report to the requested file path instead of stdout. |

When `--output` is omitted, the command writes exactly one JSON report to
stdout. When `--output` is supplied, the command writes exactly one JSON report
file and keeps stdout empty.

## Agent Skill

The repository-owned CLI skill lives at
[`skills/highergraphen/SKILL.md`](../../skills/highergraphen/SKILL.md). It is
the immediate agent integration path for this report: agents should run the CLI,
validate the report contract, and interpret the JSON according to the schema.

MCP servers, provider-specific plugin bundles, marketplace metadata, and
provider-specific manifests are future optional work. They are not required for
the current CLI plus skill integration path.

## Examples

Emit the report to stdout:

```sh
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

Lift the checked-in architecture input fixture:

```sh
./target/debug/highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json
```

Write the report to a file:

```sh
./target/debug/highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Write a lifted input report to a file:

```sh
./target/debug/highergraphen architecture input lift \
  --input schemas/inputs/architecture-lift.input.example.json \
  --format json \
  --output architecture-input-lift.report.json
```

Accept a completion candidate from a generated report:

```sh
./target/debug/highergraphen completion review accept \
  --input architecture-direct-db-access-smoke.report.json \
  --candidate candidate:billing-status-api \
  --reviewer reviewer:architecture-lead \
  --reason "Billing Service owns the API boundary." \
  --format json \
  --output completion-review.report.json
```

Validate the generated report with the repository-owned no-network validator:

```sh
python3 scripts/validate-cli-report-contract.py
```

Validate an existing report file:

```sh
python3 scripts/validate-cli-report-contract.py \
  --report architecture-direct-db-access-smoke.report.json
```

## Exit Behavior

Exit code `0` means the workflow ran and emitted a report. The current workflow
is expected to detect a direct database access architecture violation, and that
domain finding is still a successful CLI result.

Nonzero exits are reserved for command usage errors, runtime construction
failures, report serialization failures, or file output failures.

## Report Contract

The emitted report uses this stable contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.architecture.direct_db_access_smoke.report.v1` |
| Report type | `architecture_direct_db_access_smoke` |
| Report version | `1` |
| Schema file | [`architecture-direct-db-access-smoke.report.schema.json`](../../schemas/reports/architecture-direct-db-access-smoke.report.schema.json) |
| Example fixture | [`architecture-direct-db-access-smoke.report.example.json`](../../schemas/reports/architecture-direct-db-access-smoke.report.example.json) |
| Contract validator | [`validate-cli-report-contract.py`](../../scripts/validate-cli-report-contract.py) |
| Runtime runner | `higher_graphen_runtime::run_architecture_direct_db_access_smoke` |

The top-level JSON object contains:

- `schema`
- `report_type`
- `report_version`
- `metadata`
- `scenario`
- `result`
- `projection`

The current deterministic report has `result.status` set to
`"violation_detected"`, exactly one direct database access obstruction, and
exactly one billing status API completion candidate.

The architecture input lift report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.architecture.input_lift.report.v1` |
| Report type | `architecture_input_lift` |
| Report version | `1` |
| Input schema | [`architecture-lift.input.schema.json`](../../schemas/inputs/architecture-lift.input.schema.json) |
| Input fixture | [`architecture-lift.input.example.json`](../../schemas/inputs/architecture-lift.input.example.json) |
| Report schema | [`architecture-input-lift.report.schema.json`](../../schemas/reports/architecture-input-lift.report.schema.json) |
| Example fixture | [`architecture-input-lift.report.example.json`](../../schemas/reports/architecture-input-lift.report.example.json) |
| Runtime runner | `higher_graphen_runtime::run_architecture_input_lift` |

The input lift report has `result.status` set to `"lifted"`, records accepted
cell and incidence IDs under `result.accepted_fact_ids`, and records unreviewed
completion candidate IDs under `result.inferred_structure_ids`.

The completion review report uses this contract:

| Surface | Value |
| --- | --- |
| Schema ID | `highergraphen.completion.review.report.v1` |
| Report type | `completion_review` |
| Report version | `1` |
| Report schema | [`completion-review.report.schema.json`](../../schemas/reports/completion-review.report.schema.json) |
| Runtime runner | `higher_graphen_runtime::run_completion_review` |

The review report records source report metadata under
`scenario.source_report`, preserves the selected source candidate under
`scenario.candidate` and `result.review_record.candidate`, and records the
explicit request under `result.review_record.request`. Accepted reports include
`result.review_record.accepted_completion`; rejected reports include
`result.review_record.rejected_completion`.

## Semantic Rules

Consumers must preserve these semantics:

- A detected architecture violation is report data, not a CLI failure.
- The billing status API is a completion candidate, not accepted structure.
- The completion candidate must remain `review_status: "unreviewed"` until a
  later explicit review workflow accepts or rejects it.
- Accepting or rejecting a completion candidate emits a separate auditable
  review report and never edits or silently promotes the source candidate.
- The input lift path treats JSON `components` and `relations` as accepted
  facts and JSON `inferred_structures` as unreviewed candidates.
- Agent skills and future tool surfaces should consume the CLI output or runtime
  runner and validate against the schema instead of reimplementing the workflow.

## Unsupported Usage

These are intentionally unsupported in the current CLI:

- Human-readable output formats.
- Architecture input formats beyond the bounded JSON v1 document.
- MCP server behavior.
- Provider-specific plugin, marketplace, or manifest behavior.
- Provider-specific skills beyond the repository-owned
  `skills/highergraphen/SKILL.md` CLI skill.
- Additional `highergraphen` subcommands beyond `architecture smoke
  direct-db-access`, `architecture input lift`, and `completion review`.
