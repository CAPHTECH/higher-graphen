# Core Concepts

This document defines the shared vocabulary for HigherGraphen.

## Space

A Space is the top-level structural container for a target world.

A space owns cells, complexes, contexts, morphisms, invariants, constraints,
obstructions, completions, projections, and metadata. A product package usually
creates one or more spaces for a domain-specific target world.

## Cell

A Cell is a typed structural element inside a space.

Cells are not limited to ordinary graph nodes. A cell has a dimension:

| Dimension | Typical meaning |
| --- | --- |
| `0-cell` | Entity, object, concept, observation point, requirement, component. |
| `1-cell` | Relation, transition, dependency, mapping, edge. |
| `2-cell` | Relation between relations, triple interaction, consistency condition. |
| `n-cell` | Higher-order relation, higher-order constraint, or higher-order consistency condition. |

## Incidence

Incidence records how cells are connected. It is the basic relation mechanism
used by graph-like, hypergraph-like, and complex-like structures.

## Complex

A Complex is an organized collection of cells and incidences.

HigherGraphen supports several complex styles:

- Typed graph
- Hypergraph
- Simplicial complex
- Cell complex
- Custom complex

## Context

A Context defines a local region of meaning, validity, vocabulary, or rules.

The same cell can be meaningful in multiple contexts. Contexts make it possible
to express local validity and global inconsistency separately.

## Section

A Section is an assignment over a context.

Sections are useful for representing local observations, local interpretations,
or context-specific values that may later need to be glued into a globally
consistent structure.

## Morphism

A Morphism maps one structure to another.

Morphism types include abstraction, refinement, translation, projection, lift,
migration, interpretation, and custom mappings. A morphism records preserved
invariants, lost structure, distortion, and composition constraints.

## Invariant

An Invariant is a property that must remain true under specific transformations
or within a defined scope.

Examples:

- A requirement must be verified by at least one test.
- A service must not directly access a database owned by another bounded context.
- A projection must declare information loss.

## Constraint

A Constraint is a checkable condition over cells, contexts, or assignments.

Invariants describe properties that should be preserved. Constraints describe
conditions that can be checked and reported as violations.

## Obstruction

An Obstruction is a structured explanation of why a condition cannot hold, why
a transformation cannot proceed safely, or why a global structure cannot be
assembled from local structures.

Obstructions can include counterexamples, location cells, related contexts,
related morphisms, severity, and required resolution.

## Completion Candidate

A Completion Candidate is a proposed missing structure.

HigherGraphen does not silently fill missing structure as fact. It proposes a
candidate with rationale, inferred sources, confidence, and review status.

## Candidate Optimization

Candidate Optimization compares reviewable candidates under explicit
objectives, constraints, and measurements.

Optimization helps rank completion candidates, repair plans, review targets, or
scenario changes without accepting them. It may use Pareto frontiers,
lexicographic ordering, hard-blocked candidates, infeasibility obstructions, or
other bounded finite methods. Its output is a recommendation record with review
status, not an applied change.

## Projection

A Projection converts higher structure into a usable view for an audience and
purpose.

Examples:

- Architecture review report
- Developer action plan
- Operator dashboard
- Executive summary
- API response
- Query result

Every projection should declare meaningful information loss.

## Projection Loss Metric

A Projection Loss Metric is a finite, reviewable measurement of what a
projection collapses, omits, or makes ambiguous.

Projection loss metrics make information loss inspectable instead of relying
only on prose declarations. A metric can record source cardinality, projected
cardinality, collapsed source distinctions, ambiguity, missing loss
declarations, and source trace gaps. A metric does not prohibit lossy
projection. It helps humans, agents, and policies decide whether the loss is
declared, acceptable, and appropriate for the projection audience.

## Interpretation Package

An Interpretation Package gives domain meaning to the abstract HigherGraphen
model.

It defines cell type mappings, morphism type mappings, invariant templates,
projection templates, lift adapters, completion rules, and domain vocabulary.

## Provenance

Provenance records where a structural element came from, how it was extracted,
its confidence, and its review status.

Provenance is required because HigherGraphen must distinguish observed facts,
human claims, AI-generated inferences, and reviewed conclusions.

## Equivalence Claim

An Equivalence Claim is a reviewable claim that two or more structures may be
treated as equivalent under declared criteria, scope, witnesses, and quotient
effects.

It is not a silent merge and it is not accepted identity by default. An
equivalence claim remains a candidate until its scope, criteria, supporting
witnesses, quotient losses, unresolved obstructions, provenance, and review
state make acceptance safe.

## Derivation

A Derivation records how premises, inference rules, warrants, and verification
produce a conclusion.

Provenance answers where a structure came from. A derivation answers why a
conclusion follows. It can record excluded premises, counterexamples,
verification status, verifier identity, and failure modes such as missing
premises, invalid rules, circular reasoning, or unsupported jumps.

## Witness

A Witness is observable support or contradiction for a structural judgment.

Witnesses can support or refute claims, derivations, equivalence claims,
constraint violations, invariant checks, and obstructions. A witness points to a
payload such as a log entry, metric point, test result, code location, document
excerpt, counterexample, human review, machine check result, or external
reference. Its validity contexts and review status determine whether it may be
used as accepted support.

## Scenario

A Scenario represents a hypothetical, reachable, blocked, counterfactual,
planned, refuted, or accepted operational world relative to a base space.

Scenarios let HigherGraphen compare actual, proposed, reachable, and what-if
structures without confusing them with accepted current state. A scenario can
record assumptions, changed structures, reachability, affected invariants,
expected obstructions, required witnesses, valuations, provenance, and review
state.

## Capability

A Capability records which actor can perform which operation on which target in
which contexts.

Capabilities are actor-specific operational permissions over structures. They
can cover operations such as read, propose, modify, accept, reject, project,
execute morphism, merge equivalence, create scenario, or approve a policy
exception. Active use may require preconditions, postconditions, absence of
forbidden effects, explicit review, and a valid time interval.

## Policy

A Policy is a system-wide or context-bound rule for permission, prohibition,
obligation, review, projection safety, candidate acceptance, data boundaries, or
escalation.

Policies make operational and review constraints explicit instead of leaving
them as informal prose. They define applicability, rules, required witnesses,
required derivations, escalation paths, violation obstruction templates,
provenance, and review state.

## Valuation

A Valuation records a value judgment about a structure, morphism, completion
candidate, scenario, projection, or other target under an explicit evaluation
context.

Valuations are for comparing alternatives by criteria, evidence, ordering mode,
and trade-offs. They are not substitutes for invariant satisfaction. They may
use scalar scores, lexicographic order, partial order, Pareto frontiers,
threshold acceptance, or qualitative ranking, and they can explicitly mark
alternatives as incomparable.

## Order Relation

An Order Relation records that one structure is weaker, stronger, more
abstract, more refined, more supported, or otherwise ordered relative to
another structure under explicit criteria.

Order relations are useful for requirement strength, evidence support,
abstraction levels, policy precedence, and refinement checks. They are claims
unless accepted by review. A reasoning kernel may check finite consequences
such as comparability, cycles, meet or join candidates, and whether a morphism
preserves the declared order.

## Abstract State

An Abstract State is a conservative approximation of a larger or more concrete
space.

Abstract states let HigherGraphen reason over large or partially known systems
without pretending to know every concrete detail. An abstract state can mark an
invariant as definitely satisfied, possibly violated, or unknown. Unknown or
possible results are not accepted violations unless a concrete witness,
accepted derivation, or reviewed concretization supports them.

## Graph Analytic

A Graph Analytic is a bounded structural measurement over a selected graph or
incidence view.

Graph analytics support impact analysis, review targeting, boundary detection,
and dependency risk. Examples include impact cones, cut sets, articulation
points, bridges, dominators, central cells, and cycle reduction candidates.
These analytics produce prompts, rankings, and obstructions; they do not assign
final review ownership or approve changes.

## Temporal Property

A Temporal Property is a checkable statement about states, transitions, traces,
or event ordering.

Temporal properties express behavior that cannot be reduced to a single static
structure. Examples include forbidden reachability, required eventual actions,
always-before ordering, and absence of dead-end states except accepted terminal
states. Bounded temporal checks must report their bounds and counterexample
traces when they find violations.

## Diagram Construction

A Diagram Construction is a finite structural operation over spaces and
morphisms, such as a commutativity check, pullback candidate, or pushout
candidate.

Diagram constructions help compare structures, find common substructure, and
propose merges. They are candidates unless the required equivalence claims,
quotient losses, and invariant preservation checks have been reviewed and
accepted.

## Observation Action

An Observation Action is a proposed evidence-gathering step for reducing
uncertainty about claims, candidates, scenarios, or obstructions.

Observation actions can record target claims, expected evidence kind,
estimated cost, expected information gain, policy blockers, provenance, and
review state. They recommend what to observe next. They do not execute the
observation or accept the claim being investigated.

## Schema Morphism

A Schema Morphism describes evolution between schemas, ontologies,
interpretation packages, or report contracts.

HigherGraphen structures are interpreted through models that can themselves
change. Schema morphisms record source and target schemas, interpretation
packages, mapping kind, individual mappings, affected objects, compatibility,
verification, provenance, review state, and explicit loss claims. They prevent
schema migration from being treated as an ordinary data mapping with no semantic
or compatibility risk.

## Core Extension Support Objects

HigherGraphen v0.4.0 also exposes support objects used by the extension
concepts above. These include object references, lifecycle states, review
requirements, inference rules, verifiers, payload references, reachability
records, scenario changes, validity intervals, policy rules, policy
applicability, valuation criteria, trade-offs, projection loss metrics,
optimization objectives, order relations, abstract states, graph analytics,
temporal properties, observation actions, diagram constructions, schema
mappings, and schema verification records.

These support objects should be used when they carry validation, review,
projection, or agent-operation meaning. They should not be added as decorative
metadata.
