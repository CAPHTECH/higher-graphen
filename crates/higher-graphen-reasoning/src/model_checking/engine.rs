use super::*;
use higher_graphen_structure::space::{Incidence, PathStep};
use std::collections::VecDeque;

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

    /// Dispatches a unified temporal check query.
    pub fn check_temporal_property(
        &self,
        query: &TemporalCheckQuery,
    ) -> Result<TemporalCheckReport> {
        match query {
            TemporalCheckQuery::RequiredEventualTransition(query) => {
                self.check_required_event(query)
            }
            TemporalCheckQuery::AlwaysBefore(query) => self.check_always_before(query),
            TemporalCheckQuery::NoDeadEndExceptTerminal(query) => self.check_dead_ends(query),
        }
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

/// Dispatches a unified temporal check query.
pub fn check_temporal_property(
    query: &TemporalCheckQuery,
    store: &InMemorySpaceStore,
) -> Result<TemporalCheckReport> {
    ModelChecker::new(store).check_temporal_property(query)
}

/// Builds an always-before query from state labels.
pub fn always_before_query_from_labels(
    space_id: Id,
    initial_cell_ids: impl IntoIterator<Item = Id>,
    labels: &StateLabelSet,
    before_label_id: &Id,
    after_label_id: &Id,
    options: ModelCheckingOptions,
) -> Result<AlwaysBeforeQuery> {
    Ok(AlwaysBeforeQuery::new(
        space_id,
        initial_cell_ids,
        labels.cell_ids_for(before_label_id)?,
        labels.cell_ids_for(after_label_id)?,
    )
    .with_options(options))
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
        Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            required_relation_types,
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
        )
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
        Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            BTreeSet::new(),
            before_cell_ids,
            after_cell_ids,
            BTreeSet::new(),
        )
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
        Self::from_parts(
            query.space_id.clone(),
            initial_cell_ids,
            query.options.clone(),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            terminal_cell_ids,
        )
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
            exhaustive: false,
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
            exhaustive: false,
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
            exhaustive: true,
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
            exhaustive: matches!(status, TemporalCheckStatus::Satisfied)
                && witness.is_none()
                && self.frontier_cell_ids.is_empty(),
            max_depth: self.query.max_depth,
            witness,
            visited_cell_ids: self.visited_cell_ids.clone(),
            frontier_cell_ids: self.frontier_cell_ids.iter().cloned().collect(),
            obstructions,
        }
    }
}

use super::support::*;
