//! Bounded finite-state model checking substrate for HigherGraphen.
//!
//! The kernel treats cells in a [`higher_graphen_structure::space::Space`] as finite states
//! and incidences as transitions. It performs deterministic bounded
//! reachability checks for forbidden states and returns witness paths or
//! frontier reports without depending on an external solver.

use higher_graphen_core::{CoreError, Id, Result};
use higher_graphen_structure::space::{GraphPath, InMemorySpaceStore, TraversalDirection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Traversal controls used by the bounded model checker.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCheckingOptions {
    /// Direction used for directed transition incidences.
    pub direction: TraversalDirection,
    /// Optional allowed transition relation types. Empty means any relation type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_types: Vec<String>,
    /// Optional maximum number of transitions in explored witness paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Whether an initial state that is also forbidden is immediately unsafe.
    pub treat_forbidden_initial_as_unsafe: bool,
}

impl Default for ModelCheckingOptions {
    fn default() -> Self {
        Self {
            direction: TraversalDirection::Outgoing,
            relation_types: Vec::new(),
            max_depth: None,
            treat_forbidden_initial_as_unsafe: true,
        }
    }
}

impl ModelCheckingOptions {
    /// Creates default outgoing, unbounded model-checking options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns these options with a traversal direction.
    #[must_use]
    pub fn in_direction(mut self, direction: TraversalDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns these options with an allowed relation type appended.
    #[must_use]
    pub fn with_relation_type(mut self, relation_type: impl Into<String>) -> Self {
        self.relation_types
            .push(relation_type.into().trim().to_owned());
        self
    }

    /// Returns these options with a maximum witness depth.
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Returns these options with explicit initial-forbidden handling.
    #[must_use]
    pub fn with_forbidden_initial_handling(mut self, treat_as_unsafe: bool) -> Self {
        self.treat_forbidden_initial_as_unsafe = treat_as_unsafe;
        self
    }
}

/// Query asking whether any forbidden state is reachable from finite initial states.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCheckingQuery {
    /// Space that owns the states and transition incidences.
    pub space_id: Id,
    /// Initial state cells.
    pub initial_cell_ids: Vec<Id>,
    /// Forbidden state cells.
    pub forbidden_cell_ids: Vec<Id>,
    /// Bounded traversal controls.
    pub options: ModelCheckingOptions,
}

impl ModelCheckingQuery {
    /// Creates a query with default model-checking options.
    #[must_use]
    pub fn new<I, F>(space_id: Id, initial_cell_ids: I, forbidden_cell_ids: F) -> Self
    where
        I: IntoIterator<Item = Id>,
        F: IntoIterator<Item = Id>,
    {
        Self {
            space_id,
            initial_cell_ids: initial_cell_ids.into_iter().collect(),
            forbidden_cell_ids: forbidden_cell_ids.into_iter().collect(),
            options: ModelCheckingOptions::default(),
        }
    }

    /// Returns this query with explicit model-checking options.
    #[must_use]
    pub fn with_options(mut self, options: ModelCheckingOptions) -> Self {
        self.options = options;
        self
    }
}

/// Safety classification produced by a bounded model-checking run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyStatus {
    /// No forbidden state is reachable in the finite graph under the supplied options.
    Safe,
    /// A forbidden state is reachable and a witness path is available.
    Unsafe,
    /// The depth bound was exhausted before the checker could prove safety.
    Unknown,
}

/// Witness that a forbidden state is reachable.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenWitness {
    /// Forbidden state reached by the witness path.
    pub forbidden_cell_id: Id,
    /// State/transition path from an initial state to the forbidden state.
    pub path: GraphPath,
}

/// Classification produced by bounded temporal-property checks.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalCheckStatus {
    /// The bounded finite exploration found a satisfying witness.
    Satisfied,
    /// The bounded finite exploration found a counterexample.
    Violated,
    /// The depth bound was exhausted before the property could be decided.
    Unknown,
}

/// Stable obstruction category emitted by bounded temporal checks.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalObstructionType {
    /// No required event was found on any explored path.
    RequiredEventMissing,
    /// An after-state was reached before any before-state.
    OrderingViolation,
    /// A reachable non-terminal state has no outgoing transition.
    DeadEndState,
    /// The configured depth bound prevented a conclusive result.
    StateSpaceLimitExceeded,
}

/// Bounded temporal property kinds supported by the finite model-checking API.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalPropertyKind {
    /// No forbidden state should be reachable.
    ForbiddenReachability,
    /// At least one selected transition should eventually occur.
    RequiredEventualTransition,
    /// Any after-state must be preceded by a before-state on the trace.
    AlwaysBefore,
    /// Reachable dead-end states must be explicitly declared terminal.
    NoDeadEndExceptTerminal,
}

/// Minimal reviewable temporal property descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalProperty {
    /// Stable property identifier.
    pub id: Id,
    /// Property kind.
    pub property_kind: TemporalPropertyKind,
    /// Human-readable bounded scope.
    pub scope: String,
}

/// Finite state label used to construct label-based temporal checks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateLabel {
    /// Stable label identifier.
    pub id: Id,
    /// Cells carrying this label.
    pub cell_ids: Vec<Id>,
}

impl StateLabel {
    /// Creates a state label.
    #[must_use]
    pub fn new(id: Id, cell_ids: impl IntoIterator<Item = Id>) -> Self {
        Self {
            id,
            cell_ids: cell_ids.into_iter().collect(),
        }
    }
}

/// Finite state-label set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateLabelSet {
    /// Labels in the state model.
    pub labels: Vec<StateLabel>,
}

impl StateLabelSet {
    /// Creates a state-label set.
    #[must_use]
    pub fn new(labels: impl IntoIterator<Item = StateLabel>) -> Self {
        Self {
            labels: labels.into_iter().collect(),
        }
    }

    /// Returns cells carrying the requested label.
    pub fn cell_ids_for(&self, label_id: &Id) -> Result<Vec<Id>> {
        self.labels
            .iter()
            .find(|label| &label.id == label_id)
            .map(|label| {
                label
                    .cell_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            })
            .ok_or_else(|| {
                malformed(
                    "label_id",
                    format!("identifier {label_id} does not exist in the state label set"),
                )
            })
    }
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

/// Structured temporal-check obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalObstruction {
    /// Obstruction category.
    pub obstruction_type: TemporalObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
}

/// Query requiring at least one selected transition to eventually occur.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredEventQuery {
    /// Space that owns the states and transition incidences.
    pub space_id: Id,
    /// Initial state cells.
    pub initial_cell_ids: Vec<Id>,
    /// Required transition relation types.
    pub required_relation_types: Vec<String>,
    /// Bounded traversal controls.
    pub options: ModelCheckingOptions,
}

impl RequiredEventQuery {
    /// Creates a required-event query.
    #[must_use]
    pub fn new<R>(
        space_id: Id,
        initial_cell_ids: impl IntoIterator<Item = Id>,
        required_relation_types: R,
    ) -> Self
    where
        R: IntoIterator<Item = String>,
    {
        Self {
            space_id,
            initial_cell_ids: initial_cell_ids.into_iter().collect(),
            required_relation_types: required_relation_types.into_iter().collect(),
            options: ModelCheckingOptions::default(),
        }
    }

    /// Returns this query with explicit traversal controls.
    #[must_use]
    pub fn with_options(mut self, options: ModelCheckingOptions) -> Self {
        self.options = options;
        self
    }
}

/// Query requiring all after-states to be preceded by a before-state on every explored path.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AlwaysBeforeQuery {
    /// Space that owns the states and transition incidences.
    pub space_id: Id,
    /// Initial state cells.
    pub initial_cell_ids: Vec<Id>,
    /// States that must occur before any after-state.
    pub before_cell_ids: Vec<Id>,
    /// States that must not occur before a before-state.
    pub after_cell_ids: Vec<Id>,
    /// Bounded traversal controls.
    pub options: ModelCheckingOptions,
}

impl AlwaysBeforeQuery {
    /// Creates an always-before query.
    #[must_use]
    pub fn new<B, A>(
        space_id: Id,
        initial_cell_ids: impl IntoIterator<Item = Id>,
        before_cell_ids: B,
        after_cell_ids: A,
    ) -> Self
    where
        B: IntoIterator<Item = Id>,
        A: IntoIterator<Item = Id>,
    {
        Self {
            space_id,
            initial_cell_ids: initial_cell_ids.into_iter().collect(),
            before_cell_ids: before_cell_ids.into_iter().collect(),
            after_cell_ids: after_cell_ids.into_iter().collect(),
            options: ModelCheckingOptions::default(),
        }
    }

    /// Returns this query with explicit traversal controls.
    #[must_use]
    pub fn with_options(mut self, options: ModelCheckingOptions) -> Self {
        self.options = options;
        self
    }
}

/// Query forbidding reachable dead ends except declared terminal states.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeadEndQuery {
    /// Space that owns the states and transition incidences.
    pub space_id: Id,
    /// Initial state cells.
    pub initial_cell_ids: Vec<Id>,
    /// States allowed to have no outgoing transition.
    pub terminal_cell_ids: Vec<Id>,
    /// Bounded traversal controls.
    pub options: ModelCheckingOptions,
}

impl DeadEndQuery {
    /// Creates a dead-end query.
    #[must_use]
    pub fn new(
        space_id: Id,
        initial_cell_ids: impl IntoIterator<Item = Id>,
        terminal_cell_ids: impl IntoIterator<Item = Id>,
    ) -> Self {
        Self {
            space_id,
            initial_cell_ids: initial_cell_ids.into_iter().collect(),
            terminal_cell_ids: terminal_cell_ids.into_iter().collect(),
            options: ModelCheckingOptions::default(),
        }
    }

    /// Returns this query with explicit traversal controls.
    #[must_use]
    pub fn with_options(mut self, options: ModelCheckingOptions) -> Self {
        self.options = options;
        self
    }
}

/// Result of a bounded temporal-property check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalCheckReport {
    /// Space that was checked.
    pub space_id: Id,
    /// Final temporal-property classification.
    pub status: TemporalCheckStatus,
    /// True when the finite reachable state space was exhausted under the supplied options.
    pub exhaustive: bool,
    /// Maximum explored witness depth when a bound was supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Satisfying trace or counterexample trace, depending on property kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<GraphPath>,
    /// States visited in deterministic breadth-first order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visited_cell_ids: Vec<Id>,
    /// Boundary states at an exhausted depth bound.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier_cell_ids: Vec<Id>,
    /// Obstructions produced by violated or unknown checks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<TemporalObstruction>,
}

/// Unified temporal check query.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "property_kind", rename_all = "snake_case")]
pub enum TemporalCheckQuery {
    /// Required eventual transition query.
    RequiredEventualTransition(RequiredEventQuery),
    /// Always-before query.
    AlwaysBefore(AlwaysBeforeQuery),
    /// Dead-end query.
    NoDeadEndExceptTerminal(DeadEndQuery),
}

impl TemporalCheckReport {
    /// Returns true when the property is satisfied by the bounded check.
    #[must_use]
    pub fn is_satisfied(&self) -> bool {
        self.status == TemporalCheckStatus::Satisfied
    }
}

/// Result of a bounded forbidden-state reachability check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCheckingReport {
    /// Space that was checked.
    pub space_id: Id,
    /// Normalized initial states used for the run.
    pub initial_cell_ids: Vec<Id>,
    /// Normalized forbidden states used for the run.
    pub forbidden_cell_ids: Vec<Id>,
    /// Final safety classification.
    pub status: SafetyStatus,
    /// True when the finite reachable state space was exhausted under the supplied options.
    pub exhaustive: bool,
    /// Maximum explored witness depth when a bound was supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Reachability witness when the result is unsafe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<ForbiddenWitness>,
    /// States visited in deterministic breadth-first order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visited_cell_ids: Vec<Id>,
    /// Boundary states at the exhausted depth bound when the result is unknown.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier_cell_ids: Vec<Id>,
    /// Forbidden states proven unreachable when the result is safe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unreachable_forbidden_cell_ids: Vec<Id>,
    /// Diagnostic reason when the result is unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<String>,
}

impl ModelCheckingReport {
    /// Returns true when the report proves safety.
    #[must_use]
    pub fn is_safe(&self) -> bool {
        self.status == SafetyStatus::Safe
    }

    /// Returns true when the report contains a forbidden-state witness.
    #[must_use]
    pub fn is_unsafe(&self) -> bool {
        self.status == SafetyStatus::Unsafe
    }

    /// Returns true when the supplied bound prevented a conclusive result.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.status == SafetyStatus::Unknown
    }
}

/// Deterministic bounded model checker over an in-memory finite space store.
#[derive(Clone, Copy, Debug)]
pub struct ModelChecker<'a> {
    store: &'a InMemorySpaceStore,
}

mod engine;
mod support;
pub use engine::{
    always_before_query_from_labels, check_always_before, check_dead_ends, check_model,
    check_required_event, check_temporal_property,
};

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_structure::space::{Cell, Incidence, IncidenceOrientation, Space};

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid test id")
    }

    fn seeded_store() -> InMemorySpaceStore {
        let mut store = InMemorySpaceStore::new();
        store
            .insert_space(Space::new(id("space-a"), "Finite state space"))
            .expect("insert space");
        store
    }

    fn finite_state_store() -> InMemorySpaceStore {
        let mut store = seeded_store();
        for cell_id in ["start", "mid", "ok", "bad", "isolated-bad"] {
            store
                .insert_cell(Cell::new(id(cell_id), id("space-a"), 0, "state"))
                .expect("insert state");
        }
        insert_transition(&mut store, "transition-start-mid", "start", "mid");
        insert_transition(&mut store, "transition-mid-bad", "mid", "bad");
        insert_transition(&mut store, "transition-start-ok", "start", "ok");
        store
    }

    fn insert_transition(
        store: &mut InMemorySpaceStore,
        incidence_id: &str,
        from_cell_id: &str,
        to_cell_id: &str,
    ) {
        store
            .insert_incidence(Incidence::new(
                id(incidence_id),
                id("space-a"),
                id(from_cell_id),
                id(to_cell_id),
                "transition",
                IncidenceOrientation::Directed,
            ))
            .expect("insert transition");
    }

    #[test]
    fn unsafe_state_returns_witness_path() {
        let store = finite_state_store();
        let query = ModelCheckingQuery::new(id("space-a"), [id("start")], [id("bad")])
            .with_options(ModelCheckingOptions::new().with_relation_type("transition"));

        let report = ModelChecker::new(&store)
            .check(&query)
            .expect("check model");

        assert!(report.is_unsafe());
        assert_eq!(report.status, SafetyStatus::Unsafe);
        let witness = report.witness.expect("forbidden witness");
        assert_eq!(witness.forbidden_cell_id, id("bad"));
        assert_eq!(
            witness.path.cell_ids(),
            vec![id("start"), id("mid"), id("bad")]
        );
        assert_eq!(
            report.visited_cell_ids,
            vec![id("start"), id("mid"), id("ok"), id("bad")]
        );
    }

    #[test]
    fn safe_report_lists_unreachable_forbidden_states() {
        let store = finite_state_store();
        let query = ModelCheckingQuery::new(id("space-a"), [id("ok")], [id("isolated-bad")]);

        let report = check_model(&query, &store).expect("check model");

        assert!(report.is_safe());
        assert_eq!(report.status, SafetyStatus::Safe);
        assert!(report.exhaustive);
        assert!(report.witness.is_none());
        assert!(report.frontier_cell_ids.is_empty());
        assert_eq!(report.visited_cell_ids, vec![id("ok")]);
        assert_eq!(
            report.unreachable_forbidden_cell_ids,
            vec![id("isolated-bad")]
        );
    }

    #[test]
    fn depth_bound_reports_unknown_frontier() {
        let store = finite_state_store();
        let query = ModelCheckingQuery::new(id("space-a"), [id("start")], [id("bad")])
            .with_options(ModelCheckingOptions::new().with_max_depth(1));

        let report = ModelChecker::new(&store)
            .check(&query)
            .expect("check model");

        assert!(report.is_unknown());
        assert_eq!(report.status, SafetyStatus::Unknown);
        assert!(!report.exhaustive);
        assert_eq!(report.max_depth, Some(1));
        assert!(report.witness.is_none());
        assert_eq!(
            report.visited_cell_ids,
            vec![id("start"), id("mid"), id("ok")]
        );
        assert_eq!(report.frontier_cell_ids, vec![id("mid"), id("ok")]);
        assert!(report.unreachable_forbidden_cell_ids.is_empty());
        assert_eq!(
            report.unknown_reason.as_deref(),
            Some("depth bound exhausted before proving no forbidden state is reachable")
        );
    }

    #[test]
    fn malformed_query_requires_sources_and_forbidden_states() {
        let store = finite_state_store();
        let empty_sources = ModelCheckingQuery::new(id("space-a"), [], [id("bad")]);
        let empty_forbidden = ModelCheckingQuery::new(id("space-a"), [id("start")], []);

        assert_eq!(
            ModelChecker::new(&store)
                .check(&empty_sources)
                .expect_err("empty sources should fail")
                .code(),
            "malformed_field"
        );
        assert_eq!(
            ModelChecker::new(&store)
                .check(&empty_forbidden)
                .expect_err("empty forbidden should fail")
                .code(),
            "malformed_field"
        );
    }

    #[test]
    fn malformed_query_rejects_cells_outside_the_space() {
        let mut store = finite_state_store();
        store
            .insert_space(Space::new(id("space-b"), "Other state space"))
            .expect("insert other space");
        store
            .insert_cell(Cell::new(id("outside"), id("space-b"), 0, "state"))
            .expect("insert outside state");
        let query = ModelCheckingQuery::new(id("space-a"), [id("outside")], [id("bad")]);

        let error = ModelChecker::new(&store)
            .check(&query)
            .expect_err("outside cell should fail");

        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn deterministic_ordering_normalizes_initial_and_forbidden_sets() {
        let mut store = seeded_store();
        for cell_id in ["z-start", "a-start", "bad-z", "bad-a"] {
            store
                .insert_cell(Cell::new(id(cell_id), id("space-a"), 0, "state"))
                .expect("insert state");
        }
        insert_transition(&mut store, "transition-z", "z-start", "bad-z");
        insert_transition(&mut store, "transition-a", "a-start", "bad-a");
        let query = ModelCheckingQuery::new(
            id("space-a"),
            [id("z-start"), id("a-start")],
            [id("bad-z"), id("bad-a")],
        );

        let report = ModelChecker::new(&store)
            .check(&query)
            .expect("check model");

        assert_eq!(report.initial_cell_ids, vec![id("a-start"), id("z-start")]);
        assert_eq!(report.forbidden_cell_ids, vec![id("bad-a"), id("bad-z")]);
        let witness = report.witness.expect("forbidden witness");
        assert_eq!(witness.forbidden_cell_id, id("bad-a"));
        assert_eq!(witness.path.cell_ids(), vec![id("a-start"), id("bad-a")]);
    }

    #[test]
    fn serde_round_trip_preserves_query_and_report_shape() {
        let store = finite_state_store();
        let query = ModelCheckingQuery::new(id("space-a"), [id("start")], [id("bad")])
            .with_options(ModelCheckingOptions::new().with_max_depth(1));

        let query_json = serde_json::to_string(&query).expect("serialize query");
        let decoded_query: ModelCheckingQuery =
            serde_json::from_str(&query_json).expect("deserialize query");
        assert_eq!(decoded_query, query);

        let report = ModelChecker::new(&store)
            .check(&decoded_query)
            .expect("check model");
        let report_json = serde_json::to_string(&report).expect("serialize report");
        let decoded_report: ModelCheckingReport =
            serde_json::from_str(&report_json).expect("deserialize report");

        assert_eq!(decoded_report, report);
    }

    #[test]
    fn required_event_reports_satisfying_transition_or_missing_event() {
        let store = finite_state_store();
        let satisfied =
            RequiredEventQuery::new(id("space-a"), [id("start")], ["transition".to_owned()]);

        let report = ModelChecker::new(&store)
            .check_required_event(&satisfied)
            .expect("check required event");

        assert!(report.is_satisfied());
        assert_eq!(report.status, TemporalCheckStatus::Satisfied);
        assert!(!report.exhaustive);
        assert_eq!(
            report.witness.as_ref().expect("event witness").steps[0].relation_type,
            "transition"
        );

        let missing = RequiredEventQuery::new(id("space-a"), [id("start")], ["publish".to_owned()]);
        let missing_report = check_required_event(&missing, &store).expect("missing event check");
        assert_eq!(missing_report.status, TemporalCheckStatus::Violated);
        assert_eq!(
            missing_report.obstructions[0].obstruction_type,
            TemporalObstructionType::RequiredEventMissing
        );
    }

    #[test]
    fn always_before_reports_ordering_violation() {
        let store = finite_state_store();
        let query = AlwaysBeforeQuery::new(id("space-a"), [id("start")], [id("ok")], [id("bad")]);

        let report = ModelChecker::new(&store)
            .check_always_before(&query)
            .expect("check ordering");

        assert_eq!(report.status, TemporalCheckStatus::Violated);
        assert!(!report.exhaustive);
        assert_eq!(
            report.obstructions[0].obstruction_type,
            TemporalObstructionType::OrderingViolation
        );
        assert_eq!(
            report.witness.expect("counterexample").cell_ids(),
            vec![id("start"), id("mid"), id("bad")]
        );
    }

    #[test]
    fn dead_end_check_allows_declared_terminals_only() {
        let store = finite_state_store();
        let query = DeadEndQuery::new(id("space-a"), [id("start")], [id("bad")]);

        let report = check_dead_ends(&query, &store).expect("check dead ends");

        assert_eq!(report.status, TemporalCheckStatus::Violated);
        assert_eq!(
            report.obstructions[0].obstruction_type,
            TemporalObstructionType::DeadEndState
        );
        assert_eq!(
            report.witness.expect("dead-end witness").end_cell_id,
            id("ok")
        );
    }

    #[test]
    fn unified_temporal_dispatch_and_state_labels_build_queries() {
        let store = finite_state_store();
        let labels = StateLabelSet::new([
            StateLabel::new(id("label/ok"), [id("ok")]),
            StateLabel::new(id("label/bad"), [id("bad")]),
        ]);
        let query = always_before_query_from_labels(
            id("space-a"),
            [id("start")],
            &labels,
            &id("label/ok"),
            &id("label/bad"),
            ModelCheckingOptions::new(),
        )
        .expect("label query");
        let unified = TemporalCheckQuery::AlwaysBefore(query);

        let report = check_temporal_property(&unified, &store).expect("dispatch temporal query");

        assert_eq!(report.status, TemporalCheckStatus::Violated);
        assert_eq!(
            report.obstructions[0].obstruction_type,
            TemporalObstructionType::OrderingViolation
        );
    }
}
