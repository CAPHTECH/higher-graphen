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

