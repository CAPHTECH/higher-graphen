use super::*;

/// Summarizes the full cell set of a finite complex.
pub fn summarize_complex(store: &InMemorySpaceStore, complex_id: &Id) -> Result<TopologySummary> {
    let complex = require_complex(store, complex_id)?;
    summarize_active_complex(store, complex, complex.cell_ids.iter().cloned())
}

/// Summarizes an explicit active subset of a finite complex.
pub fn summarize_complex_cells(
    store: &InMemorySpaceStore,
    complex_id: &Id,
    cell_ids: impl IntoIterator<Item = Id>,
) -> Result<TopologySummary> {
    let complex = require_complex(store, complex_id)?;
    summarize_active_complex(store, complex, cell_ids)
}

/// Summarizes a cumulative filtration with default persistence options.
pub fn summarize_filtration(
    store: &InMemorySpaceStore,
    complex_id: &Id,
    stages: &[FiltrationStage],
) -> Result<PersistenceSummary> {
    summarize_filtration_with_options(store, complex_id, stages, PersistenceOptions::default())
}

/// Summarizes a cumulative filtration with explicit persistence options.
pub fn summarize_filtration_with_options(
    store: &InMemorySpaceStore,
    complex_id: &Id,
    stages: &[FiltrationStage],
    options: PersistenceOptions,
) -> Result<PersistenceSummary> {
    if stages.is_empty() {
        return Err(malformed(
            "stages",
            "at least one filtration stage is required",
        ));
    }

    let complex = require_complex(store, complex_id)?;
    let complex_cell_ids = id_set(&complex.cell_ids);
    let mut seen_stage_ids = BTreeSet::new();
    let mut previous_stage_cell_ids = BTreeSet::new();
    let mut birth_stage_by_cell_id = BTreeMap::new();
    let mut stage_summaries = Vec::new();

    for (stage_index, stage) in stages.iter().enumerate() {
        if !seen_stage_ids.insert(stage.id.clone()) {
            return Err(malformed(
                "stages",
                format!("stage id {} appears more than once", stage.id),
            ));
        }

        let active_cell_ids = validate_stage_cell_ids(
            store,
            complex,
            &complex_cell_ids,
            stage,
            &previous_stage_cell_ids,
        )?;
        validate_stage_boundary_closure(
            store,
            complex,
            &complex_cell_ids,
            stage,
            &active_cell_ids,
        )?;

        for cell_id in active_cell_ids.difference(&previous_stage_cell_ids) {
            birth_stage_by_cell_id.insert(cell_id.clone(), stage_index);
        }

        let topology = summarize_active_complex(store, complex, active_cell_ids.iter().cloned())?;
        stage_summaries.push(FiltrationStageSummary {
            stage_id: stage.id.clone(),
            stage_index,
            cell_ids: ids_from_set(active_cell_ids.clone()),
            topology,
        });

        previous_stage_cell_ids = active_cell_ids;
    }

    let intervals = build_persistence_intervals(store, complex, stages, &birth_stage_by_cell_id)?;
    let last_stage_index = stages.len() - 1;
    let persistent_intervals = intervals
        .iter()
        .filter(|interval| {
            interval.lifetime_stages(last_stage_index) >= options.min_lifetime_stages
        })
        .cloned()
        .collect::<Vec<_>>();
    let open_component_count = intervals
        .iter()
        .filter(|interval| interval.dimension == 0 && interval.is_open())
        .count();
    let open_hole_count = intervals
        .iter()
        .filter(|interval| interval.dimension == 1 && interval.is_open())
        .count();

    Ok(PersistenceSummary {
        complex_id: complex.id.clone(),
        options,
        stages: stage_summaries,
        intervals,
        persistent_intervals,
        open_component_count,
        open_hole_count,
    })
}

fn summarize_active_complex(
    store: &InMemorySpaceStore,
    complex: &Complex,
    cell_ids: impl IntoIterator<Item = Id>,
) -> Result<TopologySummary> {
    let active_cell_ids = validate_active_cell_ids(store, complex, cell_ids)?;
    let mut graph = build_graph(store, complex, &active_cell_ids)?;
    let homology = summarize_homology(store, complex, &active_cell_ids, &mut graph.findings)?;
    let components = connected_components(&graph.vertices, &graph.edges);
    let simple_cycles = simple_cycle_witnesses(&graph.vertices, &graph.edges);
    let first_betti_number = homology.betti_number(1);

    Ok(TopologySummary {
        complex_id: complex.id.clone(),
        vertex_count: graph.vertices.len(),
        graph_edge_count: graph.edges.len(),
        component_count: components.len(),
        connected_components: components,
        homology,
        first_betti_number,
        simple_hole_count: first_betti_number,
        has_simple_cycle: !simple_cycles.is_empty(),
        simple_cycles,
        findings: graph.findings,
    })
}

fn validate_active_cell_ids(
    store: &InMemorySpaceStore,
    complex: &Complex,
    cell_ids: impl IntoIterator<Item = Id>,
) -> Result<BTreeSet<Id>> {
    let complex_cell_ids = id_set(&complex.cell_ids);
    let mut active_cell_ids = BTreeSet::new();
    for cell_id in cell_ids {
        if !complex_cell_ids.contains(&cell_id) {
            return Err(malformed(
                "cell_ids",
                format!(
                    "identifier {cell_id} is not included in complex {}",
                    complex.id
                ),
            ));
        }
        require_cell_in_complex(store, complex, &cell_id)?;
        active_cell_ids.insert(cell_id);
    }
    Ok(active_cell_ids)
}

fn validate_stage_cell_ids(
    store: &InMemorySpaceStore,
    complex: &Complex,
    complex_cell_ids: &BTreeSet<Id>,
    stage: &FiltrationStage,
    previous_stage_cell_ids: &BTreeSet<Id>,
) -> Result<BTreeSet<Id>> {
    let mut active_cell_ids = BTreeSet::new();
    let mut duplicate_cell_ids = BTreeSet::new();

    for cell_id in &stage.cell_ids {
        if !active_cell_ids.insert(cell_id.clone()) {
            duplicate_cell_ids.insert(cell_id.clone());
        }
        if !complex_cell_ids.contains(cell_id) {
            return Err(malformed(
                "stages",
                format!(
                    "stage {} contains cell {cell_id} outside complex {}",
                    stage.id, complex.id
                ),
            ));
        }
        require_cell_in_complex(store, complex, cell_id)?;
    }

    if !duplicate_cell_ids.is_empty() {
        return Err(malformed(
            "stages",
            format!(
                "stage {} repeats cell ids {:?}",
                stage.id,
                id_strings(&duplicate_cell_ids)
            ),
        ));
    }

    let removed_cell_ids = previous_stage_cell_ids
        .difference(&active_cell_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    if !removed_cell_ids.is_empty() {
        return Err(malformed(
            "stages",
            format!(
                "stage {} is not cumulative; missing prior cells {:?}",
                stage.id,
                id_strings(&removed_cell_ids)
            ),
        ));
    }

    Ok(active_cell_ids)
}

fn validate_stage_boundary_closure(
    store: &InMemorySpaceStore,
    complex: &Complex,
    complex_cell_ids: &BTreeSet<Id>,
    stage: &FiltrationStage,
    active_cell_ids: &BTreeSet<Id>,
) -> Result<()> {
    for cell_id in active_cell_ids {
        let cell = require_cell_in_complex(store, complex, cell_id)?;
        for boundary_id in &cell.boundary {
            if complex_cell_ids.contains(boundary_id) && !active_cell_ids.contains(boundary_id) {
                return Err(malformed(
                    "stages",
                    format!(
                        "stage {} includes cell {cell_id} before in-complex boundary {boundary_id}",
                        stage.id
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn build_graph(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
) -> Result<GraphData> {
    let mut vertices = BTreeSet::new();
    let mut edges = Vec::new();
    let mut findings = Vec::new();

    for cell_id in active_cell_ids {
        let cell = require_cell_in_complex(store, complex, cell_id)?;
        match cell.dimension {
            0 => {
                vertices.insert(cell.id.clone());
            }
            1 => {
                if let Some(edge) =
                    graph_edge_from_cell(store, complex, active_cell_ids, cell, &mut findings)?
                {
                    edges.push(edge);
                }
            }
            _ => {}
        }
    }

    Ok(GraphData {
        vertices,
        edges,
        findings,
    })
}

fn graph_edge_from_cell(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
    cell: &Cell,
    findings: &mut Vec<TopologyFinding>,
) -> Result<Option<GraphEdge>> {
    let mut endpoint_ids = BTreeSet::new();
    let mut external_boundary_cell_ids = BTreeSet::new();
    let mut non_vertex_boundary_cell_ids = BTreeSet::new();

    for boundary_id in &cell.boundary {
        let boundary = require_cell_in_space(store, complex, boundary_id)?;
        if !active_cell_ids.contains(boundary_id) {
            external_boundary_cell_ids.insert(boundary_id.clone());
            continue;
        }
        if boundary.dimension == 0 {
            endpoint_ids.insert(boundary_id.clone());
        } else {
            non_vertex_boundary_cell_ids.insert(boundary_id.clone());
        }
    }

    if !external_boundary_cell_ids.is_empty() {
        push_finding(
            findings,
            TopologyFinding {
                finding_type: TopologyFindingKind::ExternalBoundaryCell,
                obstruction_type: Some(UNCOVERED_REGION_OBSTRUCTION_TYPE.to_owned()),
                cell_id: Some(cell.id.clone()),
                related_cell_ids: ids_from_set(external_boundary_cell_ids),
                description: format!(
                    "edge cell {} references boundary cells outside the summarized region",
                    cell.id
                ),
            },
        );
        return Ok(None);
    }

    let endpoints = ids_from_set(endpoint_ids);
    if endpoints.len() != 2 || !non_vertex_boundary_cell_ids.is_empty() {
        let mut related_cell_ids = endpoints.clone();
        related_cell_ids.extend(ids_from_set(non_vertex_boundary_cell_ids));
        push_finding(findings, TopologyFinding {
            finding_type: TopologyFindingKind::NonGraphEdgeBoundary,
            obstruction_type: None,
            cell_id: Some(cell.id.clone()),
            related_cell_ids,
            description: format!(
                "dimension-1 cell {} has {} vertex endpoints; exactly two are required for the finite graph kernel",
                cell.id,
                endpoints.len()
            ),
        });
        return Ok(None);
    }

    Ok(Some(GraphEdge {
        id: cell.id.clone(),
        source: endpoints[0].clone(),
        target: endpoints[1].clone(),
    }))
}

fn summarize_homology(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
    findings: &mut Vec<TopologyFinding>,
) -> Result<HomologySummary> {
    let cells_by_dimension = cells_by_dimension(store, complex, active_cell_ids)?;
    let boundary_data = build_boundary_data(
        store,
        complex,
        active_cell_ids,
        &cells_by_dimension,
        findings,
    )?;
    validate_boundary_composition(&boundary_data, findings);

    let mut dimensions = BTreeSet::from([0]);
    for dimension in cells_by_dimension.keys() {
        dimensions.insert(*dimension);
        if *dimension > 0 {
            dimensions.insert(dimension - 1);
        }
    }

    let mut euler_characteristic = 0;
    for (dimension, cell_ids) in &cells_by_dimension {
        if dimension % 2 == 0 {
            euler_characteristic += cell_ids.len() as i64;
        } else {
            euler_characteristic -= cell_ids.len() as i64;
        }
    }

    let mut dimension_summaries = Vec::new();
    for dimension in dimensions {
        let chain_rank = cells_by_dimension.get(&dimension).map_or(0, Vec::len);
        let boundary_rank = boundary_data
            .rank_by_dimension
            .get(&dimension)
            .copied()
            .unwrap_or(0);
        let cycle_rank = chain_rank.saturating_sub(boundary_rank);
        let bounding_chain_rank = boundary_data
            .rank_by_dimension
            .get(&dimension.saturating_add(1))
            .copied()
            .unwrap_or(0);
        let homology_rank = cycle_rank.saturating_sub(bounding_chain_rank);
        dimension_summaries.push(HomologyDimensionSummary {
            dimension,
            chain_rank,
            boundary_rank,
            cycle_rank,
            bounding_chain_rank,
            homology_rank,
        });
    }

    Ok(HomologySummary {
        coefficient_field: HomologyCoefficientField::Z2,
        dimensions: dimension_summaries,
        euler_characteristic,
    })
}

fn cells_by_dimension(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
) -> Result<BTreeMap<Dimension, Vec<Id>>> {
    let mut cells_by_dimension = BTreeMap::<Dimension, Vec<Id>>::new();
    for cell_id in active_cell_ids {
        let cell = require_cell_in_complex(store, complex, cell_id)?;
        cells_by_dimension
            .entry(cell.dimension)
            .or_default()
            .push(cell.id.clone());
    }
    Ok(cells_by_dimension)
}

fn build_boundary_data(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
    cells_by_dimension: &BTreeMap<Dimension, Vec<Id>>,
    findings: &mut Vec<TopologyFinding>,
) -> Result<BoundaryData> {
    let mut rank_by_dimension = BTreeMap::new();
    let mut boundary_cell_ids_by_cell_id = BTreeMap::new();

    for (dimension, cell_ids) in cells_by_dimension {
        if *dimension == 0 {
            continue;
        }

        let row_cell_ids = cells_by_dimension
            .get(&(dimension - 1))
            .cloned()
            .unwrap_or_default();
        let row_index_by_cell_id = row_cell_ids
            .iter()
            .enumerate()
            .map(|(index, cell_id)| (cell_id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let mut boundary_columns = Vec::new();

        for cell_id in cell_ids {
            let cell = require_cell_in_complex(store, complex, cell_id)?;
            let boundary_cell_ids = homology_boundary_cell_ids(
                store,
                complex,
                active_cell_ids,
                cell,
                &row_index_by_cell_id,
                findings,
            )?;
            let row_indices = boundary_cell_ids
                .iter()
                .filter_map(|boundary_id| row_index_by_cell_id.get(boundary_id).copied())
                .collect::<BTreeSet<_>>();
            boundary_columns.push(row_indices);
            boundary_cell_ids_by_cell_id.insert(cell.id.clone(), boundary_cell_ids);
        }

        rank_by_dimension.insert(*dimension, rank_mod2(&boundary_columns));
    }

    Ok(BoundaryData {
        rank_by_dimension,
        boundary_cell_ids_by_cell_id,
    })
}

fn homology_boundary_cell_ids(
    store: &InMemorySpaceStore,
    complex: &Complex,
    active_cell_ids: &BTreeSet<Id>,
    cell: &Cell,
    row_index_by_cell_id: &BTreeMap<Id, usize>,
    findings: &mut Vec<TopologyFinding>,
) -> Result<BTreeSet<Id>> {
    let mut boundary_cell_ids = BTreeSet::new();
    let mut external_boundary_cell_ids = BTreeSet::new();
    let mut non_codimension_one_boundary_cell_ids = BTreeSet::new();

    for boundary_id in &cell.boundary {
        let boundary = require_cell_in_space(store, complex, boundary_id)?;
        if !active_cell_ids.contains(boundary_id) {
            external_boundary_cell_ids.insert(boundary_id.clone());
            continue;
        }

        if boundary.dimension.checked_add(1) == Some(cell.dimension)
            && row_index_by_cell_id.contains_key(boundary_id)
        {
            boundary_cell_ids.insert(boundary_id.clone());
        } else {
            non_codimension_one_boundary_cell_ids.insert(boundary_id.clone());
        }
    }

    if !external_boundary_cell_ids.is_empty() {
        push_finding(
            findings,
            TopologyFinding {
                finding_type: TopologyFindingKind::ExternalBoundaryCell,
                obstruction_type: Some(UNCOVERED_REGION_OBSTRUCTION_TYPE.to_owned()),
                cell_id: Some(cell.id.clone()),
                related_cell_ids: ids_from_set(external_boundary_cell_ids),
                description: format!(
                    "cell {} references boundary cells outside the summarized chain complex",
                    cell.id
                ),
            },
        );
    }

    if !non_codimension_one_boundary_cell_ids.is_empty() {
        push_finding(
            findings,
            TopologyFinding {
                finding_type: TopologyFindingKind::NonCodimensionOneBoundary,
                obstruction_type: Some(UNSUPPORTED_DIMENSION_OBSTRUCTION_TYPE.to_owned()),
                cell_id: Some(cell.id.clone()),
                related_cell_ids: ids_from_set(non_codimension_one_boundary_cell_ids),
                description: format!(
                    "cell {} has boundary cells outside the codimension-one chain group",
                    cell.id
                ),
            },
        );
    }

    Ok(boundary_cell_ids)
}

fn validate_boundary_composition(
    boundary_data: &BoundaryData,
    findings: &mut Vec<TopologyFinding>,
) {
    for (cell_id, boundary_cell_ids) in &boundary_data.boundary_cell_ids_by_cell_id {
        let mut composed_boundary_cell_ids = BTreeSet::new();
        for boundary_id in boundary_cell_ids {
            if let Some(lower_boundary_ids) =
                boundary_data.boundary_cell_ids_by_cell_id.get(boundary_id)
            {
                xor_ids(&mut composed_boundary_cell_ids, lower_boundary_ids);
            }
        }

        if composed_boundary_cell_ids.is_empty() {
            continue;
        }

        push_finding(
            findings,
            TopologyFinding {
                finding_type: TopologyFindingKind::BoundaryOperatorCompositionNonZero,
                obstruction_type: Some(UNSUPPORTED_DIMENSION_OBSTRUCTION_TYPE.to_owned()),
                cell_id: Some(cell_id.clone()),
                related_cell_ids: ids_from_set(composed_boundary_cell_ids),
                description: format!("cell {cell_id} has nonzero composed boundary over Z2"),
            },
        );
    }
}

fn rank_mod2(columns: &[BTreeSet<usize>]) -> usize {
    let mut pivot_columns: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();

    for column in columns {
        let mut reduced = column.clone();
        while let Some(pivot) = reduced.iter().next_back().copied() {
            if let Some(existing_column) = pivot_columns.get(&pivot) {
                xor_indices(&mut reduced, existing_column);
            } else {
                pivot_columns.insert(pivot, reduced);
                break;
            }
        }
    }

    pivot_columns.len()
}

fn xor_indices(left: &mut BTreeSet<usize>, right: &BTreeSet<usize>) {
    let values = right.iter().copied().collect::<Vec<_>>();
    for value in values {
        if !left.insert(value) {
            left.remove(&value);
        }
    }
}

fn xor_ids(left: &mut BTreeSet<Id>, right: &BTreeSet<Id>) {
    let values = right.iter().cloned().collect::<Vec<_>>();
    for value in values {
        if !left.insert(value.clone()) {
            left.remove(&value);
        }
    }
}

fn push_finding(findings: &mut Vec<TopologyFinding>, finding: TopologyFinding) {
    if !findings.iter().any(|existing| {
        existing.finding_type == finding.finding_type
            && existing.obstruction_type == finding.obstruction_type
            && existing.cell_id == finding.cell_id
            && existing.related_cell_ids == finding.related_cell_ids
    }) {
        findings.push(finding);
    }
}

fn connected_components(
    vertices: &BTreeSet<Id>,
    edges: &[GraphEdge],
) -> Vec<ConnectedComponentSummary> {
    let mut union_find = UnionFind::new();
    for vertex_id in vertices {
        union_find.add(vertex_id.clone());
    }
    for edge in edges {
        union_find.union_lexicographic(&edge.source, &edge.target);
    }

    let mut component_vertices: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
    for vertex_id in vertices {
        let root = union_find.find(vertex_id);
        component_vertices
            .entry(root)
            .or_default()
            .insert(vertex_id.clone());
    }

    let mut component_edges: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
    for edge in edges {
        let root = union_find.find(&edge.source);
        component_edges
            .entry(root)
            .or_default()
            .insert(edge.id.clone());
    }

    component_vertices
        .into_iter()
        .map(
            |(representative_cell_id, vertex_cell_ids)| ConnectedComponentSummary {
                edge_cell_ids: ids_from_set(
                    component_edges
                        .remove(&representative_cell_id)
                        .unwrap_or_default(),
                ),
                representative_cell_id,
                vertex_cell_ids: ids_from_set(vertex_cell_ids),
            },
        )
        .collect()
}

fn simple_cycle_witnesses(
    vertices: &BTreeSet<Id>,
    edges: &[GraphEdge],
) -> Vec<SimpleCycleIndicator> {
    let mut forest = UnionFind::new();
    let mut adjacency = BTreeMap::new();
    let mut witnesses = Vec::new();

    for vertex_id in vertices {
        forest.add(vertex_id.clone());
        adjacency.insert(vertex_id.clone(), Vec::new());
    }

    for edge in edges {
        if forest.find(&edge.source) == forest.find(&edge.target) {
            if let Some((vertex_cell_ids, mut edge_cell_ids)) =
                find_path(&adjacency, &edge.source, &edge.target)
            {
                edge_cell_ids.push(edge.id.clone());
                witnesses.push(SimpleCycleIndicator {
                    witness_edge_id: edge.id.clone(),
                    vertex_cell_ids,
                    edge_cell_ids,
                });
            }
            continue;
        }

        forest.union_lexicographic(&edge.source, &edge.target);
        adjacency
            .entry(edge.source.clone())
            .or_default()
            .push((edge.target.clone(), edge.id.clone()));
        adjacency
            .entry(edge.target.clone())
            .or_default()
            .push((edge.source.clone(), edge.id.clone()));
    }

    witnesses
}

fn find_path(
    adjacency: &BTreeMap<Id, Vec<(Id, Id)>>,
    start: &Id,
    target: &Id,
) -> Option<(Vec<Id>, Vec<Id>)> {
    let mut queue = VecDeque::from([start.clone()]);
    let mut visited = BTreeSet::from([start.clone()]);
    let mut predecessor: BTreeMap<Id, (Id, Id)> = BTreeMap::new();

    while let Some(current) = queue.pop_front() {
        if &current == target {
            break;
        }

        let mut neighbors = adjacency.get(&current).cloned().unwrap_or_default();
        neighbors.sort();
        for (neighbor, edge_id) in neighbors {
            if visited.insert(neighbor.clone()) {
                predecessor.insert(neighbor.clone(), (current.clone(), edge_id));
                queue.push_back(neighbor);
            }
        }
    }

    if !visited.contains(target) {
        return None;
    }

    let mut vertex_cell_ids = vec![target.clone()];
    let mut edge_cell_ids = Vec::new();
    let mut current = target.clone();

    while &current != start {
        let (previous, edge_id) = predecessor.get(&current)?.clone();
        edge_cell_ids.push(edge_id);
        vertex_cell_ids.push(previous.clone());
        current = previous;
    }

    vertex_cell_ids.reverse();
    edge_cell_ids.reverse();
    Some((vertex_cell_ids, edge_cell_ids))
}

mod persistence;
use persistence::*;

fn require_complex<'a>(store: &'a InMemorySpaceStore, complex_id: &Id) -> Result<&'a Complex> {
    store.complex(complex_id).ok_or_else(|| {
        malformed(
            "complex_id",
            format!("identifier {complex_id} does not exist"),
        )
    })
}

fn require_cell_in_complex<'a>(
    store: &'a InMemorySpaceStore,
    complex: &Complex,
    cell_id: &Id,
) -> Result<&'a Cell> {
    let cell = require_cell_in_space(store, complex, cell_id)?;
    if complex.cell_ids.contains(cell_id) {
        Ok(cell)
    } else {
        Err(malformed(
            "cell_ids",
            format!(
                "identifier {cell_id} is not included in complex {}",
                complex.id
            ),
        ))
    }
}

fn require_cell_in_space<'a>(
    store: &'a InMemorySpaceStore,
    complex: &Complex,
    cell_id: &Id,
) -> Result<&'a Cell> {
    let cell = store
        .cell(cell_id)
        .ok_or_else(|| malformed("cell_ids", format!("identifier {cell_id} does not exist")))?;
    if cell.space_id == complex.space_id {
        Ok(cell)
    } else {
        Err(malformed(
            "cell_ids",
            format!(
                "identifier {cell_id} belongs to space {}, expected {}",
                cell.space_id, complex.space_id
            ),
        ))
    }
}

fn id_set(ids: &[Id]) -> BTreeSet<Id> {
    ids.iter().cloned().collect()
}

fn ids_from_set(ids: BTreeSet<Id>) -> Vec<Id> {
    ids.into_iter().collect()
}

fn id_strings(ids: &BTreeSet<Id>) -> Vec<String> {
    ids.iter().map(ToString::to_string).collect()
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

#[derive(Clone, Debug)]
struct GraphData {
    vertices: BTreeSet<Id>,
    edges: Vec<GraphEdge>,
    findings: Vec<TopologyFinding>,
}

#[derive(Clone, Debug)]
struct GraphEdge {
    id: Id,
    source: Id,
    target: Id,
}

#[derive(Clone, Debug, Default)]
struct BoundaryData {
    rank_by_dimension: BTreeMap<Dimension, usize>,
    boundary_cell_ids_by_cell_id: BTreeMap<Id, BTreeSet<Id>>,
}

#[derive(Clone, Debug, Default)]
struct UnionFind {
    parent: BTreeMap<Id, Id>,
}

impl UnionFind {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, id: Id) {
        self.parent.entry(id.clone()).or_insert(id);
    }

    fn find(&mut self, id: &Id) -> Id {
        let parent = self
            .parent
            .get(id)
            .cloned()
            .expect("union-find contains vertex");
        if &parent == id {
            return parent;
        }
        let root = self.find(&parent);
        self.parent.insert(id.clone(), root.clone());
        root
    }

    fn union_lexicographic(&mut self, left: &Id, right: &Id) -> bool {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return false;
        }

        let (survivor, loser) = if left_root <= right_root {
            (left_root, right_root)
        } else {
            (right_root, left_root)
        };
        self.parent.insert(loser, survivor);
        true
    }
}
