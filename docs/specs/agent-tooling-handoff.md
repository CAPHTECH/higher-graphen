# Agent Tooling Handoff

This handoff prepares the next agent tooling case. It should be read after
[`ai-agent-integration.md`](ai-agent-integration.md),
[`runtime-cli-scope.md`](runtime-cli-scope.md), and
[`runtime-workflow-contract.md`](runtime-workflow-contract.md).

The immediate agent path is CLI plus repository-owned skill only. It packages
the existing Architecture Product smoke workflow for agents through the
`highergraphen` CLI, report schema, fixture, validation script, and
`skills/highergraphen/SKILL.md`. MCP servers, provider plugin bundles,
marketplace metadata, and provider-specific manifests are future optional work.

## Stable Consumable Surfaces

The first stable agent-facing command is:

```sh
highergraphen architecture smoke direct-db-access --format json
```

Optional file output is:

```sh
highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Stable report contract:

| Surface | Value |
| --- | --- |
| Report type | `architecture_direct_db_access_smoke` |
| Report version | `1` |
| Schema ID | `highergraphen.architecture.direct_db_access_smoke.report.v1` |
| Schema path | `schemas/reports/architecture-direct-db-access-smoke.report.schema.json` |
| Fixture path | `schemas/reports/architecture-direct-db-access-smoke.report.example.json` |
| Runtime package | `higher-graphen-runtime` |
| Runtime runner | `run_architecture_direct_db_access_smoke` |
| CLI package | `tools/highergraphen-cli/` |
| CLI binary | `highergraphen` |
| Contract validator | `scripts/validate-cli-report-contract.py` |
| Repository skill | `skills/highergraphen/SKILL.md` |

The CLI exits `0` when the workflow runs and emits a report, even when the
report contains the expected architecture violation. Usage, runtime,
serialization, and output failures remain nonzero CLI failures.

## Already Implemented

The current implementation surface for agent tooling is the deterministic
Architecture Product direct database access smoke report:

- Runtime workflow:
  `higher_graphen_runtime::run_architecture_direct_db_access_smoke`.
- CLI command:
  `highergraphen architecture smoke direct-db-access --format json`.
- Optional `--output <path>` writing the same JSON report to a file.
- JSON report envelope with `schema`, `report_type`, `report_version`,
  `metadata`, `scenario`, `result`, and `projection`.
- Schema and fixture under `schemas/reports/`.
- Domain violation represented as successful report data:
  `result.status == "violation_detected"`.
- Exactly one obstruction for direct cross-context database access.
- Exactly one billing status API completion candidate with
  `review_status == "unreviewed"`.

The suggested billing API is not accepted structure. Agent tooling must keep it
as a reviewable candidate unless a later explicit acceptance workflow exists.

## Current Immediate Artifacts

This case should create or maintain these artifacts:

- A no-network validation command that runs or consumes the CLI report, checks
  it against the schema, and verifies key semantics such as
  `violation_detected` and `review_status: "unreviewed"`.
- A repository-owned `highergraphen` skill that tells agents when to invoke the
  CLI, how to validate the report, and how to interpret the report without
  accepting completion candidates.
- CLI and agent docs that describe MCP as out of scope for the immediate path.

## Future Optional Artifacts

Later provider-specific cases may create some or all of these artifacts:

- MCP server package exposing an agent tool for the Architecture Product smoke
  workflow.
- Claude, Codex, or other provider-specific skills generated from the
  repository-owned skill.
- Claude marketplace or plugin metadata once the provider manifest format is
  confirmed.
- A provider plugin bundle containing skills, metadata, command/tool
  definitions, schemas, examples, and optional MCP configuration.
- CI validation that the fixture validates against the schema and that generated
  CLI JSON remains compatible with both.

Provider-specific folder names, manifest fields, marketplace categories,
authentication policy, and installation metadata remain implementation
decisions for the next case. Verify provider requirements at implementation
time because marketplace and plugin formats can change.

## Out of Scope for This Handoff

This document does not implement:

- MCP server capabilities.
- Claude, Codex, or other provider-specific packaged skills beyond the
  repository-owned skill.
- Marketplace entries or plugin manifests.
- Plugin bundle packaging.
- External input parsing for real architecture sources.
- Acceptance or rejection workflows for completion candidates.
- New Rust, Cargo, schema, or `.casegraphen/` changes.

Future tooling must not reimplement the lower-crate workflow orchestration. It
should call the runtime runner directly or invoke the stable CLI command, then
consume the JSON report according to the schema.

## Acceptance Checks for CLI Plus Skill Integration

The CLI plus skill integration case should prove:

1. The CLI can be invoked:

   ```sh
   highergraphen architecture smoke direct-db-access --format json
   ```

2. Optional output works:

   ```sh
   highergraphen architecture smoke direct-db-access \
     --format json \
     --output architecture-direct-db-access-smoke.report.json
   ```

3. The emitted JSON validates against
   `schemas/reports/architecture-direct-db-access-smoke.report.schema.json`.
4. The schema ID is
   `highergraphen.architecture.direct_db_access_smoke.report.v1`.
5. The report preserves
   `report_type: "architecture_direct_db_access_smoke"` and
   `report_version: 1`.
6. The repository skill consumes the schema and fixture instead of restating or
   reimplementing the workflow contract.
7. A detected direct database access violation remains a successful report
   result, not a tool failure.
8. The billing status API remains an unreviewed completion candidate and is not
   described as accepted structure.
9. CI validates the schema/fixture pair and, where practical, generated CLI
   output against the schema.

## Agent Guidance

Agent-facing documentation should make these rules explicit:

- Use the CLI or runtime runner as the source of truth.
- Use `skills/highergraphen/SKILL.md` for the current agent procedure.
- Treat the schema as the stable machine-readable contract.
- Present obstructions and recommended actions from the report projection.
- Preserve provenance and review status.
- Do not promote completion candidates without an explicit review workflow.
- Do not hide that the current workflow is deterministic smoke coverage, not
  full architecture ingestion.
