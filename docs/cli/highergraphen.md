# highergraphen CLI

The `highergraphen` command is the operational CLI for HigherGraphen runtime
workflows. The first supported command exposes the deterministic Architecture
Product direct database access smoke workflow as a stable JSON report.

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

## Command

```sh
highergraphen architecture smoke direct-db-access --format json [--output <path>]
```

This command runs the Architecture Product direct database access smoke
workflow. The workflow is deterministic in the current implementation and does
not read external architecture files, databases, tickets, ADRs, or source code.

## Options

| Option | Required | Description |
| --- | --- | --- |
| `--format json` | Yes | Emits the stable JSON report. No human text format is supported yet. |
| `--output <path>` | No | Writes the JSON report to the requested file path instead of stdout. |

When `--output` is omitted, the command writes exactly one JSON report to
stdout. When `--output` is supplied, the command writes exactly one JSON report
file and keeps stdout empty.

## Examples

Emit the report to stdout:

```sh
./target/debug/highergraphen architecture smoke direct-db-access --format json
```

Write the report to a file:

```sh
./target/debug/highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Validate a generated report with Python when `jsonschema` is available:

```sh
python3 -c 'import json, jsonschema; \
schema=json.load(open("schemas/reports/architecture-direct-db-access-smoke.report.schema.json")); \
report=json.load(open("architecture-direct-db-access-smoke.report.json")); \
jsonschema.Draft202012Validator.check_schema(schema); \
jsonschema.validate(instance=report, schema=schema)'
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

## Semantic Rules

Consumers must preserve these semantics:

- A detected architecture violation is report data, not a CLI failure.
- The billing status API is a completion candidate, not accepted structure.
- The completion candidate must remain `review_status: "unreviewed"` until a
  later explicit review workflow accepts or rejects it.
- Agent tools, MCP servers, plugins, and skills should consume the CLI output or
  runtime runner and validate against the schema instead of reimplementing the
  workflow.

## Unsupported Usage

These are intentionally unsupported in the current CLI:

- Human-readable output formats.
- External input paths for real architecture sources.
- Accepting or rejecting completion candidates.
- Provider-specific MCP, skill, plugin, or marketplace behavior.
- Additional `highergraphen` subcommands beyond
  `architecture smoke direct-db-access`.
