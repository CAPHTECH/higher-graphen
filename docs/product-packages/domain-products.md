# Domain Products

HigherGraphen supports domain products through interpretation packages. A
domain product should define domain vocabulary, mappings, invariants,
completion rules, and projections while reusing the shared structural core.

## Contract Product

Purpose:

Represent contracts, clauses, obligations, deadlines, responsibility, and
evidence as higher structure.

Interpretation:

| HigherGraphen primitive | Contract interpretation |
| --- | --- |
| Cell | Contract, clause, party, obligation, deadline, evidence. |
| Morphism | Amendment, renewal, termination, clause-to-obligation mapping. |
| Invariant | Obligation must have a responsible party; material change requires prior notice. |
| Obstruction | Unfulfillable obligation, missing notice clause, conflicting obligation. |
| Projection | Contract review report, obligation matrix, risk summary. |

## Project Product

Purpose:

Represent tasks, dependencies, milestones, deliverables, teams, constraints, and
plan revisions as higher structure.

Interpretation:

| HigherGraphen primitive | Project interpretation |
| --- | --- |
| Cell | Task, milestone, deliverable, team, dependency, risk. |
| Morphism | Dependency, status transition, plan revision, scope change. |
| Invariant | A dependent task cannot start before its prerequisite is complete. |
| Obstruction | Impossible schedule, missing dependency, conflicting milestone. |
| Projection | Project review, delivery risk report, dependency action plan. |

## Evidence Product

Purpose:

Represent claims, observations, support relations, contradiction relations,
review status, and confidence as higher structure.

Interpretation:

| HigherGraphen primitive | Evidence interpretation |
| --- | --- |
| Cell | Claim, evidence, observation, source, reviewer. |
| Morphism | Claim-to-evidence mapping, source transformation, review transition. |
| Invariant | Accepted claims must have sufficient support and no unresolved critical contradiction. |
| Obstruction | Unsupported claim, contradiction, unreviewed inference presented as fact. |
| Projection | Evidence report, claim confidence summary, review queue. |

## DDD Review Product

Purpose:

Represent bounded contexts, domain model decisions, aggregate boundaries,
context mappings, evidence, and review gates as higher structure.

Interpretation:

| HigherGraphen primitive | DDD review interpretation |
| --- | --- |
| Cell | Bounded context, aggregate, entity, value object, service, API, database, evidence, review gate. |
| Morphism | Context mapping, anti-corruption translation, model split, equivalence claim, decision revision. |
| Invariant | Shared cross-context model requires accepted equivalence evidence; boundary translations must be explicit; review gates must be accepted before close. |
| Obstruction | Boundary semantic loss, cross-context identity collapse, missing evidence, missing boundary mapping, projection information loss. |
| Completion | Add anti-corruption mapping, split model by context, request source-backed evidence, complete domain review. |
| Projection | DDD review report, implementation risk view, AI-agent view, audit trace. |

The stable CLI contract is
[`../specs/ddd-review-cli-contract.md`](../specs/ddd-review-cli-contract.md).
DDD-specific interpretation belongs to this product workflow and must not be
promoted into `higher-graphen-core`.

## Product Package Requirements

Every product package should provide:

- Domain cell type mappings.
- Domain morphism type mappings.
- Invariant templates.
- Constraint templates.
- Completion rules.
- Projection templates.
- Lift adapters from common input formats.
- Example scenarios.
