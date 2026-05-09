# Mathematical Extension Kernels

This document designs additional mathematical kernels for HigherGraphen at the
same abstraction level as the existing structure, evidence, reasoning,
projection, interpretation, and runtime contracts.

The goal is not to expose mathematics as product-facing terminology. The goal
is to turn mathematical methods into bounded, deterministic, reviewable
operations that improve decisions about uncertainty, completion, projection,
change, and safety.

## Design Rules

Every mathematical extension must follow the existing HigherGraphen contract:

- It must operate on spaces, cells, incidences, contexts, morphisms,
  invariants, obstructions, completions, projections, evidence, scenarios,
  policies, valuations, or schema morphisms.
- It must return structured records, not untyped prose or hidden state.
- It must make approximation, information loss, unsupported cases, and
  resource limits explicit.
- It must preserve provenance, confidence, severity, and review status
  separation.
- It must not silently promote generated results into accepted facts.
- It must start with finite, deterministic kernels before adding probabilistic,
  heuristic, or provider-backed implementations.
- It must remain product-neutral. Domain products and interpretation packages
  translate domain language into these kernels.

## Kernel Families

| Kernel family | Primary package | Main object | Main output |
| --- | --- | --- | --- |
| Uncertainty and value-of-information | `higher-graphen-evidence` | Claim, evidence, observation action | Evidence priority, posterior update, uncertainty obstruction |
| Optimization | `higher-graphen-reasoning` | Candidate set, objective, constraint | Ranked option set, Pareto frontier, infeasibility obstruction |
| Information theory | `higher-graphen-projection` | Projection, selector, source structure | Loss metric, ambiguity report, projection risk |
| Order and lattice reasoning | `higher-graphen-structure` and `higher-graphen-reasoning` | Ordered structure, refinement relation | Comparability report, meet/join candidate, monotonicity check |
| Abstract interpretation | `higher-graphen-reasoning` | Concrete space, abstract domain | Conservative abstract state, unknown region, violation possibility |
| Graph analytics | `higher-graphen-structure` | Finite graph or incidence view | Impact cone, cut set, central boundary, cycle reduction candidate |
| Temporal and model checking | `higher-graphen-reasoning` | State space, transition, trace property | Reachability proof, counterexample trace, temporal obstruction |
| Categorical construction | `higher-graphen-structure` | Diagram of spaces and morphisms | Pullback/pushout candidate, non-commutative finding, merge obstruction |

## Uncertainty And Value-Of-Information

### Purpose

Use uncertainty models to decide which claim should be checked next, which
evidence would most reduce ambiguity, and when an inference remains too weak to
use as accepted support.

### Core Records

`UncertaintyState`:

- `claim_id`
- `prior_confidence`
- `posterior_confidence`
- `uncertainty_measure`
- `supporting_evidence_ids`
- `contradicting_evidence_ids`
- `unresolved_obstruction_ids`
- `review_status`
- `provenance`

`ObservationAction`:

- `id`
- `target_claim_ids`
- `expected_evidence_kind`
- `estimated_cost`
- `expected_information_gain`
- `blocked_by_policy_ids`
- `provenance`
- `review_status`

`InformationGainReport`:

- `claim_id`
- `candidate_actions`
- `recommended_action_ids`
- `calculation_kind`
- `unsupported_reason`
- `information_loss`

### MVP Kernel

The first kernel should extend the existing Bayesian-inspired confidence module
with deterministic value-of-information scoring:

```text
expected_gain = current_uncertainty - expected_posterior_uncertainty
net_value = expected_gain - normalized_observation_cost
```

The MVP may use entropy or posterior distance from the decision threshold as
the uncertainty measure. It must report the chosen measure explicitly.

Implemented MVP surface:

- `higher_graphen_reasoning::uncertainty::score_information_gain`
- `score_multi_claim_information_gain`
- `posterior_from_likelihood` and `EvidenceLikelihoodModel`
- `UncertaintyState`, `ObservationAction`, `InformationGainReport`
- `MultiClaimInformationGainReport`

### Obstructions

- `insufficient_prior`
- `missing_likelihood_model`
- `observation_blocked_by_policy`
- `cost_exceeds_budget`
- `unsupported_uncertainty_measure`

### Boundary

This kernel recommends what to observe. It does not accept the claim, execute
the observation, or treat confidence as review status.

## Optimization

### Purpose

Use finite optimization to rank completion candidates, repair plans, review
targets, or scenario changes by explicit objectives and constraints.

### Core Records

`OptimizationProblem`:

- `id`
- `candidate_ids`
- `objective_ids`
- `constraint_ids`
- `required_invariant_ids`
- `blocked_candidate_ids`
- `optimization_kind`
- `resource_limits`
- `provenance`

`Objective`:

- `id`
- `target_kind`
- `direction`
- `weight`
- `priority`
- `measurement_source_ids`
- `provenance`

`OptimizationReport`:

- `problem_id`
- `status`
- `selected_candidate_ids`
- `ranked_candidate_ids`
- `pareto_frontier`
- `dominated_candidate_ids`
- `violated_constraint_ids`
- `relaxed_constraint_ids`
- `obstructions`
- `resource_usage`
- `review_status`

### MVP Kernel

Start with deterministic finite ranking and Pareto filtering:

- reject candidates that violate hard constraints;
- compute comparable objective vectors;
- return Pareto-optimal candidates;
- optionally apply lexicographic priorities for a single recommended order.

Set cover and minimum hitting set can be added for completion planning:

```text
candidate -> obstruction ids it resolves
goal      -> cover all required obstruction ids with minimum cost
```

### Obstructions

- `infeasible_constraint_set`
- `candidate_missing_measurement`
- `objective_incomparable`
- `resource_limit_exceeded`
- `optimization_unsupported`

### Boundary

Optimization produces recommendations and rankings. It does not apply a
candidate or change its review status.

## Information Theory For Projections

### Purpose

Make projection loss more precise than free-text declarations when a projection
compresses, summarizes, filters, or identifies structures.

### Core Records

`ProjectionLossMetric`:

- `projection_id`
- `source_ids`
- `metric_kind`
- `source_cardinality`
- `projected_cardinality`
- `distinguished_pair_count`
- `collapsed_pair_count`
- `ambiguity_score`
- `declared_loss_ids`
- `provenance`

`ProjectionAmbiguityReport`:

- `projection_id`
- `ambiguous_output_ids`
- `collapsed_source_groups`
- `missing_loss_declarations`
- `risk_severity`
- `obstructions`

### MVP Kernel

Use finite structural metrics first:

- cardinality loss: source count versus projected count;
- collapsed distinctions: source pairs that map to one projected item;
- selector loss: selected source ids omitted by the projection;
- traceability coverage: projected items with source ids.

Entropy-based metrics can be added later when source distributions are
available and validated.

### Obstructions

- `undeclared_projection_loss`
- `ambiguous_projection_output`
- `source_trace_missing`
- `unsupported_loss_metric`

### Boundary

This kernel evaluates projection safety. It does not forbid lossy projections;
it forces loss to be declared and reviewable.

## Order And Lattice Reasoning

### Purpose

Represent refinement, abstraction strength, evidence support, policy strength,
and requirement implication as explicit order relations.

### Core Records

`OrderRelation`:

- `id`
- `space_id`
- `relation_type`
- `lesser_id`
- `greater_id`
- `criteria`
- `provenance`
- `review_status`

`OrderCheckReport`:

- `relation_set_id`
- `status`
- `cycle_witness`
- `incomparable_pairs`
- `least_upper_bound_candidates`
- `greatest_lower_bound_candidates`
- `monotonicity_violations`
- `obstructions`

### MVP Kernel

Start with finite partial-order checks:

- reflexive closure support;
- antisymmetry violation detection;
- transitive reachability;
- incomparability reporting;
- meet/join candidate discovery by finite search.

Monotonicity checks should evaluate whether a morphism preserves an order
relation:

```text
a <= b in source implies f(a) <= f(b) in target
```

Implemented MVP surface:

- `higher_graphen_structure::space::FiniteOrderRelationSet::analyze`
- `accepted_relations` and `selected_by_review_statuses`
- `check_order_monotonicity`
- `check_relation_order_monotonicity`
- `OrderCheckReport` with comparability, meet/join candidates, selected
  relation identifiers, and obstructions.

### Obstructions

- `order_cycle`
- `antisymmetry_violation`
- `missing_refinement_witness`
- `join_not_unique`
- `monotonicity_violation`

### Boundary

Order relations are claims unless accepted by review. The kernel can prove
finite consequences of accepted or selected relations, but it must report which
relation set it used.

## Abstract Interpretation

### Purpose

Analyze large or partially known structures with conservative approximations
instead of requiring full exact evaluation.

### Core Records

`AbstractDomain`:

- `id`
- `name`
- `abstract_value_kinds`
- `join_operation_id`
- `widening_policy`
- `soundness_assumption_ids`
- `provenance`

`AbstractionMorphism`:

- `morphism_id`
- `concrete_space_id`
- `abstract_space_id`
- `concrete_to_abstract_mapping`
- `soundness_invariant_ids`
- `lost_structure`
- `distortion`
- `provenance`

`AbstractInterpretationReport`:

- `analysis_id`
- `abstract_state_ids`
- `definitely_satisfied_invariant_ids`
- `possibly_violated_invariant_ids`
- `unknown_region_ids`
- `widening_events`
- `obstructions`
- `information_loss`

### MVP Kernel

Use finite domains and explicit join tables:

- map concrete cells to abstract cells;
- propagate abstract values over incidences;
- join values at merge points;
- return `definitely`, `possibly`, or `unknown` results for selected
  invariants.

### Obstructions

- `unsound_abstraction`
- `missing_join_operation`
- `widening_lost_required_distinction`
- `unknown_region_requires_witness`
- `abstract_counterexample_requires_concretization`

### Boundary

Abstract interpretation may report possible violations. It must not claim an
exact violation unless a concrete witness or accepted concretization exists.

## Graph Analytics

### Purpose

Extend traversal beyond reachability into practical structural diagnostics for
impact analysis, review targeting, dependency risk, and boundary detection.

### Core Records

`GraphAnalyticsInput`:

- `space_id`
- `cell_selector`
- `incidence_selector`
- `algorithm_kind`
- `resource_limits`

`GraphAnalyticsReport`:

- `algorithm_kind`
- `ranked_cell_ids`
- `ranked_incidence_ids`
- `cut_sets`
- `impact_cones`
- `component_summaries`
- `cycle_reduction_candidates`
- `obstructions`
- `information_loss`

### MVP Kernel

Add deterministic finite algorithms:

- impact cone: bounded forward and backward closure from changed cells;
- articulation points and bridges over a selected undirected view;
- minimum cut approximation for small finite graphs;
- dominator tree for directed workflow graphs;
- feedback edge candidates for cycle reduction.

Implemented MVP surface:

- `InMemorySpaceStore::analyze_graph`
- `GraphAnalyticsInput` and `GraphAnalyticsReport`
- impact cone, articulation cells, bridge incidences, connected components,
  strongly connected components, degree-style centrality scores, cut-cell
  candidates, and single-seed dominator candidates.

### Obstructions

- `boundary_crossing_unreviewed`
- `critical_connector_uncovered`
- `cycle_reduction_candidate_unreviewed`
- `impact_cone_exceeds_resource_limit`

### Boundary

Graph analytics creates review prompts and structural rankings. It does not
decide final review ownership or approve changes.

## Temporal And Model Checking

### Purpose

Check stateful behavior, workflow safety, policy sequencing, and trace
obligations that cannot be expressed as one reachability query.

### Core Records

`StateModel`:

- `space_id`
- `state_cell_ids`
- `transition_incidence_ids`
- `initial_state_ids`
- `terminal_state_ids`
- `state_label_ids`
- `provenance`

`TemporalProperty`:

- `id`
- `property_kind`
- `scope`
- `formula`
- `severity`
- `provenance`

`ModelCheckReport`:

- `property_id`
- `status`
- `satisfying_trace`
- `counterexample_trace`
- `unreachable_state_ids`
- `resource_usage`
- `obstructions`

### MVP Kernel

Support a bounded finite subset before adding a full temporal logic parser:

- forbidden state reachability;
- required eventual transition;
- always-before ordering;
- absence of dead end except terminal states;
- bounded counterexample trace generation.

Implemented MVP surface:

- `check_model` / `ModelChecker::check`
- `check_required_event`
- `check_always_before`
- `check_dead_ends`
- `ModelCheckingReport` and `TemporalCheckReport`
- exhaustive finite-pass markers and `TemporalPropertyKind`

### Obstructions

- `forbidden_state_reachable`
- `required_event_missing`
- `ordering_violation`
- `dead_end_state`
- `state_space_limit_exceeded`

### Boundary

Bounded model checking must report bounds. A bounded pass is not an unbounded
proof unless the state model is finite and exhaustively explored.

## Categorical Construction Kernels

### Purpose

Use categorical constructions as finite structural operations for merge,
comparison, common-substructure extraction, and integration.

### Core Records

`Diagram`:

- `id`
- `space_ids`
- `morphism_ids`
- `commutativity_requirements`
- `provenance`

`PullbackCandidate`:

- `id`
- `diagram_id`
- `candidate_space_id`
- `projection_morphism_ids`
- `unmatched_source_ids`
- `information_loss`
- `review_status`

`PushoutCandidate`:

- `id`
- `diagram_id`
- `candidate_space_id`
- `inclusion_morphism_ids`
- `identified_source_groups`
- `quotient_losses`
- `unresolved_obstruction_ids`
- `review_status`

`DiagramCheckReport`:

- `diagram_id`
- `commutes`
- `non_commutative_witnesses`
- `pullback_candidates`
- `pushout_candidates`
- `obstructions`

### MVP Kernel

Start with finite diagrams over explicit mappings:

- commutativity check for two paths of morphisms;
- common mapped substructure extraction as a pullback candidate;
- explicit merge over shared source ids as a pushout candidate;
- quotient loss declaration for identified structures.

Implemented MVP surface:

- `explicit_pullback_candidate`
- `explicit_pushout_candidate`
- `check_diagram_commutativity`
- `ExplicitPullbackReport`, `ExplicitPushoutReport`, and
  `DiagramCommutativityReport`
- `ExplicitPushoutReport::candidate_space_shell` for an empty reviewable
  candidate `Space` shell.

### Obstructions

- `non_commutative_diagram`
- `ambiguous_identification`
- `merge_requires_unreviewed_equivalence`
- `pullback_incomplete`
- `pushout_loses_required_invariant`

### Boundary

Pullbacks and pushouts are candidates unless all equivalence claims, losses,
and invariant preservation checks are accepted.

## Runtime Composition

The kernels should compose through the existing HigherGraphen pipeline:

```text
lifted structure
  -> graph analytics
  -> abstract interpretation
  -> invariant or model check
  -> obstruction
  -> optimization over completion candidates
  -> uncertainty-driven evidence request
  -> projection with information-loss metrics
```

No kernel should own the whole pipeline. Runtime workflows select a bounded
input snapshot, run one or more kernels, and emit a report with source
boundaries, review status, and declared information loss.

## Package Placement

| Package | Additions |
| --- | --- |
| `higher-graphen-core` | No new mathematical concepts. Reuse `Id`, `Confidence`, `Severity`, `ReviewStatus`, `Provenance`, and structured errors. |
| `higher-graphen-structure` | Order relations, graph analytics over finite incidence views, categorical diagrams, pullback and pushout candidate records. |
| `higher-graphen-evidence` | Uncertainty state, value-of-information scoring, evidence acquisition recommendations. |
| `higher-graphen-reasoning` | Optimization, abstract interpretation, temporal/model checking, new obstruction families. |
| `higher-graphen-projection` | Projection loss metrics, ambiguity reports, traceability coverage. |
| `higher-graphen-interpretation` | Domain templates mapping product concepts to objectives, temporal properties, abstract domains, order relations, and observation actions. |
| `higher-graphen-runtime` | Bounded workflows that combine kernels and emit reviewable JSON reports. |

## Implementation Order

1. Projection information-loss metrics, because they strengthen an existing
   contract without changing review semantics.
2. Optimization over completion candidates, because current workflows already
   produce reviewable candidates and obstructions.
3. Graph analytics impact cones and articulation points, because they directly
   improve PR review targets and architecture boundary prompts.
4. Order checks, because valuations, refinement, requirements, and evidence
   strength already need partial comparability.
5. Value-of-information scoring, because it improves evidence gathering after
   uncertainty is visible.
6. Bounded model checking, because stateful products need trace-level
   counterexamples.
7. Abstract interpretation, because it requires careful soundness contracts.
8. Categorical pullback and pushout candidates, because they should depend on
   mature morphism, equivalence, and projection-loss records.

## Minimal Acceptance Contract

Each kernel is ready to enter the MVP surface only when it has:

- a bounded input record;
- a structured report record;
- at least one satisfied case;
- at least one obstruction case;
- deterministic ordering of output identifiers;
- JSON round-trip tests;
- source boundary and information-loss declarations when used by runtime;
- no silent review-status promotion.
