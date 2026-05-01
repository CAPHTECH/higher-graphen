# DDD Review CLI Contract

This document defines the pre-implementation contract for promoting the
CaseGraphen DDD diagnostic prototype into a stable `highergraphen` CLI
workflow. It is documentation and schema contract work only; it does not require
Rust implementation changes.

The workflow belongs to the `highergraphen` product CLI because it is a
domain-product review workflow over bounded source structure. It must not add
DDD-specific interpretation to `higher-graphen-core` or CaseGraphen core. Lower
crates may provide shared cells, incidences, morphisms, evidence, confidence,
review status, obstructions, completions, and projection primitives, but terms
such as bounded context, aggregate, anti-corruption layer, ubiquitous language,
and aggregate ownership belong in this workflow, its schemas, fixtures, docs,
and skills.

The motivating reference is the Sales/Billing Customer fixture at
[`examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json`](../../examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json).
That fixture records a proposed shared Customer model across Sales and Billing,
accepted workshop evidence, unreviewed AI equivalence evidence, a boundary issue
for semantic loss, an unreviewed anti-corruption mapping completion candidate,
projection loss, and a non-closeable review state.

## Command Surface

The stable command namespace is:

```sh
highergraphen ddd input from-case-space \
  --case-space <path> \
  --format json \
  [--output <path>]

highergraphen ddd review \
  --input <path> \
  --format json \
  [--output <path>]
```

`ddd input from-case-space` is a deterministic adapter from a native
CaseGraphen `CaseSpace` JSON document into `highergraphen.ddd_review.input.v1`.
It reads exactly the file named by `--case-space`; it does not import into a
CaseGraphen store, replay a store, call `cg`, read network resources, or use
LLM inference. It may copy source-backed CaseSpace records as accepted facts and
copy inference-like CaseSpace records as unreviewed claims.

`ddd review` reads a bounded `highergraphen.ddd_review.input.v1` document and
emits `highergraphen.ddd_review.report.v1`. It evaluates fixed DDD review
invariants over the bounded snapshot and emits report data. Domain findings are
successful report results, not CLI failures.

When `--output` is omitted, each command writes exactly one JSON document to
stdout. When `--output` is supplied, each command writes exactly one JSON file
and keeps stdout empty. `--format json` is required; no human text format is in
scope for v1.

## Input Snapshot

Schema ID: `highergraphen.ddd_review.input.v1`

Required fields:

| Field | Contract |
| --- | --- |
| `schema` | Must equal `highergraphen.ddd_review.input.v1`. |
| `source` | Source metadata for the bounded input document and adapter. |
| `review_subject` | The design decision, model, bounded context map, or case space under review. |
| `source_boundary` | Explicit boundary of what was read, omitted, summarized, or adapted. |
| `accepted_facts` | Source-backed DDD facts accepted only as input observations. |

Optional fields:

| Field | Contract |
| --- | --- |
| `constraints` | DDD constraints, policies, invariants, compatibility rules, or review gates supplied by source material. |
| `reviews` | Explicit review requirements or review decisions supplied by source material. |
| `inferred_claims` | AI-created or adapter-inferred claims copied into the snapshot with `review_status: "unreviewed"`. |
| `completion_hints` | Existing unreviewed completion candidates from source material, such as a missing anti-corruption mapping. |
| `projection_requests` | Requested human, AI-agent, implementation, or audit views. |

Accepted input facts may include bounded contexts, aggregates, entities, value
objects, services, APIs, databases, teams, domain events, commands, messages,
ownership relations, dependency relations, translation relations, evidence
records, source notes, and explicit review records. Acceptance here means "the
workflow may use this record as supplied input." It does not mean the DDD design
is approved, closeable, or semantically correct.

`inferred_claims` are never accepted facts. They must carry
`review_status: "unreviewed"` and a source boundary such as
`ai_inference`, `adapter_inference`, or `unreviewed_note`. Confidence does not
promote them.

## Source Boundary

The source boundary is part of the scenario. The adapter and review command
must preserve:

- input file paths and schema IDs;
- adapter IDs such as `casegraphen_case_space.v1` or `ddd_review_input.v1`;
- source-backed evidence IDs separately from AI inference IDs;
- omitted store replay, omitted MorphismLog history, omitted full workshop
  notes, omitted code, omitted ADRs, omitted tickets, and omitted tests;
- projection requests that are implementation-focused and may hide domain risk.

The v1 workflow is deterministic and local. It must not fetch network data, call
provider APIs, ask an LLM for hidden interpretation, scan unrelated repository
files, or silently promote inferred claims into accepted facts.

## Structural Lift

The DDD snapshot lifts into one review space:

| DDD concept | HigherGraphen review structure |
| --- | --- |
| Bounded context | Accepted or unreviewed `ddd.bounded_context` cell. |
| Aggregate, entity, value object, service, API, database, team | Accepted or unreviewed `ddd.model_element` cell with a `ddd_type`. |
| Domain event, command, external message | `ddd.message` cell. |
| Depends on, owns, publishes, consumes, translates | Incidence with source provenance and review status. |
| Invariant, policy, compatibility rule | `ddd.constraint` cell or invariant record. |
| Context boundary or anti-corruption layer | Boundary cell, relation, or completion candidate. |
| Workshop note, ADR, source finding, test result | Evidence cell with `evidence_boundary: "source_backed"` when supplied by the source. |
| AI-produced equivalence proof or diagnostic | Inferred claim with `review_status: "unreviewed"`. |
| BoundaryIssue, MissingCase, RiskCase | Obstruction or semantic-case-like result record. |

The lift may create report-local cells, incidences, contexts, and morphism
summaries, but DDD labels remain workflow-local. No DDD-specific enum, cell
type, invariant template, or projection purpose should be added to
`higher-graphen-core`.

## Invariants

The initial review evaluates these deterministic invariants against the bounded
snapshot:

| Invariant | Contract |
| --- | --- |
| Context language preserved | A shared model across bounded contexts must preserve each context-specific meaning or declare reviewed equivalence evidence. |
| Cross-context identity not collapsed | Two context-specific entities must not be treated as one accepted model when equivalence evidence is missing or unreviewed. |
| Boundary translation explicit | Cross-context dependency that changes meaning should have an accepted boundary, translation rule, or anti-corruption mapping. |
| Review gates satisfied before close | A decision requiring domain review is not closeable until the review record is accepted. |
| Inference not accepted evidence | AI-created equivalence proofs, mappings, risks, and suggestions cannot satisfy accepted evidence requirements. |
| Projection declares loss | Every projection view must state information loss when it omits risk, evidence, review state, or boundary semantics. |

Absence of a violation means only that the bounded snapshot did not violate
these invariants. It does not prove the entire domain model is correct.

## Report Envelope

Schema ID: `highergraphen.ddd_review.report.v1`

Report type: `ddd_review`

The report uses the standard runtime-style envelope:

```json
{
  "schema": "highergraphen.ddd_review.report.v1",
  "report_type": "ddd_review",
  "report_version": 1,
  "metadata": {},
  "scenario": {},
  "result": {},
  "projection": {}
}
```

`metadata.command` must be `highergraphen ddd review`.
`scenario.input_schema` must be `highergraphen.ddd_review.input.v1`.

## Result Sections

The report result must include these sections:

| Field | Contract |
| --- | --- |
| `status` | `issues_detected`, `no_issues_in_snapshot`, or `unsupported_input`. |
| `accepted_fact_ids` | IDs of source-backed input observations used by the workflow. |
| `inferred_claim_ids` | IDs of unreviewed claims considered but not accepted. |
| `evaluated_invariant_ids` | DDD review invariant IDs evaluated against the bounded snapshot. |
| `obstructions` | Boundary, identity, evidence, contradiction, missing-case, projection-loss, and review-gate blockers. |
| `completion_candidates` | Unreviewed proposed fixes, such as an anti-corruption mapping. |
| `evidence_boundaries` | Machine-readable separation of source-backed evidence, AI inference, adapter inference, missing evidence, and omitted evidence. |
| `projection_loss` | Information loss by projection/view, including implementation views that hide domain risk or accepted evidence. |
| `review_gaps` | Missing, unaccepted, stale, or contradictory review records. |
| `closeability` | Whether the bounded design can be treated as closeable and the blockers that prevent close. |
| `source_ids` | IDs represented in the result. |

`obstructions`, `completion_candidates`, `projection_loss`, and `review_gaps`
emitted by the review workflow are suggestions or findings with
`review_status: "unreviewed"` unless they copy an explicit accepted source
record. The command must not mutate the input or accept/reject candidates.

## Obstructions

Initial obstruction types:

| Type | Witness payload |
| --- | --- |
| `boundary_semantic_loss` | Context IDs, shared model ID, lost meaning, evidence IDs, inferred claim IDs. |
| `cross_context_identity_collapse` | Collapsed entity IDs, context-specific meanings, missing accepted equivalence evidence. |
| `missing_boundary_mapping` | Source context, target context, dependency relation, missing translation or ACL candidate. |
| `missing_evidence` | Target decision or claim, required evidence kind, supplied unreviewed evidence, accepted alternatives. |
| `review_required` | Review gate ID, decision ID, required reviewer or role, current review status. |
| `projection_information_loss` | Projection ID, omitted risk/evidence/review/boundary records, affected source IDs. |
| `contradiction_candidate` | Decision, constraint, relation cluster, and conflicting source or inference IDs. |

For the Sales/Billing Customer fixture, the expected obstruction set includes a
boundary issue for the shared Customer model, missing accepted equivalence
evidence, a required domain model review, and projection information loss for
the implementation-focused view.

## Completion Candidates

DDD review completion candidates must include:

| Field | Contract |
| --- | --- |
| `id` | Stable candidate ID within the report or copied from source input. |
| `candidate_type` | `boundary_mapping`, `domain_review`, `evidence_request`, `model_split`, or `constraint_update`. |
| `target_ids` | Decisions, contexts, model elements, evidence, review gates, or obstructions addressed. |
| `obstruction_ids` | Obstructions this candidate may resolve if accepted and implemented. |
| `suggested_change` | Structured suggested action. |
| `rationale` | Why the change is relevant to the DDD review. |
| `provenance` | Source IDs and extraction method. |
| `severity` | Impact if the gap remains unresolved. |
| `confidence` | Inference confidence from `0.0` to `1.0`. |
| `review_status` | `unreviewed` when emitted or copied as unreviewed source material. |

The Sales/Billing reference should preserve
`completion:missing-sales-billing-acl` as an unreviewed candidate for an
explicit Sales-to-Billing anti-corruption mapping.

## Evidence, Review, And Promotion Boundary

The review workflow uses this boundary:

| Category | Examples | Report treatment |
| --- | --- | --- |
| Accepted source facts | CaseSpace records with source-backed evidence, accepted workshop notes, explicit source constraints, accepted reviews. | `accepted_fact_ids` and source-backed evidence boundaries. |
| AI-inferred or unreviewed claims | Equivalence proof, generated DDD diagnostic, inferred missing mapping, suggested split. | `inferred_claim_ids`, unreviewed obstructions, unreviewed completion candidates, or unreviewed evidence boundaries. |
| Missing evidence | Required equivalence proof, missing source note, absent review. | Obstruction or review gap. |
| Explicit later review | Human or authorized workflow accept/reject decision. | Separate review report or later bounded input; never silent mutation of the source report. |

Confidence never implies acceptance. Severity never implies confidence. A
candidate can become accepted only through an explicit review workflow or a
later bounded input that contains accepted evidence and review records.

## Projection Contract

The projection should follow the existing `ProjectionViewSet` style:

| View | Contract |
| --- | --- |
| `human_review` | Summarizes DDD risks, closeability, review gaps, and recommended next actions. |
| `ai_view` | Preserves stable IDs for contexts, model elements, relations, constraints, obstructions, candidates, evidence boundaries, projection loss, review gaps, confidence, severity, and review status. |
| `audit_trace` | Records source boundary, represented IDs, adapters, accepted versus inferred records, omitted material, and information loss. |

Every view must carry non-empty `source_ids` and at least one
`information_loss` entry. Projection must not change `review_status`.

## Schema And Fixture Artifacts

Contract artifacts:

- Input schema: `schemas/inputs/ddd-review.input.schema.json`
- Input fixture: `schemas/inputs/ddd-review.input.example.json`
- Report schema: `schemas/reports/ddd-review.report.schema.json`
- Report fixture: `schemas/reports/ddd-review.report.example.json`

The fixtures should mirror the Sales/Billing Customer example and should remain
small enough for deterministic schema validation.

## Verification Plan

Implementation tasks should verify:

1. JSON Schema validation for the checked-in input and report fixtures.
2. CLI behavior for stdout versus `--output` on both DDD commands.
3. Adapter behavior from the Sales/Billing CaseSpace fixture into the bounded
   input snapshot.
4. Report behavior that preserves accepted workshop evidence, keeps AI
   equivalence evidence unreviewed, emits the missing anti-corruption mapping
   candidate as unreviewed, reports projection loss, and marks closeability as
   false.
5. Absence of network calls, hidden LLM inference, provider API reads, and
   silent promotion of inferred claims.
