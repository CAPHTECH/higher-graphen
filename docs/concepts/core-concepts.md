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

HigherGraphen v0.3.0 also exposes support objects used by the extension
concepts above. These include object references, lifecycle states, review
requirements, inference rules, verifiers, payload references, reachability
records, scenario changes, validity intervals, policy rules, policy
applicability, valuation criteria, trade-offs, schema mappings, and schema
verification records.

These support objects should be used when they carry validation, review,
projection, or agent-operation meaning. They should not be added as decorative
metadata.

