# Gluing → Pushout Wiring

## Purpose

Retire the fabricated `complex:glued:{id}` and give gluing a path that returns a
**genuinely constructed** merged complex. Today `reasoning::gluing::attempt_gluing`
classifies an abstract `CorrespondenceCell` (whose participants are abstract
`ParticipantRef`s, not concrete complexes) and, on `Success`, fabricates
`merged_complex: Id::new("complex:glued:{base_id}")` — an id that points to
nothing built. This increment makes the abstract path honest (it materializes
nothing, so it claims nothing) and adds a structural path that builds a real
pushout via the increment-3 store adapter.

## Scope

In scope:

- **core**: make `GluingResult::Success.merged_complex` an `Option<Id>`.
- **reasoning::gluing**: stop fabricating in `attempt_gluing` (Success →
  `merged_complex: None`); add `attempt_structural_gluing` over a cospan + store
  that returns a real construction.
- **projection / CLI**: keep compiling with the `Option` (they already match
  `Success { .. }` and do not read the field).
- **schemas**: make `mergedComplex` optional in `gluing-attempt.schema.json`;
  add/adjust a fixture if one asserts the field.
- tests for both paths.

Explicit non-goals:

- No change to the abstract gluing REVIEW logic (invariant/difference/evidence
  classification stays as-is).
- No new CLI subcommand for structural gluing (its input would need a store +
  cospan format — a later increment). Library API only.
- No (co)limit unification (that is increment 4 / T3).
- The `completion_candidate` id (Candidate) and `obstruction` id (Failure)
  are review-artifact ids, not claimed-built structures — leave them.

## Core change

`crates/higher-graphen-core/src/correspondence.rs`, `GluingResult::Success`:

```rust
Success {
    /// Merged complex identifier when a concrete structure was materialized.
    /// `None` for abstract classification that builds nothing.
    #[serde(rename = "mergedComplex", skip_serializing_if = "Option::is_none")]
    merged_complex: Option<Id>,
    #[serde(rename = "preservationReport")]
    preservation_report: PreservationReport,
},
```

Backward compatibility is not required. Update all construction/match sites.

## reasoning::gluing changes

`reasoning` already depends on `higher-graphen-structure`, so it can call the
store adapter and the pushout construction.

1. **`attempt_gluing` (abstract, unchanged behavior except honesty):** the
   `Success` arm now sets `merged_complex: None`. The `complex:glued:{base_id}`
   construction is deleted. Nothing else about the classification changes.

2. **New `attempt_structural_gluing`:**

```rust
/// Outcome of gluing two concrete structures over a cospan.
pub struct StructuralGluing {
    /// Gluing classification in the gluing vocabulary.
    pub result: GluingResult,
    /// The materialized merged structure, present iff the pushout constructed.
    pub construction: Option<PushoutConstruction>,
}

pub fn attempt_structural_gluing(
    left: &Morphism,
    right: &Morphism,
    store: &InMemorySpaceStore,
    candidate_space_id: Id,
    candidate_space_name: impl Into<String>,
    complex_type: ComplexType,
) -> Result<StructuralGluing>;
```

Behavior: call `store.construct_pushout(left, right, candidate_space_id, name,
complex_type)` and map its `PushoutOutcome`:

| `PushoutOutcome` | `StructuralGluing.result` | `construction` |
| --- | --- | --- |
| `Constructed { construction, .. }` with `review_status == Unreviewed` (clean) | `Success { merged_complex: Some(construction.complex.id.clone()), preservation_report }` | `Some(construction)` |
| `Constructed { construction, .. }` with `review_status == Candidate` (ambiguous) | `Candidate { completion_candidate: Id "completion:structural-gluing:{candidate_space}", required_review: ReviewRequirement::new(true).with_decision_reason("ambiguous identification requires review") }` | `Some(construction)` |
| `Blocked { report }` | `Failure { obstruction: Id "obstruction:pushout:{candidate_space}:{first blocking obstruction_type}" }` | `None` |

- Derive the `Success` `preservation_report` from the construction (which
  structures/invariants were preserved); a minimal honest summary is acceptable
  (e.g. preserved_structures = the merged cell ids), but it must not claim
  unpreserved invariants.
- **Honesty invariant (must hold & be tested):** when `result` is `Success`,
  `merged_complex` is `Some(id)` AND `construction` is `Some` AND
  `construction.complex.id == id`. The id always points to a real returned
  complex — never a fabrication.
- Deterministic; no panic/unwrap on non-test paths; the result is a reviewable
  candidate (it does not insert anything into the store or accept anything).

## Consumer updates

- `crates/higher-graphen-projection/src/correspondence.rs`: matches
  `GluingResult::Success { .. }` already — confirm it still compiles; no logic
  change expected.
- `tools/highergraphen-cli/src/command_run.rs`: serializes the
  `GluingAttempt`; confirm it compiles. No new command this increment.
- `schemas/highergraphen/gluing-attempt.schema.json`: remove `mergedComplex`
  from the `success` variant's `required` list (it is now optional); keep its
  type when present. Update any fixture that asserts a present `mergedComplex`
  on an abstract attempt to reflect it being absent.

## Acceptance Contract

- **Abstract honesty:** `attempt_gluing` on a successful correspondence yields
  `Success { merged_complex: None, .. }`; assert NO id contains
  `"complex:glued"` anywhere in the serialized attempt.
- **Structural success:** build a store with apex space A, target spaces B and
  C (+ cells/incidences) and the two leg morphisms; `attempt_structural_gluing`
  returns `Success` with `merged_complex == Some(construction.complex.id)` and
  `construction` present; assert the honesty invariant.
- **Structural blocked:** incompatible source spaces (or a real gluing-axiom
  conflict) → `Failure { obstruction }`, `construction == None`.
- **Structural ambiguous:** a cospan that collapses same-side cells →
  `Candidate`, `construction` present.
- A JSON round-trip test for the `Option<Id>` `GluingResult` (Some and None).
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean (no new
  warnings; `///` docs on new public items); `cargo test --workspace` passes;
  the JSON-schema contract validation (existing repo script/tests) still
  passes for `gluing-attempt.schema.json`. No new deps; no
  `unwrap`/`expect`/`panic!` on non-test paths.

## File Placement

- `crates/higher-graphen-core/src/correspondence.rs` — `Option<Id>` change.
- `crates/higher-graphen-reasoning/src/gluing.rs` — honesty fix +
  `attempt_structural_gluing` + `StructuralGluing` (import
  `InMemorySpaceStore`, `Morphism`, `ComplexType`, `PushoutConstruction`,
  `PushoutOutcome` from `higher_graphen_structure`).
- `crates/higher-graphen-projection/src/correspondence.rs`,
  `tools/highergraphen-cli/src/command_run.rs` — compile-compat only.
- `schemas/highergraphen/gluing-attempt.schema.json` (+ any affected fixture).
