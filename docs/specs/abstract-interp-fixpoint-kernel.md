# Abstract Interpretation Fixpoint Kernel

This kernel extends the existing Abstract Interpretation family in
`docs/specs/math-extension-kernels.md`. It adds the missing computation: a
monotone least-fixpoint engine over a finite directed graph. The existing
module (`higher_graphen_reasoning::abstract_interpretation`) provides data
structures only — `AbstractElement` (the membership lattice), `AbstractJoin`,
`AbstractDomain`, `AbstractRegion`, `SoundnessStatus`, `AbstractMembership`,
and `classify()` — with no propagation loop and no widening. This kernel
supplies that loop while reusing those types unchanged.

## Purpose

Compute a conservative abstract state at every node of a finite directed graph
by propagating seed abstract states along edges and joining at merge nodes,
iterating to a least fixpoint. Termination on cyclic graphs is guaranteed by an
explicit widening policy. The result lets a workflow ask, per node, which
membership facts are *definitely* established versus only *possibly* present,
without expanding the underlying large or partially known concrete space.

## Scope

- In scope: a finite, deterministic worklist (chaotic-iteration) solver over the
  existing membership lattice; an explicit per-node revisit-count widening hook;
  per-node final abstract states; `definitely_satisfied` vs `possibly_violated`
  classification for selected membership checks; structured obstructions and
  information loss.
- Out of scope: a new abstract domain (the kernel reuses the existing
  membership lattice and `AbstractJoin`); inferring the graph from
  `higher-graphen-structure` (the graph is supplied as explicit finite adjacency
  to avoid a crate dependency cycle); probabilistic or heuristic transfer.

## Mathematical Definition

The lattice is the existing membership lattice on a finite universe of concrete
identifiers `U`. An `AbstractElement` `e` is the pair
`(D(e), P(e))` with `D(e) = definite_concrete_ids` (the must-set) and
`P(e) = possible_concrete_ids` (the may-set), constrained by `D(e) ⊆ P(e)`.

The order used by the fixpoint is the existing `AbstractJoin` order: `e₁ ⊑ e₂`
iff `D(e₁) ⊇ D(e₂)` and `P(e₁) ⊆ P(e₂)`. The join (least upper bound) is
exactly `AbstractJoin::new`: `D(e₁ ⊔ e₂) = D(e₁) ∩ D(e₂)` and
`P(e₁ ⊔ e₂) = P(e₁) ∪ P(e₂)`. The kernel never reimplements meet/join; it calls
`AbstractJoin`.

Each directed edge `n → m` carries a monotone transfer `f`. The kernel uses a
`gen`-only transfer (classic monotone forward dataflow without kill, which keeps
the result a sound over-approximation):

```text
f_edge(e) = ( D(e) ∪ edge.gen_definite ,  P(e) ∪ edge.gen_definite ∪ edge.gen_possible )
```

`gen_definite` ids are added to both the must-set and may-set (they are
introduced unconditionally by traversing the edge); `gen_possible` ids are added
to the may-set only. With empty `gen` sets the transfer is the identity, which
is the pure propagation case.

The fixpoint assignment `state: Node → AbstractElement` is the least solution of

```text
state(n) ⊒ seed(n)                        for every seeded node n
state(m) ⊒ f_(n→m)( state(n) )            for every edge n → m
```

computed by chaotic iteration from the seeds. The set of concrete ids that can
ever appear is the finite union of all seed ids and all edge `gen` ids, so the
may-set is bounded and plain join iteration already terminates. The may-set is
nonetheless the only *monotone-increasing* direction and can be "tall" relative
to that universe; the widening hook caps revisits and lets a caller relax toward
an explicit universe, and the iteration bound is the hard safety net.

### Widening

The widening policy is revisit-count based and explicit, and it relaxes *only
the may-set*. The must-set is monotone decreasing under `AbstractJoin` (joins
intersect must-sets), so it converges without help and is never widened —
preserving every definite fact through widening.

A node is widened only after it has been popped from the worklist strictly more
than `widen_threshold` times. Widening unions the configured `widen_top`
universe into the may-set:

```text
widen(joined, n) =
    if revisits(n) > widen_threshold and widen_top is non-empty:
        ( D(joined) ,  P(joined) ∪ widen_top )
    else:
        joined        (ordinary join via AbstractJoin)
```

Because `widen_top` is a fixed finite set, the may-set reaches `P ∪ widen_top`
after at most one widening and then stabilizes, so the solver terminates. Each
widening application that actually adds ids is recorded as a `WideningEvent`
carrying exactly the added may-set ids. With an empty `widen_top` widening is a
no-op (no universe to relax toward); termination then rests on the finite
gen-universe and the iteration bound.

## Inputs

`AbstractGraph` (the explicit finite directed graph + seeds):

- `domain: AbstractDomain` — the single domain all states share (joins require
  equal domains, matching `AbstractJoin`).
- `nodes: Vec<AbstractGraphNode>` — each is `{ node_id: Id, successors:
  Vec<AbstractEdge> }`; `successors` is the adjacency (deduplicated and sorted).
- `seeds: Vec<NodeSeed>` — each is `{ node_id: Id, possible_concrete_ids,
  definite_concrete_ids }`; the initial abstract state at a node.
- `soundness: SoundnessStatus` — soundness assumption applied to every derived
  state (joined with seed soundness).

`AbstractEdge`: `{ target_node_id: Id, gen_definite: BTreeSet<Id>, gen_possible:
BTreeSet<Id> }`.

`FixpointOptions`:

- `widen_threshold: usize` — revisits strictly above this trigger widening.
- `widen_top: BTreeSet<Id>` — the may-set a widened node is relaxed to. Empty
  means "no universe": widening then only shrinks the must-set.
- `max_iterations: usize` — hard resource bound on total worklist pops.

`MembershipCheck` (a selected per-node invariant/membership query):

- `id: Id`, `node_id: Id`, `concrete_id: Id` — "is `concrete_id` a member at
  `node_id`?" evaluated against the final state via `AbstractElement::classify`.

## Records

`AbstractInterpretationReport` (matches the family record in
math-extension-kernels.md):

- `analysis_id: Id`
- `node_states: Vec<NodeAbstractState>` — per node, the final
  `AbstractElement` (its `id` is the node id), sorted by node id. This carries
  the `abstract_state_ids` of the family record (each node's element id).
- `reached_fixpoint: bool` — false when the iteration bound was hit before a
  fixed point; the report is then explicitly partial.
- `iterations: usize` — total worklist pops performed (resource usage).
- `definitely_satisfied_check_ids: Vec<Id>` — checks whose concrete id is
  `Definite` at the node (sorted).
- `possibly_violated_check_ids: Vec<Id>` — checks whose concrete id is
  `Possible`, `UnknownRegion`, or `Unknown` (sorted). "Possibly" is the safe
  default; `Excluded`/`Unsound` are reported via obstructions, not as
  satisfied.
- `unknown_region_node_ids: Vec<Id>` — nodes whose final state is not sound or
  carries an unknown region (sorted).
- `widening_events: Vec<WideningEvent>` — sorted by `(node_id, iteration)`.
- `obstructions: Vec<FixpointObstruction>` — sorted.
- `information_loss: Vec<String>` — explicit, sorted, deduplicated.

`NodeAbstractState`: `{ node_id: Id, state: AbstractElement }`.

`WideningEvent`: `{ node_id: Id, iteration: usize, widened_possible_ids:
BTreeSet<Id> }` — the may-set ids added by the relaxation (the must-set is never
widened).

`FixpointObstruction`: `{ obstruction_type: FixpointObstructionType, reason:
String, node_id: Option<Id> }` (mirrors the `TemporalObstruction` convention in
`model_checking`).

## Obstruction Families

Per the Abstract Interpretation section of math-extension-kernels.md:

- `missing_join_operation` — two states to be merged are in different domains,
  so `AbstractJoin` cannot run (the join operation is missing for the pair).
- `unsound_abstraction` — a seed or derived state is `SoundnessStatus::Unsound`;
  conservative absence/exclusion reasoning is invalid.
- `widening_lost_required_distinction` — a widening step added a concrete id to
  the may-set so that a selected check on that id flips from `Excluded` to
  `Possible`, losing an exclusion the check relied on.
- `unknown_region_requires_witness` — a selected check resolves to
  `UnknownRegion` or `Unknown`; treating it as satisfied/violated requires a
  concrete witness or concretization.
- `iteration_limit_exceeded` — `max_iterations` was hit before a fixed point
  (resource limit; the report is then explicitly incomplete).

## Safety And Review Boundary

The engine produces a conservative over-approximation. `possibly_violated` is
the safe default: a membership check is never reported as a definite violation
without a concrete witness. The kernel:

- never promotes a generated state to an accepted fact; `node_states` are
  reviewable candidates and carry the supplied `SoundnessStatus`;
- only reports `definitely_satisfied` when `classify` returns `Definite`;
- emits `unknown_region_requires_witness` whenever a check is `Unknown`/
  `UnknownRegion`, so concretization is required before treating it as fact;
- treats `Unsound` states as obstructions, never as exclusions.

Possible violations therefore require concretization before being treated as
facts, matching the `abstract_counterexample_requires_concretization` boundary
of the family.

## Determinism

- All inputs are normalized into `BTreeSet`/sorted `Vec` on construction.
- The worklist is a `BTreeSet<Id>` popped in ascending id order (chaotic
  iteration in a fixed deterministic order); successors are visited in sorted
  order.
- Joins are computed by `AbstractJoin`, which already produces `BTreeSet`
  outputs.
- All report lists are sorted and deduplicated; `widening_events` are sorted by
  `(node_id, iteration)`.
- Equal input yields byte-identical serialized JSON (asserted by a determinism
  test and a JSON round-trip test).

## Minimal Acceptance Contract

- A bounded input record (`AbstractGraph` + `FixpointOptions` +
  `MembershipCheck`s).
- A structured report record (`AbstractInterpretationReport`).
- At least one satisfied case: an acyclic graph whose fixpoint converges to
  asserted per-node states with a `definitely_satisfied` check.
- At least one obstruction case per family above.
- A cyclic graph that only terminates via widening, asserting termination and a
  recorded `WideningEvent`.
- Deterministic ordering of output identifiers (determinism test).
- JSON round-trip test.
- No silent review-status promotion.

## File Placement

- Engine and records: `crates/higher-graphen-reasoning/src/abstract_interpretation/fixpoint.rs`.
- Re-exported from `crates/higher-graphen-reasoning/src/abstract_interpretation/mod.rs`.
- This design doc: `docs/specs/abstract-interp-fixpoint-kernel.md`.
- No new crate dependencies; no dependency on `higher-graphen-structure` (the
  graph is supplied explicitly to avoid a dependency cycle).
