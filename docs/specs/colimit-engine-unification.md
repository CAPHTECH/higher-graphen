# Finite (Co)limit Scaffolding Consolidation (T3 — Stage 2 of 3)

## Purpose

Now that the workspace has TWO real finite universal constructions —
`construct_explicit_pushout` (a colimit, quotient via union-find) and
`construct_explicit_pullback` (a limit, fibered pairing) — consolidate the
**genuinely duplicated scaffolding** they share, and reconcile the small API
asymmetry between them. This is a behavior-preserving refactor; the extensive
existing tests (exact-assertion, oracle, parity, determinism, round-trip) are
the safety net.

This is minimalism-driven ("same logic in 2+ places → extract"), NOT a grand
rewrite. The matching cores are INTENTIONALLY different (limit vs colimit) and
must stay separate — that is not duplication.

## What IS duplicated (extract these)

1. **Candidate Space + Complex assembly.** Both functions, after producing a
   `Vec<Cell>` + `Vec<Incidence>`, run identical logic: compute `context_ids`
   (sorted union over cells), `max_dimension` (max over cells), build a complex
   id `"{candidate_space}/.../complex"`, populate the `Complex`
   (cell_ids/incidence_ids/max_dimension/complex_type), and populate the
   candidate `Space` (cell_ids/incidence_ids/complex_ids/context_ids). Extract a
   shared helper, e.g.:
   ```rust
   pub(super) fn assemble_candidate(
       candidate_space_id: &Id, candidate_space_name: &str,
       complex_type: ComplexType, cells: &[Cell], incidences: &[Incidence],
   ) -> (Space, Complex);
   ```
   (in `morphism/mod.rs` or `helpers.rs`). Both pushout and pullback call it.

2. **Store-adapter element gather.** `InMemorySpaceStore::construct_pushout` and
   `construct_pullback` both: validate a space exists (else
   `CoreError::MalformedField`), filter `self.cells`/`self.incidences` by
   `space_id`, and sort by `Id`. Extract a shared private helper on the store,
   e.g. `fn gather_space_elements(&self, space_id: &Id) -> Result<(Vec<Cell>,
   Vec<Incidence>)>`, and call it from both adapters (pushout gathers its two
   TARGET spaces; pullback gathers its two SOURCE spaces).

3. The id-escape helper is already shared — keep it shared.

## What is NOT duplicated (do NOT force-merge)

- The two **`*Construction` structs differ**: `PushoutConstruction` carries
  `review_status`; `PullbackConstruction` carries `cell_matches` /
  `relation_matches`. Do not collapse them into one type if it means adding
  fields that one side does not use. (A shared inner `{space, complex, cells,
  incidences}` sub-struct embedded in both is acceptable ONLY if it does not
  increase total lines or complicate the public API; otherwise leave the two
  structs and just share the assembly helper.)
- The two **`*Outcome` enums** and their distinct report types
  (`ExplicitPushoutReport` vs `ExplicitPullbackReport`) stay separate.
- The **matching cores** (union-find quotient vs fibered pairing) stay separate.

## API reconciliation

Make `construct_explicit_pullback` take `candidate_space_id`,
`candidate_space_name`, and `complex_type` **inside `PullbackInputs`** (matching
`PushoutInputs`, which already carries them), instead of as separate trailing
parameters. Update the pullback test CALL SITES accordingly. This is the only
intended signature change.

## Viability gate

Apply the minimalism gate while implementing. If, beyond the two helper
extractions (assembly + gather) and the API reconciliation, any further
"unification" (shared Construction/Outcome types, a single generic engine) would
ADD lines/abstraction without removing real duplication, STOP there and report
what was and was not consolidated, with the rationale. Extracting the two
genuinely-identical helpers + reconciling the API is the floor and is clearly
net-positive; anything beyond is optional and gated.

## Hard constraints

- **Behavior-preserving.** Every existing pushout, pullback, store-adapter,
  oracle, determinism, and round-trip test must pass with its ASSERTIONS
  UNCHANGED. Only test CALL SITES may change (for the pullback API
  reconciliation). Do not weaken or delete any assertion.
- Net lines of code should not increase (this is consolidation); ideally
  decrease. Report the net delta.
- `higher-graphen-structure` only. No new deps. No `unwrap`/`expect`/`panic!`
  on non-test paths. `///` docs on any new public item; `pub(super)` for
  internal helpers. Deterministic output unchanged (byte-identical JSON for the
  same inputs as before).

## Acceptance Contract

- All pre-existing tests green, assertions unchanged.
- `assemble_candidate` and `gather_space_elements` (or equivalently-named
  helpers) exist and are called by BOTH the pushout and pullback paths /
  adapters — verify the duplicated blocks are gone (no copy-paste remains).
- `construct_explicit_pullback` signature now mirrors `construct_explicit_pushout`
  (candidate params inside `PullbackInputs`).
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean;
  `cargo test --workspace` passes; `scripts/validate-json-contracts.py` passes.
- Report the net LOC delta and exactly which blocks were consolidated.

## File Placement

- `crates/higher-graphen-structure/src/morphism/mod.rs` and/or `helpers.rs`:
  the `assemble_candidate` helper; re-base both constructions on it; the
  `PullbackInputs` API change.
- `crates/higher-graphen-structure/src/space/store.rs`: the
  `gather_space_elements` helper; re-base both store adapters on it.
- Update pullback test call sites only (no assertion changes).
