use crate::space::Dimension;
use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};

/// Direction used when traversing incidences.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalDirection {
    /// Follow directed incidences from source to target.
    Outgoing,
    /// Follow directed incidences from target to source.
    Incoming,
    /// Follow directed incidences in either direction.
    Both,
}

/// Shared traversal controls for reachability and path walking.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraversalOptions {
    /// Direction used for directed incidences.
    pub direction: TraversalDirection,
    /// Optional allowed relation types. Empty means any relation type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_types: Vec<String>,
    /// Optional maximum number of incidences in a returned path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    /// Optional maximum number of paths returned by path-walking APIs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_paths: Option<usize>,
}

impl Default for TraversalOptions {
    fn default() -> Self {
        Self {
            direction: TraversalDirection::Outgoing,
            relation_types: Vec::new(),
            max_depth: None,
            max_paths: None,
        }
    }
}

impl TraversalOptions {
    /// Creates default outgoing traversal options.
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

    /// Returns these options with a maximum path depth.
    #[must_use]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Returns these options with a maximum returned path count.
    #[must_use]
    pub fn with_max_paths(mut self, max_paths: usize) -> Self {
        self.max_paths = Some(max_paths);
        self
    }
}

/// Query asking whether one cell can reach another cell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReachabilityQuery {
    /// Space that owns the traversed cells and incidences.
    pub space_id: Id,
    /// Start cell.
    pub from_cell_id: Id,
    /// Target cell.
    pub to_cell_id: Id,
    /// Traversal controls.
    pub options: TraversalOptions,
}

impl ReachabilityQuery {
    /// Creates an outgoing reachability query with no relation-type filter.
    #[must_use]
    pub fn new(space_id: Id, from_cell_id: Id, to_cell_id: Id) -> Self {
        Self {
            space_id,
            from_cell_id,
            to_cell_id,
            options: TraversalOptions::default(),
        }
    }

    /// Returns this query with traversal options.
    #[must_use]
    pub fn with_options(mut self, options: TraversalOptions) -> Self {
        self.options = options;
        self
    }
}

/// One traversed incidence in a graph path.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathStep {
    /// Cell at the beginning of this traversed step.
    pub from_cell_id: Id,
    /// Incidence used by this step.
    pub incidence_id: Id,
    /// Cell reached by this traversed step.
    pub to_cell_id: Id,
    /// Relation type copied from the traversed incidence.
    pub relation_type: String,
}

/// Witness path through cells and incidences.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPath {
    /// First cell in the path.
    pub start_cell_id: Id,
    /// Last cell in the path.
    pub end_cell_id: Id,
    /// Traversed incidences.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<PathStep>,
}

impl GraphPath {
    /// Creates an empty path at a start cell.
    #[must_use]
    pub fn new(start_cell_id: Id) -> Self {
        Self {
            start_cell_id: start_cell_id.clone(),
            end_cell_id: start_cell_id,
            steps: Vec::new(),
        }
    }

    /// Number of incidences traversed by this path.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    /// Cell sequence represented by the path.
    #[must_use]
    pub fn cell_ids(&self) -> Vec<Id> {
        let mut cell_ids = vec![self.start_cell_id.clone()];
        cell_ids.extend(self.steps.iter().map(|step| step.to_cell_id.clone()));
        cell_ids
    }

    pub(crate) fn append(&self, step: PathStep) -> Self {
        let mut path = self.clone();
        path.end_cell_id = step.to_cell_id.clone();
        path.steps.push(step);
        path
    }
}

/// Result of a reachability query.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReachabilityResult {
    /// Space that was traversed.
    pub space_id: Id,
    /// Start cell.
    pub from_cell_id: Id,
    /// Target cell.
    pub to_cell_id: Id,
    /// Whether a path to the target was found.
    pub reachable: bool,
    /// Shortest witness path when reachable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortest_path: Option<GraphPath>,
    /// Cells visited in traversal order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visited_cell_ids: Vec<Id>,
    /// Last reached cells when the target is not reachable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier_cell_ids: Vec<Id>,
}

/// Cell selector used by path-pattern matching.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CellPattern {
    /// Optional exact cell identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<Id>,
    /// Optional required cell type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_type: Option<String>,
    /// Optional required dimension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<Dimension>,
}

impl CellPattern {
    /// Creates a pattern that matches any cell.
    #[must_use]
    pub fn any() -> Self {
        Self::default()
    }

    /// Creates a pattern that matches one cell identifier.
    #[must_use]
    pub fn by_id(cell_id: Id) -> Self {
        Self {
            cell_id: Some(cell_id),
            cell_type: None,
            dimension: None,
        }
    }

    /// Returns this pattern with a cell type constraint.
    #[must_use]
    pub fn of_type(mut self, cell_type: impl Into<String>) -> Self {
        self.cell_type = Some(cell_type.into().trim().to_owned());
        self
    }

    /// Returns this pattern with a dimension constraint.
    #[must_use]
    pub fn with_dimension(mut self, dimension: Dimension) -> Self {
        self.dimension = Some(dimension);
        self
    }
}

/// One required edge and target-cell layer in a path pattern.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathPatternSegment {
    /// Optional required relation type for the edge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_type: Option<String>,
    /// Required target-cell pattern after traversing the edge.
    pub target: CellPattern,
}

impl PathPatternSegment {
    /// Creates a segment that accepts any relation type.
    #[must_use]
    pub fn new(target: CellPattern) -> Self {
        Self {
            relation_type: None,
            target,
        }
    }

    /// Returns this segment with a required relation type.
    #[must_use]
    pub fn with_relation_type(mut self, relation_type: impl Into<String>) -> Self {
        self.relation_type = Some(relation_type.into().trim().to_owned());
        self
    }
}

/// Query for detecting a fixed single-edge-per-layer path pattern.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathPattern {
    /// Space that owns the traversed cells and incidences.
    pub space_id: Id,
    /// Start-cell pattern.
    pub start: CellPattern,
    /// Required edge and target-cell layers.
    pub segments: Vec<PathPatternSegment>,
    /// Direction used for directed incidences.
    pub direction: TraversalDirection,
    /// Optional maximum number of matches to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_matches: Option<usize>,
}

impl PathPattern {
    /// Creates a pattern query with outgoing traversal.
    #[must_use]
    pub fn new(space_id: Id, start: CellPattern) -> Self {
        Self {
            space_id,
            start,
            segments: Vec::new(),
            direction: TraversalDirection::Outgoing,
            max_matches: None,
        }
    }

    /// Returns this pattern with an appended segment.
    #[must_use]
    pub fn then(mut self, segment: PathPatternSegment) -> Self {
        self.segments.push(segment);
        self
    }

    /// Returns this pattern with traversal direction.
    #[must_use]
    pub fn in_direction(mut self, direction: TraversalDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns this pattern with a maximum match count.
    #[must_use]
    pub fn with_max_matches(mut self, max_matches: usize) -> Self {
        self.max_matches = Some(max_matches);
        self
    }
}

/// One detected path-pattern match.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathPatternMatch {
    /// Witness path that satisfied the pattern.
    pub path: GraphPath,
    /// Cell identifiers matched by the start pattern and each segment target.
    pub matched_cell_ids: Vec<Id>,
}
