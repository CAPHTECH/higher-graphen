# casegraphen Contract

This document defines the implementable contract for `casegraphen`, the case
and scenario centered intermediate tool in the primary HigherGraphen tool
family. It refines the `higher-graphen-space` row in
[`../intermediate-tools-map.md`](../intermediate-tools-map.md) without changing
core package responsibilities.

`casegraphen` is the HigherGraphen intermediate tool for cases, scenarios,
coverage, missing cases, conflicting cases, and case projections. The first
supported surfaces are CLI plus agent skill. MCP is out of scope.

The next-stage workflow reasoning contract is defined in
[`casegraphen-workflow-reasoning-engine.md`](casegraphen-workflow-reasoning-engine.md).
That document extends this baseline case graph tool toward readiness,
obstruction, completion, evidence-boundary, projection, correspondence, and
evolution reasoning inside the `higher-graphen` workspace.

## Scope

`casegraphen` captures concrete situations and scenarios as structured cases
over a HigherGraphen space. It turns examples, counterexamples, smoke scenarios,
regressions, boundary cases, and conflict probes into traceable graph records.

The tool must answer these questions:

- Which cases and scenarios are represented?
- Which cells, incidences, contexts, boundaries, invariants, or morphisms does
  each case exercise?
- Which declared coverage goals are satisfied, partially satisfied, or
  uncovered?
- Which missing cases should be proposed for review?
- Which cases or scenarios conflict with each other, and why?
- Which source IDs and information loss appear in human, AI-agent, and audit
  projections?

`casegraphen` is domain-neutral. Domain products may interpret a case as an
architecture scenario, migration scenario, incident scenario, research
scenario, or governance scenario, but the tool contract remains about
structured cases over HigherGraphen primitives.

## Conceptual Basis

`casegraphen` is based on a small set of concepts:

| Concept | Contract role |
| --- | --- |
| Case graph | A typed graph of cases, scenarios, coverage goals, and relations. |
| Scenario | A reusable situation template or parameterized path through a space. |
| Situation | The concrete observed or proposed world state represented by a case. |
| Coverage map | The relation between cases and the structures they exercise. |
| Boundary coverage | Checks that important edges, boundaries, contexts, and dimensional transitions are exercised. |
| Missing case | A reviewable candidate case needed to satisfy a declared coverage goal. |
| Conflicting case | A structured contradiction between cases, scenarios, expected outcomes, or observations. |
| Case projection | A lossy view of cases for humans, AI agents, audits, or systems. |

The tool separates these ideas:

- A `Scenario` is a reusable template or situation class.
- A `Case` is a concrete instance, example, counterexample, or regression
  record.
- A `CoverageReport` evaluates representation against declared goals; it does
  not make uncovered structure an error by itself.
- A `MissingCase` is a completion candidate and must remain unreviewed until an
  explicit review action accepts or rejects it.
- A `ConflictingCase` is a domain finding that should be reported with
  provenance and severity, not hidden as a generic validation failure.

## Package And CLI Surface

The intended package split is:

| Surface | Contract |
| --- | --- |
| Primary lower crate | `crates/higher-graphen-space/` |
| Rust crate name | `higher_graphen_space` |
| Intermediate tool package | `tools/casegraphen/` |
| CLI binary | `casegraphen` |
| Tool skill name | `casegraphen` |
| Codex skill path | `integrations/codex/skills/casegraphen/` |
| Claude skill path | `integrations/claude/skills/casegraphen/` |
| Report schema prefix | `highergraphen.case.*.report.v1` |

The first implementation should keep model and evaluator logic in lower crates
and keep command parsing, report envelopes, output paths, and agent procedures
in `tools/casegraphen/` and the skill directories.

Expected Rust-facing concepts:

```rust
pub struct CaseGraph;
pub struct Case;
pub struct Scenario;
pub struct CoverageGoal;
pub struct CoverageReport;
pub struct MissingCase;
pub struct ConflictingCase;
pub struct CaseProjectionReport;

pub fn validate_case_graph(graph: &CaseGraph) -> CaseResult<()>;
pub fn evaluate_coverage(
    graph: &CaseGraph,
    policy: CoveragePolicy,
) -> CaseResult<CoverageReport>;
pub fn detect_missing_cases(
    graph: &CaseGraph,
    policy: CoveragePolicy,
) -> CaseResult<Vec<MissingCase>>;
pub fn detect_conflicts(
    graph: &CaseGraph,
    policy: ConflictPolicy,
) -> CaseResult<Vec<ConflictingCase>>;
```

Minimum CLI commands:

```sh
casegraphen version
casegraphen --version
casegraphen validate --input case.graph.json --format json
casegraphen coverage --input case.graph.json --coverage coverage.policy.json --format json [--output <path>]
casegraphen missing --input case.graph.json --coverage coverage.policy.json --format json [--output <path>]
casegraphen conflicts --input case.graph.json --format json [--output <path>]
casegraphen project --input case.graph.json --projection projection.json --format json [--output <path>]
casegraphen compare --left case.graph.json --right case.graph.json --format json
casegraphen history topology --input case.graph.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
casegraphen history topology diff --left left.case.graph.json --right right.case.graph.json --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
```

Report-producing CLI commands must accept `--format json`. Human-readable text
report output may be added later, but it must derive from the same report data.
The `version` command is a plain text metadata command.

Domain findings are successful command results. Missing cases, partial
coverage, no coverage, and conflicting cases should produce `ok` reports and
exit `0`. Malformed input, invalid primitive values, unreadable files, schema
mismatches, unsupported options, or output failures are tool failures.

## Core Dependencies

`casegraphen` must reuse core primitives:

- `Id`
- `SourceRef`
- `Provenance`
- `Confidence`
- `Severity`
- `ReviewStatus`
- core-owned structured errors where primitive construction fails

Required lower crates:

- `higher-graphen-core` for shared primitives, provenance, review status,
  confidence, severity, and structured errors.
- `higher-graphen-space` for spaces, cells, incidences, complexes, contexts,
  boundaries, and structural locations.

Conditional lower crates:

- `higher-graphen-invariant` when coverage goals reference invariants or
  constraints.
- `higher-graphen-morphism` when cases exercise transformations, migrations,
  projections, or preservation checks.
- `higher-graphen-completion` when missing cases are emitted as reviewable
  completion candidates.
- `higher-graphen-obstruction` when conflicts need to be rendered as
  obstruction-like failure records.
- `higher-graphen-evidence` when case claims need explicit supporting or
  contradicting evidence.
- `higher-graphen-projection` in the tool layer for human, AI-agent, and audit
  projections.

The lower model and evaluator crates must not depend on runtime packages, CLI
packages, tools, apps, provider SDKs, or MCP packages.

## Input Contract

The initial graph input is a JSON document with this top-level shape:

```json
{
  "schema": "highergraphen.case.graph.v1",
  "case_graph_id": "case_graph:architecture-smoke",
  "space_id": "space:architecture-product-smoke",
  "cases": [],
  "scenarios": [],
  "coverage_goals": [],
  "relations": [],
  "review_records": [],
  "metadata": {}
}
```

Required graph fields:

| Field | Contract |
| --- | --- |
| `schema` | Exact schema identifier for the case graph format. |
| `case_graph_id` | Stable `Id` for this case graph. |
| `space_id` | Stable `Id` for the structural universe being evaluated. |
| `cases` | Concrete case records. |
| `scenarios` | Reusable scenario templates or situation classes. |
| `coverage_goals` | Declared coverage requirements. |
| `relations` | Typed edges between cases, scenarios, structures, and sources. |
| `review_records` | Explicit review actions. Empty when no review has occurred. |
| `metadata` | Downstream-owned object; must not carry required semantics. |

### Case

```json
{
  "id": "case:direct-db-access-smoke",
  "space_id": "space:architecture-product-smoke",
  "title": "Direct DB access smoke scenario",
  "case_type": "smoke",
  "situation_summary": "Order Service reads Billing DB directly.",
  "scenario_ids": ["scenario:service-db-boundary"],
  "cell_ids": ["cell:order-service", "cell:billing-db"],
  "incidence_ids": ["incidence:order-service-reads-billing-db"],
  "context_ids": ["context:orders", "context:billing"],
  "expected_outcomes": [
    {
      "id": "outcome:violation-detected",
      "summary": "Direct cross-context DB access is reported as a violation."
    }
  ],
  "observed_outcomes": [],
  "source_ids": ["source:architecture-input"],
  "tags": ["architecture", "boundary"],
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`case_type` values:

- `example`
- `counterexample`
- `smoke`
- `regression`
- `boundary`
- `negative`
- `conflict_probe`
- `scenario_instance`
- `custom:<extension>`

### Scenario

```json
{
  "id": "scenario:service-db-boundary",
  "space_id": "space:architecture-product-smoke",
  "title": "Service crosses a DB ownership boundary",
  "scenario_type": "boundary",
  "parameters": {
    "source_cell_type": "service",
    "target_cell_type": "database"
  },
  "required_context_ids": ["context:orders", "context:billing"],
  "required_cell_types": ["service", "database"],
  "coverage_target_ids": ["coverage:owned-db-access"],
  "source_ids": ["source:architecture-scenario-template"],
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`scenario_type` values:

- `reference`
- `smoke`
- `regression`
- `boundary`
- `negative`
- `adversarial`
- `exploration`
- `custom:<extension>`

### Coverage Goal

```json
{
  "id": "coverage:owned-db-access",
  "space_id": "space:architecture-product-smoke",
  "coverage_type": "boundary",
  "required_ids": [
    "cell:order-service",
    "cell:billing-db",
    "context:orders",
    "context:billing"
  ],
  "dimensions": [0, 1],
  "min_cases_per_target": 1,
  "severity_if_uncovered": "high",
  "source_ids": ["source:architecture-requirements"],
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`coverage_type` values:

- `cell`
- `incidence`
- `context`
- `boundary`
- `invariant`
- `morphism`
- `scenario_matrix`
- `custom:<extension>`

### Case Relation

```json
{
  "id": "relation:case-covers-boundary",
  "relation_type": "covers",
  "from_id": "case:direct-db-access-smoke",
  "to_id": "coverage:owned-db-access",
  "evidence_ids": [],
  "provenance": {
    "source": {"kind": "document"},
    "confidence": 1.0,
    "review_status": "unreviewed"
  }
}
```

`relation_type` values:

- `covers`
- `exercises`
- `contradicts`
- `refines`
- `duplicates`
- `depends_on`
- `derives_from`
- `projects_to`
- `custom:<extension>`

Relation endpoints must reference records in the case graph or stable source
structure IDs in the owning HigherGraphen space.

## Output Contract

All report-producing commands should emit this envelope:

```json
{
  "schema": "highergraphen.case.<operation>.report.v1",
  "report_type": "case_<operation>",
  "report_version": 1,
  "metadata": {
    "command": "casegraphen <operation> ...",
    "tool_package": "tools/casegraphen",
    "core_packages": ["higher-graphen-core", "higher-graphen-space"]
  },
  "input": {},
  "result": {},
  "projection": {}
}
```

Operation-specific result fields:

| Command | Result contract |
| --- | --- |
| `validate` | `result.valid`, `result.errors`, `result.warnings`, and graph counts. |
| `coverage` | `result.coverage_status`, per-goal coverage, represented IDs, uncovered IDs, partially covered IDs, and boundary coverage. |
| `missing` | `result.missing_cases`, each with missing type, target IDs, rationale, confidence, severity, provenance, and `review_status`. |
| `conflicts` | `result.conflicts`, each with case IDs, scenario IDs, conflict type, evidence IDs, severity, explanation, and provenance. |
| `project` | `result.projection_result`, selected source IDs, omitted source IDs, and declared information loss. |
| `compare` | Added, removed, changed, equivalent, conflicting, and not-comparable case records. |
| `history topology` | `result.topology`, `result.source_mapping`, and, only when `--higher-order` is supplied, `result.higher_order`. |
| `history topology diff` | `result.scalar_deltas`, `result.source_mapping_delta`, and, only when both sides include higher-order summaries, `result.higher_order`. |

`history topology` is read-only diagnostics over a deterministic lift of the
case graph into a finite complex. Baseline output omits `result.higher_order`.
When `--higher-order` is supplied, `result.higher_order.options` records
`include_higher_order: true`, optional `max_dimension`, and
`min_persistence_stages`; `cell_count` and `stage_count` summarize the selected
filtration; `filtration_source` identifies deterministic cell order, workflow
revision history, or native morphism log ordering; `stage_sources` maps
generated stages back to revision/morphism records when available;
`persistence` contains `stages`, `intervals`,
`persistent_intervals`, `open_component_count`, and `open_hole_count` when at
least one cell is selected. `--min-persistence` and
`--min-persistence-stages` are aliases for the same stage-lifetime threshold.
Higher-order topology findings are diagnostic signals and must not be treated
as coverage, completion, or blocker decisions by themselves.

`history topology diff` compares two lifted topology reports. It reports scalar
topology deltas, added/removed source node and relation IDs, and optional
higher-order summary deltas when `--higher-order` is supplied. It is not a full
JSON patch and does not mutate either input graph.

`projection` must include:

- `human_review`: concise case summaries, missing-case prompts, conflict
  explanations, and recommended review actions.
- `ai_view`: source-stable case, scenario, coverage, missing-case, and conflict
  records.
- `audit_trace`: per-source coverage, source IDs represented in the report, and
  declared information loss.

Missing cases and conflicts must remain visible in AI and audit projections. A
human summary may be concise, but it must disclose when coverage is incomplete
or when conflicts are unresolved.

## Invariants

- Every case graph has a non-empty `case_graph_id`, `space_id`, and exact
  schema identifier.
- Every case, scenario, coverage goal, relation, and review record has a stable
  ID.
- Every case, scenario, and coverage goal belongs to the same `space_id` as the
  graph unless an explicit cross-space relation is supported by a later schema.
- Relation endpoints must resolve to records in the case graph or source
  structure IDs in the referenced HigherGraphen space.
- A coverage goal must not be reported as covered unless at least one case or
  relation explicitly covers or exercises every required target under the
  selected policy.
- A missing case must name the uncovered target IDs and remain
  `review_status: "unreviewed"` until an explicit review action changes it.
- A conflicting case finding must reference at least two distinct records or
  one record plus one incompatible source structure.
- Conflicts must carry a conflict type, severity, explanation, provenance, and
  source IDs.
- Projections must declare represented source IDs and information loss.
- Projection output must not promote an unreviewed missing case into an
  accepted case.

## Failure Modes

Tool errors:

- unreadable input, coverage policy, projection definition, or output path;
- malformed JSON;
- invalid `Id`, `Confidence`, `Severity`, or `ReviewStatus`;
- missing required graph fields;
- dangling relation endpoint inside the case graph;
- unsupported CLI option;
- serialization or schema-validation failure.

Domain results:

- no cases provided;
- no coverage goals provided;
- coverage goals unmet;
- partially covered boundary or context;
- missing cases detected;
- conflicting cases detected;
- recognized projection produced with declared information loss;
- compare operation found non-equivalent or not-comparable case graphs.

## Validation Expectations

Implementation must include:

- constructor and serde tests for case graph, case, scenario, coverage goal,
  relation, missing case, conflicting case, and report structs;
- schema and fixture tests for `highergraphen.case.*.report.v1`;
- CLI tests proving `--format json`, `--output`, exit behavior, and invalid
  input errors;
- semantic tests proving coverage evaluation respects declared targets,
  dimensions, contexts, and boundary coverage;
- tests proving missing cases preserve uncovered target IDs, confidence,
  severity, provenance, and `ReviewStatus`;
- tests proving conflicts preserve both sides of the conflict, source IDs,
  severity, provenance, and evidence links when present;
- projection tests proving human, AI-agent, and audit views preserve source
  IDs, declare information loss, and do not hide missing or conflicting cases;
- negative tests for dangling relation endpoints, cross-space references,
  invalid primitive values, and unsupported schema identifiers.

## Non-Goals

- MCP server behavior.
- UI workflows.
- Provider SDK integration.
- General source ingestion or semantic lifting from raw text. Bounded lift
  adapters may create case graphs, but this contract starts after source
  material has become structured cases, scenarios, coverage goals, and
  relations.
- Executing scenarios against external systems.
- Replacing `evidencegraphen`, `invariantgraphen`, `completiongraphen`,
  `obstructiongraphen`, or `projectiongraphen`.
- Automatically accepting missing cases or resolving conflicts without an
  explicit review action.
- Mutating the source HigherGraphen space from a projection command.

## First Implementation Tasks

1. Create `tools/casegraphen/` with a `casegraphen` CLI entry point and JSON
   report envelope support.
2. Define serde records for case graphs, cases, scenarios, coverage goals,
   case relations, missing cases, conflicting cases, and operation reports.
3. Add JSON schemas and fixtures for `highergraphen.case.graph.v1` and the
   first `highergraphen.case.*.report.v1` reports.
4. Implement `casegraphen validate` with strict primitive validation and
   relation endpoint checks.
5. Implement deterministic coverage evaluation for cells, incidences,
   contexts, boundaries, and scenario matrices.
6. Implement missing-case detection as reviewable completion candidates without
   promoting them to accepted facts.
7. Implement conflict detection for contradictory outcomes, incompatible
   coverage claims, duplicate cases with incompatible expectations, and
   scenario/source mismatches.
8. Implement `casegraphen project` through the projection layer with human,
   AI-agent, and audit views.
9. Add initial `casegraphen` skills under `integrations/codex/skills/` and
   `integrations/claude/skills/` that call the CLI and load schema references
   on demand.
10. Add contract tests and CLI fixtures before expanding into domain product
    workflows.
