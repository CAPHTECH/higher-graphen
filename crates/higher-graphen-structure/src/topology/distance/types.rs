//! Deterministic finite distances between two persistence diagrams.
//!
//! This module compares two multisets of [`PersistenceInterval`] grouped by
//! homology dimension and reports, per dimension, the exact bottleneck distance
//! and the exact p-Wasserstein distance with explicit matched pairs. Each
//! interval becomes a point `(birth, death)` using `birth_stage_index` and a
//! death index derived from a shared ordered filtration stage list; open
//! intervals use a finite sentinel equal to the number of stages.
//!
//! All arithmetic is performed in doubled-integer units so every cost is exact
//! and every serialized field is byte-identical for equal input. The only
//! irrational quantity (the p-th root of a Wasserstein cost sum) is exposed as
//! a computed f64 accessor and is never serialized. The report is always a
//! [`ReviewStatus::Candidate`] comparison and never an accepted equivalence.

use crate::space::Dimension;
use crate::topology::PersistenceInterval;
use higher_graphen_core::{Id, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};

/// Stable obstruction type emitted when structural drift exceeds a threshold.
pub const STRUCTURAL_DRIFT_OBSTRUCTION_TYPE: &str = "structural_drift_exceeds_threshold";

/// Stable obstruction type emitted when both diagrams are empty in every dimension.
pub const EMPTY_DIAGRAM_PAIR_OBSTRUCTION_TYPE: &str = "empty_diagram_pair";

/// Bounded request comparing two persistence diagrams over a shared stage order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceDistanceRequest {
    /// Shared ordered filtration stage identifiers. Defines the death sentinel
    /// (the stage count) and the index of every referenced death stage id.
    pub stage_ids: Vec<Id>,
    /// First diagram, treated as a candidate fingerprint.
    pub left: Vec<PersistenceInterval>,
    /// Second diagram, treated as a candidate fingerprint.
    pub right: Vec<PersistenceInterval>,
    /// Wasserstein order `p`. Defaults to one. Zero is rejected.
    pub wasserstein_order: u32,
    /// Optional doubled bottleneck threshold for the drift review signal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_threshold_doubled: Option<u64>,
}

impl PersistenceDistanceRequest {
    /// Creates a request with order `p = 1` and no drift threshold.
    #[must_use]
    pub fn new(
        stage_ids: impl IntoIterator<Item = Id>,
        left: impl IntoIterator<Item = PersistenceInterval>,
        right: impl IntoIterator<Item = PersistenceInterval>,
    ) -> Self {
        Self {
            stage_ids: stage_ids.into_iter().collect(),
            left: left.into_iter().collect(),
            right: right.into_iter().collect(),
            wasserstein_order: 1,
            drift_threshold_doubled: None,
        }
    }

    /// Returns this request with an explicit Wasserstein order `p`.
    #[must_use]
    pub fn with_wasserstein_order(mut self, wasserstein_order: u32) -> Self {
        self.wasserstein_order = wasserstein_order;
        self
    }

    /// Returns this request with a doubled bottleneck drift threshold.
    #[must_use]
    pub fn with_drift_threshold_doubled(mut self, drift_threshold_doubled: u64) -> Self {
        self.drift_threshold_doubled = Some(drift_threshold_doubled);
        self
    }
}

/// Kind of matched pair in a Wasserstein optimal matching.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceMatchKind {
    /// A left point matched to a right point.
    PointToPoint,
    /// A left point matched to its diagonal projection.
    LeftToDiagonal,
    /// A right point matched to its diagonal projection.
    RightToDiagonal,
}

/// A point coordinate in a persistence diagram, in raw stage-index units.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistencePoint {
    /// Birth stage index.
    pub birth_index: usize,
    /// Death stage index, equal to the stage count for open intervals.
    pub death_index: usize,
}

/// One matched pair in a deterministic optimal Wasserstein matching.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceMatch {
    /// Match category.
    pub kind: PersistenceMatchKind,
    /// Left diagram point, absent for an unmatched right point on the diagonal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_point: Option<PersistencePoint>,
    /// Right diagram point, absent for an unmatched left point on the diagonal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_point: Option<PersistencePoint>,
    /// Doubled L-infinity (or doubled to-diagonal) cost of this match.
    pub cost_doubled: u64,
}

/// Per-dimension bottleneck and Wasserstein distance between two diagrams.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceDimensionDistance {
    /// Homology dimension compared.
    pub dimension: Dimension,
    /// Number of left points in this dimension.
    pub left_point_count: usize,
    /// Number of right points in this dimension.
    pub right_point_count: usize,
    /// Exact doubled bottleneck distance.
    pub bottleneck_doubled: u64,
    /// Exact sum of `(doubled_cost)^p` over the optimal Wasserstein matching.
    pub wasserstein_cost_power_sum_doubled: u128,
    /// Optimal Wasserstein matching, sorted deterministically.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matches: Vec<PersistenceMatch>,
}

impl PersistenceDimensionDistance {
    /// Returns the bottleneck distance in true (undoubled) units.
    #[must_use]
    pub fn bottleneck(&self) -> f64 {
        self.bottleneck_doubled as f64 / 2.0
    }

    /// Returns the p-Wasserstein distance in true units for the supplied order.
    #[must_use]
    pub fn wasserstein(&self, wasserstein_order: u32) -> f64 {
        if wasserstein_order == 0 {
            return f64::NAN;
        }
        let power_sum = self.wasserstein_cost_power_sum_doubled as f64;
        let doubled = power_sum.powf(1.0 / f64::from(wasserstein_order));
        doubled / 2.0
    }
}

/// Non-blocking review-signal obstruction from a diagram comparison.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceDistanceObstruction {
    /// Stable obstruction category.
    pub obstruction_type: String,
    /// Dimension the obstruction applies to, when dimension-specific.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<Dimension>,
    /// Advisory severity.
    pub severity: Severity,
    /// Human-readable deterministic explanation.
    pub reason: String,
}

/// Deterministic persistence-diagram distance report.
///
/// The report is always a [`ReviewStatus::Candidate`] comparison. It never
/// asserts that two diagrams are equivalent, even when all distances are zero.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PersistenceDistanceReport {
    /// Number of shared filtration stages, used as the open-interval sentinel.
    pub stage_count: usize,
    /// Wasserstein order `p` used by this report.
    pub wasserstein_order: u32,
    /// Per-dimension distances, sorted by dimension.
    pub dimensions: Vec<PersistenceDimensionDistance>,
    /// Maximum doubled bottleneck distance across dimensions, zero when empty.
    pub max_bottleneck_doubled: u64,
    /// Review status of this comparison. Always [`ReviewStatus::Candidate`].
    pub review_status: ReviewStatus,
    /// Non-blocking review-signal obstructions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<PersistenceDistanceObstruction>,
}
