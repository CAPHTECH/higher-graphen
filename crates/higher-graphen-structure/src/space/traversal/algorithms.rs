use super::{
    malformed, CycleSearchOptions, GraphPath, NormalizedCellPattern, NormalizedCycleSearchOptions,
    NormalizedPathPattern, NormalizedPathPatternSegment, NormalizedTraversalOptions, PathPattern,
    PathPatternMatch, PathStep, ReachabilityQuery, ReachabilityResult, TraversalDirection,
};
use crate::space::{Cell, InMemorySpaceStore, Incidence, IncidenceOrientation};
use crate::topology::SimpleCycleIndicator;
use higher_graphen_core::{Id, Result};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

impl InMemorySpaceStore {
    /// Returns the shortest path witness for a reachability query, if one exists.
    pub fn reachable(&self, query: &ReachabilityQuery) -> Result<ReachabilityResult> {
        self.require_cell_in_space("from_cell_id", &query.from_cell_id, &query.space_id)?;
        self.require_cell_in_space("to_cell_id", &query.to_cell_id, &query.space_id)?;
        let options = NormalizedTraversalOptions::try_from(&query.options)?;

        if query.from_cell_id == query.to_cell_id {
            return Ok(reachable_result(
                query,
                GraphPath::new(query.from_cell_id.clone()),
            ));
        }

        let mut search = ReachabilitySearch::new(self, query, &options);
        Ok(search.run())
    }

    /// Returns simple paths between the query endpoints.
    pub fn walk_paths(&self, query: &ReachabilityQuery) -> Result<Vec<GraphPath>> {
        self.require_cell_in_space("from_cell_id", &query.from_cell_id, &query.space_id)?;
        self.require_cell_in_space("to_cell_id", &query.to_cell_id, &query.space_id)?;
        let options = NormalizedTraversalOptions::try_from(&query.options)?;
        let max_depth = options
            .max_depth
            .unwrap_or_else(|| self.cell_count_in_space(&query.space_id).saturating_sub(1));

        let mut walker = PathWalker::new(self, query, &options, max_depth);
        Ok(walker.run())
    }

    /// Returns simple directed cycles over incidence `from_cell_id -> to_cell_id` edges.
    pub fn find_simple_cycles(
        &self,
        space_id: &Id,
        options: &CycleSearchOptions,
    ) -> Result<Vec<SimpleCycleIndicator>> {
        self.require_space(space_id)?;
        let options = NormalizedCycleSearchOptions::try_from(options)?;
        let mut search = SimpleCycleSearch::new(self, space_id, &options);
        Ok(search.run())
    }

    /// Returns paths that satisfy a fixed layer-by-layer path pattern.
    pub fn matches_path_pattern(&self, pattern: &PathPattern) -> Result<Vec<PathPatternMatch>> {
        let pattern = NormalizedPathPattern::try_from(pattern)?;
        self.require_space(&pattern.space_id)?;
        if let Some(cell_id) = &pattern.start.cell_id {
            self.require_cell_in_space("start.cell_id", cell_id, &pattern.space_id)?;
        }

        let mut matcher = PathPatternMatcher::new(self, &pattern);
        Ok(matcher.run())
    }

    fn require_space(&self, space_id: &Id) -> Result<()> {
        if self.spaces.contains_key(space_id) {
            Ok(())
        } else {
            Err(malformed(
                "space_id",
                format!("identifier {space_id} does not exist in the store"),
            ))
        }
    }

    fn require_cell_in_space(&self, field: &str, cell_id: &Id, space_id: &Id) -> Result<&Cell> {
        let cell = self
            .cells
            .get(cell_id)
            .ok_or_else(|| malformed(field, format!("identifier {cell_id} does not exist")))?;
        if &cell.space_id == space_id {
            Ok(cell)
        } else {
            Err(malformed(
                field,
                format!("identifier {cell_id} belongs to space {}", cell.space_id),
            ))
        }
    }

    fn cell_count_in_space(&self, space_id: &Id) -> usize {
        self.spaces
            .get(space_id)
            .map_or(0, |space| space.cell_ids.len())
    }

    fn cells_matching<'a>(
        &'a self,
        space_id: &Id,
        pattern: &NormalizedCellPattern,
    ) -> Vec<&'a Cell> {
        self.spaces
            .get(space_id)
            .into_iter()
            .flat_map(|space| &space.cell_ids)
            .filter_map(|cell_id| self.cells.get(cell_id))
            .filter(|cell| pattern.matches(cell))
            .collect()
    }

    fn neighbor_steps(
        &self,
        space_id: &Id,
        current_cell_id: &Id,
        options: &NormalizedTraversalOptions,
    ) -> Vec<PathStep> {
        self.spaces
            .get(space_id)
            .into_iter()
            .flat_map(|space| &space.incidence_ids)
            .filter_map(|incidence_id| self.incidences.get(incidence_id))
            .filter(|incidence| options.allows_relation(&incidence.relation_type))
            .filter_map(|incidence| step_from_incidence(current_cell_id, incidence, options))
            .collect()
    }
}

#[derive(Clone, Debug)]
struct CycleEdge {
    incidence_id: Id,
    to_cell_id: Id,
}

struct SimpleCycleSearch<'a> {
    store: &'a InMemorySpaceStore,
    space_id: &'a Id,
    options: &'a NormalizedCycleSearchOptions,
    adjacency: BTreeMap<Id, Vec<CycleEdge>>,
    cycles: Vec<SimpleCycleIndicator>,
    seen_cycles: BTreeSet<(Vec<Id>, Vec<Id>)>,
}

impl<'a> SimpleCycleSearch<'a> {
    fn new(
        store: &'a InMemorySpaceStore,
        space_id: &'a Id,
        options: &'a NormalizedCycleSearchOptions,
    ) -> Self {
        Self {
            store,
            space_id,
            options,
            adjacency: directed_adjacency(store, space_id, options),
            cycles: Vec::new(),
            seen_cycles: BTreeSet::new(),
        }
    }

    fn run(&mut self) -> Vec<SimpleCycleIndicator> {
        for start_cell_id in self.cell_ids() {
            if self.has_enough_cycles() {
                break;
            }
            let mut visited = BTreeSet::from([start_cell_id.clone()]);
            self.visit(
                &start_cell_id,
                &start_cell_id,
                &mut vec![start_cell_id.clone()],
                &mut Vec::new(),
                &mut visited,
            );
        }
        self.cycles.clone()
    }

    fn cell_ids(&self) -> Vec<Id> {
        let mut cell_ids = self
            .store
            .spaces
            .get(self.space_id)
            .map_or_else(Vec::new, |space| space.cell_ids.clone());
        cell_ids.sort();
        cell_ids
    }

    fn visit(
        &mut self,
        start_cell_id: &Id,
        current_cell_id: &Id,
        vertex_cell_ids: &mut Vec<Id>,
        edge_cell_ids: &mut Vec<Id>,
        visited: &mut BTreeSet<Id>,
    ) {
        if self.has_enough_cycles() {
            return;
        }

        let outgoing_edges = self
            .adjacency
            .get(current_cell_id)
            .cloned()
            .unwrap_or_default();
        for edge in outgoing_edges {
            if self.has_enough_cycles() {
                return;
            }
            let next_cycle_length = edge_cell_ids.len() + 1;
            if self.exceeds_max_path_length(next_cycle_length) {
                continue;
            }
            if &edge.to_cell_id == start_cell_id {
                self.record_cycle(vertex_cell_ids, edge_cell_ids, edge.incidence_id);
                continue;
            }
            if &edge.to_cell_id < start_cell_id || visited.contains(&edge.to_cell_id) {
                continue;
            }

            visited.insert(edge.to_cell_id.clone());
            vertex_cell_ids.push(edge.to_cell_id.clone());
            edge_cell_ids.push(edge.incidence_id);
            let next_cell_id = vertex_cell_ids.last().expect("just pushed vertex").clone();
            self.visit(
                start_cell_id,
                &next_cell_id,
                vertex_cell_ids,
                edge_cell_ids,
                visited,
            );
            let removed_cell_id = vertex_cell_ids.pop().expect("visited vertex should exist");
            visited.remove(&removed_cell_id);
            edge_cell_ids.pop();
        }
    }

    fn record_cycle(
        &mut self,
        vertex_cell_ids: &[Id],
        path_edge_cell_ids: &[Id],
        witness_edge_id: Id,
    ) {
        let mut edge_cell_ids = path_edge_cell_ids.to_vec();
        edge_cell_ids.push(witness_edge_id.clone());
        let key = (vertex_cell_ids.to_vec(), edge_cell_ids.clone());
        if !self.seen_cycles.insert(key) {
            return;
        }
        self.cycles.push(SimpleCycleIndicator {
            witness_edge_id,
            vertex_cell_ids: vertex_cell_ids.to_vec(),
            edge_cell_ids,
        });
    }

    fn exceeds_max_path_length(&self, length: usize) -> bool {
        self.options
            .max_path_length
            .is_some_and(|max_path_length| length > max_path_length)
    }

    fn has_enough_cycles(&self) -> bool {
        self.options
            .max_cycles
            .is_some_and(|max_cycles| self.cycles.len() >= max_cycles)
    }
}

struct ReachabilitySearch<'a> {
    store: &'a InMemorySpaceStore,
    query: &'a ReachabilityQuery,
    options: &'a NormalizedTraversalOptions,
    queue: VecDeque<GraphPath>,
    visited: BTreeSet<Id>,
    visited_order: Vec<Id>,
    depth_by_cell: BTreeMap<Id, usize>,
}

impl<'a> ReachabilitySearch<'a> {
    fn new(
        store: &'a InMemorySpaceStore,
        query: &'a ReachabilityQuery,
        options: &'a NormalizedTraversalOptions,
    ) -> Self {
        let start_path = GraphPath::new(query.from_cell_id.clone());
        let mut visited = BTreeSet::new();
        visited.insert(query.from_cell_id.clone());
        Self {
            store,
            query,
            options,
            queue: VecDeque::from([start_path]),
            visited,
            visited_order: vec![query.from_cell_id.clone()],
            depth_by_cell: BTreeMap::from([(query.from_cell_id.clone(), 0)]),
        }
    }

    fn run(&mut self) -> ReachabilityResult {
        while let Some(path) = self.queue.pop_front() {
            if self.reached_depth_limit(&path) {
                continue;
            }
            if let Some(result) = self.expand(path) {
                return result;
            }
        }

        unreachable_result(
            self.query,
            self.visited_order.clone(),
            frontier_cell_ids(&self.depth_by_cell),
        )
    }

    fn expand(&mut self, path: GraphPath) -> Option<ReachabilityResult> {
        for step in self
            .store
            .neighbor_steps(&self.query.space_id, &path.end_cell_id, self.options)
        {
            if !self.visited.insert(step.to_cell_id.clone()) {
                continue;
            }
            let next_path = path.append(step);
            self.record_visit(&next_path);
            if next_path.end_cell_id == self.query.to_cell_id {
                return Some(reachable_result(self.query, next_path));
            }
            self.queue.push_back(next_path);
        }
        None
    }

    fn reached_depth_limit(&self, path: &GraphPath) -> bool {
        self.options
            .max_depth
            .is_some_and(|max_depth| path.depth() >= max_depth)
    }

    fn record_visit(&mut self, path: &GraphPath) {
        self.visited_order.push(path.end_cell_id.clone());
        self.depth_by_cell
            .insert(path.end_cell_id.clone(), path.depth());
    }
}

struct PathWalker<'a> {
    store: &'a InMemorySpaceStore,
    query: &'a ReachabilityQuery,
    options: &'a NormalizedTraversalOptions,
    max_depth: usize,
    max_paths: usize,
    paths: Vec<GraphPath>,
}

impl<'a> PathWalker<'a> {
    fn new(
        store: &'a InMemorySpaceStore,
        query: &'a ReachabilityQuery,
        options: &'a NormalizedTraversalOptions,
        max_depth: usize,
    ) -> Self {
        Self {
            store,
            query,
            options,
            max_depth,
            max_paths: options.max_paths.unwrap_or(usize::MAX),
            paths: Vec::new(),
        }
    }

    fn run(&mut self) -> Vec<GraphPath> {
        let path = GraphPath::new(self.query.from_cell_id.clone());
        let mut visited = BTreeSet::from([self.query.from_cell_id.clone()]);
        self.visit(path, &mut visited);
        self.paths.clone()
    }

    fn visit(&mut self, path: GraphPath, visited: &mut BTreeSet<Id>) {
        if self.paths.len() >= self.max_paths {
            return;
        }
        if path.end_cell_id == self.query.to_cell_id {
            self.paths.push(path);
            return;
        }
        if path.depth() >= self.max_depth {
            return;
        }

        for step in self
            .store
            .neighbor_steps(&self.query.space_id, &path.end_cell_id, self.options)
        {
            if visited.contains(&step.to_cell_id) {
                continue;
            }
            visited.insert(step.to_cell_id.clone());
            self.visit(path.append(step.clone()), visited);
            visited.remove(&step.to_cell_id);
        }
    }
}

struct PathPatternMatcher<'a> {
    store: &'a InMemorySpaceStore,
    pattern: &'a NormalizedPathPattern,
    matches: Vec<PathPatternMatch>,
}

impl<'a> PathPatternMatcher<'a> {
    fn new(store: &'a InMemorySpaceStore, pattern: &'a NormalizedPathPattern) -> Self {
        Self {
            store,
            pattern,
            matches: Vec::new(),
        }
    }

    fn run(&mut self) -> Vec<PathPatternMatch> {
        for start in self
            .store
            .cells_matching(&self.pattern.space_id, &self.pattern.start)
        {
            let path = GraphPath::new(start.id.clone());
            self.match_segment(0, path, vec![start.id.clone()]);
            if self.has_enough_matches() {
                break;
            }
        }
        self.matches.clone()
    }

    fn match_segment(&mut self, index: usize, path: GraphPath, matched_cell_ids: Vec<Id>) {
        if self.has_enough_matches() {
            return;
        }
        let Some(segment) = self.pattern.segments.get(index) else {
            self.matches.push(PathPatternMatch {
                path,
                matched_cell_ids,
            });
            return;
        };

        for step in self.neighbor_steps_for_segment(&path.end_cell_id, segment) {
            let Some(target) = self.store.cells.get(&step.to_cell_id) else {
                continue;
            };
            if !segment.target.matches(target) {
                continue;
            }
            let mut next_cell_ids = matched_cell_ids.clone();
            next_cell_ids.push(step.to_cell_id.clone());
            self.match_segment(index + 1, path.append(step), next_cell_ids);
        }
    }

    fn neighbor_steps_for_segment(
        &self,
        current_cell_id: &Id,
        segment: &NormalizedPathPatternSegment,
    ) -> Vec<PathStep> {
        let options = NormalizedTraversalOptions::for_single_relation(
            self.pattern.direction,
            &segment.relation_type,
        );
        self.store
            .neighbor_steps(&self.pattern.space_id, current_cell_id, &options)
    }

    fn has_enough_matches(&self) -> bool {
        self.pattern
            .max_matches
            .is_some_and(|max_matches| self.matches.len() >= max_matches)
    }
}

fn step_from_incidence(
    current_cell_id: &Id,
    incidence: &Incidence,
    options: &NormalizedTraversalOptions,
) -> Option<PathStep> {
    let to_cell_id = next_cell_id(current_cell_id, incidence, options.direction)?;
    Some(PathStep {
        from_cell_id: current_cell_id.clone(),
        incidence_id: incidence.id.clone(),
        to_cell_id,
        relation_type: incidence.relation_type.clone(),
    })
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

fn directed_adjacency(
    store: &InMemorySpaceStore,
    space_id: &Id,
    options: &NormalizedCycleSearchOptions,
) -> BTreeMap<Id, Vec<CycleEdge>> {
    let Some(space) = store.spaces.get(space_id) else {
        return BTreeMap::new();
    };

    let mut adjacency: BTreeMap<Id, Vec<CycleEdge>> = space
        .cell_ids
        .iter()
        .map(|cell_id| (cell_id.clone(), Vec::new()))
        .collect();
    for incidence_id in &space.incidence_ids {
        let Some(incidence) = store.incidences.get(incidence_id) else {
            continue;
        };
        if !options.allows_relation(&incidence.relation_type) {
            continue;
        }
        adjacency
            .entry(incidence.from_cell_id.clone())
            .or_default()
            .push(CycleEdge {
                incidence_id: incidence.id.clone(),
                to_cell_id: incidence.to_cell_id.clone(),
            });
    }
    for edges in adjacency.values_mut() {
        edges.sort_by(|left, right| {
            left.to_cell_id
                .cmp(&right.to_cell_id)
                .then_with(|| left.incidence_id.cmp(&right.incidence_id))
        });
    }
    adjacency
}

fn reachable_result(query: &ReachabilityQuery, path: GraphPath) -> ReachabilityResult {
    ReachabilityResult {
        space_id: query.space_id.clone(),
        from_cell_id: query.from_cell_id.clone(),
        to_cell_id: query.to_cell_id.clone(),
        reachable: true,
        visited_cell_ids: path.cell_ids(),
        frontier_cell_ids: Vec::new(),
        shortest_path: Some(path),
    }
}

fn unreachable_result(
    query: &ReachabilityQuery,
    visited_cell_ids: Vec<Id>,
    frontier_cell_ids: Vec<Id>,
) -> ReachabilityResult {
    ReachabilityResult {
        space_id: query.space_id.clone(),
        from_cell_id: query.from_cell_id.clone(),
        to_cell_id: query.to_cell_id.clone(),
        reachable: false,
        shortest_path: None,
        visited_cell_ids,
        frontier_cell_ids,
    }
}

fn frontier_cell_ids(depth_by_cell: &BTreeMap<Id, usize>) -> Vec<Id> {
    let Some(max_depth) = depth_by_cell.values().max() else {
        return Vec::new();
    };
    depth_by_cell
        .iter()
        .filter(|(_, depth)| *depth == max_depth)
        .map(|(cell_id, _)| cell_id.clone())
        .collect()
}
