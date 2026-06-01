# Typed Provenance (T3 — Stage 3 of 3)

## Purpose

Make HigherGraphen's core safety rule — "a candidate must not become accepted
fact without an explicit review" — a **compile-time** guarantee for code that
opts in, instead of a runtime convention. Today `ReviewStatus`
(`Candidate`/`Unreviewed`/`Reviewed`/`Rejected`/`Accepted`) is a runtime enum
field; nothing stops code from setting `Accepted` directly. This stage adds a
typestate primitive whose ONLY path to an `Accepted` value is through an
explicit review morphism.

## Scope (bounded, additive — do NOT sprawl)

`ReviewStatus` is used pervasively (core extensions, evidence, structure,
runtime workflows). Do **NOT** convert those to typestate. Keep the runtime
enum. This stage is:

- A new, additive typestate primitive in `higher-graphen-core` (a new module,
  e.g. `core::provenance` / `core::reviewed`).
- A `compile_fail` doctest proving the guarantee (built-in; NO new dependency
  such as trybuild).
- Unit tests for the in-memory transitions and the serialization projection.
- **One real adopter** so the primitive is not dead code (see below).

Explicit non-goals: no change to the `ReviewStatus` enum or its existing
usages; no retrofit of runtime workflows / report schemas / CLI to the
typestate (that is future work — call it out, do not do it); no new crate deps.

## The primitive

```rust
// Sealed so only this crate defines review states.
pub trait ReviewState: sealed::Sealed { fn status() -> ReviewStatus; }
pub enum Candidate {}   // zero-size markers
pub enum Accepted {}
impl ReviewState for Candidate { fn status() -> ReviewStatus { ReviewStatus::Candidate } }
impl ReviewState for Accepted  { fn status() -> ReviewStatus { ReviewStatus::Accepted } }

/// An explicit review act: the only thing that authorizes promotion.
pub struct ReviewMorphism { reviewer: Id, decision: ReviewDecision, evidence: Vec<Id>, rationale: Option<String>, /* provenance */ }

/// A value carrying its review state in the type. The `Accepted` state is
/// unconstructable except via `accept`.
pub struct Reviewed<T, S: ReviewState> { value: T, provenance: Provenance, review: Option<ReviewMorphism>, _state: PhantomData<S> }

impl<T> Reviewed<T, Candidate> {
    pub fn candidate(value: T, provenance: Provenance) -> Self; // ONLY public constructor
    pub fn accept(self, review: ReviewMorphism) -> Reviewed<T, Accepted>; // ONLY path to Accepted
    pub fn reject(self, review: ReviewMorphism) -> RejectedReview<T>;     // optional
}
impl<T, S: ReviewState> Reviewed<T, S> {
    pub fn value(&self) -> &T;
    pub fn provenance(&self) -> &Provenance;
    pub fn review_status(&self) -> ReviewStatus { S::status() } // bridge to runtime enum
    pub fn review(&self) -> Option<&ReviewMorphism>;
}
```

Key guarantees (must hold):
- `Accepted` is a sealed marker; `Reviewed<T, Accepted>` has NO public
  constructor — the only way to obtain one is `Reviewed::<T, Candidate>::accept(review)`.
- `accept` CONSUMES the candidate (by value) and REQUIRES a `ReviewMorphism`.
- The private `value` / `_state` fields and the sealed `Sealed` trait prevent
  external construction or adding new states.

## Serialization soundness (critical subtlety)

`serde` deserialization can otherwise be a backdoor (constructing `Accepted`
without review). Therefore:

- Serialization of `Reviewed<T, S>` projects to a plain shape: the `T` payload
  plus `reviewStatus: S::status()` plus the optional review morphism. (Implement
  `Serialize` for both states.)
- Do **NOT** derive/allow direct `Deserialize` into `Reviewed<T, Accepted>`.
  Deserialization MUST land in `Reviewed<T, Candidate>` (an untrusted,
  re-reviewable value) regardless of any `reviewStatus` in the input — a
  persisted "accepted" value re-enters as a candidate that must be re-accepted
  in-process. Document this explicitly; it is what keeps the type guarantee
  sound across persistence boundaries.

## The `compile_fail` proof (no new dep)

A `///` doctest on the primitive, marked ```` ```compile_fail ````, that
demonstrates you cannot reach `Accepted` without `accept` — e.g. attempting to
construct `Reviewed::<X, Accepted>` directly, or calling a non-existent
`accept`-free promotion, fails to compile. Also a normal doctest/unit test
showing the LEGAL path (`candidate(..).accept(review)`) compiles and yields
`review_status() == Accepted`.

## One real adopter (so it is not dead)

Adopt the primitive at exactly one genuine candidate→accepted boundary, at the
library level, WITHOUT touching report schemas / workflows / CLI. Preferred: a
small core helper that accepts a `CompletionCandidate` (or another simple
existing core candidate type) via the typestate —
`Reviewed<CompletionCandidate, Candidate>` → `accept` →
`Reviewed<CompletionCandidate, Accepted>` — demonstrated by a passing unit test.
If no clean real-core-type adoption exists without schema/workflow churn, the
viability gate applies: deliver the primitive + proofs + a documented adoption
recipe, use a representative real core type in the test, and REPORT that
workflow/CLI retrofits are deferred. Do not force a sprawling retrofit.

## Viability gate

If the serde projection or the adoption fights Rust ergonomics so hard that it
would require touching the pervasive `ReviewStatus` usages or many files, STOP:
keep the primitive + compile_fail proof + unit tests + recipe, adopt nothing
risky, and report. The compile-time guarantee mechanism is the deliverable;
broad adoption is explicitly future work.

## Acceptance Contract

- The primitive exists in core with the guarantees above; `Reviewed<T, Accepted>`
  is unconstructable except via `accept`.
- A `compile_fail` doctest proves the no-bypass guarantee; a passing test shows
  the legal `candidate → accept` path and `review_status()` bridging.
- Serialization projects to `{ payload, reviewStatus, review? }`; deserialization
  lands in `Candidate` (a test asserts a serialized "accepted" value re-enters
  as a candidate).
- At least one real-core-type usage exists with a passing test (or, per the
  gate, a documented recipe + representative-type test).
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean (no new
  warnings; `///` docs on all new public items); `cargo test --workspace`
  passes (incl. the doctests); `scripts/validate-json-contracts.py` passes
  (no schema changes expected). No new deps; no `unwrap`/`expect`/`panic!` on
  non-test paths; no `unsafe`.

## File Placement

- `crates/higher-graphen-core/src/`: new module (e.g. `provenance.rs` /
  `reviewed.rs`) with the typestate primitive, `ReviewMorphism`,
  `ReviewDecision`, sealed `ReviewState`; re-export from `lib.rs`.
- The one adopter + tests in the most natural core location (alongside the
  adopted type).
- Update `docs/specs/typed-provenance.md` only if the adopter choice differs;
  no other doc/schema changes expected.
