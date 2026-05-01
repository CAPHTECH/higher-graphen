# Runtime Workflow Contract

This document defines the reusable `higher-graphen-runtime` contract for the
deterministic Architecture Product direct database access smoke workflow. It
refines [`runtime-cli-scope.md`](runtime-cli-scope.md) and preserves the
boundaries in [`package-boundaries.md`](package-boundaries.md),
[`core-contracts.md`](core-contracts.md), and
[`non-core-package-workplans.md`](non-core-package-workplans.md).

The CLI, MCP servers, provider plugins, and future apps should consume this
runtime workflow rather than reimplementing orchestration over lower crates.

Additional runtime workflows should keep the same envelope, review-status, and
source-boundary rules. Domain workflows such as DDD review may add
domain-specific schemas and result sections, but domain interpretation belongs
in runtime/CLI workflow modules and product docs, not in
`higher-graphen-core`.

## Identity

| Surface | Contract |
| --- | --- |
| Runtime package | `higher-graphen-runtime` |
| Runtime crate | `higher_graphen_runtime` |
| Workflow name | `architecture_direct_db_access_smoke` |
| Public runner | `run_architecture_direct_db_access_smoke` |
| Report struct | `ArchitectureDirectDbAccessSmokeReport` |
| Report type | `architecture_direct_db_access_smoke` |
| Report version | `1` |
| Schema identifier | `highergraphen.architecture.direct_db_access_smoke.report.v1` |
| CLI consumer | `highergraphen architecture smoke direct-db-access` |

The first implementation has no external input. It constructs the deterministic
scenario proven by
[`examples/architecture/tests/architecture_product_smoke.rs`](../../examples/architecture/tests/architecture_product_smoke.rs),
but runtime must not depend on the example crate or test module.

## Public API

The minimum public surface is:

```rust
pub mod reports;
pub mod workflows;

pub use reports::{ArchitectureDirectDbAccessSmokeProjection, ArchitectureDirectDbAccessSmokeReport};
pub use reports::{ArchitectureDirectDbAccessSmokeResult, ArchitectureDirectDbAccessSmokeScenario};
pub use reports::{ReportEnvelope, ReportMetadata, RuntimeError, RuntimeResult};
pub use workflows::architecture::run_architecture_direct_db_access_smoke;
```

The runner signature is:
```rust
pub fn run_architecture_direct_db_access_smoke() -> RuntimeResult<ArchitectureDirectDbAccessSmokeReport>;
```

`RuntimeResult<T>` is a runtime-owned alias for
`Result<T, RuntimeError>`. `ArchitectureDirectDbAccessSmokeReport` may be a
type alias over `ReportEnvelope<Scenario, Result, Projection>` or an equivalent
concrete struct with the same serialized fields. The reusable envelope should
remain available for later workflows.

## Report Envelope

All runtime workflow reports use this stable top-level shape:

```json
{
  "schema": "highergraphen.architecture.direct_db_access_smoke.report.v1",
  "report_type": "architecture_direct_db_access_smoke",
  "report_version": 1,
  "metadata": {},
  "scenario": {},
  "result": {},
  "projection": {}
}
```

`ReportEnvelope<S, R, P>` owns `schema`, `report_type`, `report_version`,
`metadata`, `scenario`, `result`, and `projection`. `ReportMetadata` must
include `command: "highergraphen architecture smoke direct-db-access"`,
`runtime_package: "higher-graphen-runtime"`,
`runtime_crate: "higher_graphen_runtime"`, and
`cli_package: "highergraphen-cli"`.

Metadata may later add optional runtime build or generation fields, but must not
include provider SDK objects, file handles, process handles, or nonportable
debug output.

## Scenario

`ArchitectureDirectDbAccessSmokeScenario` is the report view of deterministic
input structure. It should not expose the internal in-memory store. Required
serialized fields are `space_id`, `workflow_context_id`, `context_ids`,
`cells`, `incidences`, `invariant_id`, and `invariant_name`.

Stable scenario IDs:

| Object | ID |
| --- | --- |
| Space | `space:architecture-product-smoke` |
| Workflow context | `context:architecture-review` |
| Orders context | `context:orders` |
| Billing context | `context:billing` |
| Order Service cell | `cell:order-service` |
| Billing Service cell | `cell:billing-service` |
| Billing DB cell | `cell:billing-db` |
| Order reads Billing DB incidence | `incidence:order-service-reads-billing-db` |
| Billing owns Billing DB incidence | `incidence:billing-service-owns-billing-db` |
| Invariant | `invariant:no-cross-context-direct-database-access` |

The runtime may build the scenario using `higher-graphen-structure::space`
types such as `Space`, `Cell`, `Incidence`, `Complex`, and
`InMemorySpaceStore`, then map it into the report view. The invariant name must
be `No cross-context direct database access`.

## Result

`ArchitectureDirectDbAccessSmokeResult` records machine-checkable workflow
outcome data. A detected architecture violation is a successful workflow result,
not a runtime error.

Required serialized fields:

| Field | Contract |
| --- | --- |
| `status` | `violation_detected` for the first deterministic scenario. |
| `violated_invariant_id` | `invariant:no-cross-context-direct-database-access`. |
| `check_result` | The violated `higher_graphen_reasoning::invariant::CheckResult` or a stable report view of it. |
| `obstructions` | Exactly one invariant-violation obstruction. |
| `completion_candidates` | Exactly one unreviewed billing API candidate. |

The runtime-owned status enum must serialize with lower snake case and include
at least `satisfied` and `violation_detected`.

Required obstruction values:

| Field | Value |
| --- | --- |
| `id` | `obstruction:order-service-direct-billing-db-access` |
| `obstruction_type` | `invariant_violation` |
| `location_cell_ids` | `cell:order-service`, `cell:billing-db` |
| `location_context_ids` | `context:orders`, `context:billing` |
| `severity` | `critical` |
| `counterexample` | Present. |
| `required_resolution` | Present. |

Required completion candidate values:

| Field | Value |
| --- | --- |
| `id` | `candidate:billing-status-api` |
| `review_status` | `unreviewed` |
| `suggested_structure.structure_id` | `cell:billing-status-api` |
| `suggested_structure.structure_type` | `api` |
| `inferred_from` | `obstruction:order-service-direct-billing-db-access`, `incidence:order-service-reads-billing-db` |
| `confidence` | `0.9` |

Runtime must preserve `ReviewStatus::Unreviewed`. Accepting, rejecting, or
promoting the suggested billing API is out of scope. The suggested API must not
be added to accepted scenario cells.

## Projection

`ArchitectureDirectDbAccessSmokeProjection` is the stable report projection for
human architecture review. It can be assembled from `higher-graphen-projection`
types or from a runtime-owned view that preserves the same semantics.

Required serialized fields:

| Field | Contract |
| --- | --- |
| `audience` | `human` |
| `purpose` | `architecture_review` |
| `summary` | States that Order Service directly reads Billing DB across context boundaries. |
| `recommended_actions` | At least one action to route access through Billing Service or a Billing API. |
| `information_loss` | Declares that the projection summarizes the full space, check, obstruction, and candidate. |
| `source_ids` | IDs represented in the projection. |

If `higher-graphen-projection::ProjectionPurpose` does not yet have an
`ArchitectureReview` variant, runtime may own the serialized
`architecture_review` purpose while still using projection crate structures
internally where they fit. Do not add product-only purpose variants to lower
crates merely to satisfy this report.

## Error Behavior

Runtime errors are reserved for failures to construct or serialize the workflow,
not for domain findings.

Runtime errors include invalid static IDs or confidence values, deterministic
scenario construction failures, lower-crate constructor failures, unsupported
future report versions, and serialization failures in runtime tests or CLI
consumers.

Domain findings are report data:

- A violated invariant returns `Ok(report)` with
  `result.status == violation_detected`.
- The obstruction is returned inside `result.obstructions`.
- The unreviewed completion candidate is returned inside
  `result.completion_candidates`.

The CLI should exit `0` when this workflow detects the expected violation and
successfully emits the report. Nonzero exit belongs to CLI usage errors, runtime
construction failures, serialization failures, or output failures.

`RuntimeError` should be structured and machine-readable. It may wrap
`higher_graphen_core::CoreError`, but public runtime APIs must not expose
`anyhow::Error`, boxed dynamic errors, or stringly typed failures as their
stable contract.

## Serialization

Runtime report structs must implement `serde::Serialize` and
`serde::Deserialize` where deserialization is meaningful for tests and
downstream consumers. Field names and enum values must serialize as lower snake
case. Required fields must always be present. Optional fields may be omitted
only when explicitly documented as optional.

JSON object key order is not contractual. Array order is contractual for this
deterministic smoke workflow. Additive fields are allowed only when they do not
rename, remove, or change existing fields. Incompatible report changes require a
new schema identifier and an incremented `report_version`.

Golden JSON tests should compare parsed JSON values or selected fields rather
than raw pretty-printed bytes unless the implementation intentionally locks a
formatter.

## Dependency Direction

`higher-graphen-runtime` may depend on `higher-graphen-core`,
`higher-graphen-structure::space`, `higher-graphen-reasoning::invariant`,
`higher-graphen-reasoning::obstruction`, `higher-graphen-reasoning::completion`,
`higher-graphen-projection`, `higher-graphen-interpretation` only when
interpretation templates or lift adapters are needed, and workspace-approved
serialization dependencies.

`higher-graphen-runtime` must not depend on `tools/highergraphen-cli`, MCP
server packages, provider SDKs, marketplace or plugin manifests, UI frameworks,
or `examples/architecture`.

Lower-level crates must not depend on runtime, CLI, providers, MCP, plugin
packaging, or Architecture Product workflow code. Report-specific adapters,
scenario builders, and convenience views belong inside runtime.

## DDD Review Extension Contract

The DDD review workflow is specified in
[`ddd-review-cli-contract.md`](ddd-review-cli-contract.md). Its command surface
is:

```text
highergraphen ddd input from-case-space --case-space <path> --format json
highergraphen ddd review --input <path> --format json
```

Runtime ownership follows the report-first pattern:

| Surface | Contract |
| --- | --- |
| Workflow name | `ddd_review` |
| Input schema | `highergraphen.ddd_review.input.v1` |
| Report schema | `highergraphen.ddd_review.report.v1` |
| Report type | `ddd_review` |
| CLI consumer | `highergraphen ddd review` |
| Reference fixture | `examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json` |

The workflow may use shared HigherGraphen primitives for source refs,
confidence, review status, cells, incidences, morphism summaries, obstructions,
completion candidates, evidence, and projections. It must keep DDD-specific
labels and invariants, such as bounded context, Customer identity collapse,
anti-corruption mapping, and domain model closeability, out of
`higher-graphen-core`.

The report result must preserve these sections as first-class JSON fields:
`obstructions`, `completion_candidates`, `evidence_boundaries`,
`projection_loss`, `review_gaps`, and `closeability`. Domain findings are
successful report data; they are not runtime errors.

## Implementation Layout

The next implementation task should add:

```text
crates/higher-graphen-runtime/
  Cargo.toml
  src/
    lib.rs
    error.rs
    reports.rs
    workflows/
      mod.rs
      architecture.rs
      architecture_direct_db_access_smoke.rs
```

`lib.rs` owns public re-exports. `error.rs` owns `RuntimeError` and
`RuntimeResult`. `reports.rs` owns `ReportEnvelope`, `ReportMetadata`, and
reusable report helpers. `workflows/architecture.rs` owns architecture workflow
exports. `workflows/architecture_direct_db_access_smoke.rs` owns scenario
construction, result assembly, projection assembly, and the public runner.

If the workflow file approaches the static-analysis size limit in
[`static-analysis-policy.md`](static-analysis-policy.md), split local helpers
into runtime-private modules. Do not move product-specific orchestration down
into core model crates.

## Required Tests

The implementation task should add focused tests that verify:

- The runner returns `Ok(report)` for the deterministic violation scenario.
- `schema`, `report_type`, and `report_version` match this contract.
- Scenario IDs match the stable IDs in this document.
- `result.status` serializes as `violation_detected`.
- The violated check identifies
  `invariant:no-cross-context-direct-database-access`.
- The obstruction has type `invariant_violation`, a counterexample, and a
  required resolution.
- The completion candidate has ID `candidate:billing-status-api`, suggested
  structure ID `cell:billing-status-api`, confidence `0.9`, and serialized
  `review_status: "unreviewed"`.
- The suggested billing API is not present in accepted scenario cells.
- Projection fields include `audience: "human"`,
  `purpose: "architecture_review"`, non-empty `recommended_actions`,
  non-empty `information_loss`, and traceable `source_ids`.
- JSON serialization round-trips through `serde_json` without losing required
  fields.
- Dependency direction remains inward: lower crates do not depend on runtime or
  CLI.

The existing architecture smoke test remains the behavioral reference. Runtime
tests should reproduce its expected facts through the public runner instead of
depending on the example test module.

## Out of Scope
This contract does not require implementing runtime or CLI packages, external
input parsing, persistent storage, candidate review actions, integration
surfaces such as MCP or WebAssembly, or lower-level crate API changes.
