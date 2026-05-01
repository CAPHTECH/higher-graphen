# Runtime and CLI Scope

This document locks the immediate runtime and CLI scope for the next tooling
layer. It is a pre-implementation contract for `hg_runtime_cli_foundation` and
should be read with [`package-boundaries.md`](package-boundaries.md),
[`intermediate-tools-map.md`](intermediate-tools-map.md),
[`ai-agent-integration.md`](ai-agent-integration.md), and
[`non-core-package-workplans.md`](non-core-package-workplans.md).

The goal is to expose the first Architecture Product workflow through a stable
runtime API and a stable CLI command without changing lower-level core package
boundaries.

## Locked Decisions

| Surface | Locked name or location |
| --- | --- |
| Runtime Cargo package | `higher-graphen-runtime` |
| Runtime Rust crate | `higher_graphen_runtime` |
| Runtime package location | `crates/higher-graphen-runtime/` |
| CLI Cargo package | `highergraphen-cli` |
| CLI Rust crate | `highergraphen_cli` |
| CLI package location | `tools/highergraphen-cli/` |
| Installed command | `highergraphen` |
| First command path | `highergraphen architecture smoke direct-db-access` |
| First JSON report filename | `architecture-direct-db-access-smoke.report.json` |
| First JSON report type | `architecture_direct_db_access_smoke` |
| First JSON schema identifier | `highergraphen.architecture.direct_db_access_smoke.report.v1` |
| First runtime report struct | `ArchitectureDirectDbAccessSmokeReport` |
| First runtime runner | `run_architecture_direct_db_access_smoke` |

The CLI package lives under `tools/` because it is an operational surface, not a
core model crate. The command is `highergraphen` because it is the umbrella
product CLI; intermediate tools such as `casegraphen` remain separate tool
families.

DDD review follows the same ownership rule. The stable command namespace is
`highergraphen ddd ...` because it is a product workflow over bounded source
structure. DDD-specific interpretation must not be moved into
`higher-graphen-core` or CaseGraphen core.

## Runtime Package Scope

`higher-graphen-runtime` owns orchestration APIs for humans, tools, and AI
agents. For the immediate scope it should provide only the smallest stable
surface required to run the first Architecture Product smoke workflow.

Minimum package shape:

```text
crates/higher-graphen-runtime/
  Cargo.toml
  src/
    lib.rs
    reports.rs
    workflows/
      mod.rs
      architecture.rs
```

Minimum public API names:

```rust
pub mod reports;
pub mod workflows;

pub use reports::{
    ArchitectureDirectDbAccessSmokeReport, ReportEnvelope, ReportMetadata,
};
pub use workflows::architecture::run_architecture_direct_db_access_smoke;
```

Runtime responsibilities in this first scope:

- Build the deterministic Architecture Product smoke structure from the
  existing scenario contract.
- Coordinate the existing lower crates for space construction, invariant
  result representation, obstruction construction, completion candidate
  detection, and projection/report assembly.
- Preserve the distinction between accepted facts, violated checks,
  obstructions, and reviewable completion candidates.
- Return stable Rust report structs that serialize to stable JSON.
- Keep runtime-specific orchestration errors inside the runtime crate.

Allowed runtime dependencies:

- `higher-graphen-core`
- `higher-graphen-structure::space`
- `higher-graphen-reasoning::invariant`
- `higher-graphen-reasoning::obstruction`
- `higher-graphen-reasoning::completion`
- `higher-graphen-projection`
- `higher-graphen-interpretation` only when interpretation templates or lift
  adapters are needed
- Serialization dependencies already accepted by the workspace

Runtime must not depend on:

- `tools/highergraphen-cli/`
- MCP server packages
- provider SDKs
- marketplace or plugin manifests
- UI frameworks
- `examples/architecture`

The existing
[`examples/architecture/tests/architecture_product_smoke.rs`](../../examples/architecture/tests/architecture_product_smoke.rs)
test is the behavioral reference, not a library dependency. Implementation may
copy the scenario constants into runtime or move reusable scenario construction
behind runtime-local modules, but it must not make lower packages depend on the
example.

## CLI Package Scope

`highergraphen-cli` owns command-line parsing, process exit behavior, stdout,
stderr, and file output. It should call `higher_graphen_runtime` instead of
directly orchestrating lower model crates.

Minimum package shape:

```text
tools/highergraphen-cli/
  Cargo.toml
  src/
    main.rs
```

The installed binary command is:

```text
highergraphen
```

The first supported workflow command is:

```text
highergraphen architecture smoke direct-db-access --format json
```

Optional file output uses:

```text
highergraphen architecture smoke direct-db-access \
  --format json \
  --output architecture-direct-db-access-smoke.report.json
```

Command behavior:

- `--format json` is required for the first implementation. Human text output
  can be added later after the JSON contract is stable.
- With no `--output`, write the JSON report to stdout.
- With `--output`, write exactly one JSON report file at the requested path.
- If the command chooses a default output filename in a future convenience mode,
  it must use `architecture-direct-db-access-smoke.report.json`.
- Exit code `0` means the workflow ran and produced a report, even if the
  report contains a detected architecture violation.
- Nonzero exit codes are reserved for command usage errors, runtime failures,
  serialization failures, or file output failures.

## DDD Review Workflow Scope

The DDD review workflow is a bounded, deterministic extension of the
`highergraphen` CLI surface:

```text
highergraphen ddd input from-case-space --case-space <path> --format json
highergraphen ddd review --input <path> --format json
```

`ddd input from-case-space` adapts one native CaseGraphen `CaseSpace` JSON file
into `highergraphen.ddd_review.input.v1`. `ddd review` reads that bounded input
and emits `highergraphen.ddd_review.report.v1`.

The workflow uses the Sales/Billing Customer example as its first reference:

```text
examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json
```

The DDD workflow boundary is intentionally narrow:

- source-backed CaseSpace records and supplied DDD input records may become
  accepted input observations;
- AI-inferred equivalence proofs, generated diagnostics, inferred missing
  mappings, and unreviewed notes remain unreviewed claims or candidates;
- report sections must expose obstructions, completion candidates, evidence
  boundaries, projection loss, review gaps, and closeability;
- the CLI must not fetch network data, call provider APIs, invoke hidden LLM
  inference, scan unrelated repository files, or silently promote inferred
  claims.

The report schema and fixture live at:

```text
schemas/inputs/ddd-review.input.schema.json
schemas/inputs/ddd-review.input.example.json
schemas/reports/ddd-review.report.schema.json
schemas/reports/ddd-review.report.example.json
```

## First Workflow Contract

The first workflow is the Architecture Product direct database access smoke
workflow. It is deterministic and has no external input in the initial scope.

Workflow name:

```text
architecture_direct_db_access_smoke
```

Runtime function:

```rust
run_architecture_direct_db_access_smoke()
```

CLI command:

```text
highergraphen architecture smoke direct-db-access
```

Scenario facts:

| Object | Stable ID | Meaning |
| --- | --- | --- |
| Space | `space:architecture-product-smoke` | Architecture smoke structure. |
| Context | `context:architecture-review` | Workflow context. |
| Context | `context:orders` | Order Service ownership context. |
| Context | `context:billing` | Billing ownership context. |
| Cell | `cell:order-service` | Order Service component. |
| Cell | `cell:billing-service` | Billing Service component. |
| Cell | `cell:billing-db` | Billing database. |
| Incidence | `incidence:order-service-reads-billing-db` | Order Service reads Billing DB. |
| Incidence | `incidence:billing-service-owns-billing-db` | Billing Service owns Billing DB. |
| Invariant | `invariant:no-cross-context-direct-database-access` | Components must not directly access another context's database. |
| Obstruction | `obstruction:order-service-direct-billing-db-access` | Detected direct cross-context database access. |
| Completion candidate | `candidate:billing-status-api` | Reviewable proposal for a Billing Service API. |
| Suggested cell | `cell:billing-status-api` | Proposed API cell, not accepted structure. |

Required workflow result:

1. Build the architecture space with the three cells and two incidences.
2. Represent the invariant named `No cross-context direct database access`.
3. Produce a violated check result for Order Service reading Billing DB.
4. Produce an invariant-violation obstruction with a counterexample and required
   resolution.
5. Produce exactly one completion candidate for a billing status API.
6. Preserve `ReviewStatus::Unreviewed` for the completion candidate.
7. Emit a JSON report using the contract below.

## JSON Report Contract

The first JSON report is named:

```text
architecture-direct-db-access-smoke.report.json
```

The report type is:

```text
architecture_direct_db_access_smoke
```

The schema identifier is:

```text
highergraphen.architecture.direct_db_access_smoke.report.v1
```

The top-level JSON object must use this envelope:

```json
{
  "schema": "highergraphen.architecture.direct_db_access_smoke.report.v1",
  "report_type": "architecture_direct_db_access_smoke",
  "report_version": 1,
  "metadata": {
    "command": "highergraphen architecture smoke direct-db-access",
    "runtime_package": "higher-graphen-runtime",
    "cli_package": "highergraphen-cli"
  },
  "scenario": {},
  "result": {},
  "projection": {}
}
```

Required `scenario` fields:

- `space_id`
- `workflow_context_id`
- `cells`
- `incidences`
- `invariant_id`

Required `result` fields:

- `status`: use `violation_detected` for the first scenario.
- `violated_invariant_id`
- `obstructions`
- `completion_candidates`

Required `completion_candidates` fields:

- `id`
- `review_status`
- `suggested_structure`
- `inferred_from`
- `confidence`

The first scenario must serialize the candidate review status as:

```json
"review_status": "unreviewed"
```

Required `projection` fields:

- `audience`: `human`
- `purpose`: `architecture_review`
- `summary`
- `recommended_actions`
- `information_loss`
- `source_ids`

The report may include additional fields only when they are additive and do not
rename or remove the fields above. If a future report shape is incompatible, it
must use a new schema identifier and increment the report version.

## Lower Package Boundary Rules

This scope must not require changes to existing lower-level package boundaries.

- Do not add Architecture Product concepts to `higher-graphen-core`.
- Do not add CLI, MCP, plugin, marketplace, provider, or UI concepts to any core
  package.
- Do not make `higher-graphen-structure::space`, `higher-graphen-reasoning::invariant`,
  `higher-graphen-reasoning::obstruction`, `higher-graphen-reasoning::completion`,
  `higher-graphen-projection`, or `higher-graphen-interpretation` depend on
  runtime or tools.
- Do not move review workflow acceptance into the CLI. The CLI may display and
  emit candidates, but accepting or rejecting candidates requires an explicit
  runtime workflow added later.
- Do not treat the suggested billing API as accepted structure. It remains a
  completion candidate with `unreviewed` status.
- Do not make the example crate an implementation dependency.

If the runtime needs a convenience adapter or report-specific view, implement it
inside `higher-graphen-runtime` or the CLI package instead of expanding lower
model crates.

## Out of Scope

The following work is intentionally outside this case:

- Adding `crates/higher-graphen-runtime/` to the workspace.
- Adding `tools/highergraphen-cli/` to the workspace.
- Implementing the CLI command.
- Implementing MCP server capabilities.
- Implementing marketplace metadata or provider-specific plugin packaging.
- Creating agent skills or plugin bundles beyond handoff documentation.
- Implementing `casegraphen`, `morphographen`, or other intermediate tool
  command families.
- Adding external input parsing for architecture documents, OpenAPI files,
  database schemas, ADRs, tickets, or tests.
- Adding persistent storage, snapshots, migrations, or remote services.
- Adding UI, studio, playground, Python, Node, WebAssembly, or SDK packaging.
- Accepting or rejecting completion candidates.
- Changing lower-level core package public API boundaries.

MCP, marketplace, provider plugin packaging, and agent skills remain future
handoffs that should consume this CLI and JSON report contract rather than
define a competing workflow contract.

## Implementation Handoff

The next implementation task can proceed by:

1. Adding `crates/higher-graphen-runtime/` as a workspace crate.
2. Implementing `ArchitectureDirectDbAccessSmokeReport` and
   `run_architecture_direct_db_access_smoke`.
3. Adding `tools/highergraphen-cli/` as a workspace package with binary
   `highergraphen`.
4. Wiring
   `highergraphen architecture smoke direct-db-access --format json` to the
   runtime runner.
5. Adding tests that compare key JSON fields against the stable IDs and report
   names in this document.

The implementation should keep any future MCP, plugin, marketplace, and skill
work as consumers of the same runtime and report contract.
