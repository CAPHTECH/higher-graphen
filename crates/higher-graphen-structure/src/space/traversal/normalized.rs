use super::{
    malformed, CellPattern, PathPattern, PathPatternSegment, TraversalDirection, TraversalOptions,
};
use crate::space::{Cell, Dimension};
use higher_graphen_core::{CoreError, Id, Result};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(crate) struct NormalizedTraversalOptions {
    pub(crate) direction: TraversalDirection,
    relation_types: BTreeSet<String>,
    pub(crate) max_depth: Option<usize>,
    pub(crate) max_paths: Option<usize>,
}

impl NormalizedTraversalOptions {
    pub(crate) fn allows_relation(&self, relation_type: &str) -> bool {
        self.relation_types.is_empty() || self.relation_types.contains(relation_type)
    }

    pub(crate) fn for_single_relation(
        direction: TraversalDirection,
        relation_type: &Option<String>,
    ) -> Self {
        Self {
            direction,
            relation_types: relation_type.iter().cloned().collect(),
            max_depth: None,
            max_paths: None,
        }
    }
}

impl TryFrom<&TraversalOptions> for NormalizedTraversalOptions {
    type Error = CoreError;

    fn try_from(options: &TraversalOptions) -> Result<Self> {
        if options.max_paths == Some(0) {
            return Err(malformed("max_paths", "value must be greater than zero"));
        }
        Ok(Self {
            direction: options.direction,
            relation_types: normalize_string_set("relation_types", &options.relation_types)?,
            max_depth: options.max_depth,
            max_paths: options.max_paths,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NormalizedCellPattern {
    pub(crate) cell_id: Option<Id>,
    cell_type: Option<String>,
    dimension: Option<Dimension>,
}

impl NormalizedCellPattern {
    pub(crate) fn matches(&self, cell: &Cell) -> bool {
        self.matches_id(cell) && self.matches_type(cell) && self.matches_dimension(cell)
    }

    fn matches_id(&self, cell: &Cell) -> bool {
        self.cell_id.as_ref().map_or(true, |id| &cell.id == id)
    }

    fn matches_type(&self, cell: &Cell) -> bool {
        self.cell_type
            .as_ref()
            .map_or(true, |cell_type| &cell.cell_type == cell_type)
    }

    fn matches_dimension(&self, cell: &Cell) -> bool {
        self.dimension
            .map_or(true, |dimension| cell.dimension == dimension)
    }
}

impl TryFrom<&CellPattern> for NormalizedCellPattern {
    type Error = CoreError;

    fn try_from(pattern: &CellPattern) -> Result<Self> {
        Ok(Self {
            cell_id: pattern.cell_id.clone(),
            cell_type: normalize_optional_string("cell_type", &pattern.cell_type)?,
            dimension: pattern.dimension,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NormalizedPathPatternSegment {
    pub(crate) relation_type: Option<String>,
    pub(crate) target: NormalizedCellPattern,
}

impl TryFrom<&PathPatternSegment> for NormalizedPathPatternSegment {
    type Error = CoreError;

    fn try_from(segment: &PathPatternSegment) -> Result<Self> {
        Ok(Self {
            relation_type: normalize_optional_string("relation_type", &segment.relation_type)?,
            target: NormalizedCellPattern::try_from(&segment.target)?,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NormalizedPathPattern {
    pub(crate) space_id: Id,
    pub(crate) start: NormalizedCellPattern,
    pub(crate) segments: Vec<NormalizedPathPatternSegment>,
    pub(crate) direction: TraversalDirection,
    pub(crate) max_matches: Option<usize>,
}

impl TryFrom<&PathPattern> for NormalizedPathPattern {
    type Error = CoreError;

    fn try_from(pattern: &PathPattern) -> Result<Self> {
        if pattern.segments.is_empty() {
            return Err(malformed("segments", "path pattern must include a segment"));
        }
        if pattern.max_matches == Some(0) {
            return Err(malformed("max_matches", "value must be greater than zero"));
        }
        Ok(Self {
            space_id: pattern.space_id.clone(),
            start: NormalizedCellPattern::try_from(&pattern.start)?,
            segments: normalized_segments(&pattern.segments)?,
            direction: pattern.direction,
            max_matches: pattern.max_matches,
        })
    }
}

fn normalized_segments(
    segments: &[PathPatternSegment],
) -> Result<Vec<NormalizedPathPatternSegment>> {
    segments
        .iter()
        .map(NormalizedPathPatternSegment::try_from)
        .collect()
}

fn normalize_string_set(field: &str, values: &[String]) -> Result<BTreeSet<String>> {
    values
        .iter()
        .map(|value| normalize_required(field, value))
        .collect()
}

fn normalize_optional_string(field: &str, value: &Option<String>) -> Result<Option<String>> {
    value
        .as_ref()
        .map(|value| normalize_required(field, value))
        .transpose()
}

fn normalize_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        Err(malformed(field, "value must not be empty after trimming"))
    } else {
        Ok(normalized)
    }
}
