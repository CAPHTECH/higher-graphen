# Native CaseGraphen Case Management Design

Status: design contract for case `casegraphen-native-case-management`, task
`task_native_case_management_design_docs`.

This document defines the target native CaseGraphen case management model for
the `higher-graphen` repository. It is intentionally not a clone of the
installed `cg` workspace tracker. The installed `cg` binary is only the
meta-workflow driver used to manage this implementation case. The product
design below is native CaseGraphen: a `CaseSpace` containing typed `CaseCell`
records, connected by `CaseRelation` records, evolved by `CaseMorphism`
records, and persisted through an append-only `MorphismLog`.

The design extends the current baseline and workflow contracts:

- [`casegraphen.md`](casegraphen.md)
- [`casegraphen-current-surface-inventory.md`](casegraphen-current-surface-inventory.md)
- [`casegraphen-workflow-contracts.md`](casegraphen-workflow-contracts.md)
- [`casegraphen-feature-completion-contract.md`](casegraphen-feature-completion-contract.md)
- [`../../guides/product-integration-for-ai-agents.md`](../../guides/product-integration-for-ai-agents.md)

## Purpose And Non-Goals

Native CaseGraphen case management must let an operator create, inspect,
reason about, review, project, evolve, and close structured cases without
storing readiness as mutable task state. The durable model is a case space plus
an append-only morphism history. Readiness, frontier, blockers, close status,
projection views, and evolution summaries are derived projections.

The design must support these product questions:

- Which case cells exist in this case space, and what role does each one play?
- Which relations make one cell depend on, verify, obstruct, complete, review,
  or project another?
- Which morphisms changed the case space, which invariants did they preserve,
  and which invariants did they violate?
- Which cells are ready, blocked, frontier, complete, or closeable when the log
  is replayed under the current rules?
- Which evidence is source-backed, inferred, reviewed, accepted, rejected, or
  insufficient?
- Which projections are suitable for human review, AI operation, audit, system
  integration, or migration?

Explicit non-goals:

- Re-implementing installed `cg` behavior, command names, workspace cache, task
  tracker states, or clone behavior as the native product model.
- Changing the external `/Users/rizumita/Workspace/casegraphen` repository.
- Adding MCP support, provider SDK integration, marketplace packages, or
  agent-provider runtime dependencies.
- Rewriting external repositories or external issue/task systems.
- Treating projection output, AI inference, or generated completion candidates
  as accepted evidence without explicit review.
- Treating missing coverage, blocked cells, obstructions, or unreviewed
  completion candidates as tool failures. They are successful domain findings.

## Product Integration Alignment

Native CaseGraphen follows the product-integration pipeline defined in
[`product-integration-for-ai-agents.md`](../../guides/product-integration-for-ai-agents.md).
The native product object is not a task list, report, or UI state. It is a
bounded case world lifted into HigherGraphen-compatible structure.

Mapping:

| Product integration concept | Native CaseGraphen target |
| --- | --- |
| Bounded source snapshot | `CaseSourceBoundary` contract, encoded in v1 under `CaseSpace.metadata.source_boundary`. |
| `Space` | `CaseSpace`. |
| `Cell` | `CaseCell`. |
| Incidence | `CaseRelation`. |
| `Context` | Context cells or external HigherGraphen context refs, represented in v1 by `custom:context`, `external_ref`, and context relations. |
| `Morphism` | `CaseMorphism` plus append-only `MorphismLog`. |
| `Invariant` | Readiness, close, evidence, projection, policy, and morphism-preservation rules. |
| `Obstruction` | Derived obstruction reports or durable obstruction cells when the obstruction itself must be tracked. |
| `CompletionCandidate` | Completion cells, generated completion report records, and review morphisms. |
| Evidence and witness | Evidence cells, evidence relations, provenance, and generated core-extension `Witness` records. |
| Review boundary | Review cells, review morphisms, `ReviewStatus`, and explicit accept/reject/reopen/waive transitions. |
| Projection | Named projection records with represented IDs, omitted IDs, information loss, and allowed operations. |
| Policy and capability | Close policies, projection operation gates, supplied core-extension `Policy`/`Capability` records, and generated decision-gate extensions. |

This alignment changes the modeling priority:

```text
source material
  -> CaseSourceBoundary
  -> CaseSpace
  -> CaseCell / CaseRelation / context refs
  -> CaseMorphism and MorphismLog
  -> derived readiness, obstructions, completions, evidence, review gaps
  -> projections and allowed operations
```

Reports and CLI outputs are projections over the replayed case space. They must
not become the source of truth.

## Conceptual Model

### CaseSourceBoundary

`CaseSourceBoundary` defines what the native case space is allowed to treat as
accepted input. It is required as a product concept and as a v1 validation
contract even though the current schema stores it in `metadata`.

In `highergraphen.case.space.v1`, the recommended encoding is:

```json
{
  "metadata": {
    "source_boundary": {
      "id": "source_boundary:example",
      "included_sources": [],
      "excluded_sources": [],
      "adapters": [],
      "accepted_fact_policy": "Only records supplied by the input file, import adapter, or reviewed morphism are accepted input facts.",
      "inference_policy": "AI or heuristic records start unreviewed unless review-promoted.",
      "information_loss": []
    }
  }
}
```

Fields:

| Field | Contract |
| --- | --- |
| `id` | Stable source-boundary ID used by operation gates to prove the operation is bound to the same lifted source snapshot. |
| `included_sources` | Source IDs, paths, commands, case IDs, workflow graph IDs, or external systems included in the bounded snapshot. |
| `excluded_sources` | Known omitted sources or scopes. Absence from the snapshot is not evidence of absence in the real world. |
| `adapters` | Importers or deterministic adapters that produced accepted records, such as `native.case.import.v1`, `workflow.graph.import.v1`, `cg.workspace.snapshot.v1`, or `manual.review.v1`. |
| `accepted_fact_policy` | Rule for which records may be treated as accepted input facts. |
| `inference_policy` | Rule for AI-generated, heuristic, or derived records. |
| `information_loss` | Source text, events, payload fields, review detail, or external state omitted or summarized during the lift. |

Native `case new` creates an empty case universe and records a minimal source
boundary. Native `case import` and workflow migration inputs must record the
input file, source schema, source revision or workflow graph ID, adapter name,
and import loss. Native validation rejects case spaces whose top-level
`metadata.source_boundary` is missing or underspecified, and the first
`MorphismLogEntry` must preserve the lift boundary in
`morphism.metadata.source_boundary` with `morphism.metadata.lift_semantics`.
A future v2 schema may promote `source_boundary` from metadata into a
first-class `CaseSpace` field.

### CaseSpace

`CaseSpace` is the bounded native case universe. It replaces the current
workflow graph wrapper as the primary case management aggregate.

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Exact native schema identifier, initially `highergraphen.case.space.v1`. |
| `schema_version` | Native schema version, initially `1`. |
| `case_space_id` | Stable ID for the case space. |
| `space_id` | HigherGraphen structural space being exercised. |
| `case_cells` | Typed case records, work records, proof obligations, reviews, and projections. |
| `case_relations` | Typed incidences between cells and stable external HigherGraphen structures. |
| `revision` | Current materialized revision derived from the log. |
| `close_policy_id` | Optional policy selecting close invariants. |
| `metadata` | Downstream-owned object. In v1, native CaseGraphen reserves `metadata.source_boundary` and `metadata.higher_graphen_extensions` as documented integration contracts. |

The materialized `CaseSpace` is a replay result, not the source of truth. The
source of truth is the ordered `MorphismLog`.

### CaseCell Taxonomy

`CaseCell` is the native unit of case management. A task-like item is only one
cell type, not the organizing abstraction.

| Cell type | Role |
| --- | --- |
| `case` | Concrete example, counterexample, regression, smoke case, boundary case, or scenario instance. |
| `scenario` | Reusable situation pattern or parameterized path through a space. |
| `goal` | Desired outcome or coverage objective. |
| `work` | Human or AI action needed to evolve the case space. |
| `decision` | Chosen interpretation, tradeoff, or policy outcome. |
| `event` | Observed external event or recorded milestone. |
| `evidence` | Source-backed, inferred, reviewed, accepted, or rejected support. |
| `proof` | Proof obligation or proof result over a cell, relation, morphism, or invariant. |
| `review` | Explicit review action or review requirement. |
| `obstruction` | Durable obstruction finding when the obstruction itself must be tracked. |
| `completion` | Reviewable candidate for missing structure. |
| `projection` | Named view contract with represented IDs, omitted IDs, and information loss. |
| `revision` | Materialized replay point. |
| `morphism` | First-class reference to a transformation recorded in the log. |
| `external_ref` | Stable pointer to external structures when a full cell is not owned here. |
| `custom:context` | v1-compatible context cell for local vocabulary, review scope, policy scope, source boundary, or projection scope. A future v2 schema may add a first-class `context` value. |
| `custom:<extension>` | Extension cell type, valid only when the schema explicitly preserves its metadata. |

Common fields:

| Field | Contract |
| --- | --- |
| `id` | Stable cell ID preserved across projections. |
| `cell_type` | Taxonomy value. |
| `space_id` | Owning HigherGraphen space. |
| `title` | Human-readable label. |
| `summary` | Optional concise description. |
| `lifecycle` | Input lifecycle fact such as `proposed`, `active`, `waiting`, `resolved`, `retired`, `accepted`, or `rejected`. |
| `source_ids` | Source references represented by the cell. |
| `structure_ids` | Referenced cells, incidences, contexts, invariants, morphisms, or external stable structures. |
| `provenance` | Source, confidence, actor, timestamp, and review status. |
| `metadata` | Extension data that cannot carry hidden readiness semantics. |

`lifecycle` is an input fact. `ready`, `frontier`, `blocked`, and `closeable`
are not stored on the cell.

### CaseRelation

`CaseRelation` is a typed incidence between cells or between cells and external
HigherGraphen structures.

Core relation types:

- `depends_on`
- `waits_for`
- `requires_evidence`
- `requires_proof`
- `satisfies_evidence_requirement`
- `verifies`
- `covers`
- `exercises`
- `blocks`
- `unblocks`
- `contradicts`
- `invalidates`
- `completes`
- `derives_from`
- `refines`
- `projects_to`
- `transitions_to`
- `corresponds_to`
- `accepts`
- `rejects`
- `supersedes`
- `custom:in_context`
- `custom:governed_by_policy`
- `custom:allowed_by_capability`
- `custom:source_boundary_of`

Relations may be `hard`, `soft`, or `diagnostic`. Only hard relations can block
readiness or closure by default. Soft and diagnostic relations can emit
warnings, review recommendations, or transfer hints without changing readiness.

Context modeling in v1 should use one of these patterns:

| Need | v1 encoding | v2 direction |
| --- | --- | --- |
| A local vocabulary or review scope is owned by the native case space. | `CaseCell` with `cell_type: "custom:context"` plus `custom:in_context` relations. | First-class `context` cell type. |
| The context already exists in HigherGraphen structure. | `external_ref` cell or `structure_ids` pointing at the external context ID. | Explicit `context_refs` field or relation target policy. |
| A projection hides or collapses contexts. | `Projection.information_loss` names represented and omitted context IDs. | Projection contract remains first-class. |
| A policy applies only in one scope. | Policy ID in `close_policy_id`, core-extension `Policy`, or `custom:governed_by_policy` relation. | First-class policy/capability gate records. |

### CaseMorphism

`CaseMorphism` is a reviewable transformation from one case-space revision to
another. It generalizes current workflow transition records and patch review
records.

Required fields:

| Field | Contract |
| --- | --- |
| `morphism_id` | Stable ID. |
| `morphism_type` | `create`, `update`, `retire`, `relate`, `unrelate`, `review`, `evidence_attach`, `completion_accept`, `completion_reject`, `projection`, `migration`, `close`, or `custom:<extension>`. Use `create`, `migration`, or `custom:lift` to record source-snapshot-to-case-space lifts in v1. |
| `source_revision_id` | Revision the morphism applies to, or `null` for genesis. |
| `target_revision_id` | Revision produced by replaying the morphism. |
| `added_ids` | Cell or relation IDs added. |
| `updated_ids` | Cell or relation IDs updated. |
| `retired_ids` | Cell or relation IDs retired. |
| `preserved_ids` | IDs whose meaning is declared preserved. |
| `violated_invariant_ids` | Invariants violated by this morphism. Empty for accepted ordinary changes. |
| `review_status` | Review status of the morphism itself. |
| `evidence_ids` | Evidence supporting the change. |
| `source_ids` | Source material behind the change. |
| `metadata` | Bounded payload for operation-specific details. |

A valid morphism is not automatically applied. Application requires appending
it to the `MorphismLog`. A generated morphism must remain `unreviewed` until an
explicit review morphism accepts or rejects it.

The first materializing morphism in a case space should be interpreted as a
lift morphism from a bounded source snapshot into a native case universe. In
v1, this can be represented as:

- `morphism_type: "create"` for empty `case new`;
- `morphism_type: "migration"` for workflow graph or external case imports;
- `morphism_type: "custom:lift"` when a downstream integration needs to make
  the lift explicit without waiting for a v2 enum.

Lift morphism metadata should preserve:

- `source_boundary_id` or inline `source_boundary` summary;
- source schema and source revision or snapshot ID;
- adapter name and version;
- preserved source IDs and lost source fields;
- generated cell and relation IDs;
- review status of any AI-created or heuristic structures.

### MorphismLog

`MorphismLog` is the append-only source of truth for a native case space.

Each entry includes:

- log entry schema and schema version;
- `case_space_id`;
- monotonic log sequence or content-addressed entry ID;
- `morphism_id`;
- `source_revision_id` and `target_revision_id`;
- serialized morphism payload or stable pointer to it;
- actor, timestamp, provenance, and source IDs;
- hash of the previous entry when content-addressing is enabled;
- replay checksum for the produced revision.

Reducers replay the log into a materialized `CaseSpace`, revision index,
projection cache, and close-check cache. Caches are disposable. If cache and log
disagree, replay wins.

### Revision

`Revision` is a replay point. It records which log entries have been applied
and the derived checksum of the materialized case space.

Revision records support:

- deterministic replay;
- diff and evolution reports;
- stale-write checks through `base_revision` or `source_revision_id`;
- migration from current workflow graph snapshots;
- close evidence that names the exact case-space state being closed.

### Projection

`Projection` is a named, lossy view of a case space. A projection never changes
truth, review status, evidence status, or readiness. It reports:

- audience: `human_review`, `ai_agent`, `audit`, `system`, `migration`;
- represented cell IDs and relation IDs;
- omitted cell IDs and relation IDs;
- information-loss records;
- allowed operations in that view;
- source IDs required to interpret the view;
- warnings when the projection hides blockers, unreviewed inference, or close
  invariant failures.

`allowed_operations` is an operation view, not authorization by itself. A
projection may state that a view exposes `inspect`, `propose_morphism`, or
`accept_completion`, but the operation is executable only when policy and
capability gates allow it.

## Policy And Capability Gates

CaseGraphen is an agent-operated product surface. Operations that mutate,
approve, export, close, or otherwise change durable case state must be gated by
policy and capability, not by projection visibility alone.

Current native close-check enforces this at the package API boundary by adding
`close:native-policy-capability-gate`: a close attempt must select a close
policy, name operation source evidence, and carry a first-class operation gate.
The gate binds `actor_id`, `operation`, `operation_scope_id`, `audience`,
`capability_ids`, and `source_boundary_id`. The close-check gate passes only
when the operation is `close-check`, the scope matches the case space, the
audience is `audit` or `system`, at least one capability is named, and the
gate source-boundary ID matches the lifted case-space source boundary.

Minimum operation gates:

| Operation | Gate |
| --- | --- |
| Inspect a case space | Projection permits the view, and no policy blocks the audience. |
| Propose a morphism | Actor has capability to propose against the target revision and source boundary. |
| Apply a morphism | Morphism check passes, base revision matches, required review exists, and policy permits the mutation. |
| Reject a morphism | Actor has review capability for the morphism target. |
| Accept/reject/reopen a completion | Completion review policy permits the action, reviewer is recorded, and linked evidence or decision IDs satisfy the rule. |
| Close a case space | Close-check passes, close policy is selected, validation evidence is named, and actor has close capability. |
| Export a projection | Projection audience is allowed, information loss is declared, and source boundary permits export. |

In v1, gates may be represented by:

- `close_policy_id`;
- core-extension `Policy` and `Capability` records under
  `metadata.higher_graphen_extensions`;
- generated `core_extensions` in close-check, morphism-check, and workflow
  reports;
- `custom:governed_by_policy` and `custom:allowed_by_capability` relations;
- explicit review morphisms naming reviewer, reason, evidence, and decisions.

Generated capabilities are explanatory unless a supplied policy or review
workflow consumes them. A report that says an operation is visible or suggested
must not imply that it has been authorized.

## Derived Readiness, Frontier, And Blockers

Native readiness is derived by replaying the `MorphismLog` into a `CaseSpace`
and evaluating projection rules over cells, relations, evidence, proof, and
review records. It is not stored as a mutable task state.

The readiness projection emits:

| Output | Derivation |
| --- | --- |
| `ready_cell_ids` | Active cells whose hard dependencies are resolved, waits are satisfied, required evidence/proof is accepted or source-backed, no hard contradiction blocks them, and required reviews are accepted. |
| `not_ready_cell_ids` | Active cells that fail at least one readiness rule. |
| `frontier_cell_ids` | Minimal ready cells whose downstream work is not already completed, accepted, retired, or superseded. |
| `blocked_cell_ids` | Cells with at least one hard obstruction. |
| `waiting_cell_ids` | Cells blocked only by waits or explicit external references. |
| `rule_results` | Per-cell readiness rule witnesses. |
| `projection_loss` | Disclosure of hidden cells, relations, and source records. |

Default readiness rules:

- dependency closure: hard `depends_on` targets must be resolved, accepted, or
  otherwise complete under the selected policy;
- wait resolution: hard `waits_for` targets must be recorded, accepted, or
  explicitly waived by review;
- evidence availability: required evidence must be source-backed or
  review-promoted, and must meet the minimum review status;
- proof availability: required proof cells must be accepted or explicitly
  waived by review;
- contradiction absence: hard `contradicts` or `invalidates` relations produce
  blockers until resolved;
- review status: generated completions, inferred evidence, and generated
  morphisms must be reviewed before they can satisfy hard requirements.

This keeps native CaseGraphen compatible with the current workflow evaluator's
principle that state is an input fact and readiness is derived.

## Obstruction, Completion, Evidence, And Review Semantics

### Obstructions

An obstruction is a domain finding explaining why a cell, relation, morphism,
projection, or close attempt cannot proceed under the selected rules. It is not
a tool failure.

Obstruction types:

- `unresolved_dependency`
- `external_wait`
- `missing_evidence`
- `missing_proof`
- `invalid_morphism`
- `contradiction`
- `impossible_closure`
- `projection_loss`
- `correspondence_mismatch`
- `review_required`

Each obstruction must carry stable witness IDs, source constraint IDs,
severity, provenance, review status, and recommended completion types.

### Completions

A completion is a reviewable candidate for missing or corrective structure.
Completions generalize current missing cases and workflow completion
candidates.

Completion types:

- `missing_case`
- `missing_scenario`
- `missing_work`
- `missing_decision`
- `missing_event`
- `missing_evidence`
- `missing_proof`
- `missing_relation`
- `missing_projection`
- `missing_review`
- `replacement_morphism`

Generated completions start with `review_status: "unreviewed"`. Accepting a
completion appends an explicit review morphism and, when needed, a follow-up
case morphism that adds the accepted structure. Rejecting or reopening also
appends review morphisms; it does not erase the original candidate.

### Evidence

Evidence cells and evidence relations distinguish origin from review outcome.

Evidence origins:

- `source_backed`: comes from an explicit source reference or command output;
- `inferred`: generated by an AI or heuristic and not sufficient for hard
  requirements by default;
- `review_promoted`: inferred or interpreted evidence promoted by explicit
  review;
- `rejected`: reviewed and rejected as support;
- `contradicting`: evidence against a cell, relation, morphism, or claim.

Hard evidence requirements default to allowing only `source_backed` and
`review_promoted` evidence with sufficient review status. Inferred evidence may
appear in AI and audit projections, but it cannot silently satisfy close or
readiness requirements.

### Reviews

Review is represented as both a cell type and morphism type:

- review requirement cells state that something needs review;
- review action cells summarize human or delegated decisions;
- review morphisms record the durable transition that accepts, rejects,
  reopens, waives, or supersedes a candidate, evidence item, relation, or
  morphism.

Every review morphism must name reviewer ID, reason, target IDs, source or
evidence IDs, and the review outcome. A review projection may suggest actions,
but the durable result is only the appended review morphism.

## Projection And Evolution Views

Native CaseGraphen should provide a common projection engine over the replayed
case space.

Required projection views:

| View | Purpose |
| --- | --- |
| `human_review` | Small action-oriented view with open reviews, blockers, close failures, and recommended next steps. |
| `ai_agent` | Operational view with frontier cells, blocked cells, candidate morphisms, evidence boundaries, and allowed commands. |
| `audit` | Complete provenance, log entries, review decisions, evidence origins, and projection loss. |
| `system` | Stable machine surface for package API consumers. |
| `migration` | Mapping from current workflow graph records and `cg` case records into native cells, relations, and morphisms. |

Evolution reports are also projections. They compare revisions and emit:

- appeared, resolved, and persisted obstruction IDs;
- added, updated, retired, and preserved cell/relation IDs;
- accepted, rejected, reopened, and pending completion IDs;
- morphisms grouped by type and review status;
- invariant violations introduced or resolved;
- projection loss relative to the requested audience.

## Close Invariants And Close-Check Semantics

Closing a native case space appends a `close` morphism only after close-check
passes. Close-check is a projection over a specific revision and close policy.

Default close invariants:

1. No critical or high hard obstruction remains unresolved.
2. Required goals are covered by accepted cases, scenarios, proof, or explicit
   waiver reviews.
3. Required evidence is source-backed or review-promoted.
4. Generated completions that affect required goals are accepted, rejected, or
   explicitly deferred by review.
5. Candidate morphisms that affect required goals are accepted/applied,
   rejected, or explicitly deferred by review.
6. Required projections disclose represented IDs, omitted IDs, and information
   loss.
7. The case-space source boundary is declared and any excluded source scopes
   relevant to closeability are either represented as projection loss,
   reviewed waivers, or explicit residual risk.
8. The actor or workflow attempting closure is allowed by the selected close
   policy and capability gate.
9. The case-space revision being closed matches the caller's `base_revision`.
10. Storage replay from `MorphismLog` produces the same revision checksum as the
   close-check input.
11. Migration bridges, if used, have recorded their source revision or snapshot
   IDs.
12. The close morphism names evidence for the validation commands used at close
    time.

Close-check results are domain reports:

- `closable: true` means a close morphism may be appended.
- `closable: false` means the report lists invariant failures and completion
  candidates.
- malformed input, unsupported schema versions, unreadable stores, stale base
  revisions, and replay checksum mismatches are tool failures.

## Store Layout Proposal

The native store should live under a repo-owned store root supplied by CLI or
package API. It must not require direct writes to installed `cg` internals.

Proposed layout:

```text
case_spaces/
  <case_space_segment>/
    manifest.json
    morphisms.jsonl
    revisions/
      <revision_segment>.case_space.json
    morphisms/
      <morphism_segment>.case_morphism.json
    projections/
      <revision_segment>/
        human_review.json
        ai_agent.json
        audit.json
        system.json
        migration.json
    close_checks/
      <revision_segment>.close_check.json
```

Contracts:

- `morphisms.jsonl` is append-only and authoritative.
- `revisions/`, `projections/`, and `close_checks/` are replayable caches.
- Path segments are encoded from IDs using a deterministic safe-segment
  function.
- Appends require `source_revision_id` or `base_revision` to avoid stale
  writes.
- Validation must replay the log, verify revision checksums, check schema
  versions, and verify that cached projections do not claim authority.
- Import/migration records preserve source workflow graph IDs, installed `cg`
  case IDs, event IDs, and snapshot paths when available.

Implementation note: `tools/casegraphen/src/native_store.rs` currently provides
the first file-backed native store under `native_case_spaces/`. It records
`morphism_log.jsonl`, deterministic revision snapshots, list/inspect/history,
replay, validation, and conservative append support for metadata-only
morphisms. Typed reducers for materializing arbitrary cell or relation payloads
remain out of scope until the native reasoning and CLI tasks define those
operation contracts.

## CLI And Package API Target Surface

All commands must accept `--format json`; `--output <path>` may write the same
report envelope to disk. Domain findings exit successfully with structured
reports. Tool failures use structured errors.

Implemented native CLI surface:

```sh
casegraphen case new --store <dir> --case-space-id <id> --space-id <id> --title <text> --revision-id <id> --format json
casegraphen case import --store <dir> --input <native.case.space.json> --revision-id <id> --format json
casegraphen case list --store <dir> --format json
casegraphen case inspect --store <dir> --case-space-id <id> --format json
casegraphen case validate --store <dir> --case-space-id <id> --format json
casegraphen case history --store <dir> --case-space-id <id> --format json
casegraphen case history topology --store <dir> --case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
casegraphen case history topology diff --left-store <dir> --left-case-space-id <id> --right-store <dir> --right-case-space-id <id> --format json [--higher-order [--max-dimension <n>] [--min-persistence <n>|--min-persistence-stages <n>]] [--output <path>]
casegraphen case replay --store <dir> --case-space-id <id> --format json
casegraphen case reason --store <dir> --case-space-id <id> --format json
casegraphen case frontier --store <dir> --case-space-id <id> --format json
casegraphen case obstructions --store <dir> --case-space-id <id> --format json
casegraphen case completions --store <dir> --case-space-id <id> --format json
casegraphen case evidence --store <dir> --case-space-id <id> --format json
casegraphen case project --store <dir> --case-space-id <id> --format json
casegraphen case close-check --store <dir> --case-space-id <id> --base-revision-id <id> --validation-evidence-id <id> --format json
```

Morphism commands:

```sh
casegraphen morphism propose --store <dir> --case-space-id <id> --input <case_morphism.json> --format json
casegraphen morphism check --store <dir> --case-space-id <id> --morphism-id <id> --format json
casegraphen morphism apply --store <dir> --case-space-id <id> --morphism-id <id> --base-revision-id <id> --reviewer-id <id> --reason <text> --format json
casegraphen morphism reject --store <dir> --case-space-id <id> --morphism-id <id> --reviewer-id <id> --reason <text> --revision-id <id> --format json
```

The first implementation intentionally bounds morphism application to
metadata-only morphisms because typed reducers for arbitrary cell/relation
payloads are not yet part of the native store. Candidate morphisms are proposed
into a native proposal area under the supplied store root, checked against the
current replayed case-space revision, then either appended through
`morphism apply` or rejected by an explicit review morphism.

Planned review commands:

```sh
casegraphen review list --store <dir> --case-space-id <id> --format json
casegraphen review accept --store <dir> --case-space-id <id> --target-id <id> --reviewer-id <id> --reason <text> --format json
casegraphen review reject --store <dir> --case-space-id <id> --target-id <id> --reviewer-id <id> --reason <text> --format json
casegraphen review reopen --store <dir> --case-space-id <id> --target-id <id> --reviewer-id <id> --reason <text> --format json
casegraphen review waive --store <dir> --case-space-id <id> --target-id <id> --reviewer-id <id> --reason <text> --format json
```

These `review ...` commands are not implemented in the current CLI. Until that
surface exists, review state is represented through native morphism
proposal/check/apply/reject flows and metadata-only review morphisms.

`case history topology` emits a native CLI operation report with topology
diagnostics under `result.topology`. Baseline output omits
`result.topology.higher_order`. When `--higher-order` is supplied, output
includes `result.topology.higher_order` with the options used
(`include_higher_order`, optional `max_dimension`, and
`min_persistence_stages`), selected `cell_count`, cumulative `stage_count`, and
an optional `persistence` summary. Native topology uses
`filtration_source: native_morphism_log` when replay history is available and
emits `stage_sources` for morphism log entries plus any deterministic remainder
needed to close the final complex. The persistence summary includes per-stage
topology, all intervals, thresholded `persistent_intervals`,
`open_component_count`, and `open_hole_count`. `--min-persistence` and
`--min-persistence-stages` are aliases for the same stage-lifetime threshold.
Higher-order topology remains a read-only diagnostic and does not mutate the
case space or change frontier, obstruction, completion, evidence, close-check,
or morphism semantics.

`case history topology diff` replays two native case spaces from their stores,
builds the same topology reports, and emits `result.topology_diff` with scalar
topology deltas, source-mapping additions/removals, and optional higher-order
summary deltas. It is a comparison surface only; it does not append morphisms.

Package API targets:

```rust
pub struct CaseSpace;
pub struct CaseCell;
pub struct CaseRelation;
pub struct CaseMorphism;
pub struct MorphismLogEntry;
pub struct Revision;
pub struct Projection;
pub struct CloseCheck;

pub fn replay_case_space(log: &MorphismLog) -> CaseResult<CaseSpace>;
pub fn validate_case_space(space: &CaseSpace) -> CaseResult<ValidationReport>;
pub fn evaluate_readiness(space: &CaseSpace, policy: ReadinessPolicy) -> CaseResult<ReadinessProjection>;
pub fn detect_obstructions(space: &CaseSpace, policy: ReadinessPolicy) -> CaseResult<Vec<Obstruction>>;
pub fn propose_completions(space: &CaseSpace, obstructions: &[Obstruction]) -> CaseResult<Vec<Completion>>;
pub fn project_case_space(space: &CaseSpace, projection: ProjectionRequest) -> CaseResult<ProjectionReport>;
pub fn check_morphism(space: &CaseSpace, morphism: &CaseMorphism) -> CaseResult<MorphismCheck>;
pub fn apply_morphism(space: &CaseSpace, morphism: &CaseMorphism) -> CaseResult<CaseSpace>;
pub fn check_close(space: &CaseSpace, policy: ClosePolicy) -> CaseResult<CloseCheck>;
```

The API must remain independent of CLI parsing, provider SDKs, MCP servers,
and product runtime packages.

## Schema And Package Contract Notes

The native package contract defines strict serde model boundaries in
`tools/casegraphen/src/native_model.rs`, a stable package-level report envelope
in `tools/casegraphen/src/native_report.rs`, and the current file-backed native
store, evaluator, close-check, and CLI routing in `tools/casegraphen/src/`.
Review commands and arbitrary payload materialization remain planned; current
mutation is intentionally bounded to metadata-only morphism append/reject
flows.

Versioned JSON contracts live in:

- `schemas/casegraphen/native.case.space.schema.json`
- `schemas/casegraphen/native.case.space.example.json`
- `schemas/casegraphen/native.case.report.schema.json`
- `schemas/casegraphen/native.case.report.example.json`
- `examples/casegraphen/native/README.md`
- `examples/casegraphen/native/reports/README.md`

The Rust contract preserves the design vocabulary directly:

- `CaseSpace`, `CaseCell`, `CaseRelation`, `CaseMorphism`,
  `MorphismLogEntry`, `Revision`, `Projection`, `ReviewRecord`,
  `ClosePolicy`, `CloseCheck`, and `NativeCaseReport` are package-level public
  types exported by `casegraphen`.
- Stable IDs, confidence, severity, provenance, and review status reuse
  `higher-graphen-core` primitives already used by the existing casegraphen
  models. No new dependency edge is introduced.
- Unknown fields are rejected at Rust serde boundaries. Extension points are
  explicit: downstream-owned taxonomy values must use `custom:<extension>`,
  while free-form payloads are isolated under `metadata`.
- Readiness/frontier/blocker/closeability outputs remain projections. The
  native `CaseCell` contract does not store `ready`, `frontier`, `blocked`, or
  `closeable` as mutable cell fields.
- Evidence origin and review outcome remain separate: `EvidenceBoundary`
  distinguishes `source_backed`, `inferred`, `review_promoted`, `rejected`, and
  `contradicting`, while `ReviewRecord` and core `ReviewStatus` preserve the
  explicit review decision.

## Relation To Current Workflow Graphs And Bridge

The current `highergraphen.case.workflow.graph.v1` model is the migration
source for native case management, not the final native aggregate.

Mapping:

| Current workflow record | Native target |
| --- | --- |
| `WorkflowCaseGraph` | Imported `CaseSpace` revision plus migration or lift morphism. |
| `WorkItem` | `CaseCell` with `cell_type` mapped from `item_type`. |
| `WorkflowRelation` | `CaseRelation`. |
| `ReadinessRule` | Readiness policy cell or relation constraint. |
| `EvidenceRecord` | Evidence `CaseCell` plus evidence relations. |
| `CompletionCandidate` | Completion `CaseCell`, initially unreviewed. |
| `CompletionReviewRecord` | Review `CaseCell` and review `CaseMorphism`. |
| `TransitionRecord` | `CaseMorphism`. |
| `ProjectionProfile` | Projection `CaseCell` or projection request. |
| `CorrespondenceRecord` | Correspondence relation or diagnostic cell. |
| workflow history entry | `MorphismLog` migration/import entry and revision record. |
| workflow metadata/source records | `CaseSpace.metadata.source_boundary`, source-boundary cells, or source-boundary relations. |

The existing `casegraphen cg workflow ...` bridge remains a compatibility
bridge while native case management is built. Its purpose is to import,
inspect, replay, and review current workflow graph snapshots. It should not
become the native command namespace and should not imply that installed `cg`
semantics are the native product model.

Migration sequence:

1. Build a `CaseSourceBoundary` from the workflow graph, store path, source
   schema, source revision, projection loss, and imported adapter.
2. Import a current workflow graph into a native store as a genesis migration
   or lift morphism.
3. Replay the imported cells, relations, contexts, and source-boundary records
   into a native `CaseSpace`.
4. Preserve source workflow graph ID, revision ID, store path, report IDs,
   accepted input facts, inferred records, and import loss in migration
   metadata.
5. Run native validation, readiness projections, and policy/capability gates.
6. Compare native reports against current workflow reports until parity is
   sufficient.
7. Move operator docs from `casegraphen cg workflow ...` bridge commands to
   native `casegraphen case ...` and `casegraphen morphism ...` commands;
   include `casegraphen review ...` only after that planned command surface is
   implemented.

## Verification Strategy And Implementation Sequence

Verification layers:

- source-boundary tests proving included/excluded sources and lift information
  loss survive import, replay, projection, and close-check;
- schema validation for native case space, cell, relation, morphism, log entry,
  projection, and close-check records;
- replay determinism tests from `MorphismLog` to `CaseSpace`;
- context modeling tests proving local context relations or external context
  references are preserved and disclosed in projection loss when omitted;
- readiness projection tests proving ready/frontier/blocker values are derived,
  not stored;
- evidence-boundary tests proving unreviewed inference cannot satisfy hard
  evidence requirements;
- policy/capability gate tests proving projected allowed operations are not
  treated as authorization, and blocked supplied core extensions make
  close-check not closeable or morphism-check not applicable;
- completion review tests proving generated completions start unreviewed and
  change only through review morphisms;
- morphism check/apply/reject tests with stale-revision and invariant failure
  cases;
- close-check tests for success, unresolved obstruction, missing evidence,
  unreviewed completion, projection loss, and replay checksum mismatch;
- migration parity tests from current workflow graph examples;
- CLI JSON report tests for every native command;
- storage validation tests proving caches are replayable and non-authoritative.

Recommended implementation sequence and status:

1. Add native schemas and Rust model records for `CaseSpace`, `CaseCell`,
   `CaseRelation`, `CaseMorphism`, `MorphismLogEntry`, `Revision`, and
   projection requests. Implemented.
2. Implement log replay, validation, and deterministic revision checksums.
   Implemented.
3. Implement derived readiness, obstruction, completion, evidence, projection,
   and evolution evaluators over the replayed space. Implemented.
4. Implement native store operations and storage validation. Implemented.
5. Add read-only CLI commands: `case list`, `case inspect`, `case history`,
   `case replay`, `case validate`, `case reason`, `case frontier`,
   `case obstructions`, `case completions`, `case evidence`, and
   `case project`. Implemented.
6. Add morphism proposal/check/apply/reject commands with conservative
   metadata-only materialization. Implemented.
7. Add close-check. Implemented.
8. Add full review commands, arbitrary typed morphism reducers, and native
   `case close`. Planned.
9. Add migration from current workflow graph stores and keep the bridge
   documented as transitional.
10. Update skills, examples, and verification gates after command names are
    implemented.

Implementation note: `tools/casegraphen/src/native_eval.rs` now provides the
read-only package evaluator for step 3. It validates the native `CaseSpace` and
`MorphismLog` contracts before deriving readiness, frontier, hard blockers,
obstructions, unreviewed completion candidates, evidence-boundary findings,
review gaps, projection loss, correspondence summaries, morphism evolution, and
a close-check skeleton. These are successful domain reports; only malformed
spaces, dangling native references, and invalid morphism-log contracts are
structured errors.

Implementation note: `tools/casegraphen/src/native_review.rs` provides the
native review and close-check package API for step 7 and the package-level part
of step 8. Review actions produce metadata-only `CaseMorphism` records that can
be appended by callers through the native store; generated review morphisms are
not silently applied. Close-check derivation treats completion candidates,
evidence, morphisms, residual risks, waivers, and projection loss as closeable
only after explicit review, deferral, waiver, or caller declaration. AI
inference remains separate from accepted evidence unless an explicit review
morphism promotes or waives it.

Final verification status for `task_native_case_e2e_verification`: the native
reference fixture and CLI integration tests exercise create, import, list,
inspect, history, replay, reason, frontier, obstructions, completions,
evidence, project, close-check, and morphism propose/check/apply/reject flows.
The native case-space and native report example JSON files are validated
against their checked-in JSON Schema files by package integration tests. The
release gate run on 2026-04-26 passed `cargo fmt --all --check`,
`cargo test -p casegraphen`, `cargo test --workspace`,
`sh scripts/static-analysis.sh`,
`python3 integrations/cli-skill-bundle/check-bundle.py`, `git diff --check`,
`cg validate --case casegraphen-native-case-management`,
`cg validate storage`, and
`cg history topology --case casegraphen-native-case-management --higher-order
--format json`.

## Risks And Limitations

- The native model is more general than the current workflow graph; migration
  needs parity tests to avoid losing current operator behavior.
- Source-boundary metadata is a v1 compatibility encoding. If downstream
  integrations rely on it heavily, v2 should promote it into a first-class
  field instead of leaving it in metadata.
- Context modeling through `custom:context` and custom relations is compatible
  with v1 but less self-documenting than a first-class native context record.
- Derived readiness is safer than stored readiness but requires deterministic
  replay and clear policy versioning.
- Completion and evidence review semantics can become noisy unless projections
  separate human action views from audit detail.
- Append-only logs need compaction or snapshot strategy once case spaces grow.
- Content-addressed entries and revision checksums are useful, but they add
  compatibility constraints for future schema migrations.
- Bridge commands may confuse operators if docs do not clearly distinguish
  native `casegraphen case ...` from transitional
  `casegraphen cg workflow ...`.
- Close invariants are policy-sensitive; early implementation should keep the
  default policy strict and make waivers explicit review morphisms.
- Projection `allowed_operations` can be misread as authorization unless
  policy/capability gates remain explicit in docs, schemas, examples, and
  skills.
- Store layout and package APIs should be treated as target contracts until the
  implementation tasks prove them with schemas, tests, and reference fixtures.
