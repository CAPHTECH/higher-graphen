# casegraphen Workflow Reasoning Engine

Status: foundation contract for case `casegraphen-highergraphen-rewrite`.

This document defines the next stage of `casegraphen` inside the
`higher-graphen` workspace. It does not target a direct rewrite of the external
`casegraphen reference workspace` repository. That repository may remain
reference material or become a later migration target, but the implementation
target for this case is the HigherGraphen intermediate tool package:

```text
tools/casegraphen/
docs/specs/intermediate-tools/casegraphen.md
schemas/casegraphen/
future agent skill surfaces for casegraphen
```

The current implementation surface is inventoried in
[`casegraphen-current-surface-inventory.md`](casegraphen-current-surface-inventory.md).
The implementable model and report contracts for the next implementation slice
are defined in
[`casegraphen-workflow-contracts.md`](casegraphen-workflow-contracts.md).

## Target Definition

`casegraphen` should become a higher-order workflow reasoning engine over
HigherGraphen structures.

The existing tool captures cases, scenarios, coverage goals, missing cases,
conflicts, comparison, and projections. The next stage generalizes this into a
workflow case space that can represent work, readiness, proof, blockers,
completions, decisions, evidence, and evolution.

```text
casegraphen
  = Case-centered HigherGraphen intermediate tool
    + workflow reasoning model
    + CLI and skill surfaces for AI agents
```

It remains an intermediate tool, not a business product and not an MCP server.

## Current Baseline

The current implementation already provides:

- `tools/casegraphen` Rust CLI package;
- `casegraphen create`, `inspect`, `list`, `validate`, `coverage`, `missing`,
  `conflicts`, `project`, and `compare`;
- JSON schemas and fixtures under `schemas/casegraphen/`;
- report envelopes under `highergraphen.case.<operation>.report.v1`;
- reviewable `MissingCase` records;
- `ConflictingCase` findings;
- human, AI, and audit projections;
- dependency on `higher-graphen-core` primitives.

This baseline should be preserved. New workflow reasoning should extend it
instead of replacing the existing case graph contract.

## HigherGraphen Concept Map

| Workflow concept | HigherGraphen concept | `casegraphen` role |
| --- | --- | --- |
| Workflow case graph | Space | Bounded reasoning universe for a case set or workflow. |
| Case | 0-cell | Concrete example, counterexample, regression, or situation. |
| Scenario | 0-cell or template cell | Reusable situation pattern. |
| Work item | 0-cell | Task, goal, decision, event, evidence, or external wait. |
| Relation | Incidence | Dependency, coverage, contradiction, derivation, projection. |
| Readiness rule | Constraint | Checkable condition for ready work. |
| Blocker | Obstruction | Structured explanation of non-readiness. |
| Missing task or proof | Completion candidate | Proposed structure requiring review. |
| Evidence | Evidence cell or provenance-backed claim | Source-backed proof, separate from inference. |
| Patch or transition | Morphism | Reviewable transformation between workflow spaces. |
| Frontier | Projection | View of ready work with declared loss. |
| Skill view | Projection | AI-agent-specific view and allowed operation set. |
| Similar workflow | Correspondence | Reusable structural match or mismatch. |
| Revision sequence | Evolution | Temporal trace of workflow structure. |

## Workflow Reasoning Questions

The engine should answer:

- What work is ready now?
- Why is this work blocked?
- Which proof, test, decision, or case is missing?
- Which inferred structures are still unreviewed?
- Which constraints or invariants would a patch violate?
- Which cases or workflows are structurally similar?
- Which projection is suitable for a human, an AI agent, or an audit trail?
- Which information is hidden by that projection?
- Which workflow structures changed over time?

These questions are structural. Domain-specific interpretation should live in
product packages or skills.

## Required Capabilities

### Readiness Projection

Add a workflow readiness model without breaking the current coverage model.

Minimum records:

- work item ID;
- work item type;
- current state;
- hard dependency IDs;
- external wait IDs;
- evidence requirement IDs;
- ready or not-ready result;
- obstruction IDs explaining non-readiness.

Readiness is a projection, not a stored fact. A stored state can participate in
readiness, but the ready set is derived.

### Obstruction Reports

Blockers, contradictions, invalid transitions, and impossible closure should be
reported as obstructions.

Minimum fields:

- obstruction ID;
- obstruction type;
- affected work item or case IDs;
- source constraint or invariant;
- witness IDs;
- explanation;
- severity;
- required resolution when known;
- provenance.

### Completion Candidates

The engine should propose missing structure without silently accepting it.

Initial completion types:

- missing task;
- missing evidence;
- missing test;
- missing decision;
- missing dependency relation;
- missing case;
- missing projection;
- missing review action.

Each candidate must carry confidence, provenance, rationale, target IDs, and
`review_status: "unreviewed"` until an explicit review action changes it.

### Evidence Boundary

AI inference must remain separate from accepted evidence.

The engine may produce inferred reports, but it must not turn them into
evidence by projection alone. Evidence requires a source-backed record or an
explicit review action that promotes a candidate.

### Projection Loss

Every report that presents a subset of the workflow space must state what it
omits.

Examples:

- a frontier report omits non-ready work and most evidence detail;
- an AI skill report may omit human prose but keep machine IDs;
- an audit report may omit convenience summaries but keep source IDs;
- a comparison report may omit full JSON diff and show case-level differences.

### Correspondence

Comparison should evolve from case-level diffing into structural
correspondence.

The engine should distinguish:

- equivalent;
- similar with information loss;
- matching by scenario pattern;
- conflicting;
- not comparable;
- transferable mitigation or completion pattern.

Correspondence must include mismatch evidence. Similarity is not identity.

### Evolution

The first implementation can be file-based, but the model should leave room for
revision-indexed reasoning.

Evolution should eventually answer:

- when a blocker appeared;
- when proof was attached;
- when a completion candidate was accepted or rejected;
- which workflow shape persisted across revisions;
- which transition broke an invariant.

## Package Placement

The implementation belongs in the `higher-graphen` repository.

Recommended placement:

| Artifact | Location |
| --- | --- |
| Tool docs | `docs/specs/intermediate-tools/` |
| CLI package | `tools/casegraphen/` |
| JSON schemas | `schemas/casegraphen/` |
| Examples | `examples/casegraphen/` or tool fixtures |
| Codex/Claude skills | `integrations/*/skills/casegraphen/` when added |
| Runtime integration | Optional, only if the general `highergraphen` CLI hosts the tool |

The existing external CaseGraphen repository is out of scope for this case.

## Implementation Phases

1. Foundation docs.
   Define this target, link it from the docs index, and preserve the existing
   case graph contract.
2. Current surface inventory.
   Map current CLI commands, schemas, tests, and reports to the workflow
   reasoning concepts in this document.
3. Contract and schema design.
   Define report schemas for readiness, obstructions, completions, evidence
   boundaries, workflow projection, and correspondence.
4. Substrate model.
   Refactor or extend `tools/casegraphen` model types so workflow cases can be
   represented through HigherGraphen primitives and provenance.
5. Reasoning engines.
   Implement readiness, obstruction, completion, evidence-boundary,
   correspondence, projection, and evolution hooks as focused modules.
6. CLI and skill surfaces.
   Add commands and skill instructions for AI agents to inspect and operate the
   workflow reasoning engine without MCP.
7. Reference verification.
   Add examples, fixtures, schema checks, CLI smoke tests, package tests, and
   static analysis gates.

## Compatibility Rules

- Existing `casegraphen` commands keep their current behavior.
- Existing schemas remain valid unless explicitly versioned.
- New reports are additive and use new schema identifiers.
- Missing cases and workflow completions remain reviewable candidates.
- Conflicts and blockers are successful domain findings, not CLI failures.
- Invalid input, unsupported schema versions, malformed IDs, and unreadable
  files remain tool failures.
- CLI output must remain machine-readable JSON for agent use.

## Completion Criteria For This Rewrite Case

The case is complete when:

- the documentation and schemas define the workflow reasoning model;
- `tools/casegraphen` implements the first workflow reasoning slice;
- CLI commands expose the slice with stable JSON reports;
- examples prove readiness, obstruction, completion, and projection behavior;
- package tests and workspace static analysis pass;
- `cg validate --case casegraphen-highergraphen-rewrite` and
  `cg validate storage` pass;
- evidence is attached to the case for each completed task.
