# Pullback Construction Kernel (T3 — Stage 1 of 3)

## Where this sits

Increment 4 (T3) is staged to avoid a risky one-shot refactor and to follow
minimalism (consolidate only after duplication exists):

- **Stage 1 (THIS doc): make pullback actually construct** the fibered product,
  mirroring `construct_explicit_pushout`. After this, the workspace has TWO real
  finite universal constructions (pushout + pullback).
- **Stage 2 (follow-up):** extract the shared finite (co)limit engine and
  re-base pushout + pullback on it (consolidate the duplication Stage 1
  introduces). Not in this increment.
- **Stage 3 (follow-up):** typed provenance (candidate→accepted only via an
  explicit review morphism, enforced at the type level). Not in this increment.

## Purpose

Today `explicit_pullback_candidate` returns an `ExplicitPullbackReport` with the
matched source pairs (`cell_matches: [{left_cell_id, right_cell_id,
target_cell_id}]`) but **builds no candidate `Space`/`Complex`** — the same
report-only gap the pushout had before
`construct_explicit_pushout`. This stage constructs the real pullback object as
a reviewable candidate.

## Mathematical definition

A pullback is a LIMIT over a cospan (note: opposite shape to the pushout's
span):

```text
        left           right
   A  ---------> T <--------- B
```

`left: A → T`, `right: B → T` share target space `T` (`target_space_id`). The
pullback `P = A ×_T B = { (a, b) : left(a) = right(b) }` — the fibered product.

- One pullback cell per matched pair `(a, b)` with `left(a) = right(b)` (these
  are exactly `ExplicitPullbackReport.cell_matches`). A target cell with several
  left- and right-preimages yields every (left × right) combination — that is
  the correct fibered product, NOT an obstruction.
- A pullback incidence connects `(a1, b1) → (a2, b2)` when there is an incidence
  `a1 → a2` in A and an incidence `b1 → b2` in B that agree (a
  `relation_matches` pair), with matching `relation_type`/`orientation`.

Unlike the pushout (which QUOTIENTS via union-find), the pullback PAIRS — there
is no identification/union-find. The construction is simpler: emit one cell per
matched pair, one incidence per agreeing incidence pair.

## Inputs (mirror `PushoutInputs`)

```text
PullbackInputs {
    left: &Morphism,                  // A -> T
    right: &Morphism,                 // B -> T
    candidate_space_id: Id,
    candidate_space_name: String,
    complex_type: ComplexType,
    left_source_cells: &[Cell],       // cells of A (the SOURCE space; pairs draw attributes from here)
    right_source_cells: &[Cell],      // cells of B
    left_source_incidences: &[Incidence],   // incidences of A
    right_source_incidences: &[Incidence],  // incidences of B
}
```

Reuse `helpers::pullback_matches(&left.cell_mapping, &right.cell_mapping)` for
the matched cell pairs and `pullback_matches(&left.relation_mapping,
&right.relation_mapping)` for matched relation pairs (these already exist).

## Construction

For each matched cell pair `(a, b)` (deterministic order, sorted):

- Create one pullback `Cell` in the candidate space. Deterministic id, e.g.
  `"{candidate_space}/pullback/cell/{a}+{b}"` (escape reserved separators the
  same way the pushout kernel does, so the id is injective — reuse that helper
  if practical).
- `dimension` / `cell_type`: must be EQUAL for `a` and `b` (a coherent fiber
  cell). Unequal ⇒ blocking `incompatible_fiber` obstruction; skip the cell.
- `boundary`: pullback cell `(a,b)` includes pullback cell `(a',b')` on its
  boundary when `a' ∈ boundary(a)`, `b' ∈ boundary(b)`, and `(a',b')` is itself
  a matched pair. Remap to pullback-cell ids; sort/dedup.
- `context_ids`: union of `a` and `b` contexts, sorted/deduped (mirror pushout).
- `provenance`: drop on a paired cell and record a concrete `information_loss`
  string (two sources); never fabricate.

For each matched relation pair whose endpoints' pairs both exist as pullback
cells and whose `relation_type`/`orientation` agree: create one pullback
`Incidence` between the two pullback cells. Disagreeing `relation_type`/
`orientation` ⇒ blocking `incompatible_fiber`. Endpoints that are not both
matched pairs ⇒ drop the incidence with an `information_loss` note.

`max_dimension` = max over pullback cells. Assemble the candidate `Space`
(cell_ids / incidence_ids / complex_ids / context_ids) and `Complex`.

## Output (mirror `PushoutOutcome` / `PushoutConstruction`)

```text
PullbackConstruction { space, complex, cells, incidences, review_status }
PullbackOutcome = Constructed { construction: Box<PullbackConstruction>, report: ExplicitPullbackReport }
               | Blocked { report: ExplicitPullbackReport }
pub fn construct_explicit_pullback(inputs: PullbackInputs<'_>) -> PullbackOutcome
```

- Box the large `Constructed` variant (no `#[allow(clippy::large_enum_variant)]`).
- `review_status` is `Unreviewed` (clean) or `Candidate` — NEVER `Accepted`.
- Any BLOCKING obstruction (`incompatible_target_space`, `pullback_incomplete`,
  `incompatible_fiber`) ⇒ `Blocked`, no materialized accepted structure.
- Concrete `information_loss` entries on the report (dropped provenance, dropped
  incidence, etc.). Deterministic ordering (sort all id lists; equal input ⇒
  byte-identical JSON).

## Obstruction families

| Code | Severity | Meaning |
| --- | --- | --- |
| `incompatible_target_space` | blocking | `left.target_space_id != right.target_space_id` (exists). |
| `pullback_incomplete` | blocking | An explicit mapping has no partner with the same target (exists). |
| `incompatible_fiber` | blocking (NEW) | A matched pair `(a,b)` has conflicting `dimension`/`cell_type`, or a matched relation pair has conflicting `relation_type`/`orientation` — cannot form one coherent fiber element. |

Keep the existing two; add `incompatible_fiber`. (This is the pullback analog
of the pushout's `incompatible_identification`.)

## Store adapter (mirror increment 3)

Add `InMemorySpaceStore::construct_pullback(&self, left, right,
candidate_space_id, candidate_space_name, complex_type) -> Result<PullbackOutcome>`:
gather the two SOURCE spaces' cells/incidences (`left.source_space_id`,
`right.source_space_id`) by `space_id`, sorted by `id`; missing source space ⇒
`CoreError::MalformedField`; `&self` (no mutation); delegate to
`construct_explicit_pullback`.

## Acceptance Contract

- A clean cospan: assert the constructed pullback cells/incidences/max_dimension
  and candidate `Space` membership EXACTLY for a small hand-built example
  (including a target cell with 2 left- and 2 right-preimages → 4 fiber cells).
- `incompatible_fiber` (paired cells differing in dimension) → `Blocked`, no
  accepted structure.
- `incompatible_target_space` → `Blocked`.
- Determinism (same input → identical serialized output).
- JSON round-trip for `PullbackOutcome`.
- Store-adapter parity vs the pure function + missing-source-space error +
  store-unchanged.
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean (no new
  warnings, no `#[allow]` for the enum; `///` docs on new public items);
  `cargo test --workspace` passes. No new deps; no `unwrap`/`expect`/`panic!`
  on non-test paths.

## Template to mirror

`construct_explicit_pushout` + `PushoutInputs`/`PushoutConstruction`/
`PushoutOutcome` (crates/higher-graphen-structure/src/morphism/mod.rs) and the
increment-3 `InMemorySpaceStore::construct_pushout`
(crates/higher-graphen-structure/src/space/store.rs) are the structural
template. The pullback is simpler (pairing, no union-find).

## File Placement

- `crates/higher-graphen-structure/src/morphism/mod.rs`: `PullbackInputs`,
  `PullbackConstruction`, `PullbackOutcome`, `construct_explicit_pullback`, the
  `IncompatibleFiber` obstruction variant. Keep `explicit_pullback_candidate`
  (the report) as-is.
- `crates/higher-graphen-structure/src/morphism/helpers.rs`: small pairing
  helpers if useful (`pub(super)`); reuse `pullback_matches` and the id-escape
  helper.
- `crates/higher-graphen-structure/src/space/store.rs`: the `construct_pullback`
  adapter.
- Tests alongside the existing pushout/pullback tests.
