//! Finite graph analytics over selected HigherGraphen incidence views.

use super::{malformed, InMemorySpaceStore, IncidenceOrientation};
use crate::space::traversal::TraversalDirection;
use higher_graphen_core::{Id, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Bounded graph analytics input over one space.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAnalyticsInput {
    /// Space to analyze.
    pub space_id: Id,
    /// Seed cells for the impact cone. Empty means the cone is not computed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seed_cell_ids: Vec<Id>,
    /// Optional allowed relation types. Empty means any relation type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_types: Vec<String>,
    /// Direction used for the impact cone.
    pub direction: TraversalDirection,
    /// Optional maximum impact-cone traversal depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
}

impl GraphAnalyticsInput {
    /// Creates graph analytics input for a space.
    #[must_use]
    pub fn new(space_id: Id) -> Self {
        Self {
            space_id,
            seed_cell_ids: Vec::new(),
            relation_types: Vec::new(),
            direction: TraversalDirection::Both,
            max_depth: None,
        }
    }

    /// Returns this input with impact-cone seed cells.
    #[must_use]
    pub fn with_seed_cell_ids(mut self, seed_cell_ids: impl IntoIterator<Item = Id>) -> Self {
        self.seed_cell_ids = unique_ids(seed_cell_ids);
        self
    }

    /// Returns this input with an allowed relation type appended.
    pub fn with_relation_type(mut self, relation_type: impl Into<String>) -> Result<Self> {
        self.relation_types
            .push(required_text("relation_types", relation_type)?);
        self.relation_types = unique_texts(std::mem::take(&mut self.relation_types));
        Ok(self)
    }

    /// Returns this input with an impact-cone direction.
    #[must_use]
    pub fn in_direction(mut self, direction: TraversalDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns this input with a maximum impact-cone depth.
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    fn validate(&mut self) -> Result<()> {
        self.seed_cell_ids = unique_ids(std::mem::take(&mut self.seed_cell_ids));
        self.relation_types = self
            .relation_types
            .drain(..)
            .map(|relation_type| required_text("relation_types", relation_type))
            .collect::<Result<Vec<_>>>()?;
        self.relation_types = unique_texts(std::mem::take(&mut self.relation_types));
        Ok(())
    }
}

impl<'de> Deserialize<'de> for GraphAnalyticsInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            space_id: Id,
            #[serde(default)]
            seed_cell_ids: Vec<Id>,
            #[serde(default)]
            relation_types: Vec<String>,
            direction: TraversalDirection,
            max_depth: Option<usize>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let mut input = Self {
            space_id: wire.space_id,
            seed_cell_ids: wire.seed_cell_ids,
            relation_types: wire.relation_types,
            direction: wire.direction,
            max_depth: wire.max_depth,
        };
        input.validate().map_err(serde::de::Error::custom)?;
        Ok(input)
    }
}

/// Connected component in the selected traversal graph view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAnalyticsComponent {
    /// Deterministic representative cell.
    pub representative_cell_id: Id,
    /// Cells in the component.
    pub cell_ids: Vec<Id>,
}

/// Finite graph analytics report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAnalyticsReport {
    /// Space analyzed.
    pub space_id: Id,
    /// Seed cells used by impact-cone traversal.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seed_cell_ids: Vec<Id>,
    /// Impact cone from seed cells in traversal order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impact_cone_cell_ids: Vec<Id>,
    /// Articulation cells in the selected traversal view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub articulation_cell_ids: Vec<Id>,
    /// Bridge incidences in the selected traversal view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bridge_incidence_ids: Vec<Id>,
    /// Connected components in the selected traversal view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connected_components: Vec<GraphAnalyticsComponent>,
    /// Strongly connected components in the selected directed traversal view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strongly_connected_components: Vec<GraphAnalyticsComponent>,
}

impl InMemorySpaceStore {
    /// Runs finite graph analytics over one selected incidence view.
    pub fn analyze_graph(&self, input: &GraphAnalyticsInput) -> Result<GraphAnalyticsReport> {
        let mut input = input.clone();
        input.validate()?;
        let space = self.spaces.get(&input.space_id).ok_or_else(|| {
            malformed(
                "space_id",
                format!("identifier {} does not exist in the store", input.space_id),
            )
        })?;
        for seed_cell_id in &input.seed_cell_ids {
            let cell = self.cells.get(seed_cell_id).ok_or_else(|| {
                malformed(
                    "seed_cell_ids",
                    format!("identifier {seed_cell_id} does not exist"),
                )
            })?;
            if cell.space_id != input.space_id {
                return Err(malformed(
                    "seed_cell_ids",
                    format!(
                        "identifier {seed_cell_id} belongs to space {}, expected {}",
                        cell.space_id, input.space_id
                    ),
                ));
            }
        }

        let view = GraphView::from_store(self, &input, &space.cell_ids);
        let impact_cone_cell_ids = impact_cone(&view, &input);
        let connected_components = connected_components(&view);
        let strongly_connected_components = strongly_connected_components(&view);
        let articulation_cell_ids = articulation_cells(&view);
        let bridge_incidence_ids = bridge_incidences(&view);

        Ok(GraphAnalyticsReport {
            space_id: input.space_id,
            seed_cell_ids: input.seed_cell_ids,
            impact_cone_cell_ids,
            articulation_cell_ids,
            bridge_incidence_ids,
            connected_components,
            strongly_connected_components,
        })
    }
}

#[derive(Clone, Debug)]
struct GraphView {
    adjacency: BTreeMap<Id, Vec<GraphEdge>>,
}

impl GraphView {
    fn from_store(
        store: &InMemorySpaceStore,
        input: &GraphAnalyticsInput,
        cell_ids: &[Id],
    ) -> Self {
        let allowed_relation_types = input
            .relation_types
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut adjacency = cell_ids
            .iter()
            .map(|cell_id| (cell_id.clone(), Vec::new()))
            .collect::<BTreeMap<_, _>>();
        let Some(space) = store.spaces.get(&input.space_id) else {
            return Self { adjacency };
        };

        for incidence_id in &space.incidence_ids {
            let Some(incidence) = store.incidences.get(incidence_id) else {
                continue;
            };
            if !allowed_relation_types.is_empty()
                && !allowed_relation_types.contains(&incidence.relation_type)
            {
                continue;
            }
            if matches!(
                input.direction,
                TraversalDirection::Outgoing | TraversalDirection::Both
            ) || incidence.orientation == IncidenceOrientation::Undirected
            {
                adjacency
                    .entry(incidence.from_cell_id.clone())
                    .or_default()
                    .push(GraphEdge {
                        incidence_id: incidence.id.clone(),
                        to_cell_id: incidence.to_cell_id.clone(),
                    });
            }
            if matches!(
                input.direction,
                TraversalDirection::Incoming | TraversalDirection::Both
            ) || incidence.orientation == IncidenceOrientation::Undirected
            {
                adjacency
                    .entry(incidence.to_cell_id.clone())
                    .or_default()
                    .push(GraphEdge {
                        incidence_id: incidence.id.clone(),
                        to_cell_id: incidence.from_cell_id.clone(),
                    });
            }
        }

        for edges in adjacency.values_mut() {
            edges.sort_by(|left, right| {
                left.to_cell_id
                    .cmp(&right.to_cell_id)
                    .then_with(|| left.incidence_id.cmp(&right.incidence_id))
            });
        }

        Self { adjacency }
    }

    fn neighbors(&self, cell_id: &Id) -> Vec<GraphEdge> {
        self.adjacency.get(cell_id).cloned().unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
struct GraphEdge {
    incidence_id: Id,
    to_cell_id: Id,
}

fn impact_cone(view: &GraphView, input: &GraphAnalyticsInput) -> Vec<Id> {
    if input.seed_cell_ids.is_empty() {
        return Vec::new();
    }

    let mut visited = BTreeSet::new();
    let mut ordered = Vec::new();
    let mut queue = VecDeque::new();
    for seed_cell_id in &input.seed_cell_ids {
        if visited.insert(seed_cell_id.clone()) {
            ordered.push(seed_cell_id.clone());
            queue.push_back((seed_cell_id.clone(), 0usize));
        }
    }

    while let Some((cell_id, depth)) = queue.pop_front() {
        if input.max_depth.is_some_and(|max_depth| depth >= max_depth) {
            continue;
        }
        for edge in view.neighbors(&cell_id) {
            if visited.insert(edge.to_cell_id.clone()) {
                ordered.push(edge.to_cell_id.clone());
                queue.push_back((edge.to_cell_id, depth + 1));
            }
        }
    }

    ordered
}

fn connected_components(view: &GraphView) -> Vec<GraphAnalyticsComponent> {
    let mut remaining = view.adjacency.keys().cloned().collect::<BTreeSet<_>>();
    let mut components = Vec::new();

    while let Some(start) = remaining.iter().next().cloned() {
        let mut queue = VecDeque::from([start.clone()]);
        let mut component = BTreeSet::from([start.clone()]);
        remaining.remove(&start);

        while let Some(cell_id) = queue.pop_front() {
            for edge in view.neighbors(&cell_id) {
                if remaining.remove(&edge.to_cell_id) {
                    component.insert(edge.to_cell_id.clone());
                    queue.push_back(edge.to_cell_id);
                }
            }
        }

        let cell_ids = component.into_iter().collect::<Vec<_>>();
        components.push(GraphAnalyticsComponent {
            representative_cell_id: cell_ids[0].clone(),
            cell_ids,
        });
    }

    components
}

fn strongly_connected_components(view: &GraphView) -> Vec<GraphAnalyticsComponent> {
    let mut remaining = view.adjacency.keys().cloned().collect::<BTreeSet<_>>();
    let mut components = Vec::new();

    while let Some(start) = remaining.iter().next().cloned() {
        let forward = reachable_cells(view, &start);
        let component = forward
            .iter()
            .filter(|cell_id| reachable_cells(view, cell_id).contains(&start))
            .cloned()
            .collect::<BTreeSet<_>>();
        for cell_id in &component {
            remaining.remove(cell_id);
        }
        let cell_ids = component.into_iter().collect::<Vec<_>>();
        components.push(GraphAnalyticsComponent {
            representative_cell_id: cell_ids[0].clone(),
            cell_ids,
        });
    }

    components
}

fn reachable_cells(view: &GraphView, start: &Id) -> BTreeSet<Id> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(cell_id) = queue.pop_front() {
        if !visited.insert(cell_id.clone()) {
            continue;
        }
        for edge in view.neighbors(&cell_id) {
            queue.push_back(edge.to_cell_id);
        }
    }
    visited
}

fn articulation_cells(view: &GraphView) -> Vec<Id> {
    let baseline = connected_components(view).len();
    view.adjacency
        .keys()
        .filter(|cell_id| {
            let reduced = remove_cell(view, cell_id);
            connected_components(&reduced).len() > baseline
        })
        .cloned()
        .collect()
}

fn bridge_incidences(view: &GraphView) -> Vec<Id> {
    let baseline = connected_components(view).len();
    let incidence_ids = view
        .adjacency
        .values()
        .flat_map(|edges| edges.iter().map(|edge| edge.incidence_id.clone()))
        .collect::<BTreeSet<_>>();

    incidence_ids
        .into_iter()
        .filter(|incidence_id| {
            let reduced = remove_incidence(view, incidence_id);
            connected_components(&reduced).len() > baseline
        })
        .collect()
}

fn remove_cell(view: &GraphView, removed_cell_id: &Id) -> GraphView {
    let adjacency = view
        .adjacency
        .iter()
        .filter(|(cell_id, _)| *cell_id != removed_cell_id)
        .map(|(cell_id, edges)| {
            (
                cell_id.clone(),
                edges
                    .iter()
                    .filter(|edge| &edge.to_cell_id != removed_cell_id)
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    GraphView { adjacency }
}

fn remove_incidence(view: &GraphView, removed_incidence_id: &Id) -> GraphView {
    let adjacency = view
        .adjacency
        .iter()
        .map(|(cell_id, edges)| {
            (
                cell_id.clone(),
                edges
                    .iter()
                    .filter(|edge| &edge.incidence_id != removed_incidence_id)
                    .cloned()
                    .collect(),
            )
        })
        .collect();
    GraphView { adjacency }
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    if normalized.is_empty() {
        Err(malformed(field, "value must not be empty after trimming"))
    } else {
        Ok(normalized)
    }
}

fn unique_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_texts(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::{Cell, Incidence, Space};

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn analytics_store() -> InMemorySpaceStore {
        let mut store = InMemorySpaceStore::new();
        store
            .insert_space(Space::new(id("space-a"), "Graph analytics"))
            .expect("insert space");
        for cell_id in ["a", "b", "c", "d"] {
            store
                .insert_cell(Cell::new(id(cell_id), id("space-a"), 0, "node"))
                .expect("insert cell");
        }
        for (incidence_id, from, to) in [
            ("edge-ab", "a", "b"),
            ("edge-bc", "b", "c"),
            ("edge-cd", "c", "d"),
        ] {
            store
                .insert_incidence(Incidence::new(
                    id(incidence_id),
                    id("space-a"),
                    id(from),
                    id(to),
                    "depends_on",
                    IncidenceOrientation::Undirected,
                ))
                .expect("insert incidence");
        }
        store
    }

    #[test]
    fn impact_cone_respects_seed_depth_and_reports_connectors() {
        let store = analytics_store();
        let input = GraphAnalyticsInput::new(id("space-a"))
            .with_seed_cell_ids([id("a")])
            .with_max_depth(2);

        let report = store.analyze_graph(&input).expect("analyze graph");

        assert_eq!(report.impact_cone_cell_ids, vec![id("a"), id("b"), id("c")]);
        assert_eq!(report.articulation_cell_ids, vec![id("b"), id("c")]);
        assert_eq!(
            report.bridge_incidence_ids,
            vec![id("edge-ab"), id("edge-bc"), id("edge-cd")]
        );
        assert_eq!(report.connected_components.len(), 1);

        let json = serde_json::to_string(&report).expect("serialize");
        let roundtrip: GraphAnalyticsReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, report);
    }

    #[test]
    fn relation_filter_limits_the_selected_graph_view() {
        let store = analytics_store();
        let input = GraphAnalyticsInput::new(id("space-a"))
            .with_seed_cell_ids([id("a")])
            .with_relation_type("other")
            .expect("valid relation type");

        let report = store.analyze_graph(&input).expect("analyze graph");

        assert_eq!(report.impact_cone_cell_ids, vec![id("a")]);
        assert_eq!(report.connected_components.len(), 4);
        assert!(report.articulation_cell_ids.is_empty());
        assert!(report.bridge_incidence_ids.is_empty());
    }

    #[test]
    fn strongly_connected_components_group_mutually_reachable_directed_cells() {
        let mut store = InMemorySpaceStore::new();
        store
            .insert_space(Space::new(id("space-a"), "Directed graph analytics"))
            .expect("insert space");
        for cell_id in ["a", "b", "c"] {
            store
                .insert_cell(Cell::new(id(cell_id), id("space-a"), 0, "node"))
                .expect("insert cell");
        }
        for (incidence_id, from, to) in [
            ("edge-ab", "a", "b"),
            ("edge-ba", "b", "a"),
            ("edge-bc", "b", "c"),
        ] {
            store
                .insert_incidence(Incidence::new(
                    id(incidence_id),
                    id("space-a"),
                    id(from),
                    id(to),
                    "flows_to",
                    IncidenceOrientation::Directed,
                ))
                .expect("insert incidence");
        }
        let input = GraphAnalyticsInput::new(id("space-a"))
            .with_relation_type("flows_to")
            .expect("relation type")
            .in_direction(TraversalDirection::Outgoing);

        let report = store.analyze_graph(&input).expect("analyze graph");

        assert_eq!(
            report.strongly_connected_components,
            vec![
                GraphAnalyticsComponent {
                    representative_cell_id: id("a"),
                    cell_ids: vec![id("a"), id("b")],
                },
                GraphAnalyticsComponent {
                    representative_cell_id: id("c"),
                    cell_ids: vec![id("c")],
                },
            ]
        );
    }

    #[test]
    fn graph_analytics_rejects_seed_cells_outside_the_space() {
        let mut store = analytics_store();
        store
            .insert_space(Space::new(id("space-b"), "Other"))
            .expect("insert second space");
        store
            .insert_cell(Cell::new(id("outside"), id("space-b"), 0, "node"))
            .expect("insert outside cell");
        let input = GraphAnalyticsInput::new(id("space-a")).with_seed_cell_ids([id("outside")]);

        let error = store
            .analyze_graph(&input)
            .expect_err("outside seed should fail");

        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn graph_analytics_deserialization_rejects_empty_relation_type() {
        let value = serde_json::json!({
            "space_id": "space-a",
            "seed_cell_ids": ["a"],
            "relation_types": ["  "],
            "direction": "both"
        });

        assert!(serde_json::from_value::<GraphAnalyticsInput>(value).is_err());
    }
}
