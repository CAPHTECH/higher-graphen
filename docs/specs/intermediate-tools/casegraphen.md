# casegraphen Contract

This document defines the implementable contract for `casegraphen`, the
higher-order structure operation tool in the primary HigherGraphen tool
family. It refines the `higher-graphen-structure::space` row in
[`../intermediate-tools-map.md`](../intermediate-tools-map.md) without changing
core package responsibilities.

`casegraphen` is the HigherGraphen intermediate tool for lifting bounded source
snapshots into case spaces, deriving obstructions and completions, checking
invariants, applying reviewed morphisms, comparing structural correspondence,
and projecting lossy views. Cases and scenarios are important cell types, but
they are not the command model. The first supported surfaces are CLI plus agent
skill. MCP is out of scope.

The next-stage workflow reasoning contract is defined in
[`casegraphen-workflow-reasoning-engine.md`](casegraphen-workflow-reasoning-engine.md).
That document records the transitional workflow reasoning surface that should
be lifted into the higher-order command model described here.

## Scope

`casegraphen` captures concrete situations and scenarios as structured cells
inside a replayable case space. It turns examples, counterexamples, smoke
scenarios, regressions, boundary cases, conflict probes, workflow records, and
external snapshots into traceable structure that can be transformed only
through reviewed morphisms.

The tool must answer these questions:

- Which bounded sources were lifted into the case space, and what information
  was lost during the lift?
- Which cells, relations, contexts, boundaries, invariants, projections, and
  morphisms are present in the current replayed space?
- Which hard dependencies, waits, missing evidence, missing proof, conflicts,
  policy gates, or projection losses obstruct readiness or closure?
- Which missing cells, relations, evidence, reviews, projections, or morphisms
  should be proposed as reviewable completions?
- Which morphisms preserve required invariants, and which violate them?
- Which source IDs, omitted IDs, and information loss appear in human,
  AI-agent, audit, system, and migration projections?
- Are two spaces, revisions, or projections equivalent, similar with loss,
  conflicting, or not comparable?

`casegraphen` is domain-neutral. Domain products may interpret a case as an
architecture scenario, migration scenario, incident scenario, research
scenario, or governance scenario, but the tool contract remains about
structured higher-order operations over HigherGraphen primitives.

## Conceptual Basis

`casegraphen` is based on higher-order structural operations:

| Concept | Contract role |
| --- | --- |
| Source boundary | The bounded source snapshot that may be treated as accepted input. |
| Lift | A source-to-case-space morphism that records represented IDs, generated IDs, and information loss. |
| Case space | The replayed HigherGraphen-compatible space of cells and relations. |
| Case cell | A typed unit such as case, scenario, goal, work, decision, evidence, proof, obstruction, completion, projection, revision, morphism, or external reference. |
| Case relation | A typed incidence between cells or external structures, including dependency, evidence, proof, contradiction, projection, transition, correspondence, review, and policy relations. |
| Morphism | A reviewable transformation from one case-space revision to another. |
| Obstruction | A domain finding that prevents readiness, closure, projection, correspondence, or morphism application under selected rules. |
| Completion | A reviewable candidate for missing or corrective structure. |
| Projection | A lossy view with represented IDs, omitted IDs, information loss, and allowed operations. |
| Equivalence | A structural correspondence result between spaces, revisions, or projections. |
| Invariant | A rule over readiness, evidence, projection, policy, closeability, or morphism preservation. |

The tool separates these ideas:

- A source `lift` is the boundary-crossing operation; legacy graph or workflow
  JSON is an input adapter, not the durable model.
- A `CaseSpace` replayed from a morphism log is the product object; reports and
  CLI output are projections over that object.
- A `Case` or `Scenario` is a cell inside the space, not a top-level command
  namespace.
- A generated obstruction, completion, inferred evidence item, or morphism is a
  domain finding and must remain unreviewed until an explicit review morphism
  accepts or rejects it.
- A projection can expose actions, but it cannot mutate state, promote
  evidence, accept completions, or hide information loss.
- Invariant failures, missing evidence, unresolved obstructions, and
  non-equivalent correspondences are successful domain reports unless the input
  itself is malformed or unreadable.

## Package And CLI Surface

The intended package split is:

| Surface | Contract |
| --- | --- |
| Primary lower crate | `crates/higher-graphen-structure/` module `space` |
| Rust crate name | `higher_graphen_structure` module `space` |
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
pub struct CaseSpace;
pub struct CaseCell;
pub struct CaseRelation;
pub struct CaseMorphism;
pub struct MorphismLog;
pub struct SourceBoundary;
pub struct LiftReport;
pub struct ObstructionReport;
pub struct CompletionReport;
pub struct ProjectionReport;
pub struct EquivalenceReport;
pub struct InvariantReport;

pub fn lift_source(input: SourceSnapshot, adapter: LiftAdapter) -> CaseResult<LiftReport>;
pub fn replay_space(log: &MorphismLog) -> CaseResult<CaseSpace>;
pub fn validate_space(space: &CaseSpace) -> CaseResult<InvariantReport>;
pub fn reason_space(space: &CaseSpace) -> CaseResult<CaseReasoningReport>;
pub fn propose_morphism(space: &CaseSpace, proposal: MorphismProposal) -> CaseResult<CaseMorphism>;
pub fn check_morphism(space: &CaseSpace, morphism: &CaseMorphism) -> CaseResult<InvariantReport>;
pub fn project_space(space: &CaseSpace, projection: ProjectionDefinition) -> CaseResult<ProjectionReport>;
pub fn compare_spaces(left: &CaseSpace, right: &CaseSpace) -> CaseResult<EquivalenceReport>;
```

Minimum CLI commands:

```sh
casegraphen version
casegraphen --version
casegraphen lift workflow --input workflow.graph.json --store <dir> --revision-id <id> --format json [--output <path>]
casegraphen lift case-graph --input case.graph.json --store <dir> --revision-id <id> --format json [--output <path>]
casegraphen space new --store <dir> --case-space-id <id> --space-id <id> --title <text> --revision-id <id> --format json [--output <path>]
casegraphen space list --store <dir> --format json [--output <path>]
casegraphen space inspect --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen space replay --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen space validate --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen space reason --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen space topology --store <dir> --case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
casegraphen space topology diff --left-store <dir> --left-case-space-id <id> --right-store <dir> --right-case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
casegraphen morphism propose --store <dir> --case-space-id <id> --input case_morphism.json --format json [--output <path>]
casegraphen morphism check --store <dir> --case-space-id <id> --morphism-id <id> --format json [--output <path>]
casegraphen morphism apply --store <dir> --case-space-id <id> --morphism-id <id> --base-revision-id <id> --reviewer-id <id> --reason <text> --format json [--output <path>]
casegraphen morphism reject --store <dir> --case-space-id <id> --morphism-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json [--output <path>]
casegraphen obstruction list --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen completion candidates --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen projection apply --store <dir> --case-space-id <id> --projection <projection.json> --format json [--output <path>]
casegraphen equivalence check --left-store <dir> --left-case-space-id <id> --right-store <dir> --right-case-space-id <id> --format json [--output <path>]
casegraphen invariant check --store <dir> --case-space-id <id> --format json [--output <path>]
casegraphen invariant close-check --store <dir> --case-space-id <id> --base-revision-id <id> --validation-evidence-id <id> --format json [--output <path>]
```

The legacy `casegraphen create`, `inspect`, `list`, `validate`, `coverage`,
`missing`, `conflicts`, `project`, `compare`, and top-level `history topology`
commands are transitional aliases for the old `highergraphen.case.graph.v1`
surface. The destructive redesign removes them from the canonical command
surface. Their remaining value is represented by `lift case-graph`, `space
validate`, `obstruction list`, `completion candidates`, `projection apply`,
`equivalence check`, and `space topology`.

Report-producing CLI commands must accept `--format json`. Human-readable text
report output may be added later, but it must derive from the same report data.
The `version` command is a plain text metadata command.

Domain findings are successful command results. Missing evidence, missing
proof, unresolved obstructions, unreviewed completions, invariant failures,
projection loss, and non-equivalent correspondence should produce `ok` reports
and exit `0`. Malformed input, invalid primitive values, unreadable files,
schema mismatches, unsupported options, or output failures are tool failures.

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
- `higher-graphen-structure::space` for spaces, cells, incidences, complexes, contexts,
  boundaries, and structural locations.

Conditional lower crates:

- `higher-graphen-reasoning::invariant` when coverage goals reference invariants or
  constraints.
- `higher-graphen-structure::morphism` when cases exercise transformations, migrations,
  projections, or preservation checks.
- `higher-graphen-reasoning::completion` when missing cases are emitted as reviewable
  completion candidates.
- `higher-graphen-reasoning::obstruction` when conflicts need to be rendered as
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
    "core_packages": ["higher-graphen-core", "higher-graphen-structure::space"]
  },
  "input": {},
  "result": {},
  "projection": {}
}
```

Operation-specific result fields:

| Command | Result contract |
| --- | --- |
| `lift` | `result.case_space_id`, source boundary, adapter metadata, represented source IDs, generated IDs, omitted IDs, lift information loss, and initial revision ID. |
| `space validate` | `result.valid`, invariant errors, warnings, replay checksums, source-boundary status, graph counts, and projection cache status. |
| `space reason` | readiness, frontier, obstructions, completions, evidence findings, projection summaries, and closeability hints. |
| `space topology` | `result.topology`, `result.source_mapping`, and, only when `--higher-order` is supplied, `result.higher_order`. |
| `space topology diff` | `result.scalar_deltas`, `result.source_mapping_delta`, and, only when both sides include higher-order summaries, `result.higher_order`. |
| `morphism check` | preservation status, invariant results, stale-base diagnostics, required review/evidence findings, and violated invariant IDs. |
| `obstruction list` | `result.obstructions`, each with witness IDs, blocked target IDs, severity, provenance, review status, and recommended completion types. |
| `completion candidates` | `result.completions`, each with candidate type, target IDs, rationale, confidence, severity, provenance, and `review_status`. |
| `projection apply` | represented IDs, omitted IDs, information loss, allowed operations, hidden blocker warnings, and source IDs required to interpret the view. |
| `equivalence check` | equivalent, similar-with-loss, conflicting, not-comparable, transferable pattern, and mismatch witness records. |
| `invariant check` / `invariant close-check` | invariant pass/fail results, closeability, policy/capability gates, validation evidence, and residual risks. |

`space topology` is read-only diagnostics over a deterministic lift of the
case space into a finite complex. Baseline output omits `result.higher_order`.
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

`space topology diff` compares two lifted topology reports. It reports scalar
topology deltas, added/removed source node and relation IDs, and optional
higher-order summary deltas when `--higher-order` is supplied. It is not a full
JSON patch and does not mutate either input space.

`projection` must include:

- `human_review`: concise open reviews, blockers, close failures, completion
  prompts, and recommended review actions.
- `ai_view`: source-stable cells, relations, obstructions, completions,
  evidence boundaries, morphism candidates, and allowed operations.
- `audit_trace`: per-source provenance, source IDs represented in the report,
  review decisions, morphism history, and declared information loss.

Completions, obstructions, inferred evidence, and rejected morphisms must remain
visible in AI and audit projections. A human summary may be concise, but it
must disclose unresolved blockers, missing proof/evidence, unreviewed
completions, and projection loss.

## Invariants

- Every case space has a non-empty `case_space_id`, `space_id`, exact schema
  identifier, and declared source boundary.
- Every cell, relation, morphism, projection, and revision has a stable ID.
- Every materialized revision is replayable from the morphism log.
- Relation endpoints must resolve to records in the case space or declared
  external structure IDs in the referenced HigherGraphen space.
- A hard requirement must not be reported as satisfied unless accepted,
  source-backed, or review-promoted evidence/proof satisfies it under the
  selected policy.
- A generated completion must name target IDs and remain
  `review_status: "unreviewed"` until an explicit review morphism changes it.
- An obstruction must reference stable witness IDs and carry type, severity,
  explanation, provenance, source IDs, and review status.
- A morphism must not be applied unless its source revision matches the replayed
  revision and required invariant checks pass or are explicitly waived.
- Projections must declare represented IDs, omitted IDs, source IDs, allowed
  operations, and information loss.
- Projection output must not promote unreviewed completions, inferred evidence,
  or generated morphisms into accepted state.

## Failure Modes

Tool errors:

- unreadable input, store, source snapshot, morphism, projection definition, or
  output path;
- malformed JSON;
- invalid `Id`, `Confidence`, `Severity`, or `ReviewStatus`;
- missing required case-space, source-boundary, or morphism fields;
- dangling relation endpoint inside the case space;
- unsupported CLI option;
- serialization or schema-validation failure.

Domain results:

- empty or minimal case spaces;
- no goals or close policy provided;
- required goals unmet;
- missing evidence or proof;
- unresolved obstructions;
- unreviewed completion candidates;
- invalid or rejected morphism candidates;
- recognized projection produced with declared information loss;
- equivalence operation found non-equivalent or not-comparable spaces.

## Validation Expectations

Implementation must include:

- constructor and serde tests for case space, cell, relation, morphism log,
  source boundary, obstruction, completion, projection, equivalence, invariant,
  and report structs;
- schema and fixture tests for `highergraphen.case.*.report.v1`;
- CLI tests proving `--format json`, `--output`, exit behavior, and invalid
  input errors;
- semantic tests proving readiness, obstruction, completion, evidence,
  projection, equivalence, and invariant checks respect declared relations,
  dimensions, contexts, source boundaries, and policies;
- tests proving completions preserve target IDs, confidence, severity,
  provenance, and `ReviewStatus`;
- tests proving obstructions preserve witnesses, source IDs, severity,
  provenance, and evidence links when present;
- projection tests proving human, AI-agent, audit, system, and migration views
  preserve source IDs, declare information loss, and do not hide obstructions
  or unreviewed completions;
- negative tests for dangling relation endpoints, cross-space references,
  invalid primitive values, and unsupported schema identifiers.

## Non-Goals

- MCP server behavior.
- UI workflows.
- Provider SDK integration.
- General source ingestion or semantic lifting from raw text. Bounded lift
  adapters may create case spaces from structured source snapshots, but raw
  source interpretation remains outside this contract.
- Executing scenarios against external systems.
- Replacing `evidencegraphen`, `invariantgraphen`, `completiongraphen`,
  `obstructiongraphen`, or `projectiongraphen`.
- Automatically accepting completions, evidence, or morphisms without an
  explicit review action.
- Mutating the source HigherGraphen space from a projection command.

## First Implementation Tasks

1. Create or update `tools/casegraphen/` with a `casegraphen` CLI entry point
   and JSON report envelope support.
2. Define serde records for case spaces, cells, relations, source boundaries,
   morphism logs, obstructions, completions, projections, equivalence results,
   invariants, and operation reports.
3. Add JSON schemas and fixtures for native case spaces, morphisms, lift
   reports, and the higher-order `highergraphen.case.*.report.v1` reports.
4. Implement `lift case-graph` and `lift workflow` adapters for existing
   structured sources.
5. Implement `space validate`, `space replay`, `space reason`, and `space
   topology` over the replayed morphism log.
6. Implement `obstruction list` and `completion candidates` as review-boundary
   reports without promoting generated records to accepted facts.
7. Implement `morphism propose`, `morphism check`, `morphism apply`, and
   `morphism reject` with source-revision and invariant checks.
8. Implement `projection apply`, `equivalence check`, `invariant check`, and
   `invariant close-check` through shared package APIs.
9. Add or update `casegraphen` skills under `integrations/codex/skills/` and
   `integrations/claude/skills/` that call the CLI and load schema references
   on demand.
10. Add contract tests and CLI fixtures before expanding into domain product
    workflows.
