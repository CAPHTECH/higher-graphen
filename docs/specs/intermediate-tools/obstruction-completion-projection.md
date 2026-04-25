# Obstruction, Completion, and Projection Tool Contracts

This document defines implementable contracts for the first
obstruction/completion/projection intermediate tools:
`obstructiongraphen`, `completiongraphen`, and `projectiongraphen`.

The contracts refine the responsibilities in
[`../intermediate-tools-map.md`](../intermediate-tools-map.md),
[`../package-boundaries.md`](../package-boundaries.md),
[`../non-core-package-workplans.md`](../non-core-package-workplans.md), and
[`../runtime-workflow-contract.md`](../runtime-workflow-contract.md). MCP is
out of scope for these contracts. The immediate implementation target is
package, runtime, CLI, schema, and skill surfaces.

## Shared Contract

These tools are operational intermediate tools, not core model crates. The core
crates own reusable records and engines. The `*graphen` tools own command
surfaces, report envelopes, validation entry points, workflow composition, and
agent-facing procedures around those records.

Shared rules:

- Tool packages live under `tools/<tool-name>/`.
- Installed command names use the bare tool name, such as
  `obstructiongraphen`.
- Rust model types remain in `crates/higher-graphen-*`.
- Runtime product workflows may call the same lower crates directly when a
  bundled product report is more appropriate than a standalone tool command.
- JSON output must be the first stable format.
- Human-readable text output may be added later, but it must derive from the
  same report data.
- Tool reports must use stable lower snake case field names and enum values.
- Domain findings are report data. Tool failures are reserved for invalid
  input, validation failures, unsupported options, serialization failures, and
  output failures.
- Completion candidates and inferred structures must not become accepted facts
  without an explicit review action.
- Projection output must declare information loss and source identifiers.

Recommended report envelope:

```json
{
  "schema": "highergraphen.<tool>.<operation>.report.v1",
  "report_type": "<tool>_<operation>",
  "report_version": 1,
  "metadata": {
    "command": "<tool> ...",
    "tool_package": "tools/<tool>",
    "core_packages": []
  },
  "input": {},
  "result": {},
  "projection": {}
}
```

`input`, `result`, and `projection` are operation-specific. If a runtime
workflow already uses `scenario` instead of `input`, as the current
Architecture Product reports do, the runtime report may keep `scenario` for
compatibility. Dedicated tool reports should use `input` for new contracts.

## `obstructiongraphen`

### Conceptual Basis

`obstructiongraphen` explains why a structure, plan, invariant, morphism, or
projection cannot hold. An obstruction is a structured domain result, not an
exception string.

The core idea is that failure should carry:

- the failed condition or operation;
- locations in cells, contexts, or morphisms;
- an optional concrete counterexample or witness;
- severity;
- provenance and confidence;
- required resolution hints;
- source identifiers that downstream tools can trace.

The conceptual sources are unsatisfiability, counterexamples,
non-commutative diagrams, failed gluing, local/global inconsistency, uncovered
regions, and unacceptable projection loss.

### Package And CLI Surface

| Surface | Contract |
| --- | --- |
| Tool package | `tools/obstructiongraphen/` |
| Installed command | `obstructiongraphen` |
| Agent skill | `obstructiongraphen` |
| Primary core crate | `higher-graphen-obstruction` |
| Report schema prefix | `highergraphen.obstruction.*.report.v1` |

Minimum CLI commands:

```sh
obstructiongraphen explain --input <path> --format json [--output <path>]
obstructiongraphen validate --input <path> --format json
```

`explain` converts invariant check results, morphism preservation failures,
lost-structure reports, or runtime workflow findings into obstruction records
and a traceable projection. `validate` verifies an obstruction report without
recomputing the source check.

### Core Dependencies

Required lower crates:

- `higher-graphen-core` for `Id`, `SourceRef`, `Provenance`, `Confidence`,
  `Severity`, `ReviewStatus`, structured errors, and serde contracts.
- `higher-graphen-space` for space, cell, incidence, context ID, and location
  references.
- `higher-graphen-obstruction` for `Obstruction`, `ObstructionType`,
  `ObstructionExplanation`, `Counterexample`, `RequiredResolution`, and
  related morphism references.

Conditional lower crates:

- `higher-graphen-invariant` when the input is an invariant or constraint
  check result.
- `higher-graphen-morphism` when the input is a failed composition,
  preservation failure, or lost-structure report.
- `higher-graphen-context` when the input is a failed gluing or context
  mismatch.
- `higher-graphen-projection` only in the tool layer when rendering a report
  projection. The core obstruction crate must remain projection-neutral.

### Input Contract

`obstructiongraphen explain` accepts one bounded input document. The first
version should support either a direct obstruction input or a runtime report
that already contains check results.

Required normalized input fields:

| Field | Contract |
| --- | --- |
| `source_schema` | Stable schema identifier for the source document. |
| `space_id` | Space in which the failure was detected. |
| `source_ids` | IDs used to derive the obstruction. Must not be empty. |
| `failure_kind` | One of invariant violation, constraint unsatisfied, failed composition, failed gluing, missing morphism, context mismatch, projection loss, uncovered region, or custom extension. |
| `summary` | Non-empty projection-neutral explanation summary. |
| `details` | Optional explanation detail. |
| `location_cell_ids` | Cell locations, empty only when the failure is not cell-local. |
| `location_context_ids` | Context locations, empty only when the failure is not context-local. |
| `related_morphisms` | Related morphism IDs with optional roles. |
| `severity` | Core `Severity`; impact only. |
| `counterexample` | Optional concrete witness with assignments, path cells, and contexts. |
| `required_resolution` | Optional hint describing what must change before the obstruction clears. |
| `provenance` | Source, confidence, and review status. |

The tool may adapt existing runtime reports by selecting
`result.check_result`, `result.obstructions`, `scenario` or `input` source IDs,
and projection source IDs.

### Output Contract

`explain` emits an obstruction report:

| Field | Contract |
| --- | --- |
| `result.status` | `obstruction_detected`, `no_obstruction`, or `unsupported_input`. |
| `result.obstructions` | Zero or more `higher_graphen_obstruction::Obstruction` records. |
| `result.source_ids` | IDs represented by the result. Must not be empty when obstructions are present. |
| `projection.human_review` | Human explanation and recommended resolution actions. |
| `projection.ai_view` | Source-stable obstruction records with IDs, severity, provenance, and review status. |
| `projection.audit_trace` | Per-source coverage trace. |

`no_obstruction` is a successful domain result. `unsupported_input` is a
successful parse when the input schema is recognized but no obstruction adapter
exists. Malformed JSON, missing required fields, invalid IDs, invalid
confidence values, and unsupported command arguments are tool errors.

### Invariants

- Every obstruction has a non-empty ID, space ID, type, explanation summary,
  severity, and provenance.
- An obstruction produced from a violated invariant must preserve the violated
  invariant ID in `source_ids`.
- A counterexample must name at least one witness assignment, path cell, or
  context.
- Required resolution hints must not claim the obstruction is resolved. They
  only describe what must be reviewed or changed.
- Projection-specific wording must not be stored inside the core explanation
  as the only machine-readable contract.
- A `ProjectionLoss` obstruction must name the projection or source IDs whose
  required structure would be lost.

### Failure Modes

Tool errors:

- unreadable input or output path;
- malformed JSON;
- invalid `Id`, `Confidence`, `Severity`, or `ReviewStatus`;
- missing required input fields;
- unsupported CLI option;
- serialization or schema-validation failure.

Domain results:

- no obstruction detected;
- recognized input without an implemented adapter;
- obstruction detected with no concrete counterexample available;
- obstruction detected with no suggested resolution available.

### Validation Expectations

Implementation must include:

- constructor and serde tests for any tool-owned input and report structs;
- schema and fixture tests for `highergraphen.obstruction.*.report.v1`;
- CLI tests proving `--format json`, `--output`, exit behavior, and invalid
  input errors;
- semantic tests that invariant violations preserve source IDs, severity,
  provenance, counterexamples when present, and required resolution hints when
  present;
- projection tests proving human, AI-agent, and audit views preserve
  obstruction IDs and declare information loss.

## `completiongraphen`

### Conceptual Basis

`completiongraphen` turns missing structure into reviewable candidate
structure. A completion candidate is a proposal, not an accepted fact.

The conceptual sources are graph completion, constrained completion, free
construction, structural analogy, holes in complexes, missing tests, missing
APIs, missing constraints, and unresolved obstructions.

Completion has two separate phases:

1. Detect or materialize unreviewed candidates.
2. Accept or reject exactly one candidate through an explicit review action.

Review is auditable and does not mutate the source candidate report.

### Package And CLI Surface

| Surface | Contract |
| --- | --- |
| Tool package | `tools/completiongraphen/` |
| Installed command | `completiongraphen` |
| Agent skill | `completiongraphen` |
| Primary core crate | `higher-graphen-completion` |
| Report schema prefix | `highergraphen.completion.*.report.v1` |

Minimum CLI commands:

```sh
completiongraphen detect --input <path> --rules <path> --format json [--output <path>]
completiongraphen review accept --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]
completiongraphen review reject --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]
completiongraphen validate --input <path> --format json
```

Until the dedicated tool package exists, the current umbrella command
`highergraphen completion review accept|reject ...` is the reference
implementation for explicit review semantics.

### Core Dependencies

Required lower crates:

- `higher-graphen-core` for IDs, confidence, review status, provenance,
  structured errors, and serde contracts.
- `higher-graphen-space` for source space and created-structure references.
- `higher-graphen-completion` for `CompletionRule`,
  `CompletionDetectionInput`, `CompletionCandidate`, review requests, review
  records, accepted completions, and rejected completions.

Conditional lower crates:

- `higher-graphen-obstruction` when candidates are inferred from obstructions
  or required resolutions.
- `higher-graphen-invariant` when missing constraints or invariant templates
  are inferred from check results.
- `higher-graphen-morphism` when missing morphisms or preservation links are
  inferred.
- `higher-graphen-projection` only in the tool layer for human, AI-agent, and
  audit report views.

### Input Contract

`detect` accepts:

| Field | Contract |
| --- | --- |
| `space_id` | Space in which missing structure is detected. |
| `context_ids` | Contexts used to match rules. |
| `source_ids` | Source structures, obstructions, or checks used by detection. |
| `rules` | Explicit completion rules or a reference to a rule bundle. |
| `provenance` | Source and confidence for detection inputs when available. |

Rule payloads must map to `higher_graphen_completion::CompletionRule`:

- `id`;
- `candidate_id`;
- `missing_type`;
- `suggested_structure`;
- `context_ids`;
- `inferred_from`;
- `rationale`;
- `confidence`.

`review accept` and `review reject` accept a report or snapshot containing
completion candidates plus a reviewer request:

| Field | Contract |
| --- | --- |
| `candidate` | Candidate ID selected for review. |
| `reviewer` | Reviewer or workflow ID. |
| `reason` | Non-empty reviewer rationale. |
| `reviewed_at` | Optional externally supplied review time text. |
| `decision` | `accepted` or `rejected`, derived from the command path. |

### Output Contract

`detect` emits:

| Field | Contract |
| --- | --- |
| `result.status` | `candidates_detected` or `no_candidates`. |
| `result.candidates` | Zero or more `CompletionCandidate` records. |
| `result.source_ids` | IDs used by detection. |
| `projection.human_review` | Candidate summaries and review prompts. |
| `projection.ai_view` | Source-stable candidate records with confidence and review status. |
| `projection.audit_trace` | Per-source and per-candidate trace. |

Every detected candidate must have `review_status: "unreviewed"`.

`review accept` and `review reject` emit:

| Field | Contract |
| --- | --- |
| `result.status` | `accepted` or `rejected`. |
| `result.review_record` | `CompletionReviewRecord` preserving request, candidate snapshot, and outcome. |
| `result.review_record.accepted_completion` | Present only for accepted review. |
| `result.review_record.rejected_completion` | Present only for rejected review. |
| `projection.human_review` | Decision summary and follow-up action. |
| `projection.ai_view` | Source candidate plus outcome record. |
| `projection.audit_trace` | Candidate, source report, reviewer request, and outcome trace. |

Review output is a new report. It must not rewrite the source report, mutate the
candidate snapshot, or add accepted structure to the source space.

### Invariants

- Candidate detection must produce only unreviewed candidates.
- Candidate IDs must be stable inside a report.
- `inferred_from` must point to source IDs, obstructions, checks, or accepted
  structures used to justify the candidate.
- `confidence` describes inference confidence only. It is not acceptance,
  severity, or priority.
- Acceptance requires an explicit reviewer ID and non-empty reason.
- Rejection requires an explicit reviewer ID and non-empty reason.
- The review request candidate ID must match the selected candidate snapshot.
- Accepted and rejected outcomes are separate records; the source candidate is
  preserved unchanged.
- A rejected candidate must not be accepted later by a helper that only sees
  the rejected candidate state.
- Tool projections must not describe an unreviewed candidate as accepted
  structure.

### Failure Modes

Tool errors:

- unreadable input, rules, or output path;
- malformed JSON;
- invalid candidate, reviewer, confidence, or review status fields;
- missing rule fields;
- duplicate candidate IDs inside one detection result;
- selected candidate not found;
- review request candidate mismatch;
- attempt to accept a rejected candidate or reject an accepted candidate;
- serialization or schema-validation failure.

Domain results:

- no candidates detected;
- rules skipped because required contexts are absent;
- candidate generated with no safe auto-creation action;
- accepted review that still requires a downstream creation workflow;
- rejected review that leaves the original obstruction unresolved.

### Validation Expectations

Implementation must include:

- unit tests for rule matching, skipped rules, candidate creation, acceptance,
  rejection, and review records;
- tests proving detected candidates remain unreviewed;
- tests proving review does not mutate the source candidate;
- schema and fixture tests for detection and review reports;
- CLI tests for `detect`, `review accept`, `review reject`, `--output`, missing
  arguments, and bad candidate IDs;
- semantic tests that accepted review emits only accepted outcome fields and
  rejected review emits only rejected outcome fields;
- projection tests that candidate confidence, review status, source IDs, and
  audit traces survive every view.

## `projectiongraphen`

### Conceptual Basis

`projectiongraphen` turns higher structure into audience-specific views. A
projection is a lens or quotient over source structure: it selects, summarizes,
groups, or renders structure for a purpose while declaring what was omitted,
simplified, or collapsed.

Projection is not a UI framework and not an API transport. It produces
transport-neutral report data that humans, AI agents, audits, CLIs, runtimes,
and later apps can consume.

### Package And CLI Surface

| Surface | Contract |
| --- | --- |
| Tool package | `tools/projectiongraphen/` |
| Installed command | `projectiongraphen` |
| Agent skill | `projectiongraphen` |
| Primary core crate | `higher-graphen-projection` |
| Report schema prefix | `highergraphen.projection.*.report.v1` |

Minimum CLI commands:

```sh
projectiongraphen project --input <path> --projection <path> --format json [--output <path>]
projectiongraphen validate-loss --input <path> --format json
projectiongraphen validate --input <path> --format json
```

`project` applies a projection definition to a source bundle or report.
`validate-loss` verifies traceability and information-loss declarations without
rerendering the source. `validate` checks the report schema and semantic
projection invariants.

### Core Dependencies

Required lower crates:

- `higher-graphen-core` for IDs, severity, provenance, structured errors, and
  serde contracts.
- `higher-graphen-space` for selected cells, incidences, spaces, and context
  IDs.
- `higher-graphen-projection` for `Projection`, `ProjectionSelector`,
  `ProjectionAudience`, `ProjectionPurpose`, `OutputSchema`,
  `InformationLoss`, `RendererKind`, `ProjectionOutput`, and
  `ProjectionResult`.

Conditional lower crates:

- `higher-graphen-obstruction` when rendering obstruction reports.
- `higher-graphen-completion` when rendering completion candidates or review
  reports.
- `higher-graphen-invariant` and `higher-graphen-morphism` when check results,
  preservation reports, or lost-structure records must be represented.

The projection core crate must remain free of runtime, CLI, UI, provider SDK,
and product-package dependencies.

### Input Contract

`project` accepts a source bundle plus a projection definition.

Source bundle requirements:

| Field | Contract |
| --- | --- |
| `source_schema` | Stable schema identifier for the source document. |
| `space_id` | Source space, when the source has a space. |
| `cells` | Selected or selectable cells. |
| `incidences` | Selected or selectable incidences. |
| `obstructions` | Obstructions available to select. |
| `completion_candidates` | Candidates available to select. |
| `check_results` | Optional invariant or constraint results. |
| `source_ids` | IDs present in the source bundle. |

Projection definition requirements map to
`higher_graphen_projection::Projection`:

| Field | Contract |
| --- | --- |
| `id` | Stable projection ID. |
| `source_space_id` | Space the projection reads from, if applicable. |
| `name` | Non-empty projection name. |
| `audience` | Target consumer such as human, AI agent, audit, developer, architect, operator, or external system. |
| `purpose` | Explanation, report, dashboard, action plan, review, query result, API response, or future custom purpose. |
| `input_selector` | Cell, cell type, obstruction, context, and severity filters. |
| `output_schema` | Text, sections, table, key-value, or custom schema. |
| `information_loss` | Non-empty loss declarations with source IDs. |
| `renderer` | Optional transport-neutral renderer kind. |

### Output Contract

`project` emits:

| Field | Contract |
| --- | --- |
| `result.status` | `projected` or `empty_selection`. |
| `result.projection_result` | `ProjectionResult` with output, source IDs, renderer, and information loss. |
| `result.coverage` | Source IDs represented, omitted, and selected but not rendered. |
| `projection.human_review` | Human view when requested or when the source is a review workflow. |
| `projection.ai_view` | Source-stable records for AI-agent use. |
| `projection.audit_trace` | Per-source trace and information-loss coverage. |

For workflow reports that need multiple audiences at once, the output should
use a view set with:

- `human_review`;
- `ai_view`;
- `audit_trace`.

The current runtime `ProjectionViewSet` is the reference shape for this
multi-view report pattern.

### Invariants

- Every projection definition must declare at least one information-loss entry.
- Every projection result must include at least one represented source ID,
  unless the status is `empty_selection`.
- Every rendered section, row, entry, or record must be traceable to source
  IDs.
- Information-loss declarations must name the source IDs affected by the loss.
- The projection must not remove or invent review status for completion
  candidates.
- Human views may summarize, but AI-agent and audit views must preserve stable
  IDs for represented records.
- Audit traces must identify each represented source ID and the views that
  include it.
- Projection errors must not be used to represent domain obstructions; when a
  requested projection would hide required structure, emit or preserve a
  `projection_loss` obstruction in the source result.

### Failure Modes

Tool errors:

- unreadable source, projection, or output path;
- malformed JSON;
- invalid projection ID, audience, purpose, selector, renderer, output schema,
  or source ID;
- empty information-loss declarations;
- selected source IDs missing from the source bundle;
- rendered output that violates its declared schema;
- table rows with a different width than declared columns;
- missing trace coverage for rendered records;
- serialization or schema-validation failure.

Domain results:

- empty selector result;
- projection produced with declared loss;
- projection produced while preserving upstream obstructions;
- projection loss detected and returned as an obstruction for review.

### Validation Expectations

Implementation must include:

- unit tests for projection definitions, output schemas, renderer choices,
  information-loss declarations, and projection results;
- schema and fixture tests for projection reports;
- CLI tests for `project`, `validate-loss`, `validate`, `--output`, invalid
  selectors, and schema violations;
- semantic tests that every projected view has source IDs and information-loss
  declarations;
- tests for human, AI-agent, and audit view sets over obstruction and
  completion reports;
- tests proving projection never promotes unreviewed completion candidates.

## Composition With Review Workflows

The three tools compose in a fixed review-safe order:

```text
source structure or runtime report
  -> obstructiongraphen explain
  -> completiongraphen detect
  -> projectiongraphen project
  -> completiongraphen review accept|reject
  -> projectiongraphen project
```

Architecture review uses the same pattern:

```text
lift accepted facts
  -> check invariants
  -> produce obstructions
  -> propose completion candidates
  -> project human, AI-agent, and audit views
  -> explicitly review one candidate
  -> project the review result
```

Composition rules:

- Obstructions can justify completion candidates through `inferred_from`.
- Completion candidates can appear in projections, but projections cannot
  accept or reject them.
- Review decisions produce new reports and audit records.
- Projection view sets should present different audience views over the same
  source IDs rather than recomputing different facts per audience.
- Audit views must preserve enough trace to connect accepted facts,
  obstructions, candidates, reviewer decisions, and information loss.
- A detected domain violation, unresolved obstruction, or unreviewed candidate
  is a successful workflow result when the command emitted a valid report.

## Minimum Acceptance Checks

An implementation case for these contracts is complete only when it proves the
following:

- `obstructiongraphen explain` can produce a valid obstruction report from at
  least one invariant violation source.
- `completiongraphen detect` can produce an unreviewed candidate inferred from
  an obstruction.
- `completiongraphen review accept|reject` can emit separate audit reports
  without mutating the source candidate report.
- `projectiongraphen project` can produce human, AI-agent, and audit views over
  obstruction and completion data.
- Every generated report validates against its JSON schema.
- CLI commands return exit code `0` for successful domain findings and nonzero
  only for tool/runtime failures.
- Source IDs, provenance, review status, confidence, severity, and declared
  information loss survive report serialization.
- MCP behavior, provider-specific manifests, marketplace packaging, and UI
  rendering remain out of scope.
