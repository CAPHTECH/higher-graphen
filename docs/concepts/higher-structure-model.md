# Higher Structure Model

HigherGraphen extends ordinary graphs into a higher-structure model that can
represent local meaning, higher-order relations, transformations, constraints,
obstructions, and projections.

## From Graphs to Higher Structures

An ordinary graph is centered on nodes and edges:

```text
Node --Edge--> Node
```

HigherGraphen generalizes this into cells and complexes:

```text
0-cell: Object, concept, observation point.
1-cell: Relation, transition, dependency, mapping.
2-cell: Relation between relations, triple interaction, consistency condition.
n-cell: Higher-order relation, higher-order constraint, higher-order consistency condition.
```

This lets the model represent cases where correctness depends on combinations
of multiple elements, not only pairwise relationships.

## Structural Pipeline

A typical HigherGraphen workflow moves through the following structural stages:

```text
Source material
  -> Space
  -> Cells and incidences
  -> Complexes
  -> Contexts and sections
  -> Morphisms
  -> Invariants and constraints
  -> Obstructions
  -> Completion candidates
  -> Projections
```

This pipeline is conceptual. Implementations may run checks incrementally or in
different orders, but the distinction between each stage must remain explicit.

## Local and Global Structure

Contexts let HigherGraphen represent local validity without assuming global
validity.

Example:

```text
Context A: Term "Account" means user account.
Context B: Term "Account" means billing account.
Global check: A mapping between A and B must not assume semantic identity.
```

Sections can be attached to contexts and then checked for compatibility. A
failed compatibility check can produce an obstruction.

## Transformations and Preservation

Morphisms represent transformations between spaces or structures. They must
record:

- Source and target spaces.
- Cell mappings.
- Relation mappings.
- Preserved invariants.
- Lost structure.
- Distortion.
- Composition compatibility.

This makes transformation quality inspectable. HigherGraphen should not treat a
projection, abstraction, migration, or interpretation as lossless unless the
preservation checks support that claim.

## Obstruction and Completion

An obstruction is a structured failure. It is not just an error message.

Typical obstruction types include:

- Constraint unsatisfied
- Invariant violation
- Failed gluing
- Failed composition
- Missing morphism
- Context mismatch
- Projection loss
- Uncovered region

A completion candidate proposes missing structure that could resolve or reduce
an obstruction. Completion is reviewable:

```text
Detected missing structure
  -> CompletionCandidate
  -> Human or policy review
  -> Accept or reject
  -> Created structure or recorded rejection
```

## Projection Discipline

A projection makes higher structure usable by a target audience. It must not
hide the fact that information was lost or simplified.

Each projection should define:

- Audience.
- Purpose.
- Input selector.
- Output schema.
- Information loss.
- Optional renderer.

This keeps reports, dashboards, action plans, and API responses connected to
their source structure.

