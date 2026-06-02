# Wire Stability Contract

HigherGraphen JSON wire forms are compatibility surfaces for downstream tools,
bindings, fixtures, and schema validation. Within a schema major version, these
forms must remain stable unless a spec explicitly declares a breaking change.

## Wire Surface

The wire surface includes:

- Core primitives: `Id`, `SourceRef`, `Provenance`, `Confidence`, `Severity`,
  and `ReviewStatus`.
- Core correspondence records: `CorrespondenceCell`, `OverlapWitness`,
  `DifferenceWitness`, `GluingAttempt`, `GluingResult`,
  `OverlapWitnessKind`, and `DifferenceSeverity`.
- Reasoning records that cross crate or tool boundaries: `Obstruction`,
  `ObstructionType`, `Counterexample`, `RequiredResolution`,
  `CompletionCandidate`, `MissingType`, `CheckResult`, and `Violation`.
- Projection records: `Projection`, `ProjectionResult`, `ProjectionOutput`,
  `ProjectionLossReport`, and related projection loss enums.
- Runtime and tool reports declared under `schemas/`.

## Stable Forms

Camel-case field names, lower-snake-case enum strings, and
`#[serde(tag = "kind")]` discriminants are part of the contract wherever those
forms are already used. New optional fields may be added when they serialize
only when present. Existing field names, required fields, and enum discriminant
strings must not change within the same schema major version.

Consumers should use stable accessors such as `serialized_value()` and `kind()`
for discriminant strings. They should not concatenate against Rust variant names
or serde internals.

## Enforcement

The `schemas/` directory declares checked JSON shapes for public reports and
fixtures. `scripts/validate-json-contracts.py` is the repository gate for
schema-bearing examples and report contracts. Changes that legitimately alter a
wire shape must update the relevant schema and fixture in the same change.

Rust unit tests should continue to assert round trips for public wire records
and that discriminant accessors match serde output.
