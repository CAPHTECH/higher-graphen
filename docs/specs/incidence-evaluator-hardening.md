# Incidence and Evaluator Hardening

Follow-up to the advisory domain extensions (`advisory-domain-extensions.md`,
shipped in v0.7.0). Three general-purpose primitive improvements requested by
AdvisoryGraphen. All are additive and deterministic, carry no consumer domain
vocabulary, and add no dependencies. Target version: `0.7.1`. One branch,
`feat/incidence-evaluator-hardening`.

Priority: Issue 1 > Issue 2 > Issue 3 (1 is a latent correctness gap, 2 is parity
with an existing capability + bespoke-code deletion, 3 is a usability nicety).

| Issue | Area | Crate | Kind |
| --- | --- | --- | --- |
| I1 | Caller-supplied severity on incidence obstructions | `higher-graphen-reasoning` | additive field + thread |
| I2 | Acyclicity check: all cycles + edge witnesses via `find_simple_cycles` | `higher-graphen-reasoning` | consolidation (net-negative LOC) |
| I3 | Per-relation cross-context opt-out | `higher-graphen-reasoning` | additive flag |

---

## I1 — Caller-supplied severity on incidence obstructions

### Problem

`reasoning::incidence` emits every obstruction at a fixed `Severity::Medium`:
`base_obstruction` (`incidence/mod.rs:358`) hardcodes `Severity::Medium`, and all
three append paths (`append_dangling_obstruction` ~`:257`,
`append_context_mismatch` ~`:292`, `append_uncovered_region` ~`:324`) funnel
through it. Severity is not part of `IncidenceRelation` / `RequiredRegion` /
`IncidenceConsistencyInput`. This is inconsistent with the rest of the codebase,
where severity is a property of the finding supplied by the caller:
`Violation::new(message, severity)`, `Obstruction::new(..., severity, ...)`, and
the evaluator's acyclicity passing `rule.severity`.

### Change

Add optional per-element severity, defaulting to `Severity::Medium` when unset so
existing behavior and fixtures are preserved:

- `IncidenceRelation::with_dangling_severity(Severity)` — severity for the
  dangling-endpoint obstruction of this relation.
- `IncidenceRelation::with_context_mismatch_severity(Severity)` — severity for
  the context-mismatch obstruction of this relation.
- `RequiredRegion::with_severity(Severity)` — severity for this region's
  uncovered obstruction.

Store each as `Option<Severity>` with `#[serde(skip_serializing_if = "Option::is_none")]`.
Add a `severity: Severity` parameter to `base_obstruction` and pass the resolved
value (`opt.unwrap_or(Severity::Medium)`) from each append path.

### Safety / determinism

Pure; default preserves current output. No new wire fields appear when unset.

### Minimal Acceptance Contract

- A relation with `with_dangling_severity(High)` yields a dangling obstruction at
  `High`; an unset relation yields `Medium` (unchanged).
- Same for `with_context_mismatch_severity` and `RequiredRegion::with_severity`.
- Existing incidence tests/goldens unchanged.
- JSON round-trips with and without the severities; absent → not serialized.
- fmt / clippy / test / static-limits green.

---

## I2 — Acyclicity: all cycles with edge witnesses via `find_simple_cycles`

### Problem

`evaluate_acyclicity` (`invariant/evaluator/algorithms.rs:3`) uses a bespoke DFS
`directed_cycle` / `directed_cycle_from` (`algorithms.rs:276`/`:332`, ~82 lines)
that returns `Option<Vec<Id>>`: a single cycle, **cells only, no edge witnesses**.
The workspace already ships a richer deterministic primitive on the same store
type: `InMemorySpaceStore::find_simple_cycles(space_id, &CycleSearchOptions)`
(`space/traversal/algorithms.rs:43`) returning `Vec<SimpleCycleIndicator>`, where
`SimpleCycleIndicator` (`topology/mod.rs:70`) carries `witness_edge_id`,
`vertex_cell_ids`, and `edge_cell_ids`. `CycleSearchOptions` already supports a
`relation_types` filter (plus `max_cycles`, `max_path_length`). The evaluator
check is strictly weaker than what HG can already compute.

### Change (consolidation — delete bespoke DFS)

Back `evaluate_acyclicity` with `find_simple_cycles`:

1. Build `CycleSearchOptions` from `check.relation_types` (reuse `with_relation_type`).
2. Call `context.space_store.find_simple_cycles(&space_id, &options)`.
3. Empty → `CheckResult::satisfied`.
4. Non-empty → one `CheckResult::violated` with a single enriched `Violation`
   (keeps the check's 1:1 result contract) reporting **all** detected cycles:
   - `message`: deterministic summary, e.g. `"N simple cycle(s) detected"`.
   - `location_cell_ids`: sorted-unique union of `vertex_cell_ids` across all
     cycles (`with_location_cells`).
   - `related_morphism_ids`: sorted-unique union of edge witnesses across all
     cycles (`witness_edge_id` + `edge_cell_ids`) — the primary machine-readable
     edge witness, via the existing field.
   - `counterexample` (from R1): one assignment per cycle enumerating its vertices
     and edges, preserving per-cycle structure without a new report type.
   - severity stays `rule.severity` (unchanged).
5. **Delete** `directed_cycle`, `directed_cycle_from`, and the now-unused
   `VisitState` enum (`algorithms.rs:276`–~`:360`). Confirm no other caller.

### Behavior change (not golden-preserving — intentional)

Existing acyclicity tests assert the single-cycle message/cells from the bespoke
DFS. Switching to `find_simple_cycles` changes the reported representation (all
cycles, edge witnesses, new message). Update those tests to the new contract;
this is an enhancement, not a regression. Determinism is preserved
(`find_simple_cycles` is deterministic; all unions sorted-unique).

### Minimal Acceptance Contract

- A single-cycle graph yields a violation whose `related_morphism_ids` include the
  cycle's incidence/edge ids and whose counterexample names the cycle.
- A multi-cycle graph reports every simple cycle (counts/witnesses cover all).
- The `relation_types` filter still scopes which edges form cycles.
- A DAG yields `satisfied`.
- Determinism test: same store → byte-identical `CheckResult`.
- Net reasoning LOC does not increase (bespoke DFS removed).
- fmt / clippy / test / static-limits green.

---

## I3 — Per-relation cross-context opt-out

### Problem

`append_context_mismatch` (`incidence/mod.rs:292`) raises a `ContextMismatch`
obstruction for any incidence whose endpoints have disjoint contexts, with no way
to declare that a particular relation is a legitimate cross-context edge
(ownership, derivation, cross-team dependency).

### Change (minimal — opt-out flag, NOT a relation-type allow-list)

`IncidenceRelation` currently has **no `relation_type` field**, so a relation-type
allow-list would require introducing a new taxonomy (scope creep). Use the minimal
additive form instead:

- `IncidenceRelation::with_cross_context_allowed()` setting a
  `allow_cross_context: bool` field (default `false`, `#[serde(default,
  skip_serializing_if = "is_false")]` or equivalent so unset relations serialize
  unchanged).
- In `append_context_mismatch`, early-return when `incidence.allow_cross_context`
  is set, before computing/raising the mismatch.

Default `false` = current behavior exactly.

### Safety / determinism

Pure; default preserves current output.

### Minimal Acceptance Contract

- A relation with `with_cross_context_allowed()` and disjoint endpoint contexts
  yields **no** context-mismatch obstruction; without it, still does.
- Dangling and uncovered-region detection are unaffected by the flag.
- JSON round-trips; flag absent when unset.
- Existing incidence tests/goldens unchanged.
- fmt / clippy / test / static-limits green.
