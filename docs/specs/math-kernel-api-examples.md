# Mathematical Kernel API Examples

This page shows minimal Rust API shapes for the implemented mathematical
extension kernels. The examples are intentionally small: they show how to call
the kernels without implying that generated reports are accepted facts.

## Graph Analytics

```rust
use higher_graphen_core::Id;
use higher_graphen_structure::space::{GraphAnalyticsInput, TraversalDirection};

let input = GraphAnalyticsInput::new(Id::new("space:a")?)
    .with_seed_cell_ids([Id::new("cell:start")?])
    .in_direction(TraversalDirection::Outgoing)
    .with_max_depth(3);

let report = store.analyze_graph(&input)?;
let impact = report.impact_cone_cell_ids;
let cut_candidates = report.cut_cell_candidate_ids;
```

The report may include impact cones, connected components, strongly connected
components, articulation cells, bridge incidences, degree-style centrality,
cut-cell candidates, and single-seed dominator candidates.

## Value Of Information

```rust
use higher_graphen_core::{Confidence, Id};
use higher_graphen_reasoning::uncertainty::{
    score_information_gain, InformationGainOptions, ObservationAction,
    UncertaintyMeasure, UncertaintyState,
};

let state = UncertaintyState::new(
    Id::new("claim:api-risk")?,
    Confidence::new(0.4)?,
    Confidence::new(0.5)?,
    UncertaintyMeasure::BinaryEntropy,
);
let action = ObservationAction::new(
    Id::new("observe:logs")?,
    [Id::new("claim:api-risk")?],
    "logs",
    0.05,
)?
.with_expected_posterior_confidence(Confidence::new(0.85)?);

let report = score_information_gain(&state, &[action], &InformationGainOptions::new())?;
```

Use `EvidenceLikelihoodModel` or `posterior_from_likelihood` only when an
explicit likelihood model is available.

## Order And Lattice

```rust
use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_structure::space::{FiniteOrderRelationSet, OrderRelation};

let relation = OrderRelation::new(
    Id::new("order:a-b")?,
    Id::new("space:req")?,
    "refines",
    Id::new("req:a")?,
    Id::new("req:b")?,
)?
.with_review_status(ReviewStatus::Accepted);

let set = FiniteOrderRelationSet::new(
    Id::new("order:set")?,
    Id::new("space:req")?,
    "refines",
    [Id::new("req:a")?, Id::new("req:b")?],
)?
.with_relation(relation);

let report = set.accepted_relations().analyze()?;
```

The report includes selected relation identifiers, incomparable pairs,
antisymmetry violations, and meet/join candidates.

## Model Checking

```rust
use higher_graphen_core::Id;
use higher_graphen_reasoning::model_checking::{
    check_temporal_property, ModelCheckingOptions, RequiredEventQuery, TemporalCheckQuery,
};

let query = RequiredEventQuery::new(
    Id::new("space:workflow")?,
    [Id::new("state:start")?],
    ["transition:publish".to_owned()],
)
.with_options(ModelCheckingOptions::new().with_max_depth(5));

let report = check_temporal_property(
    &TemporalCheckQuery::RequiredEventualTransition(query),
    &store,
)?;
```

Reports distinguish bounded unknown results from exhaustive finite passes.

## Diagram Construction

```rust
use higher_graphen_core::Id;
use higher_graphen_structure::morphism::{
    check_diagram_requirements, DiagramCommutativityRequirement,
};

let requirement = DiagramCommutativityRequirement::new(
    Id::new("requirement:square")?,
    vec![direct_morphism],
    vec![left_leg, right_leg],
);

let report = check_diagram_requirements(Id::new("diagram:integration")?, &[requirement]);
```

Pullback and pushout APIs produce candidates. They do not accept equivalence
claims, quotient losses, or invariant preservation automatically.
