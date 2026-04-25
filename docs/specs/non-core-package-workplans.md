# Non-Core Package Workplans

These notes define the smallest implementation surface for the MVP packages
that come after `higher-graphen-core` verification. They derive from
[`package-boundaries.md`](package-boundaries.md),
[`rust-core-model.md`](rust-core-model.md),
[`engine-traits.md`](engine-traits.md), and
[`core-contracts.md`](core-contracts.md).

Before starting any package below, `higher-graphen-core` should already provide
validated `Id`, `SourceRef`, `Provenance`, `Confidence`, `Severity`,
`ReviewStatus`, structured errors, and stable serde behavior. Each package
should keep product, tool, binding, app, UI, provider SDK, and AI-runtime
concepts outside the reusable model crate.

## `higher-graphen-space`

- Purpose: Own structural storage primitives: `Space`, `Cell`, `Incidence`,
  `Complex`, and the MVP Space Kernel for creation and query.
- Inputs: space creation data, cell definitions, incidence definitions, complex
  definitions, and query selectors by space, type, dimension, and context ID.
- Outputs: created or retrieved spaces, cells, incidences, complexes, and query
  result sets with structured errors for invalid input.
- Core dependencies: `Id`, `Provenance`, `SourceRef`, shared result/error
  types, and serde contracts from `higher-graphen-core`.
- Forbidden dependencies: `higher-graphen-morphism`,
  `higher-graphen-invariant`, `higher-graphen-obstruction`,
  `higher-graphen-completion`, `higher-graphen-projection`,
  `higher-graphen-interpretation`, runtime, tools, bindings, apps, and product
  packages.
- Smallest verification command: `cargo test -p higher-graphen-space --lib`.
- Split criteria: Split only when durable storage backends, indexing/query
  planning, or complex-specific algorithms make the crate hard to test as a
  kernel. Keep the MVP in-memory store inside this package.

## `higher-graphen-morphism`

- Purpose: Own `Morphism`, structure mappings, explicit composition,
  preservation checks, lost structure, and distortion reports.
- Inputs: source and target structure references, cell mappings, relation
  mappings, selected invariant IDs, and candidate composition chains.
- Outputs: morphism definitions, verified composition results, preservation
  summaries, lost-structure reports, and distortion records.
- Core dependencies: `Id`, `Provenance`, shared result/error types, and serde
  contracts from `higher-graphen-core`; structural IDs and lookup traits from
  `higher-graphen-space` when checks need concrete cells or relations.
- Forbidden dependencies: `higher-graphen-obstruction`,
  `higher-graphen-completion`, `higher-graphen-projection`,
  `higher-graphen-interpretation`, runtime, tools, bindings, apps, and product
  packages. Do not assume a morphism chain is valid without explicit
  compatibility checks.
- Smallest verification command: `cargo test -p higher-graphen-morphism --lib`.
- Split criteria: Split when mapping storage, composition search, or
  preservation evaluation become independently testable subsystems. Keep basic
  model types and deterministic composition checks together for MVP.

## `higher-graphen-invariant`

- Purpose: Own `Invariant`, `Constraint`, invariant checks, constraint checks,
  and structured check results.
- Inputs: invariant definitions, constraint definitions, target space IDs,
  changed cell sets, context ID sets, and optional morphism preservation data.
- Outputs: check results that identify satisfied, violated, or unsupported
  invariants and constraints, including enough location data for obstruction
  construction.
- Core dependencies: `Id`, `Provenance`, `Severity`, shared result/error types,
  and serde contracts from `higher-graphen-core`; structure access from
  `higher-graphen-space`; morphism summaries from `higher-graphen-morphism`
  when preservation checks are part of the rule.
- Forbidden dependencies: `higher-graphen-obstruction`,
  `higher-graphen-completion`, `higher-graphen-projection`,
  `higher-graphen-interpretation`, runtime, tools, bindings, apps, and product
  packages. If a public API must return `Obstruction`, place that integration in
  the obstruction package to avoid an invariant-obstruction cycle.
- Smallest verification command: `cargo test -p higher-graphen-invariant --lib`.
- Split criteria: Split when invariant language/parsing, check execution, or
  incremental changed-cell evaluation grows beyond simple rule evaluation.
  Keep constraint result types with the invariant crate until there is a second
  consumer.

## `higher-graphen-obstruction`

- Purpose: Own `Obstruction`, counterexamples, obstruction engines, and direct
  human-readable explanations for structured failure.
- Inputs: invariant or constraint check results, morphism preservation failures,
  lost-structure reports, affected cells, affected context IDs, and severity.
- Outputs: obstructions with location cells, location contexts, related
  morphisms, explanations, optional counterexamples, severity, and required
  resolution hints.
- Core dependencies: `Id`, `Provenance`, `Severity`, `SourceRef`, shared
  result/error types, and serde contracts from `higher-graphen-core`; structure
  access from `higher-graphen-space`; violation summaries from
  `higher-graphen-invariant`; morphism summaries from
  `higher-graphen-morphism`.
- Forbidden dependencies: `higher-graphen-completion`,
  `higher-graphen-projection`, `higher-graphen-interpretation`, runtime, tools,
  bindings, apps, and product packages. Projection-specific explanation formats
  belong in the projection package.
- Smallest verification command: `cargo test -p higher-graphen-obstruction --lib`.
- Split criteria: Split when counterexample generation, explanation templates,
  or obstruction aggregation become separately reusable. Keep the obstruction
  record and simple constructors together for MVP.

## `higher-graphen-completion`

- Purpose: Own reviewable completion proposals: `CompletionCandidate`,
  completion rules, candidate detection, accept workflow, and reject workflow.
- Inputs: a space, missing-structure rules, context ID sets, inferred-from
  source identifiers, and reviewer decisions.
- Outputs: completion candidates, accepted created structure references,
  rejection records, rationale, confidence, and review status.
- Core dependencies: `Id`, `Confidence`, `ReviewStatus`, `Provenance`, shared
  result/error types, and serde contracts from `higher-graphen-core`; structure
  creation/query access from `higher-graphen-space`; optional violation or
  obstruction summaries from invariant and obstruction packages.
- Forbidden dependencies: `higher-graphen-projection`,
  `higher-graphen-interpretation`, runtime, tools, bindings, apps, and product
  packages. The engine must not silently promote inferred structure to accepted
  fact; acceptance requires an explicit review action.
- Smallest verification command: `cargo test -p higher-graphen-completion --lib`.
- Split criteria: Split when rule evaluation, candidate storage, or review
  workflow require independent adapters. Keep candidate data and in-memory
  accept/reject behavior together for MVP.

## `higher-graphen-projection`

- Purpose: Own `Projection`, selectors, projection results, information-loss
  declarations, and non-UI renderers.
- Inputs: projection definitions, source space IDs, selectors for cells,
  obstructions, context IDs, severity, audience, purpose, output schema, and
  optional renderer choice.
- Outputs: projection output with audience, purpose, declared information loss,
  source identifiers, and structured renderer result data.
- Core dependencies: `Id`, `Severity`, `Provenance`, shared result/error types,
  and serde contracts from `higher-graphen-core`; selected structure from
  `higher-graphen-space`; obstruction records from
  `higher-graphen-obstruction`.
- Forbidden dependencies: `higher-graphen-interpretation`, runtime, UI
  frameworks, tools, bindings, apps, product packages, and provider SDKs. Keep
  UI presentation and API transport outside projection.
- Smallest verification command: `cargo test -p higher-graphen-projection --lib`.
- Split criteria: Split when selector evaluation, output schema validation, or
  renderer implementations become independently versioned. Keep traceable
  projection output in this package.

## `higher-graphen-interpretation`

- Purpose: Own reusable interpretation support: type mappings, invariant
  templates, projection templates, and lift adapters from domain structures into
  HigherGraphen structures.
- Inputs: domain type mappings, source structures or references, invariant
  templates, projection templates, lift requests, and adapter configuration.
- Outputs: reusable interpretation definitions, generated space or morphism
  construction requests, invariant templates, projection templates, and adapter
  diagnostics.
- Core dependencies: `Id`, `SourceRef`, `Provenance`, `Confidence`,
  `ReviewStatus`, shared result/error types, and serde contracts from
  `higher-graphen-core`; construction targets from `higher-graphen-space`;
  mapping support from `higher-graphen-morphism`; templates from invariant and
  projection packages when needed.
- Forbidden dependencies: runtime, tools, bindings, apps, provider SDKs, UI
  frameworks, and product-package code. Product packages may depend on
  interpretation templates, but interpretation must not depend on a specific
  product workflow.
- Smallest verification command:
  `cargo test -p higher-graphen-interpretation --lib`.
- Split criteria: Split when a domain family such as architecture, contracts,
  or projects accumulates its own vocabulary, templates, and adapters. Move
  execution orchestration to runtime rather than expanding interpretation into
  an application layer.
