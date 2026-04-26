---
name: casegraphen-ddd-diagnostics
description: Diagnose DDD and domain model design risks using CaseGraphen CaseSpace/MorphismLog reports. Use for DDD診断, bounded context review, aggregate review, domain model validation, BoundaryIssue, MissingCase, RiskCase, semantic loss, or context consistency checks.
---

# CaseGraphen DDD Diagnostics

Use this skill when a task asks whether a domain model, bounded context map, or
DDD design decision is structurally acceptable. Keep DDD interpretation in the
skill layer; do not add DDD-specific behavior to CaseGraphen core unless the
user explicitly asks for a new generic primitive.

## Inputs

Prefer one of these inputs:

- a native CaseGraphen `CaseSpace` JSON file;
- an installed `cg` case that contains ontology records;
- a domain model sketch, ADR, bounded context map, or code/doc excerpt that can
  be lifted into a CaseSpace.

If no input exists, create a small CaseSpace fixture first and mark AI-derived
claims as inference, not accepted evidence.

## DDD Mapping

Map DDD concepts into CaseGraphen records before judging them:

| DDD concept | CaseGraphen representation |
| --- | --- |
| Bounded Context | `CaseCell` with `cell_type: "custom:context"` |
| Aggregate, Entity, Value Object, Service, API, Database, Team | `CaseCell` with `cell_type: "custom:entity"` and metadata |
| Domain Event, command, external message | `custom:entity` with event/message metadata |
| Depends on, owns, publishes, consumes, translates | `CaseRelation` or domain ontology relation |
| Joint rule or multi-party risk | `custom:semantic_case`, relation cluster, or higher-order structure |
| Invariant, policy, compatibility rule | `custom:constraint` |
| Context boundary or ACL | `custom:boundary` or completion candidate |
| Mapping / translation rule | relation or transformation metadata |
| ADR, workshop note, source finding, test result | `evidence` with `evidence_boundary: "source_backed"` |
| AI-produced diagnostic | inference-like evidence with `evidence_boundary: "inferred"` |
| RiskCase / MissingCase / BoundaryIssue | `custom:semantic_case` |

## Native CaseGraphen Flow

For a native CaseSpace file:

Command shorthand: `casegraphen case reason`,
`casegraphen case obstructions`, `casegraphen case completions`,
`casegraphen case evidence`, `casegraphen case project`, and
`casegraphen case close-check`.

```sh
cargo run -q -p casegraphen -- case import --store <store> --input <case-space.json> --revision-id <revision_id> --format json
cargo run -q -p casegraphen -- case validate --store <store> --case-space-id <case_space_id> --format json
cargo run -q -p casegraphen -- case reason --store <store> --case-space-id <case_space_id> --format json
cargo run -q -p casegraphen -- case obstructions --store <store> --case-space-id <case_space_id> --format json
cargo run -q -p casegraphen -- case completions --store <store> --case-space-id <case_space_id> --format json
cargo run -q -p casegraphen -- case evidence --store <store> --case-space-id <case_space_id> --format json
cargo run -q -p casegraphen -- case project --store <store> --case-space-id <case_space_id> --format json
```

Run close-check only after naming the exact replay revision and validation
evidence:

```sh
cargo run -q -p casegraphen -- case close-check --store <store> --case-space-id <case_space_id> --base-revision-id <revision_id> --validation-evidence-id <evidence_id> --format json
```

## Diagnostic Patterns

Use these patterns as heuristics and state uncertainty explicitly:

- `boundary_semantic_loss`: a context boundary or transformation drops semantics
  that a downstream constraint needs.
- `cross_context_identity_collapse`: two context-specific concepts are treated
  as one model without accepted equivalence evidence.
- `missing_case_candidate`: pairwise relations exist but no semantic case,
  constraint, test, or review covers the joint condition.
- `contradiction_candidate`: a design decision and a constraint pull the same
  structure in incompatible directions.
- `evidence_gap`: a decision depends on inferred or unreviewed evidence.
- `projection_loss`: a reviewer or implementation view hides risk, evidence, or
  boundary information.

## Example Fixture

Use the repository fixture when demonstrating the workflow:

```sh
cargo run -q -p casegraphen -- \
  case import \
  --store /tmp/casegraphen-ddd-store \
  --input examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json \
  --revision-id revision:ddd-sales-billing-imported \
  --format json
```

Expected findings for that fixture:

- the unified Customer decision is blocked;
- the Sales/Billing Customer collapse is represented as a boundary issue;
- the equivalence proof is AI inference and does not satisfy accepted evidence;
- a missing Sales-to-Billing anti-corruption mapping is proposed;
- implementation projection hides the risk and accepted workshop evidence;
- close-check is not closeable.

## Output Style

Lead with findings, not a generic explanation. For each finding include:

- the affected context/entity/decision;
- the CaseGraphen evidence or obstruction ID;
- why it matters to the domain model;
- what completion, review, or source-backed evidence is needed next.

Do not treat AI inference as accepted evidence. Do not describe non-closeable,
blocked, missing evidence, or projection loss reports as CLI failures.
