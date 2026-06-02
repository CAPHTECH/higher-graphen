# Advisory Domain Extensions

This spec designs the changes requested by AdvisoryGraphen, a downstream product
built on HigherGraphen. The requests are concrete friction points where an
AI-authored, incidence-centric, non-numeric advisory domain loses richness when
it flows through HigherGraphen's deterministic kernels.

Every change here follows the existing contract rules in
[`math-extension-kernels.md`](./math-extension-kernels.md): structured records
only, explicit loss/unsupported cases, preserved provenance/severity/review
separation, no silent promotion of generated results into accepted facts, and
finite deterministic behavior. No new dependencies. Backward compatibility is
not a goal — constructors may be tightened or relaxed when the new contract is
cleaner.

The six requests map to one branch, `feat/advisory-requests`, with logically
separated commits.

| Req | Area | Crate | Kind |
| --- | --- | --- | --- |
| R1 | Carry custom obstruction type + counterexample + required resolution through `CheckResult::to_obstruction` | `higher-graphen-reasoning` | additive plumbing |
| R2 | Native handling of untraced / empty projection output in `measure_projection_loss` | `higher-graphen-projection` | invariant relaxation + status path |
| R3 | Custom missing-type taxonomy in detection/completion | `higher-graphen-reasoning` | additive variant |
| R4 | Stable discriminant accessors on wire enums | `higher-graphen-core` | additive accessor |
| R5 | Documented wire-stability contract | `docs` / `schemas` | governance doc |
| R6 | Lightweight incidence/categorical engine variant | `higher-graphen-reasoning` | bounded new engine |

---

## R1 — Obstruction richness through `CheckResult::to_obstruction`

### Problem

`CheckResult::to_obstruction` (`reasoning/src/invariant/mod.rs:414`) hardcodes
`obstruction_type` to `InvariantViolation` / `ConstraintUnsatisfied` and moves
only `location_cell_ids` / `location_context_ids` / `related_morphism_ids` from
`Violation` (`reasoning/src/invariant/mod.rs:262`). The richer obstruction fields
already exist on the target type — `ObstructionType::Custom`
(`reasoning/src/obstruction/mod.rs:18`, `::custom` at `:35`),
`Obstruction::with_counterexample` (`:355`), `Obstruction::with_required_resolution`
(`:361`) — but `Violation`/`CheckResult` cannot carry them, so the EvaluatorKernel
→ CheckResult → to_obstruction path drops the advisory domain's standard payload.

### Change

Add three optional fields to `Violation`, all serde-skippable when absent so the
existing wire form is unchanged when unused:

- `obstruction_type: Option<ObstructionType>` — when `Some`, overrides the
  `target_kind`-derived default in `to_obstruction`. When `None`, the existing
  `Invariant`/`Constraint` mapping is preserved.
- `counterexample: Option<Counterexample>`
- `required_resolution: Option<RequiredResolution>`

Add builder methods on `Violation` mirroring the existing `Obstruction` builders
(`with_obstruction_type`, `with_counterexample`, `with_required_resolution`).

In `to_obstruction`, replace the hardcoded `match self.target_kind { ... }` with:
use `violation.obstruction_type` if present, else fall back to the `target_kind`
mapping. After constructing the `Obstruction`, thread `counterexample` and
`required_resolution` through the existing builders when present.

### Safety / review boundary

No status semantics change: only `Violated` results still produce an obstruction.
A custom `obstruction_type` is downstream-owned text validated by
`ObstructionType::custom` (non-empty, normalized). Provenance/severity untouched.

### Determinism

`to_obstruction` remains a pure function of `self` + arguments.

### Minimal Acceptance Contract

- A violated `CheckResult` with no new fields still produces the same
  `InvariantViolation` / `ConstraintUnsatisfied` obstruction (regression).
- A violated `CheckResult` carrying a custom obstruction type + counterexample +
  required resolution produces an `Obstruction` whose `obstruction_type.is_custom()`
  holds and whose counterexample/required_resolution are present.
- `Satisfied` / `Unsupported` still return `Ok(None)`.
- `Violation` JSON round-trips with and without the new fields; absent fields do
  not appear in serialized output.
- fmt / clippy / test / static-limits green.

---

## R2 — Untraced and empty projection output in `measure_projection_loss`

### Problem

`ProjectionOutput::key_value` (`projection/src/lib.rs:406`) rejects empty
`entries` via `collect_non_empty_items`, and `ProjectionEntry::new`
(`projection/src/lib.rs` ~`:460`) / `ProjectionSection::new` reject empty
`source_ids` via `collect_non_empty_ids`. So an advisory output where *zero items
carry source attribution* cannot be constructed as `KeyValue`/`Sections`; the
consumer is forced to switch to `ProjectionOutput::Text` purely to bypass the
kernel. `measure_projection_loss` (`projection/src/loss_metrics.rs:114`) then
conflates two distinct conditions: it emits `SourceTraceMissing` **and**
`UnsupportedLossMetric` together for `Text`/`Table` only
(`loss_metrics.rs` `obstructions()` ~`:382`), and never for an attributable kind
whose items lack attribution.

### Change

Separate **item-level missing attribution** from **kind-level unsupported metric**:

1. Relax `ProjectionEntry::new` and `ProjectionSection::new` to accept empty
   `source_ids` (replace `collect_non_empty_ids` with the de-duplicating collect
   that allows empty). An item with empty `source_ids` means *present but
   untraced*, not malformed.
2. Relax `ProjectionOutput::key_value` and `::sections` to accept empty
   collections (replace `collect_non_empty_items` with a plain collect). An empty
   attributable output means *no attribution present*.
3. In `loss_metrics.rs`, keep `OutputItem.source_ids: Option<Vec<Id>>` where
   `None` = kind cannot carry attribution (`Text`/`Table`), `Some(vec)` =
   attributable kind (vec may now be empty). Compute two independent flags:
   - `has_untraced_attributable_item`: any attributable output (Sections/KeyValue)
     that has zero items, or any item with `Some(vec)` where `vec.is_empty()`.
   - `unsupported_per_item_metrics`: kind is `Text`/`Table` (existing
     `has_untraced_output_kind`).
4. Update `obstructions(...)`:
   - `has_untraced_attributable_item` → `SourceTraceMissing` (only).
   - `unsupported_per_item_metrics` → `UnsupportedLossMetric` **and**
     `SourceTraceMissing` (unchanged: Text/Table lack per-item attribution, so
     both hold).

`risk_severity` already treats both as the same review tier; no change needed.

### Safety / review boundary

The kernel still never mutates the result or changes review status. Untraced
items are surfaced as a structured `SourceTraceMissing` obstruction — the advisory
consumer reads the obstruction instead of hand-rolling a `Text` detour.

### Determinism

All flags derive deterministically from output structure; ordering via existing
`sorted_unique_ids` / `BTreeSet`.

### Minimal Acceptance Contract

- A `KeyValue` output with one traced entry and one entry with empty `source_ids`
  is constructible and yields `SourceTraceMissing` **without** `UnsupportedLossMetric`.
- An empty `KeyValue` output is constructible and yields `SourceTraceMissing`.
- `Text` / `Table` outputs still yield both `SourceTraceMissing` and
  `UnsupportedLossMetric` (regression — existing tests
  `projection_loss_text_reports_unsupported_and_untraced_metrics`,
  `projection_loss_table_...` must still pass).
- A fully traced `KeyValue` with no collapse/omission yields neither.
- `ProjectionEntry` / `ProjectionOutput` JSON round-trip with empty source_ids /
  empty entries.
- fmt / clippy / test / static-limits green.

---

## R3 — Custom missing-type taxonomy in detection/completion

### Problem

`ObstructionType` is already open via `Custom(String)`, but `MissingType`
(`reasoning/src/completion/mod.rs:16`) is a closed enum. `CompletionRule`
(`:84`) keys on `MissingType`, and `SimpleCompletionEngine` (`:383`) /
`detect_completion_candidates` (`:433`) propagate it into `CompletionCandidate`
(`:454`). An advisory domain whose vocabulary is `ObstructionType::custom(...)`
cannot register a rule for a domain-specific missing kind, so it hand-builds
`CompletionCandidate::new` and bypasses rule-driven detection.

### Change

Add `Custom(String)` to `MissingType`, mirroring `ObstructionType`:

- `MissingType::custom(extension) -> Result<Self>` using the same
  `normalized_required_text` helper as `ObstructionType::custom`.
- `MissingType::is_custom()` and a `serialized_value() -> Result<String>` (or the
  existing serde representation) with a stable `custom:`-style prefix mirroring
  `CUSTOM_OBSTRUCTION_PREFIX`, so the wire form is collision-free.

Because `CompletionRule` already stores a `MissingType` and `to_candidate`
(`:151`) copies it into the candidate verbatim, no detection/engine signature
changes — a rule with a custom missing type flows through `SimpleCompletionEngine`
unchanged.

### Safety / review boundary

Custom missing types are downstream-owned, normalized, non-empty. Detection still
only materializes candidates as **unreviewed**; acceptance still requires the
typed-provenance review path. No silent promotion.

### Determinism

No behavioral change to matching (`applies_to`) — it is independent of
`MissingType`.

### Minimal Acceptance Contract

- A `CompletionRule` with `MissingType::custom("advisory:focus")` is detected by
  `SimpleCompletionEngine::detect_candidates` and materializes an unreviewed
  candidate carrying the custom missing type.
- `MissingType::custom("")` is rejected.
- `MissingType` JSON round-trips for both built-in and custom variants;
  custom serialized form is prefixed and collision-free with built-ins.
- Existing detection tests unchanged in count and outcome.
- fmt / clippy / test / static-limits green.

---

## R4 — Stable discriminant accessors on wire enums

### Problem

`GluingResult` (`core/src/correspondence.rs:693`), `DifferenceSeverity` (`:299`),
and `OverlapWitnessKind` (`:231`) expose no stable discriminant accessor; the
only stable string is the serde representation. Consumers (advisory included)
concatenate against serde names (`value["result"]["kind"]`). `ObstructionType`
already sets the precedent with `serialized_value()`
(`reasoning/src/obstruction/mod.rs:48`).

### Change

Add a `kind(&self) -> &'static str` accessor to `GluingResult`,
`DifferenceSeverity`, and `OverlapWitnessKind`, returning exactly the serde
discriminant string (`"success"`/`"candidate"`/`"failure"` for `GluingResult`;
the existing `rename_all` strings for the other two). No serde changes.

### Safety / review boundary

Pure read accessor; no state.

### Minimal Acceptance Contract

- For every variant, `kind()` equals the discriminant produced by
  `serde_json` serialization (asserted by a round-trip test that serializes each
  variant and compares the tag/string).
- fmt / clippy / test / static-limits green.

---

## R5 — Documented wire-stability contract

### Problem

Multiple downstream consumers depend on the camelCase serde representation as an
implicit contract. It is enforced by tests (`assert_serde_contract`) and the
`schemas/` directory but never declared as a versioned compatibility surface.

### Change

Add `docs/specs/wire-stability-contract.md` declaring: which types form the wire
surface, that camelCase/`tag="kind"` serde forms and the `serialized_value()` /
`kind()` discriminant strings are stable within a schema major version, the role
of `schemas/` and `validate-json-contracts.py` as the enforcement gate, and the
discriminant accessors from R4 as the recommended consumer access path (do not
concatenate against serde-internal names). Cross-link from `core-contracts.md`.

No code change. The existing `validate-json-contracts.py` gate remains the
enforcement mechanism.

### Minimal Acceptance Contract

- Doc exists, lists the wire-surface types, references the enforcement gate, and
  is cross-linked. (No gate impact beyond markdown.)

---

## R6 — Lightweight incidence/categorical engine variant

### Problem

The unused heavy engines (model checking, abstract interpretation, topology
homology/persistence, sheaf gluing, pushout/pullback) all require substrates an
AI-authored advisory domain cannot supply: cell-bounded `Complex`, state
transition graphs, numeric priors, second morphisms / cospans. Advisory supplies
**non-numeric, incidence-centric structural claims**. There is currently no
deterministic engine that consumes that lighter input.

### Scope discipline (no fabrication)

R6 is intentionally bounded to the **smallest honest engine** that adds value on
incidence/categorical input *without* fabricating structure. It does **not**
reimplement the five heavy engines. It detects structural obstructions that are
decidable purely from an incidence view, reusing existing obstruction families —
it never materializes merged complexes, numeric measures, or accepted facts.

### Change

Add a `reasoning::incidence` module exposing a deterministic
`IncidenceConsistencyEngine` over a finite, non-numeric incidence view (cells +
incidences + context labels, all `Id`-based; no coordinates, no priors). It emits
structured `Obstruction`s only, reusing existing `ObstructionType` variants and
the R1/R3 custom extensions:

- **Dangling incidence** → `ObstructionType::MissingMorphism` or a custom type:
  an incidence references a cell absent from the view.
- **Uncovered region** → `ObstructionType::UncoveredRegion`: a declared required
  cell set / context has no covering incidence.
- **Context mismatch** → `ObstructionType::ContextMismatch`: an incidence joins
  cells whose declared contexts are incompatible.

Each obstruction carries location cells/contexts and, where the input supplies
them, a `Counterexample` (the specific dangling/uncovered witness) and
`RequiredResolution` — the same richness R1 unlocks. Output is a structured
`IncidenceConsistencyReport { obstructions, ... }`; nothing is promoted to
accepted state.

### Safety / review boundary

No fabrication: the engine only reports obstructions derivable from explicit
input. It constructs no new cells, morphisms, or complexes. All findings are
unreviewed obstructions; acceptance stays on the existing review-gated path.

### Determinism

Pure function of the incidence view; deterministic ordering via `BTreeSet` /
`sorted_unique_ids`-style helpers.

### Minimal Acceptance Contract

- A consistent incidence view yields an empty obstruction set.
- A view with a dangling incidence yields exactly one `MissingMorphism`-class
  obstruction naming the missing cell, with a counterexample.
- A view with an uncovered required region yields one `UncoveredRegion`
  obstruction.
- A view with incompatible contexts on an incidence yields one `ContextMismatch`
  obstruction.
- The engine constructs no cells/morphisms/complexes (asserted structurally).
- Report JSON round-trips; determinism test (same input → byte-identical report).
- fmt / clippy / test / static-limits (≤700 lines/file, ≤80/fn, ≤12 decision
  points/fn — split into submodules if needed) green.
