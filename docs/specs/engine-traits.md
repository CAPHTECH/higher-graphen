# Engine Trait Specification

This document defines the engine interfaces implied by the HigherGraphen model.
The names are provisional, but the responsibilities are part of the intended
architecture.

## Space Kernel

The Space Kernel owns basic structural creation and query operations.

Required capabilities:

- Create a space.
- Retrieve a space by ID.
- Add a cell.
- Add an incidence.
- Create a complex.
- Query cells by space, type, dimension, and context.

The Space Kernel should be usable with an in-memory store during the MVP and
with durable storage later.

## Morphism Engine

The Morphism Engine defines and evaluates structure mappings.

Required capabilities:

- Define a morphism.
- Compose morphisms.
- Check whether selected invariants are preserved.
- Report preserved invariants, violated invariants, lost structure, and
  distortion.

Composition must be explicit. A chain of morphisms should not be assumed valid
unless the engine can verify compatible source and target structure.

## Consistency Engine

The Consistency Engine evaluates invariants and constraints.

Required capabilities:

- Check invariants for a space and changed cell set.
- Check constraints for a space and context set.
- Return obstructions for violations.
- Explain an obstruction directly or through a projection.

Invariant and constraint checks should return structured obstructions rather
than untyped error strings.

## Completion Engine

The Completion Engine detects missing structure and manages reviewable
completion proposals.

Required capabilities:

- Detect missing structure from a space, rule set, and context set.
- Produce completion candidates.
- Accept a completion candidate and return created structure.
- Reject a completion candidate with reviewer and reason.

Completion must be reviewable. The engine must not silently promote inferred
structure to accepted fact.

## Projection Engine

The Projection Engine turns higher structure into usable output.

Required capabilities:

- Define a projection.
- Project selected cells and obstructions.
- Return projection output, audience, purpose, information loss, and source
  identifiers.

Projection output must remain traceable to the source structure used to produce
it.

## Runtime Layer

The Runtime Layer should provide a stable API for humans, tools, and AI agents.

Runtime responsibilities:

- Structural query execution.
- Transformation execution.
- Projection execution.
- Review workflow execution.
- Adapter coordination for product packages.

The runtime should depend on the core engines. The engines should not depend on
UI or product-specific presentation code.

