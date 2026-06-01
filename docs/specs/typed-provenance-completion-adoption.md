# Typed-Provenance Adoption: Completion Review

## Purpose

Make the typed-provenance typestate (`core::typed_provenance::Reviewed<T, S>`,
T3 stage 3) **load-bearing in a real production path** instead of only
demonstrated in a test. The natural site is completion review: today
`reasoning::completion::accept_completion` builds an `AcceptedCompletion` (the
materialized accepted structure that flows into the completion-review report)
by a plain struct literal — nothing stops code from fabricating an
`AcceptedCompletion` without an actual review. After this change, an
`AcceptedCompletion` can be produced ONLY by going through
`Reviewed::<CompletionCandidate, Candidate>::accept(ReviewMorphism)`, so the
candidate→accepted boundary is compile-time-enforced on the real path.

## Scope

In scope (`higher-graphen-reasoning` primarily; `reasoning` depends on `core`,
so it can use `core::typed_provenance`):

- Route `accept_completion` (and `CompletionCandidate::accept`) through the
  typestate, and make `AcceptedCompletion` constructable only via that path.
- Keep the completion-review REPORT OUTPUT byte-identical.

Explicit non-goals:

- No change to the report schema, the `completion-review.report.example.json`
  fixture, the runtime workflow's behavior, or the CLI. Output must be
  identical.
- No change to `RejectedCompletion` (out of scope; leave as-is).
- No change to `core::typed_provenance` itself (it is done). No new deps.

## Design

In `crates/higher-graphen-reasoning/src/completion/mod.rs`:

1. **Gate `AcceptedCompletion` construction.** Add `#[non_exhaustive]` to
   `AcceptedCompletion` so external crates cannot build it via struct literal
   (fields stay `pub` — still readable and serde-(de)serializable; serialization
   output is unchanged). The ONLY construction site in this crate becomes the
   type-gated path below.

2. **Route acceptance through the typestate.** Rewrite `accept_completion` so
   it:
   - validates as today (reject if `candidate.review_status.is_rejected()`);
   - builds `Reviewed::<CompletionCandidate, Candidate>::candidate(candidate.clone(), provenance)`
     where `provenance` is a candidate-level provenance (the candidate carries
     `confidence`; use a `Provenance` consistent with the existing semantics —
     a candidate/AI source at the candidate's confidence);
   - calls `.accept(ReviewMorphism { reviewer_id, review_note: Some(reason) })`
     to obtain `Reviewed<CompletionCandidate, Accepted>`;
   - builds the `AcceptedCompletion` via a new crate-internal constructor
     `fn from_accepted_review(reviewed: &Reviewed<CompletionCandidate, Accepted>) -> Result<AcceptedCompletion>`
     (or equivalent) that reads the candidate fields via `reviewed.value()` and
     the reviewer/reason via `reviewed.review()`, and sets
     `review_status: ReviewStatus::Accepted`.
   - The reason must remain non-empty-validated (keep `required_text("reason", ..)`).

   Result: the only way any code obtains an `AcceptedCompletion` is by first
   producing a `Reviewed<_, Accepted>` — which itself can only come from
   `.accept(ReviewMorphism)`. The boundary is compile-time-enforced.

3. **Output identical.** `AcceptedCompletion`'s fields, names, serde attributes,
   and values are unchanged, so the serialized `CompletionReviewRecord` /
   completion-review report is byte-identical. `review_completion` and
   `CompletionReviewRecord` keep their shapes.

## Viability gate

If `#[non_exhaustive]` conflicts with the existing
`#[serde(deny_unknown_fields)]` or breaks serde round-trip, OR if reaching a
load-bearing gate forces touching many field-readers, FALL BACK: route the
production `accept_completion` path through the typestate internally (so the
real flow is type-gated) without `#[non_exhaustive]`, add a test asserting the
production path goes through `Reviewed::accept`, and REPORT the residual (that
external struct-literal construction is still technically possible). Do not
sprawl into readers/schema.

## Constraints (safety net)

- **Output byte-identical.** All existing completion tests, the
  `completion-review.report.example.json` fixture, and
  `scripts/validate-json-contracts.py` must pass UNCHANGED. The completion-review
  report serialization must not change.
- Update only test/consumer CALL SITES that struct-literal `AcceptedCompletion`
  in OTHER crates (if any) to use the type-gated constructor — do not weaken
  assertions.
- No new deps; no `unwrap`/`expect`/`panic!` on non-test paths; `///` docs on
  any new public item; deterministic.

## Acceptance Contract

- `AcceptedCompletion` is constructable only via the typestate path (verify: no
  other construction site in the crate; `#[non_exhaustive]` blocks external
  struct literals — or, per the gate, the production path provably routes
  through `Reviewed::accept`).
- A test demonstrates: `candidate.accept(reviewer, reason)` still yields an
  `AcceptedCompletion` with `review_status == Accepted` and the same field
  values as before; and that the acceptance went through `Reviewed::accept`
  (e.g. the constructor requires the typed value).
- A negative/typed test: there is no way to build `AcceptedCompletion` from an
  unaccepted candidate without the review morphism (compile-time via the
  constructor signature, or a `compile_fail` doctest if practical).
- All pre-existing completion + completion-review tests, the example fixture,
  and schema validation pass UNCHANGED.
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean;
  `cargo test --workspace` passes; `scripts/validate-json-contracts.py` passes.

## File Placement

- `crates/higher-graphen-reasoning/src/completion/mod.rs`: `#[non_exhaustive]`
  on `AcceptedCompletion`; the type-gated constructor; rewired
  `accept_completion` + `CompletionCandidate::accept`.
- Any external (e.g. `higher-graphen-runtime`) test that struct-literals
  `AcceptedCompletion`: update the call site to the constructor (no assertion
  changes).
