# Pushout Store Adapter

## Purpose

Make the real finite pushout construction reachable from an
`InMemorySpaceStore` without the caller hand-assembling cell and incidence
slices. Today `construct_explicit_pushout(PushoutInputs { .. })` is a pure
function that requires explicit `left_cells`, `right_cells`, `left_incidences`,
and `right_incidences`. That is ideal for unit tests but awkward for runtime /
CLI callers that already hold a populated store. This adapter gathers those
slices from the store's two target spaces and delegates.

## Scope

In scope (`higher-graphen-structure` only, additive):

- One read-only method on `InMemorySpaceStore` that assembles `PushoutInputs`
  from the two morphisms' target spaces and calls `construct_explicit_pushout`.
- Tests: a store-driven clean construction, a missing-space error, a blocked
  case passthrough, and determinism independent of insertion order.

Explicit non-goals:

- No change to `construct_explicit_pushout`, `PushoutInputs`, `PushoutOutcome`,
  or any existing type/behavior.
- The adapter MUST NOT mutate the store (no inserting the constructed
  space/complex/cells). It returns the reviewable `PushoutOutcome` candidate;
  whether to persist it is the caller's explicit decision. Use `&self`.
- No CLI/runtime workflow wiring (a later step). No new crate dependencies.

## API

```rust
impl InMemorySpaceStore {
    /// Constructs a finite pushout candidate over a cospan whose legs are the
    /// two morphisms, gathering target-space cells and incidences from this
    /// store. Read-only: the constructed candidate is returned, never inserted.
    pub fn construct_pushout(
        &self,
        left: &Morphism,
        right: &Morphism,
        candidate_space_id: Id,
        candidate_space_name: impl Into<String>,
        complex_type: ComplexType,
    ) -> Result<PushoutOutcome>;
}
```

(`Morphism`, `ComplexType`, `PushoutOutcome`, `construct_explicit_pushout`, and
`PushoutInputs` are all in this crate — `crate::morphism` / `crate::space`.)

## Behavior

1. Resolve the two target spaces: `left.target_space_id` (space B) and
   `right.target_space_id` (space C). If EITHER is absent from the store, return
   `CoreError::MalformedField` (field `left`/`right`, reason naming the missing
   space id). Do not silently treat a missing space as empty.
2. Gather elements by ownership, deterministically:
   - `left_cells` = every `Cell` whose `space_id == left.target_space_id`,
     sorted by `id`.
   - `right_cells` = every `Cell` whose `space_id == right.target_space_id`,
     sorted by `id`.
   - `left_incidences` / `right_incidences` = likewise for `Incidence`, sorted
     by `id`.
   Filtering by the element's own `space_id` (not by `Space.cell_ids`) keeps the
   gather robust to membership-list drift; sorting makes the result independent
   of store insertion order.
3. Build `PushoutInputs` from the morphisms, the supplied
   `candidate_space_id` / name / `complex_type`, and the gathered slices, and
   return `construct_explicit_pushout(inputs)`.

The adapter adds no new obstruction semantics. All identification, gluing-axiom
obstructions, blocking/blocked behavior, `quotient_losses`, and
`review_status` come from `construct_explicit_pushout` unchanged. A
`PushoutOutcome::Blocked` (e.g. incompatible source spaces) is returned as-is.

## Determinism / Safety

- Gathering is sorted by `Id`, so equal store contents yield identical inputs
  and therefore identical `PushoutOutcome` JSON regardless of insertion order.
- `&self` only; the store is never mutated. The constructed structure stays a
  reviewable candidate (its `review_status` is never `Accepted`).

## Minimal Acceptance Contract

- A store-driven clean case: insert an apex space A, target spaces B and C with
  their cells/incidences, and the two leg morphisms (A→B, A→C); call
  `construct_pushout`; assert `PushoutOutcome::Constructed` with the SAME merged
  cells / incidences / `max_dimension` / populated candidate `Space` membership
  that the equivalent pure-function call produces (parity with
  `construct_explicit_pushout`).
- A missing-target-space case → `Err(CoreError::MalformedField ..)`.
- A blocked case (e.g. `left.source_space_id != right.source_space_id`) →
  `Ok(PushoutOutcome::Blocked { .. })` passthrough (no panic, no error).
- A determinism test: insert the same cells/incidences in two different orders
  into two stores; assert byte-identical serialized `PushoutOutcome`.
- The store is unchanged after the call (cell/incidence/space/complex counts
  equal before and after).
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean (no new
  warnings; `///` docs on the new public method); `cargo test --workspace`
  passes. No new deps; no `unwrap`/`expect`/`panic!` on non-test paths.

## File Placement

- `crates/higher-graphen-structure/src/space/store.rs`: the new
  `construct_pushout` method (and a small private gather helper if useful).
  Import the pushout API from `crate::morphism`.
- Tests in the existing store test module
  (`crates/higher-graphen-structure/src/space/tests.rs` or `store.rs`'s
  `#[cfg(test)]`, matching the crate's convention).
- Optionally extend the `docs/specs/math-kernel-api-examples.md` Diagram
  Construction example with a one-line store-adapter call.
