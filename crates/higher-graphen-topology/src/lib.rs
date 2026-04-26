//! Finite topology, homology, and persistence summaries for HigherGraphen complexes.
//!
//! This crate implements a deterministic finite homology kernel over Z2. It
//! computes boundary ranks, cycle ranks, Betti numbers, and persistence
//! intervals from HigherGraphen cell boundaries. The legacy graph-oriented
//! fields remain available for connected-component and simple-cycle diagnostics.

use higher_graphen_core::{CoreError, Id, Result};
use higher_graphen_space::{Cell, Complex, Dimension, InMemorySpaceStore};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Stable obstruction type used when topology detects uncovered boundary cells.
pub const UNCOVERED_REGION_OBSTRUCTION_TYPE: &str = "uncovered_region";

/// Stable obstruction type used when cells are outside the supported finite kernel.
pub const UNSUPPORTED_DIMENSION_OBSTRUCTION_TYPE: &str = "custom:unsupported_topology_dimension";

/// Kind of structured finding emitted by topology summarization.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyFindingKind {
    /// A cell boundary is present in the store but outside the summarized region.
    ExternalBoundaryCell,
    /// A dimension-1 cell cannot be treated as a simple graph edge.
    NonGraphEdgeBoundary,
    /// A boundary reference skips at least one chain dimension.
    NonCodimensionOneBoundary,
    /// The finite boundary operators violate d(d(cell)) = 0 over Z2.
    BoundaryOperatorCompositionNonZero,
    /// A cell dimension is outside a bounded topology kernel.
    UnsupportedDimension,
}

/// Structured, serializable topology finding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyFinding {
    /// Finding category.
    pub finding_type: TopologyFindingKind,
    /// Stable obstruction type when the finding maps to an obstruction family.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obstruction_type: Option<String>,
    /// Cell where the finding was detected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_id: Option<Id>,
    /// Related cells that explain or witness the finding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_cell_ids: Vec<Id>,
    /// Human-readable deterministic explanation.
    pub description: String,
}

/// Connected component over the valid 1-skeleton vertices and graph edges.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectedComponentSummary {
    /// Deterministic representative vertex for the component.
    pub representative_cell_id: Id,
    /// Dimension-0 cells in the component.
    pub vertex_cell_ids: Vec<Id>,
    /// Valid dimension-1 graph edges whose endpoints are in this component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_cell_ids: Vec<Id>,
}

/// Simple cycle witness created by a redundant graph edge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SimpleCycleIndicator {
    /// Edge whose insertion closed the witnessed cycle.
    pub witness_edge_id: Id,
    /// Ordered path vertices found in the deterministic spanning forest.
    pub vertex_cell_ids: Vec<Id>,
    /// Path edges plus the witness edge.
    pub edge_cell_ids: Vec<Id>,
}

/// Coefficient field used by the finite homology engine.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum HomologyCoefficientField {
    /// Field with two elements. Boundary orientations are ignored.
    #[default]
    Z2,
}

/// Per-dimension homology summary over the selected finite cell set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomologyDimensionSummary {
    /// Chain dimension.
    pub dimension: Dimension,
    /// Rank of the chain group C_n.
    pub chain_rank: usize,
    /// Rank of the boundary operator d_n: C_n -> C_{n-1}.
    pub boundary_rank: usize,
    /// Rank of the cycle group ker d_n.
    pub cycle_rank: usize,
    /// Rank of the boundary subgroup im d_{n+1}.
    pub bounding_chain_rank: usize,
    /// Rank of H_n = ker d_n / im d_{n+1}.
    pub homology_rank: usize,
}

/// Deterministic finite homology summary over Z2.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomologySummary {
    /// Coefficient field used by this summary.
    pub coefficient_field: HomologyCoefficientField,
    /// Per-dimension ranks ordered by dimension.
    pub dimensions: Vec<HomologyDimensionSummary>,
    /// Alternating sum of chain ranks.
    pub euler_characteristic: i64,
}

impl HomologySummary {
    /// Returns the Betti number for one dimension, or zero when absent.
    #[must_use]
    pub fn betti_number(&self, dimension: Dimension) -> usize {
        self.dimensions
            .iter()
            .find(|summary| summary.dimension == dimension)
            .map_or(0, |summary| summary.homology_rank)
    }
}

/// Deterministic topology summary for a finite complex or active stage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologySummary {
    /// Complex that was summarized.
    pub complex_id: Id,
    /// Number of dimension-0 cells included in the valid 1-skeleton.
    pub vertex_count: usize,
    /// Number of valid dimension-1 graph edges included in the 1-skeleton.
    pub graph_edge_count: usize,
    /// Number of connected components over the valid 1-skeleton.
    pub component_count: usize,
    /// Connected components ordered by representative cell id.
    pub connected_components: Vec<ConnectedComponentSummary>,
    /// Finite homology summary over Z2 for every represented dimension.
    #[serde(default)]
    pub homology: HomologySummary,
    /// First Betti number from the Z2 homology summary.
    pub first_betti_number: usize,
    /// Alias for the Z2 H1 rank.
    pub simple_hole_count: usize,
    /// True when the 1-skeleton contains at least one simple cycle witness.
    pub has_simple_cycle: bool,
    /// Deterministic simple cycle witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub simple_cycles: Vec<SimpleCycleIndicator>,
    /// Findings for uncovered boundaries, invalid chain structure, and graph diagnostics.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<TopologyFinding>,
}

impl TopologySummary {
    /// Returns true when all valid vertices are in a single component.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.vertex_count > 0 && self.component_count == 1
    }
}

/// Cumulative filtration stage over a complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FiltrationStage {
    /// Stable stage identifier.
    pub id: Id,
    /// Cumulative set of active complex cells at this stage.
    pub cell_ids: Vec<Id>,
}

impl FiltrationStage {
    /// Creates a cumulative filtration stage.
    #[must_use]
    pub fn new(id: Id, cell_ids: impl IntoIterator<Item = Id>) -> Self {
        Self {
            id,
            cell_ids: cell_ids.into_iter().collect(),
        }
    }
}

/// Options controlling persistence interval reporting.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceOptions {
    /// Minimum lifetime in stage steps required for `persistent_intervals`.
    pub min_lifetime_stages: usize,
}

impl PersistenceOptions {
    /// Creates options that report every interval as persistent.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns options with a minimum interval lifetime threshold.
    #[must_use]
    pub fn with_min_lifetime_stages(mut self, min_lifetime_stages: usize) -> Self {
        self.min_lifetime_stages = min_lifetime_stages;
        self
    }
}

/// Topology summary for one filtration stage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FiltrationStageSummary {
    /// Stable stage identifier.
    pub stage_id: Id,
    /// Zero-based stage position in the supplied filtration.
    pub stage_index: usize,
    /// Cumulative active cells at this stage, sorted by id.
    pub cell_ids: Vec<Id>,
    /// Topology summary for this active stage.
    pub topology: TopologySummary,
}

/// Persistence interval for a simple H0 or H1 feature.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceInterval {
    /// Homology dimension.
    pub dimension: Dimension,
    /// Stage where the feature appears.
    pub birth_stage_id: Id,
    /// Zero-based birth stage index.
    pub birth_stage_index: usize,
    /// Stage where the feature disappears, absent for open intervals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_stage_id: Option<Id>,
    /// Zero-based death stage index, absent for open intervals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_stage_index: Option<usize>,
    /// Cells that deterministically generated the interval.
    pub generator_cell_ids: Vec<Id>,
}

impl PersistenceInterval {
    /// Returns true when the interval has no death stage.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.death_stage_id.is_none()
    }

    /// Returns the lifetime in stage steps through the last supplied stage.
    #[must_use]
    pub fn lifetime_stages(&self, last_stage_index: usize) -> usize {
        match self.death_stage_index {
            Some(death_stage_index) => death_stage_index.saturating_sub(self.birth_stage_index),
            None => last_stage_index
                .saturating_add(1)
                .saturating_sub(self.birth_stage_index),
        }
    }
}

/// Persistence summary across cumulative filtration stages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceSummary {
    /// Complex whose filtration was summarized.
    pub complex_id: Id,
    /// Options used to derive `persistent_intervals`.
    pub options: PersistenceOptions,
    /// Per-stage topology summaries.
    pub stages: Vec<FiltrationStageSummary>,
    /// All component and simple-cycle intervals.
    pub intervals: Vec<PersistenceInterval>,
    /// Intervals whose lifetime satisfies `options.min_lifetime_stages`.
    pub persistent_intervals: Vec<PersistenceInterval>,
    /// Number of open H0 component intervals.
    pub open_component_count: usize,
    /// Number of open H1 simple-cycle intervals.
    pub open_hole_count: usize,
}

mod analysis;
pub use analysis::{
    summarize_complex, summarize_complex_cells, summarize_filtration,
    summarize_filtration_with_options,
};

#[cfg(test)]
mod tests;
