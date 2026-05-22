# Correspondence and Overlap

HigherGraphen treats information overlap as a reviewable structure, not as a
similarity score. A `CorrespondenceCell` records that two or more structures are
related under a context, then keeps the shared parts and differing parts as
explicit witnesses.

Core vocabulary:

- `CorrespondenceCell`: higher-order cell for a relationship among structures.
- `OverlapWitness`: reviewable evidence for what is shared.
- `DifferenceWitness`: reviewable evidence for what differs or conflicts.
- `GluingAttempt`: result of checking whether participants can be joined.

Correspondence is not equality, acceptance, or merge. A correspondence can be
agreeing, conflicting, refining, projecting, ambiguous, or unknown. Conflicts are
valid correspondences when the participants share an explicit structure such as
the same subject, relation, object, invariant, evidence, or context.

## Phase 1 Scope

The initial implementation adds the schema, data model, deterministic overlap
and difference extraction, deterministic gluing checks, projection explanation,
and audience-specific correspondence projection. It deliberately does not add
semantic overlap detection, embeddings, LLM-based merge, automatic acceptance,
or a pushout-style gluing algorithm.

The implemented invariants are:

- accepted correspondences require evidence;
- conflicting correspondences require at least one overlap witness;
- accepted semantic overlap requires a `NormalizedClaim`, `PredicateSet`, or
  `FeatureSet` witness;
- gluing success requires a preservation report;
- a blocking difference prevents silent gluing success without explicit review.

The deterministic detector currently extracts overlap from exact identifiers,
normalized labels, shared evidence, shared invariants, and typed relation
triples. It also extracts difference witnesses for modality, evidence,
invariant satisfaction, and context mismatches when an explicit overlap exists.

The deterministic gluing checker classifies a correspondence into:

- `Failure` when a blocking difference or failed invariant check is present;
- `Candidate` when review is required because overlap, evidence, review status,
  or major differences prevent a silent merge;
- `Success` only when explicit overlap and evidence are present, no blocking or
  major differences exist, and the preservation report is non-empty.

AI-derived overlap should start as `candidate`. Projection code must not render a
candidate as accepted fact, and any omitted witnesses, evidence, contexts, or
status changes must be represented as projection loss.

Audience-specific correspondence projection supports human reviewer, AI agent,
and audit-oriented consumers. The first CLI renderer supports Markdown for
human review and JSON for machine consumers. Both renderers keep review status,
confidence, evidence, gluing result, obstruction, and `ProjectionLoss` visible.

Semantic candidate generation is implemented as a bounded trust boundary:
external LLM, embedding, tool, import, or human adapters may supply
`semanticSignals`, but HigherGraphen converts them only into
`SemanticOverlap` candidates with explicit `OverlapWitness` records. Confidence
is calibrated and capped by signal source, and candidates are ranked by
calibrated confidence. LLM and embedding signals are never accepted
automatically; explicit correspondence review is required to accept or reject a
semantic candidate.

## Example

The fixture at
`schemas/highergraphen/correspondence.graph.example.json` models a direct
database access conflict:

- observed claim: Order Service accesses Billing DB;
- invariant: direct cross-context DB access is forbidden;
- evidence: repository scan observed the access.

The shared normalized claim is the overlap witness. The observed-vs-forbidden
modality mismatch is the blocking difference. The gluing attempt fails with the
obstruction `obstruction:direct-db-access-violates-boundary`.
