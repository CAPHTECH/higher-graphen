use super::*;

pub(super) fn connected_components(
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

pub(super) fn simple_cycle_witnesses(
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

#[derive(Clone, Debug)]
pub(super) struct GraphData {
    pub(super) vertices: BTreeSet<Id>,
    pub(super) edges: Vec<GraphEdge>,
    pub(super) findings: Vec<TopologyFinding>,
}

#[derive(Clone, Debug)]
pub(super) struct GraphEdge {
    pub(super) id: Id,
    pub(super) source: Id,
    pub(super) target: Id,
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
