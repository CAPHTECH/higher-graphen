# Persistence Distance Kernel

This kernel adds deterministic, finite distances between two persistence
diagrams produced by the existing topology engine in `higher-graphen-structure`
(`src/topology/`). It follows the contract in `math-extension-kernels.md`: it
operates on existing structures (`PersistenceInterval`, `FiltrationStage`,
`PersistenceSummary`), returns structured records, makes information loss and
unsupported cases explicit, keeps provenance/review separation, and never
promotes a comparison into an accepted equivalence or morphism.

## Purpose

Quantify how far two persistence diagrams are, per homology dimension, so a
workflow can ask "did the structural fingerprint of this complex drift between
two filtrations / two snapshots?" as a bounded numeric report rather than a
free-text claim. The report is a review signal, never an accepted fact.

## Scope

In scope:

- Bottleneck distance per dimension (exact, finite).
- p-Wasserstein distance per dimension (exact assignment; `p` exposed,
  defaults documented as `p = 1` and `p = 2`).
- Explicit matched pairs (point-to-point or point-to-diagonal).
- A non-blocking review-signal obstruction when a supplied drift threshold is
  exceeded.

Out of scope (explicitly reported, never silently approximated):

- Continuous/geometric birth-death values. Coordinates are filtration **stage
  indices**, which is the only ordering the topology engine exposes.
- Stability theorems, bottleneck-Wasserstein inequalities, or any claim beyond
  the two diagrams supplied.
- Promotion to an `EquivalenceClaim`, morphism, or accepted review status.

## Mathematical Definition

A persistence diagram in dimension `d` is the multiset of points
`(birth, death)` derived from every `PersistenceInterval` with
`dimension == d`:

- `birth = birth_stage_index` (an existing field).
- `death = death_stage_index` when present; otherwise `death = S`, the number
  of supplied filtration stages (a finite sentinel for never-dying features).
  `S` is shared by both diagrams via a single supplied stage ordering, so birth
  and death indices are comparable across the two diagrams.

The diagonal is `Delta = { (t, t) }`. The cost of matching a point to the
diagonal is the L-infinity distance to its diagonal projection,
`(death - birth) / 2`.

### Doubled-integer arithmetic (determinism + exactness)

All coordinates are non-negative integers, so every relevant quantity is a
half-integer at worst. The kernel scales every value by `2` ("doubled units")
so all intermediate costs are exact integers:

- doubled L-infinity between points `u = (b_u, d_u)`, `v = (b_v, d_v)`:
  `2 * max(|b_u - b_v|, |d_u - d_v|)`.
- doubled to-diagonal cost of a point `(b, d)`: `2 * ((d - b) / 2) = d - b`
  (`d >= b` always holds for a persistence interval).

Because matching decisions are driven only by integer comparisons with fixed
deterministic tie-breaking, the matching and every serialized field is
byte-identical for equal input.

### Bottleneck

`bottleneck_d = min over matchings M of max over matched costs`, where every
point of either diagram is matched to a point of the other diagram or to the
diagonal. The kernel computes it exactly: it collects the sorted distinct set
of candidate doubled costs (all point-to-point doubled L-infinity values plus
all doubled to-diagonal values, plus `0`), then binary-searches for the
smallest threshold `t` that is feasible. Feasibility at `t` is decided by a
bipartite matching that must saturate all points: a point of `A` may match a
point of `B` when their doubled L-infinity `<= t`, or its own diagonal
projection when its doubled to-diagonal cost `<= t`; remaining `B` points must
likewise reach the diagonal. The smallest feasible `t` (in doubled units) is
the doubled bottleneck distance.

### p-Wasserstein

`wasserstein_d(p) = ( min over matchings M of sum over matched (cost^p) )^(1/p)`.
The kernel builds the standard augmented square cost matrix of size
`(n + m) x (n + m)` (Kerber-Morozov-Nigmetov layout) over `n = |A_d|`,
`m = |B_d|`:

- top-left `n x m`: `doubled_Linf(a_i, b_j)^p`.
- top-right `n x n`: `a_i` to its diagonal projection on the diagonal entry,
  `+inf` off-diagonal.
- bottom-left `m x m`: `b_j` to its diagonal projection on the diagonal entry,
  `+inf` off-diagonal.
- bottom-right `m x n`: diagonal-to-diagonal, cost `0`.

A deterministic exact min-cost perfect matching (Kuhn-Munkres / Hungarian) over
integer costs yields `cost_power_sum_doubled = min sum (doubled_cost)^p`. The
true distance is `(cost_power_sum_doubled)^(1/p) / 2`, exposed as an f64
accessor; the exact integer power sum is the serialized, byte-identical field.

## Inputs

`PersistenceDistanceRequest` (constructed, validated):

- `stage_ids: Vec<Id>` — the shared, ordered filtration stages. Defines `S` and
  the index of every death stage id. Must be non-empty and have unique ids.
- `left: Vec<PersistenceInterval>` — diagram `A` (any provenance; treated as a
  candidate fingerprint).
- `right: Vec<PersistenceInterval>` — diagram `B`.
- `wasserstein_order: u32` — `p`, default `1`; `0` is rejected.
- `drift_threshold_doubled: Option<u64>` — optional doubled bottleneck
  threshold; when the doubled bottleneck of any dimension exceeds it the report
  emits a non-blocking review signal.

Every interval whose `death_stage_id` is `Some` must reference a known stage id;
otherwise the request is rejected with a structured `malformed_field` error
(unsupported input made explicit, not silently dropped).

## Records

`PersistenceDistanceReport`:

- `stage_count: usize` — `S`, the open-interval death sentinel.
- `wasserstein_order: u32` — `p` actually used.
- `dimensions: Vec<PersistenceDimensionDistance>` — sorted by `dimension`.
- `max_bottleneck_doubled: u64` — max over dimensions (0 when empty).
- `review_status: ReviewStatus` — always `Candidate`; this is a comparison, not
  an accepted equivalence.
- `obstructions: Vec<PersistenceDistanceObstruction>` — review signals.

`PersistenceDimensionDistance`:

- `dimension: Dimension`.
- `left_point_count`, `right_point_count: usize`.
- `bottleneck_doubled: u64` — exact doubled bottleneck.
- `wasserstein_cost_power_sum_doubled: u128` — exact `sum (doubled_cost)^p`.
- `matches: Vec<PersistenceMatch>` — the Wasserstein optimal matching, sorted.

`PersistenceMatch` (one of: point-to-point, left-to-diagonal,
right-to-diagonal) carries the matched interval coordinates
(`birth_index`, `death_index_or_sentinel`) and the doubled cost. Match kind is
an explicit enum so a reader never has to infer diagonal matches.

Exact f64 accessors (`bottleneck`, `wasserstein`) are methods, not fields, so
the records keep `Eq`/`Ord`/`Hash` and byte-identical JSON; the only irrational
quantity (the p-th root) is computed on demand and never serialized.

## Obstruction Families

`PersistenceDistanceObstructionType`:

- `structural_drift_exceeds_threshold` — doubled bottleneck of a dimension
  exceeds `drift_threshold_doubled`. Severity `Medium`. Review signal only.
- `empty_diagram_pair` — a dimension where both diagrams have zero points is not
  emitted as a distance row; if every dimension is empty the report notes it so
  a reader does not mistake "no data" for "distance zero everywhere". Severity
  `Low`.

Obstructions are advisory: they carry `obstruction_type`, `dimension` (when
applicable), `severity`, and `reason`. They never change `review_status` away
from `Candidate` and never assert equivalence.

## Safety / Review Boundary

- The report is always `ReviewStatus::Candidate`. Two diagrams at distance `0`
  are *not* asserted equivalent; only a human/workflow review may accept any
  downstream equivalence claim.
- No new accepted facts, morphisms, or `EquivalenceClaim`s are produced.
- Information loss is explicit: coordinates are stage indices, open intervals
  use the stage-count sentinel, and the doubled-integer scaling is documented in
  field names (`_doubled`).
- Resource use is finite: matching is `O((n+m)^3)` per dimension over the
  supplied intervals; no store traversal, no unbounded recursion.

## Determinism

- Input intervals are normalized into doubled integer points and sorted by
  `(birth, death, generator_cell_ids)` before matching.
- Bottleneck threshold candidates are a sorted deduplicated integer set.
- The Hungarian assignment uses fixed row/column iteration order and integer
  costs; ties resolve by the smallest column index, which is fully determined.
- All output id/point lists are sorted; equal input yields byte-identical
  serialized JSON. A determinism test and a JSON round-trip test enforce this.

## Minimal Acceptance Contract

- Identical diagrams: every `bottleneck_doubled` and
  `wasserstein_cost_power_sum_doubled` is `0`; f64 accessors are `0.0`.
- One shifted feature plus one extra (diagonal-matched) feature: assert exact
  doubled bottleneck `= 2`, exact W1 doubled sum `= 3` (`bottleneck = 1.0`,
  `wasserstein(1) = 1.5`), and W2 doubled power sum `= 5`
  (`wasserstein(2) = sqrt(5)/2`).
- Symmetry: `d(A, B) == d(B, A)` for bottleneck and Wasserstein power sums.
- A threshold below the bottleneck emits exactly one
  `structural_drift_exceeds_threshold` review signal and keeps
  `review_status == Candidate`.
- Determinism: shuffled input intervals produce byte-identical JSON.
- JSON round-trip: `to_string` then `from_str` reproduces the report.

## File Placement

- Types and the public entry point `persistence_distance` live in
  `crates/higher-graphen-structure/src/topology/distance.rs`, re-exported from
  `src/topology/mod.rs`. This sits beside the existing persistence engine in the
  same crate/area named by the increment, adds no new dependency, and reuses
  `PersistenceInterval`, `FiltrationStage`, `Id`, `Dimension`, `Severity`, and
  `ReviewStatus`.
- Tests live in `src/topology/distance.rs` under `#[cfg(test)]`.
