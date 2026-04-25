# Rust Core Model Specification

This document specifies the intended Rust data model at the contract level. It
does not prescribe final file names or exact module boundaries.

## Shared Types

| Type | Purpose |
| --- | --- |
| `Id` | Stable identifier for structures. |
| `Confidence` | Floating-point confidence score for extracted or inferred structures. |
| `Dimension` | Cell dimension, usually represented as an unsigned integer. |
| `SourceKind` | Source category such as document, log, API, human, AI, code, or external source. |
| `SourceRef` | Reference to source material, including optional URI, title, and capture time. |
| `ReviewStatus` | Review state such as unreviewed, reviewed, rejected, or accepted. |
| `Provenance` | Source, extraction, confidence, reviewer, and review status. |
| `Severity` | Low, medium, high, or critical impact classification. |

## Space

A `Space` represents a target world or structural universe.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Space identifier. |
| `name` | Human-readable name. |
| `description` | Optional description. |
| `cell_ids` | Cells owned by the space. |
| `complex_ids` | Complexes in the space. |
| `context_ids` | Contexts in the space. |
| `morphism_ids` | Morphisms related to the space. |
| `invariant_ids` | Invariants scoped to the space. |
| `constraint_ids` | Constraints scoped to the space. |
| `metadata` | Product-specific metadata. |

## Cell

A `Cell` is a typed structural element.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Cell identifier. |
| `space_id` | Owning space. |
| `dimension` | Cell dimension. |
| `cell_type` | Domain or abstract type. |
| `label` | Optional display label. |
| `attributes` | Structured attributes. |
| `boundary` | Lower-dimensional cells on the boundary. |
| `coboundary` | Higher-dimensional cells that reference this cell. |
| `context_ids` | Contexts in which the cell participates. |
| `provenance` | Source and review metadata. |

## Incidence

An `Incidence` records a relation between cells.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Incidence identifier. |
| `space_id` | Owning space. |
| `from_cell_id` | Source cell. |
| `to_cell_id` | Target cell. |
| `relation_type` | Type of relation. |
| `orientation` | Directed or undirected. |
| `weight` | Optional relation weight. |
| `provenance` | Source and review metadata. |

## Complex

A `Complex` groups cells and incidences into a structural form.

Supported complex types:

- Typed graph
- Hypergraph
- Simplicial complex
- Cell complex
- Custom type

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Complex identifier. |
| `space_id` | Owning space. |
| `name` | Complex name. |
| `cell_ids` | Cells included in the complex. |
| `incidence_ids` | Incidences included in the complex. |
| `max_dimension` | Highest cell dimension in the complex. |
| `complex_type` | Structural kind. |
| `metadata` | Product-specific metadata. |

## Context and Section

A `Context` defines a local scope of validity, vocabulary, and rules.

Required `Context` fields:

| Field | Meaning |
| --- | --- |
| `id` | Context identifier. |
| `space_id` | Owning space. |
| `name` | Context name. |
| `description` | Optional description. |
| `parent_context_id` | Optional parent context. |
| `covered_by` | Child contexts or covering contexts. |
| `valid_cell_types` | Cell types valid in this context. |
| `valid_morphism_types` | Morphism types valid in this context. |
| `local_rule_ids` | Local rule identifiers. |
| `local_vocabulary` | Context-specific vocabulary. |

A `Section` records an assignment over a context.

Required `Section` fields:

| Field | Meaning |
| --- | --- |
| `id` | Section identifier. |
| `space_id` | Owning space. |
| `context_id` | Context for the assignment. |
| `assignment` | Structured assignment payload. |
| `valid_from` | Optional validity start. |
| `valid_to` | Optional validity end. |
| `provenance` | Source and review metadata. |

## Morphism

A `Morphism` maps one structure to another.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Morphism identifier. |
| `source_space_id` | Source space. |
| `target_space_id` | Target space. |
| `name` | Morphism name. |
| `morphism_type` | Abstraction, refinement, translation, projection, lift, migration, interpretation, or custom. |
| `cell_mapping` | Source cell to target cell mapping. |
| `relation_mapping` | Source relation to target relation mapping. |
| `preserved_invariant_ids` | Invariants known to be preserved. |
| `lost_structure` | Source elements lost by the mapping. |
| `distortion` | Source-target distortions introduced by the mapping. |
| `composable_with` | Compatible morphisms. |
| `provenance` | Source and review metadata. |

## Invariant and Constraint

An `Invariant` defines a property that must be preserved.

A `Constraint` defines a condition that can be checked and reported as a
violation.

Both must include scope information, severity, and provenance.

## Obstruction

An `Obstruction` records structured failure.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Obstruction identifier. |
| `space_id` | Owning space. |
| `obstruction_type` | Type of failure. |
| `location_cell_ids` | Cells where the obstruction occurs. |
| `location_context_ids` | Contexts where the obstruction occurs. |
| `related_morphism_ids` | Related morphisms. |
| `explanation` | Human-readable explanation. |
| `counterexample` | Optional counterexample. |
| `severity` | Impact classification. |
| `required_resolution` | Optional resolution requirement. |
| `provenance` | Source and review metadata. |

## Completion Candidate

A `CompletionCandidate` represents a reviewable proposal for missing structure.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Candidate identifier. |
| `space_id` | Owning space. |
| `missing_type` | Missing structure kind. |
| `suggested_structure` | Proposed structure payload. |
| `inferred_from` | Source structure identifiers. |
| `rationale` | Explanation for the proposal. |
| `confidence` | Confidence score. |
| `review_status` | Review state. |

## Projection

A `Projection` defines a usable view over higher structure.

Required fields:

| Field | Meaning |
| --- | --- |
| `id` | Projection identifier. |
| `source_space_id` | Source space. |
| `name` | Projection name. |
| `audience` | Human, AI, developer, architect, executive, operator, or external system. |
| `purpose` | Explanation, report, dashboard, action plan, review, query result, or API response. |
| `input_selector` | Selector for cells, obstructions, contexts, and severity. |
| `output_schema` | Schema of the projected output. |
| `information_loss` | Declared information loss. |
| `renderer` | Optional renderer. |

