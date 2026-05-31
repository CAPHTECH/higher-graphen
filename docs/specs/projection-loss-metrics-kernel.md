# Projection Loss Metrics Kernel

## Purpose

Make projection information loss **computed and verifiable** instead of a
free-text declaration. Today a `Projection`/`ProjectionResult` carries only
`InformationLoss { description, source_ids }` — a human-written claim. Nothing
checks whether the projection actually collapsed distinctions, dropped sources,
or rendered a source ambiguously, and nothing checks that measurable loss was
declared.

This kernel computes finite structural loss metrics over a `ProjectionResult`,
detects undeclared and ambiguous loss as structured obstructions, and thereby
hardens HigherGraphen's core promise that "a human view should declare
meaningful information loss." It is the design doc's implementation-order #1
because it strengthens an existing contract without changing review semantics.

## Scope

In scope (`higher-graphen-projection` only):

- A pure, deterministic function that measures finite loss metrics over a
  `ProjectionResult` plus an optional eligible source universe, and returns the
  two records the math spec names (`ProjectionLossMetric`,
  `ProjectionAmbiguityReport`) bundled in one report with obstructions.
- Honest handling of output kinds: `Sections`/`KeyValue` carry per-item
  `source_ids`; `Text`/`Table` do not. Metrics that need per-item attribution
  are reported `unsupported` rather than guessed.
- Tests for every metric, every obstruction, the unsupported/untraced paths,
  determinism, and JSON round-trip.

Explicit non-goals this cycle (do **not** do):

- No change to existing `Projection`, `ProjectionResult`, `ProjectionOutput`,
  `InformationLoss`, or any existing public API. Purely additive new types +
  one entry function.
- No entropy / probability metrics (the doc defers these). Finite structural
  only.
- No `InMemorySpaceStore`/runtime/CLI wiring. Pure kernel over explicit inputs.
- This kernel does NOT forbid lossy projections. It evaluates safety and forces
  loss to be declared and reviewable. Obstructions are review signals, not
  hard failures; there is no "blocked" outcome.
- No new crate dependencies.

## Inputs

```text
measure_projection_loss(
    result: &ProjectionResult,
    eligible_source_ids: &[Id],   // the sources that COULD have been projected
) -> ProjectionLossReport
```

`eligible_source_ids` is the source universe (e.g. what the projection's
selector made available). Empty slice = "omission not evaluated" (selector-loss
metrics are simply not computed; this is not itself an obstruction). A non-empty
universe enables omission detection.

All needed data is reachable through existing public accessors:
`result.projection_id()`, `result.output()`, `result.source_ids()`,
`result.information_loss()`.

## Output-Item Model

Define a uniform notion of "output item" with a stable identifier and its
traced sources, derived from `result.output()`:

| `ProjectionOutput` | Items | Item id (deterministic) | Per-item source_ids |
| --- | --- | --- | --- |
| `Sections { sections }` | one per section | `section:{index}:{title}` | `section.source_ids` (constructor guarantees non-empty) |
| `KeyValue { entries }` | one per entry | `entry:{index}:{key}` | `entry.source_ids` (non-empty) |
| `Text { .. }` | one item, the whole blob | `text` | **none** (no per-item attribution) |
| `Table { rows, .. }` | one per row | `row:{index}` | **none** (rows carry only strings) |

So `Sections`/`KeyValue` are **traced** (per-item sources available);
`Text`/`Table` are **untraced** (only the result-level `source_ids()` union is
known).

## Metric Definitions (deterministic, finite)

Let `traced` = the set of distinct source ids drawn from per-item `source_ids`
(traced outputs only). Let `represented` = `result.source_ids()` as a set
(result-level union; available for all output kinds).

- **source_cardinality** = `|eligible_source_ids|` if non-empty, else
  `|represented|`. (Report which basis was used via `metric_kind`/a field.)
- **projected_cardinality** = number of output items.
- **collapsed_pair_count** = count of unordered pairs `{a,b}` of distinct
  traced sources that co-occur in at least one traced item (their distinction
  is collapsed into one output unit). Untraced outputs contribute 0 and trigger
  `unsupported`/`source_trace_missing` (below).
- **distinguished_pair_count** = `C(|traced|, 2) - collapsed_pair_count` (pairs
  of traced sources that never co-occur in one item).
- **omitted_source_ids** = `eligible_source_ids \ represented` (sources eligible
  but absent from the output). Empty when `eligible_source_ids` is empty.
- **ambiguous_source_ids** = traced sources appearing in **two or more**
  distinct output items (same source rendered in multiple places).
- **traceability**: items with no source attribution. For `Text`/`Table` every
  item is untraced.
- **ambiguity_score** = `ambiguous_source_ids.len() / traced.len()` as `f64` in
  `[0.0, 1.0]` (`0.0` when `traced` is empty). Store raw counts too so
  consumers need not recompute.

All sets are materialized as sorted `Vec<Id>`; all counts are integers; the one
`f64` is a deterministic rational of two integers (serde_json round-trips it
exactly).

## Records

Follow the math spec's names. Suggested shapes (adjust to crate conventions,
keep the semantics and field names recognizable):

```text
ProjectionLossMetricKind { FiniteStructural }   // only MVP variant; leaves room for Entropy later

ProjectionLossMetric {
    projection_id, metric_kind,
    source_cardinality, projected_cardinality,
    collapsed_pair_count, distinguished_pair_count,
    omitted_source_ids,            // sorted
    ambiguity_score,               // f64 in [0,1]
    declared_loss_source_ids,      // sorted union of all InformationLoss.source_ids
}

ProjectionAmbiguityReport {
    projection_id,
    ambiguous_output_ids,          // sorted item ids that share a source with another item
    collapsed_source_groups,       // per traced item with >1 source: (item_id, sorted source_ids)
    missing_loss_declarations,     // sorted source ids involved in measurable loss but NOT covered by any declared InformationLoss.source_ids
    risk_severity,                 // core Severity
    obstructions,                  // Vec<ProjectionLossObstruction>
}

ProjectionLossReport { metric: ProjectionLossMetric, ambiguity: ProjectionAmbiguityReport }
```

"Covered by a declared loss" means: a source id appears in the union of
`result.information_loss()[*].source_ids`.

## Obstruction Families

Stable `snake_case` codes. None are "blocking" — they are review signals.

| Code | Fires when |
| --- | --- |
| `undeclared_projection_loss` | There is measurable loss — a collapsed pair, or an omitted source — whose involved source id(s) are not all covered by a declared `InformationLoss`. This is the central check. |
| `ambiguous_projection_output` | At least one source appears in ≥2 distinct output items and is not covered by a declared loss. |
| `source_trace_missing` | An output item has no source attribution (always true for `Text` and `Table`), so loss cannot be verified for it. |
| `unsupported_loss_metric` | A per-item metric (collapse / ambiguity) cannot be computed for the output kind (`Text`/`Table`). Report the metric values that ARE computable (cardinality, omission) and mark the rest unsupported. |

`risk_severity` mapping (use core `Severity`): if any `undeclared_projection_loss`
or `ambiguous_projection_output` → higher severity (e.g. `Severity::High` /
project's nearest equivalent); else if only `source_trace_missing` /
`unsupported_loss_metric` → medium; else low/none. Pick from the existing
`Severity` variants; do not invent new ones.

## Boundary / Safety Rules

- The kernel never mutates the projection and never changes review status.
- It does not forbid lossy projections; it surfaces measurable + undeclared loss
  for review.
- Declared `InformationLoss` is respected: loss whose sources are declared does
  not raise `undeclared_projection_loss`.
- Output identifier and all id-list ordering is deterministic (sorted); equal
  input yields byte-identical serialized JSON.

## Minimal Acceptance Contract

- Bounded input, structured report records.
- A traced (`Sections` or `KeyValue`) case with a genuine collapse that is
  declared → no `undeclared_projection_loss`; correct `collapsed_pair_count`,
  `distinguished_pair_count`, `projected_cardinality`.
- The same collapse left undeclared → `undeclared_projection_loss` with the
  right `missing_loss_declarations`.
- An ambiguous case (one source in two items), declared vs undeclared → metric
  populated; `ambiguous_projection_output` only when undeclared.
- An omission case (non-empty `eligible_source_ids` with a dropped source),
  undeclared → `undeclared_projection_loss` listing the omitted source.
- A `Text` and a `Table` case → `source_trace_missing` +
  `unsupported_loss_metric`, with cardinality/omission still computed.
- A determinism test (same input → identical serialized output).
- A JSON round-trip test for `ProjectionLossReport`.
- `cargo fmt` clean; `cargo clippy --workspace --all-targets` clean (no new
  warnings; `///` docs on all new public items); `cargo test --workspace`
  passes. No new deps; no `unwrap`/`expect`/`panic!` on non-test paths.

## File Placement

- New module `crates/higher-graphen-projection/src/loss_metrics.rs` (or inline
  in `lib.rs` if small) with the records, obstruction enum, and
  `measure_projection_loss`. Re-export from `lib.rs`.
- Tests in `crates/higher-graphen-projection/src/tests.rs` (the crate's existing
  test module) or a new `tests`-style submodule consistent with the crate.
- Add a short example to `docs/specs/math-kernel-api-examples.md` under a new
  "Projection Loss Metrics" section (a few lines).
- Optionally add the kernel to the `Information Theory For Projections` MVP
  surface list in `docs/specs/math-extension-kernels.md` (mark it implemented).
