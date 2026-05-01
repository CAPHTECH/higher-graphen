# casegraphen Current Surface Inventory

Status: inventory for case `casegraphen-highergraphen-rewrite`, task
`task_casegraphen_current_surface_inventory`.

This document maps the current HigherGraphen `casegraphen` intermediate tool
surface to the workflow reasoning engine target. It is descriptive only: it
does not change Rust code, schemas, or the external
`casegraphen reference workspace` repository.

## Sources Inspected

- `tools/casegraphen/`
- `schemas/casegraphen/`
- `docs/specs/intermediate-tools/casegraphen.md`
- `docs/specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md`
- `tools/casegraphen/tests/command.rs`
- `examples/architecture/reference/`
- `examples/architecture/tests/reference_workflow.rs`
- repo-owned skill directories under `skills/` and `integrations/`

## Current Package Surface

| Surface | Current implementation |
| --- | --- |
| Package | `tools/casegraphen`, Rust package `casegraphen`, binary `casegraphen`. |
| Entry point | `tools/casegraphen/src/main.rs` delegates to `casegraphen::cli::main_entry()`. |
| CLI module | `tools/casegraphen/src/cli.rs` parses commands and requires `--format json`. |
| Model module | `tools/casegraphen/src/model.rs` defines `CaseGraph`, `CaseRecord`, `Scenario`, `CoverageGoal`, `CaseRelation`, `ReviewRecord`, `CoveragePolicy`, `ProjectionDefinition`, `MissingCase`, and `ConflictingCase`. |
| Evaluation module | `tools/casegraphen/src/eval.rs` validates graph structure, evaluates coverage, detects missing cases, detects conflicts, compares graphs, and builds a projection result. |
| Report module | `tools/casegraphen/src/report.rs` emits `highergraphen.case.<operation>.report.v1` envelopes with `human_review`, `ai_view`, and `audit_trace` projections. |
| Store module | `tools/casegraphen/src/store.rs` reads strict JSON inputs, enforces schema identifiers, writes reports, creates graph files, and lists local graph files. |
| Lower dependencies | `higher-graphen-core`, `higher-graphen-structure::space`, and `higher-graphen-projection` are declared; the current code directly relies on core IDs, provenance, confidence, review status, and severity. |

The current CLI is file based. It does not use MCP, provider SDKs, runtime
services, or the external CaseGraphen repository.

## Current CLI Commands

| Command | Current behavior | Workflow reasoning relevance |
| --- | --- | --- |
| `casegraphen create` | Creates an empty `highergraphen.case.graph.v1` graph in a local directory. | Provides a file-based graph creation seed, but not a workflow state transition. |
| `casegraphen inspect` | Reads one graph and reports IDs and counts. | Useful as a future workflow-space summary surface. |
| `casegraphen list` | Lists graph files in a local store. | Provides store discovery, but no revision or evolution model. |
| `casegraphen validate` | Checks duplicate IDs, same-space membership, relation endpoints, review targets, and warning conditions. | Baseline structural validation. It does not validate readiness, evidence requirements, transitions, or workflow invariants. |
| `casegraphen coverage` | Evaluates coverage goals against represented cell, incidence, context, scenario, and explicit `covers`/`exercises` relations. | Closest current analog to proof coverage, but it is coverage-oriented rather than work-readiness-oriented. |
| `casegraphen missing` | Emits unreviewed `MissingCase` records for uncovered coverage targets. | Existing completion boundary for missing cases. It does not cover missing task, evidence, test, decision, dependency, projection, or review action candidates. |
| `casegraphen conflicts` | Emits `ConflictingCase` findings from `contradicts` relations and expected-vs-observed outcome mismatches. | Existing obstruction-like finding surface, but not typed as an obstruction report. |
| `casegraphen project` | Reads a projection definition and emits selected source IDs, omitted source IDs, information loss, and the standard projections. | Existing projection-loss boundary. The current definition is mostly validated rather than used to tailor output. |
| `casegraphen compare` | Reports equivalent, added, removed, changed, conflicting, and not-comparable case IDs. | Seed for correspondence, but currently limited to case-level equality and diffing. |
| `casegraphen history topology` | Emits lifted first-order topology diagnostics for a case graph. `--higher-order` adds optional persistence diagnostics with `--max-dimension` and `--min-persistence` / `--min-persistence-stages`. | Provides a read-only structural signal over the graph. Higher-order persistence is opt-in and diagnostic; baseline output omits `result.higher_order`. |
| `casegraphen history topology diff` | Compares two lifted topology reports and emits scalar topology deltas plus source-mapping added/removed IDs. `--higher-order` adds summary deltas when both sides include higher-order summaries. | Provides topology-specific pairwise change detection without overloading raw case `compare` or workflow correspondence. |
| `casegraphen cg workflow history topology` | Replays a stored workflow graph and, with `--higher-order`, orders filtration stages from workflow revision history. | Store-backed topology can expose `filtration_source: workflow_history` and `stage_sources` for revision-aware diagnostics. |
| `casegraphen case history topology diff` | Replays two native case spaces and compares their topology reports. | Native topology diff uses morphism-log-aware topology reports and emits `result.topology_diff` without mutating either store. |

All commands require `--format json`. `--output` writes pretty JSON to a file
and suppresses stdout. Domain findings such as missing cases, conflicts, and
partial coverage are successful command results. Malformed input, unsupported
schema identifiers, invalid primitive values, unsupported options, and I/O
failures are CLI failures.

## Current Schemas And Fixtures

| File | Current contract |
| --- | --- |
| `schemas/casegraphen/case.graph.schema.json` | Strict `highergraphen.case.graph.v1` input with top-level `case_graph_id`, `space_id`, cases, scenarios, coverage goals, relations, review records, and metadata. |
| `schemas/casegraphen/coverage.policy.schema.json` | `highergraphen.case.coverage_policy.v1` with selected coverage goal IDs, explicit-relation mode, and metadata. |
| `schemas/casegraphen/projection.schema.json` | `highergraphen.case.projection.v1` with audience, source inclusion flag, and metadata. |
| `schemas/casegraphen/case.report.schema.json` | Shared report envelope for `highergraphen.case.*.report.v1` commands. The schema intentionally keeps operation result payloads broad, but validates the optional `result.higher_order` fragment used by topology reports when present. |
| `schemas/casegraphen/*.example.json` | Minimal graph, coverage policy, and projection fixtures used by command tests. |
| `examples/architecture/reference/casegraphen-reference.*.json` | Reference workflow graph, policy, projection request, and generated reports. |

The v1 graph schema and Rust structs are strict: unknown fields are rejected.
Most semantic types such as `case_type`, `scenario_type`, `coverage_type`, and
`relation_type` are strings rather than closed enums, which leaves room for
extension values but also means the current schema does not enforce the
documented value sets.

## Tests And Example Coverage

| Location | Current coverage |
| --- | --- |
| `tools/casegraphen/tests/command.rs` | CLI report shape, coverage and missing as successful domain reports, projection source preservation, local create/list/inspect, `--output`, invalid schema errors, and JSON validity of schemas/fixtures. |
| Unit tests in `model.rs` and `store.rs` | Empty graph schema, primitive validation during deserialization, unreviewed missing-case boundary, and local store round trip. |
| `examples/architecture/tests/reference_workflow.rs` | End-to-end architecture reference checks, including runtime obstruction/completion reports and `casegraphen` validation, coverage, and projection over the reference case graph. |
| `examples/architecture/reference/reports/` | Checked-in reports for runtime lift/smoke/review and `casegraphen` validate/coverage/project. |

No repo-owned `casegraphen` agent skill was found under `skills/` or
`integrations/`. The current usable skill surface is therefore the CLI plus
global agent instructions outside this repository.

## Mapping To Workflow Reasoning Target

| Workflow reasoning target | Current surface | Gap to close |
| --- | --- | --- |
| Workflow case graph | `CaseGraph` names a `space_id` and stores cases, scenarios, coverage goals, relations, reviews, and metadata. | It references HigherGraphen structure IDs but does not model workflow work items, states, constraints, waits, evidence requirements, or transitions as first-class graph records. |
| Case | `CaseRecord` captures concrete examples, smoke cases, review cases, expected and observed outcomes, source IDs, tags, and provenance. | Case records have no workflow lifecycle state beyond provenance review status. Work item state must be additive or versioned. |
| Scenario | `Scenario` captures reusable situation patterns with parameters and coverage targets. | Scenario matching and parameterized workflow transfer are not implemented. |
| Work item | No dedicated type. Cases, scenarios, and relations can approximate some work structure. | Need typed work items such as task, goal, decision, event, evidence, proof, external wait, and review action. |
| Relation or incidence | `CaseRelation` has `relation_type`, endpoints, evidence IDs, and provenance; validation allows internal graph IDs plus stable external structure prefixes. | Need workflow relation semantics for hard dependency, wait, evidence requirement, readiness constraint, derivation, transition, projection, and accepted/rejected review. |
| Readiness projection | None. Coverage can show represented and uncovered IDs. | Need derived ready/not-ready results with dependency IDs, wait IDs, evidence requirement IDs, and obstruction IDs. |
| Blocker or obstruction | `ConflictingCase` reports contradictions and outcome mismatches. Runtime architecture examples emit obstructions outside `tools/casegraphen`. | Need casegraphen-owned obstruction records and reports for blockers, invalid transitions, impossible closure, missing evidence, failed constraints, and conflict witnesses. |
| Completion candidate | `MissingCase` is generated for uncovered coverage targets and remains `review_status: "unreviewed"`. Runtime architecture examples emit completion candidates outside `tools/casegraphen`. | Need generalized workflow completions for missing task, evidence, test, decision, dependency relation, case, projection, and review action. |
| Evidence boundary | Cases, relations, missing cases, and conflicts carry provenance, source IDs, and sometimes evidence IDs. | There is no evidence record model, source-backed proof attachment model, or promotion path from inference to accepted evidence inside casegraphen. |
| Projection loss | Every report has `projection`; project reports include information-loss strings and selected/omitted source IDs. | Projection definitions are not yet used to tailor human, AI, audit, or system views; omitted source IDs are currently always empty in operation projection. |
| Correspondence | `compare` reports case-level equality, added/removed/changed IDs, conflicts, and not-comparable space IDs. | Need structural correspondence: equivalent, similar-with-loss, scenario-pattern match, conflict, non-comparable, and transferable mitigation/completion pattern with mismatch evidence. |
| Evolution | Local store create/list records graph files. Review records can carry `reviewed_at`. | No revision-indexed graph model, event history, transition record, or answer to when blockers/proof/completions appeared. |
| Patch or morphism | `compare` can identify differences between two graphs. | No patch, transition, morphism, invariant-preservation, or reviewable transformation model. |
| AI skill surface | No repo-owned casegraphen skill found. | Need Codex/Claude skill docs that call the CLI, load schemas on demand, and preserve evidence/review boundaries. |

## Current Gaps Against The Target

1. No first-class workflow item model.
   The current graph can describe situations, but not ready work, in-progress
   work, external waits, decisions, events, or evidence obligations.

2. No readiness engine.
   There is no derived frontier, dependency resolver, wait-state resolver, or
   evidence-requirement checker.

3. Obstruction support is partial and indirect.
   Conflict findings are obstruction-like, and runtime examples already
   contain obstructions, but `casegraphen` has no owned obstruction schema or
   report.

4. Completion support is limited to missing cases.
   `MissingCase` establishes the review boundary, but the workflow target
   needs additional missing-structure candidates.

5. Evidence is represented by references, not records.
   `source_ids`, `evidence_ids`, provenance, confidence, and review status are
   useful anchors, but the tool cannot yet distinguish source-backed evidence
   records from AI-generated inference records in its own graph.

6. Projection configuration is shallow.
   The tool validates a projection definition, but the output does not yet vary
   meaningfully by audience or `include_sources`.

7. Correspondence is only a case diff.
   The current comparison does not reason over scenarios, coverage goals,
   structure-preserving mappings, information loss, or transferable patterns.

8. Evolution is absent.
   The current local store is a file collection, not a revisioned workflow
   history.

9. Repo-owned skill surfaces are absent.
   CLI usage is test-covered, but agent procedures for workflow reasoning are
   not yet checked in.

10. Documentation and implementation have a few contract details to reconcile.
    The baseline document describes enumerated semantic values and richer
    conflict detection than the code enforces today. The next contract task
    should decide whether to preserve string extension points, tighten only new
    schemas, or version stricter v2 contracts.

## Compatibility Constraints For The Next Contract Task

- Keep `highergraphen.case.graph.v1`, `highergraphen.case.coverage_policy.v1`,
  `highergraphen.case.projection.v1`, and existing
  `highergraphen.case.<operation>.report.v1` outputs valid.
- Keep current commands and flags working, especially `--format json` and
  `--output`.
- Add workflow reasoning reports with additive schema identifiers instead of
  overloading existing coverage, missing, conflicts, project, or compare
  result shapes.
- Preserve the distinction between domain findings and tool failures. Missing
  cases, conflicts, blockers, partial coverage, and not-comparable structures
  should remain successful JSON reports unless the input or command is invalid.
- Preserve the review boundary: generated missing cases and workflow
  completions stay unreviewed until an explicit review record or command
  accepts or rejects them.
- Preserve the evidence boundary: AI inference must not become accepted
  evidence merely because it appears in a projection.
- Preserve projection disclosure. Human, AI-agent, audit, and future workflow
  projections must declare represented source IDs and information loss.
- Do not add workflow-only fields to strict v1 input records unless the schema
  version changes. Prefer additive workflow schemas or new record arrays with
  explicit versioning.
- Continue accepting stable external HigherGraphen IDs in relations, or define
  a versioned prefix policy for workflow-specific IDs before validation starts
  enforcing them.
- Keep implementation inside `tools/casegraphen`, `schemas/casegraphen`, and
  repository-owned docs/skills. The external CaseGraphen repository remains out
  of scope for this rewrite case.
- Do not introduce MCP or provider-specific SDK requirements into the
  intermediate tool contract.
- Keep lower dependency direction clean: model and evaluator code should depend
  on core/HigherGraphen primitives, not runtime products or agent integrations.

## Contract Task Recommendations

The next contract task should define new, versioned workflow records before
implementation:

- `WorkflowCaseGraph` or an additive workflow section that can coexist with
  `CaseGraph`;
- `WorkItem`, `WorkflowRelation`, `ReadinessRule`, `ReadinessReport`;
- `ObstructionReport` and obstruction witness records;
- generalized `CompletionCandidate` records with explicit review status;
- evidence records and evidence-requirement records;
- audience-specific projection definitions with machine-checkable information
  loss;
- correspondence records that separate equivalence, similarity, conflict, and
  non-comparability;
- revision or transition records for evolution.

Those records should reuse `Id`, `SourceRef`, `Provenance`, `Confidence`,
`Severity`, and `ReviewStatus`, and should keep current casegraphen reports
stable while extending the tool toward workflow reasoning.
