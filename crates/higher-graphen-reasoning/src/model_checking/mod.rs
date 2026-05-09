//! Bounded finite-state model checking substrate for HigherGraphen.
//!
//! The kernel treats cells in a [`higher_graphen_structure::space::Space`] as finite states
//! and incidences as transitions. It performs deterministic bounded
//! reachability checks for forbidden states and returns witness paths or
//! frontier reports without depending on an external solver.

use higher_graphen_core::{CoreError, Id, Result};
use higher_graphen_structure::space::{
    GraphPath, InMemorySpaceStore, Incidence, IncidenceOrientation, PathStep, TraversalDirection,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

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

impl<'a> ModelChecker<'a> {
    /// Creates a checker over the supplied finite space store.
    #[must_use]
    pub fn new(store: &'a InMemorySpaceStore) -> Self {
        Self { store }
    }

    /// Checks whether any forbidden state is reachable from the query initial states.
    pub fn check(&self, query: &ModelCheckingQuery) -> Result<ModelCheckingReport> {
        let normalized = NormalizedQuery::try_from_query(self.store, query)?;
        let mut search = Search::new(self.store, normalized);
        Ok(search.run())
    }

    /// Checks whether a required transition eventually appears on an explored path.
    pub fn check_required_event(&self, query: &RequiredEventQuery) -> Result<TemporalCheckReport> {
        let normalized = NormalizedTemporalQuery::try_from_required_event(self.store, query)?;
        Ok(TemporalSearch::new(self.store, normalized).run_required_event())
    }

    /// Checks whether after-states are always preceded by a before-state.
    pub fn check_always_before(&self, query: &AlwaysBeforeQuery) -> Result<TemporalCheckReport> {
        let normalized = NormalizedTemporalQuery::try_from_always_before(self.store, query)?;
        Ok(TemporalSearch::new(self.store, normalized).run_always_before())
    }

    /// Checks whether reachable dead ends are limited to terminal states.
    pub fn check_dead_ends(&self, query: &DeadEndQuery) -> Result<TemporalCheckReport> {
        let normalized = NormalizedTemporalQuery::try_from_dead_end(self.store, query)?;
        Ok(TemporalSearch::new(self.store, normalized).run_dead_ends())
    }
}

/// Checks whether any forbidden state is reachable from the query initial states.
pub fn check_model(
    query: &ModelCheckingQuery,
    store: &InMemorySpaceStore,
) -> Result<ModelCheckingReport> {
    ModelChecker::new(store).check(query)
}

/// Checks whether a required transition eventually appears on an explored path.
pub fn check_required_event(
    query: &RequiredEventQuery,
    store: &InMemorySpaceStore,
) -> Result<TemporalCheckReport> {
    ModelChecker::new(store).check_required_event(query)
}

/// Checks whether after-states are always preceded by a before-state.
pub fn check_always_before(
    query: &AlwaysBeforeQuery,
    store: &InMemorySpaceStore,
) -> Result<TemporalCheckReport> {
    ModelChecker::new(store).check_always_before(query)
}

/// Checks whether reachable dead ends are limited to terminal states.
pub fn check_dead_ends(
    query: &DeadEndQuery,
    store: &InMemorySpaceStore,
) -> Result<TemporalCheckReport> {
    ModelChecker::new(store).check_dead_ends(query)
}

#[derive(Clone, Debug)]
struct NormalizedQuery {
    space_id: Id,
    initial_cell_ids: Vec<Id>,
    forbidden_cell_ids: Vec<Id>,
    direction: TraversalDirection,
    relation_types: BTreeSet<String>,
    max_depth: Option<usize>,
    treat_forbidden_initial_as_unsafe: bool,
}

impl NormalizedQuery {
    fn try_from_query(store: &InMemorySpaceStore, query: &ModelCheckingQuery) -> Result<Self> {
        let initial_cell_ids = normalized_ids("initial_cell_ids", &query.initial_cell_ids)?;
        let forbidden_cell_ids = normalized_ids("forbidden_cell_ids", &query.forbidden_cell_ids)?;
        let relation_types = normalize_relation_types(&query.options.relation_types)?;

        require_space(store, &query.space_id)?;
        for cell_id in &initial_cell_ids {
            require_cell_in_space(store, "initial_cell_ids", cell_id, &query.space_id)?;
        }
        for cell_id in &forbidden_cell_ids {
            require_cell_in_space(store, "forbidden_cell_ids", cell_id, &query.space_id)?;
        }

        Ok(Self {
            space_id: query.space_id.clone(),
            initial_cell_ids,
            forbidden_cell_ids,
            direction: query.options.direction,
            relation_types,
            max_depth: query.options.max_depth,
            treat_forbidden_initial_as_unsafe: query.options.treat_forbidden_initial_as_unsafe,
        })
    }

    fn allows_relation(&self, relation_type: &str) -> bool {
        self.relation_types.is_empty() || self.relation_types.contains(relation_type)
    }
}

#[derive(Clone, Debug)]
struct NormalizedTemporalQuery {
    space_id: Id,
    initial_cell_ids: Vec<Id>,
    required_relation_types: BTreeSet<String>,
    before_cell_ids: BTreeSet<Id>,
    after_cell_ids: BTreeSet<Id>,
    terminal_cell_ids: BTreeSet<Id>,
    direction: TraversalDirection,
    relation_types: BTreeSet<String>,
    max_depth: Option<usize>,
}

impl NormalizedTemporalQuery {
    fn try_from_required_event(
        store: &InMemorySpaceStore,
        query: &RequiredEventQuery,
    ) -> Result<Self> {
        let initial_cell_ids = normalized_ids("initial_cell_ids", &query.initial_cell_ids)?;
        let required_relation_types = normalize_relation_types(&query.required_relation_types)?;
        if required_relation_types.is_empty() {
            return Err(malformed(
                "required_relation_types",
                "value must include at least one required relation type",
            ));
        }
        require_space(store, &query.space_id)?;
        for cell_id in &initial_cell_ids {
            require_cell_in_space(store, "initial_cell_ids", cell_id, &query.space_id)?;
        }
        Ok(Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            required_relation_types,
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
        )?)
    }

    fn try_from_always_before(
        store: &InMemorySpaceStore,
        query: &AlwaysBeforeQuery,
    ) -> Result<Self> {
        let initial_cell_ids = normalized_ids("initial_cell_ids", &query.initial_cell_ids)?;
        let before_cell_ids = normalized_ids("before_cell_ids", &query.before_cell_ids)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        let after_cell_ids = normalized_ids("after_cell_ids", &query.after_cell_ids)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        require_space(store, &query.space_id)?;
        for cell_id in initial_cell_ids
            .iter()
            .chain(before_cell_ids.iter())
            .chain(after_cell_ids.iter())
        {
            require_cell_in_space(store, "temporal_cell_ids", cell_id, &query.space_id)?;
        }
        Ok(Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            BTreeSet::new(),
            before_cell_ids,
            after_cell_ids,
            BTreeSet::new(),
        )?)
    }

    fn try_from_dead_end(store: &InMemorySpaceStore, query: &DeadEndQuery) -> Result<Self> {
        let initial_cell_ids = normalized_ids("initial_cell_ids", &query.initial_cell_ids)?;
        let terminal_cell_ids = query
            .terminal_cell_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        require_space(store, &query.space_id)?;
        for cell_id in initial_cell_ids.iter().chain(terminal_cell_ids.iter()) {
            require_cell_in_space(store, "temporal_cell_ids", cell_id, &query.space_id)?;
        }
        Ok(Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            terminal_cell_ids,
        )?)
    }

    fn from_parts(
        space_id: Id,
        initial_cell_ids: Vec<Id>,
        options: ModelCheckingOptions,
        required_relation_types: BTreeSet<String>,
        before_cell_ids: BTreeSet<Id>,
        after_cell_ids: BTreeSet<Id>,
        terminal_cell_ids: BTreeSet<Id>,
    ) -> Result<Self> {
        Ok(Self {
            space_id,
            initial_cell_ids,
            required_relation_types,
            before_cell_ids,
            after_cell_ids,
            terminal_cell_ids,
            direction: options.direction,
            relation_types: normalize_relation_types(&options.relation_types)?,
            max_depth: options.max_depth,
        })
    }

    fn allows_relation(&self, relation_type: &str) -> bool {
        self.relation_types.is_empty() || self.relation_types.contains(relation_type)
    }
}

struct Search<'a> {
    store: &'a InMemorySpaceStore,
    query: NormalizedQuery,
    queue: VecDeque<GraphPath>,
    visited: BTreeSet<Id>,
    visited_cell_ids: Vec<Id>,
    frontier_cell_ids: BTreeSet<Id>,
}

impl<'a> Search<'a> {
    fn new(store: &'a InMemorySpaceStore, query: NormalizedQuery) -> Self {
        Self {
            store,
            query,
            queue: VecDeque::new(),
            visited: BTreeSet::new(),
            visited_cell_ids: Vec::new(),
            frontier_cell_ids: BTreeSet::new(),
        }
    }

    fn run(&mut self) -> ModelCheckingReport {
        if self.query.treat_forbidden_initial_as_unsafe {
            if let Some(witness) = self.initial_forbidden_witness() {
                return self.unsafe_report(witness);
            }
        }

        self.seed_initial_states();
        while let Some(path) = self.queue.pop_front() {
            if self.reached_depth_bound(&path) {
                self.frontier_cell_ids.insert(path.end_cell_id.clone());
                continue;
            }

            for step in self.neighbor_steps(&path.end_cell_id) {
                if !self.visited.insert(step.to_cell_id.clone()) {
                    continue;
                }
                let next_path = append_path(&path, step);
                self.visited_cell_ids.push(next_path.end_cell_id.clone());
                if self.is_forbidden(&next_path.end_cell_id) {
                    return self.unsafe_report(ForbiddenWitness {
                        forbidden_cell_id: next_path.end_cell_id.clone(),
                        path: next_path,
                    });
                }
                self.queue.push_back(next_path);
            }
        }

        if self.query.max_depth.is_some() && !self.frontier_cell_ids.is_empty() {
            return self.unknown_report();
        }

        self.safe_report()
    }

    fn initial_forbidden_witness(&self) -> Option<ForbiddenWitness> {
        self.query
            .initial_cell_ids
            .iter()
            .find(|cell_id| self.is_forbidden(cell_id))
            .map(|cell_id| ForbiddenWitness {
                forbidden_cell_id: cell_id.clone(),
                path: GraphPath::new(cell_id.clone()),
            })
    }

    fn seed_initial_states(&mut self) {
        for cell_id in &self.query.initial_cell_ids {
            if self.visited.insert(cell_id.clone()) {
                self.visited_cell_ids.push(cell_id.clone());
                self.queue.push_back(GraphPath::new(cell_id.clone()));
            }
        }
    }

    fn reached_depth_bound(&self, path: &GraphPath) -> bool {
        self.query
            .max_depth
            .is_some_and(|max_depth| path.depth() >= max_depth)
    }

    fn neighbor_steps(&self, current_cell_id: &Id) -> Vec<PathStep> {
        let Some(space) = self.store.space(&self.query.space_id) else {
            return Vec::new();
        };
        let mut steps = space
            .incidence_ids
            .iter()
            .filter_map(|incidence_id| self.store.incidence(incidence_id))
            .filter(|incidence| self.query.allows_relation(&incidence.relation_type))
            .filter_map(|incidence| self.step_from_incidence(current_cell_id, incidence))
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| {
            (
                &left.incidence_id,
                &left.to_cell_id,
                &left.from_cell_id,
                &left.relation_type,
            )
                .cmp(&(
                    &right.incidence_id,
                    &right.to_cell_id,
                    &right.from_cell_id,
                    &right.relation_type,
                ))
        });
        steps
    }

    fn step_from_incidence(&self, current_cell_id: &Id, incidence: &Incidence) -> Option<PathStep> {
        let to_cell_id = next_cell_id(current_cell_id, incidence, self.query.direction)?;
        Some(PathStep {
            from_cell_id: current_cell_id.clone(),
            incidence_id: incidence.id.clone(),
            to_cell_id,
            relation_type: incidence.relation_type.clone(),
        })
    }

    fn is_forbidden(&self, cell_id: &Id) -> bool {
        self.query.forbidden_cell_ids.binary_search(cell_id).is_ok()
    }

    fn unsafe_report(&self, witness: ForbiddenWitness) -> ModelCheckingReport {
        let visited_cell_ids = if self.visited_cell_ids.is_empty() {
            witness.path.cell_ids()
        } else {
            self.visited_cell_ids.clone()
        };

        ModelCheckingReport {
            space_id: self.query.space_id.clone(),
            initial_cell_ids: self.query.initial_cell_ids.clone(),
            forbidden_cell_ids: self.query.forbidden_cell_ids.clone(),
            status: SafetyStatus::Unsafe,
            max_depth: self.query.max_depth,
            witness: Some(witness),
            visited_cell_ids,
            frontier_cell_ids: Vec::new(),
            unreachable_forbidden_cell_ids: Vec::new(),
            unknown_reason: None,
        }
    }

    fn unknown_report(&self) -> ModelCheckingReport {
        ModelCheckingReport {
            space_id: self.query.space_id.clone(),
            initial_cell_ids: self.query.initial_cell_ids.clone(),
            forbidden_cell_ids: self.query.forbidden_cell_ids.clone(),
            status: SafetyStatus::Unknown,
            max_depth: self.query.max_depth,
            witness: None,
            visited_cell_ids: self.visited_cell_ids.clone(),
            frontier_cell_ids: self.frontier_cell_ids.iter().cloned().collect(),
            unreachable_forbidden_cell_ids: Vec::new(),
            unknown_reason: Some(
                "depth bound exhausted before proving no forbidden state is reachable".to_owned(),
            ),
        }
    }

    fn safe_report(&self) -> ModelCheckingReport {
        ModelCheckingReport {
            space_id: self.query.space_id.clone(),
            initial_cell_ids: self.query.initial_cell_ids.clone(),
            forbidden_cell_ids: self.query.forbidden_cell_ids.clone(),
            status: SafetyStatus::Safe,
            max_depth: self.query.max_depth,
            witness: None,
            visited_cell_ids: self.visited_cell_ids.clone(),
            frontier_cell_ids: Vec::new(),
            unreachable_forbidden_cell_ids: self.query.forbidden_cell_ids.clone(),
            unknown_reason: None,
        }
    }
}

struct TemporalSearch<'a> {
    store: &'a InMemorySpaceStore,
    query: NormalizedTemporalQuery,
    queue: VecDeque<GraphPath>,
    visited: BTreeSet<Id>,
    visited_cell_ids: Vec<Id>,
    frontier_cell_ids: BTreeSet<Id>,
}

impl<'a> TemporalSearch<'a> {
    fn new(store: &'a InMemorySpaceStore, query: NormalizedTemporalQuery) -> Self {
        Self {
            store,
            query,
            queue: VecDeque::new(),
            visited: BTreeSet::new(),
            visited_cell_ids: Vec::new(),
            frontier_cell_ids: BTreeSet::new(),
        }
    }

    fn run_required_event(mut self) -> TemporalCheckReport {
        self.seed_initial_states();
        while let Some(path) = self.queue.pop_front() {
            if self.reached_depth_bound(&path) {
                self.frontier_cell_ids.insert(path.end_cell_id.clone());
                continue;
            }
            for step in self.neighbor_steps(&path.end_cell_id) {
                let next_path = append_path(&path, step.clone());
                if self
                    .query
                    .required_relation_types
                    .contains(&step.relation_type)
                {
                    self.visited_cell_ids.push(next_path.end_cell_id.clone());
                    return self.report(
                        TemporalCheckStatus::Satisfied,
                        Some(next_path),
                        Vec::new(),
                    );
                }
                if self.visited.insert(step.to_cell_id.clone()) {
                    self.visited_cell_ids.push(step.to_cell_id.clone());
                    self.queue.push_back(next_path);
                }
            }
        }
        if self.query.max_depth.is_some() && !self.frontier_cell_ids.is_empty() {
            return self.report(
                TemporalCheckStatus::Unknown,
                None,
                vec![TemporalObstruction {
                    obstruction_type: TemporalObstructionType::StateSpaceLimitExceeded,
                    reason: "depth bound exhausted before finding a required event".to_owned(),
                }],
            );
        }
        self.report(
            TemporalCheckStatus::Violated,
            None,
            vec![TemporalObstruction {
                obstruction_type: TemporalObstructionType::RequiredEventMissing,
                reason: "no required event was reachable from the initial states".to_owned(),
            }],
        )
    }

    fn run_always_before(mut self) -> TemporalCheckReport {
        self.seed_initial_states();
        while let Some(path) = self.queue.pop_front() {
            if self.path_violates_always_before(&path) {
                return self.report(
                    TemporalCheckStatus::Violated,
                    Some(path),
                    vec![TemporalObstruction {
                        obstruction_type: TemporalObstructionType::OrderingViolation,
                        reason: "an after-state was reached before any before-state".to_owned(),
                    }],
                );
            }
            if self.reached_depth_bound(&path) {
                self.frontier_cell_ids.insert(path.end_cell_id.clone());
                continue;
            }
            for step in self.neighbor_steps(&path.end_cell_id) {
                if self.visited.insert(step.to_cell_id.clone()) {
                    let next_path = append_path(&path, step);
                    self.visited_cell_ids.push(next_path.end_cell_id.clone());
                    self.queue.push_back(next_path);
                }
            }
        }
        if self.query.max_depth.is_some() && !self.frontier_cell_ids.is_empty() {
            return self.report(
                TemporalCheckStatus::Unknown,
                None,
                vec![TemporalObstruction {
                    obstruction_type: TemporalObstructionType::StateSpaceLimitExceeded,
                    reason: "depth bound exhausted before proving always-before ordering"
                        .to_owned(),
                }],
            );
        }
        self.report(TemporalCheckStatus::Satisfied, None, Vec::new())
    }

    fn run_dead_ends(mut self) -> TemporalCheckReport {
        self.seed_initial_states();
        while let Some(path) = self.queue.pop_front() {
            if self.reached_depth_bound(&path) {
                self.frontier_cell_ids.insert(path.end_cell_id.clone());
                continue;
            }
            let steps = self.neighbor_steps(&path.end_cell_id);
            if steps.is_empty() && !self.query.terminal_cell_ids.contains(&path.end_cell_id) {
                return self.report(
                    TemporalCheckStatus::Violated,
                    Some(path),
                    vec![TemporalObstruction {
                        obstruction_type: TemporalObstructionType::DeadEndState,
                        reason: "a reachable non-terminal state has no outgoing transition"
                            .to_owned(),
                    }],
                );
            }
            for step in steps {
                if self.visited.insert(step.to_cell_id.clone()) {
                    let next_path = append_path(&path, step);
                    self.visited_cell_ids.push(next_path.end_cell_id.clone());
                    self.queue.push_back(next_path);
                }
            }
        }
        if self.query.max_depth.is_some() && !self.frontier_cell_ids.is_empty() {
            return self.report(
                TemporalCheckStatus::Unknown,
                None,
                vec![TemporalObstruction {
                    obstruction_type: TemporalObstructionType::StateSpaceLimitExceeded,
                    reason: "depth bound exhausted before proving absence of dead ends".to_owned(),
                }],
            );
        }
        self.report(TemporalCheckStatus::Satisfied, None, Vec::new())
    }

    fn seed_initial_states(&mut self) {
        for cell_id in &self.query.initial_cell_ids {
            if self.visited.insert(cell_id.clone()) {
                self.visited_cell_ids.push(cell_id.clone());
                self.queue.push_back(GraphPath::new(cell_id.clone()));
            }
        }
    }

    fn reached_depth_bound(&self, path: &GraphPath) -> bool {
        self.query
            .max_depth
            .is_some_and(|max_depth| path.depth() >= max_depth)
    }

    fn neighbor_steps(&self, current_cell_id: &Id) -> Vec<PathStep> {
        let Some(space) = self.store.space(&self.query.space_id) else {
            return Vec::new();
        };
        let mut steps = space
            .incidence_ids
            .iter()
            .filter_map(|incidence_id| self.store.incidence(incidence_id))
            .filter(|incidence| self.query.allows_relation(&incidence.relation_type))
            .filter_map(|incidence| {
                let to_cell_id = next_cell_id(current_cell_id, incidence, self.query.direction)?;
                Some(PathStep {
                    from_cell_id: current_cell_id.clone(),
                    incidence_id: incidence.id.clone(),
                    to_cell_id,
                    relation_type: incidence.relation_type.clone(),
                })
            })
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| {
            (
                &left.incidence_id,
                &left.to_cell_id,
                &left.from_cell_id,
                &left.relation_type,
            )
                .cmp(&(
                    &right.incidence_id,
                    &right.to_cell_id,
                    &right.from_cell_id,
                    &right.relation_type,
                ))
        });
        steps
    }

    fn path_violates_always_before(&self, path: &GraphPath) -> bool {
        let mut seen_before = false;
        for cell_id in path.cell_ids() {
            if self.query.before_cell_ids.contains(&cell_id) {
                seen_before = true;
            }
            if self.query.after_cell_ids.contains(&cell_id) && !seen_before {
                return true;
            }
        }
        false
    }

    fn report(
        &self,
        status: TemporalCheckStatus,
        witness: Option<GraphPath>,
        obstructions: Vec<TemporalObstruction>,
    ) -> TemporalCheckReport {
        TemporalCheckReport {
            space_id: self.query.space_id.clone(),
            status,
            max_depth: self.query.max_depth,
            witness,
            visited_cell_ids: self.visited_cell_ids.clone(),
            frontier_cell_ids: self.frontier_cell_ids.iter().cloned().collect(),
            obstructions,
        }
    }
}

fn append_path(path: &GraphPath, step: PathStep) -> GraphPath {
    let mut next_path = path.clone();
    next_path.end_cell_id = step.to_cell_id.clone();
    next_path.steps.push(step);
    next_path
}

fn next_cell_id(
    current_cell_id: &Id,
    incidence: &Incidence,
    direction: TraversalDirection,
) -> Option<Id> {
    match incidence.orientation {
        IncidenceOrientation::Directed => {
            directed_next_cell_id(current_cell_id, incidence, direction)
        }
        IncidenceOrientation::Undirected => undirected_next_cell_id(current_cell_id, incidence),
    }
}

fn directed_next_cell_id(
    current_cell_id: &Id,
    incidence: &Incidence,
    direction: TraversalDirection,
) -> Option<Id> {
    match direction {
        TraversalDirection::Outgoing if &incidence.from_cell_id == current_cell_id => {
            Some(incidence.to_cell_id.clone())
        }
        TraversalDirection::Incoming if &incidence.to_cell_id == current_cell_id => {
            Some(incidence.from_cell_id.clone())
        }
        TraversalDirection::Both => undirected_next_cell_id(current_cell_id, incidence),
        _ => None,
    }
}

fn undirected_next_cell_id(current_cell_id: &Id, incidence: &Incidence) -> Option<Id> {
    if &incidence.from_cell_id == current_cell_id {
        Some(incidence.to_cell_id.clone())
    } else if &incidence.to_cell_id == current_cell_id {
        Some(incidence.from_cell_id.clone())
    } else {
        None
    }
}

fn require_space(store: &InMemorySpaceStore, space_id: &Id) -> Result<()> {
    if store.space(space_id).is_some() {
        Ok(())
    } else {
        Err(malformed(
            "space_id",
            format!("identifier {space_id} does not exist in the store"),
        ))
    }
}

fn require_cell_in_space(
    store: &InMemorySpaceStore,
    field: &str,
    cell_id: &Id,
    space_id: &Id,
) -> Result<()> {
    let cell = store
        .cell(cell_id)
        .ok_or_else(|| malformed(field, format!("identifier {cell_id} does not exist")))?;
    if &cell.space_id == space_id {
        Ok(())
    } else {
        Err(malformed(
            field,
            format!("identifier {cell_id} belongs to space {}", cell.space_id),
        ))
    }
}

fn normalized_ids(field: &str, values: &[Id]) -> Result<Vec<Id>> {
    if values.is_empty() {
        return Err(malformed(
            field,
            "value must include at least one cell identifier",
        ));
    }
    Ok(values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

fn normalize_relation_types(values: &[String]) -> Result<BTreeSet<String>> {
    values
        .iter()
        .map(|value| {
            let normalized = value.trim().to_owned();
            if normalized.is_empty() {
                Err(malformed(
                    "relation_types",
                    "value must not be empty after trimming",
                ))
            } else {
                Ok(normalized)
            }
        })
        .collect()
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_structure::space::{Cell, Incidence, Space};

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
}
