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

use super::PersistenceInterval;
use crate::space::Dimension;
use higher_graphen_core::{CoreError, Id, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

/// Computes deterministic bottleneck and p-Wasserstein distances between two
/// persistence diagrams grouped by homology dimension.
///
/// # Errors
///
/// Returns a structured error when `stage_ids` is empty, contains a duplicate,
/// when `wasserstein_order` is zero, or when an interval references a death
/// stage id outside `stage_ids`.
pub fn persistence_distance(
    request: &PersistenceDistanceRequest,
) -> Result<PersistenceDistanceReport, CoreError> {
    if request.stage_ids.is_empty() {
        return Err(malformed(
            "stage_ids",
            "at least one filtration stage id is required",
        ));
    }
    if request.wasserstein_order == 0 {
        return Err(malformed(
            "wasserstein_order",
            "Wasserstein order must be at least one",
        ));
    }

    let mut stage_index_by_id = BTreeMap::new();
    for (index, stage_id) in request.stage_ids.iter().enumerate() {
        if stage_index_by_id.insert(stage_id.clone(), index).is_some() {
            return Err(malformed(
                "stage_ids",
                format!("stage id {stage_id} appears more than once"),
            ));
        }
    }
    let stage_count = request.stage_ids.len();

    let left_by_dimension = group_points("left", &request.left, &stage_index_by_id, stage_count)?;
    let right_by_dimension =
        group_points("right", &request.right, &stage_index_by_id, stage_count)?;

    let mut dimensions = BTreeSet::new();
    dimensions.extend(left_by_dimension.keys().copied());
    dimensions.extend(right_by_dimension.keys().copied());

    let empty_default = Vec::new();
    let mut dimension_distances = Vec::new();
    let mut max_bottleneck_doubled = 0u64;
    let mut obstructions = Vec::new();

    for dimension in dimensions {
        let left = left_by_dimension.get(&dimension).unwrap_or(&empty_default);
        let right = right_by_dimension.get(&dimension).unwrap_or(&empty_default);

        let bottleneck_doubled = bottleneck_doubled(left, right);
        let (wasserstein_cost_power_sum_doubled, matches) =
            wasserstein_matching(left, right, request.wasserstein_order);

        max_bottleneck_doubled = max_bottleneck_doubled.max(bottleneck_doubled);

        if let Some(threshold) = request.drift_threshold_doubled {
            if bottleneck_doubled > threshold {
                obstructions.push(PersistenceDistanceObstruction {
                    obstruction_type: STRUCTURAL_DRIFT_OBSTRUCTION_TYPE.to_owned(),
                    dimension: Some(dimension),
                    severity: Severity::Medium,
                    reason: format!(
                        "dimension {dimension} doubled bottleneck {bottleneck_doubled} exceeds drift threshold {threshold}"
                    ),
                });
            }
        }

        dimension_distances.push(PersistenceDimensionDistance {
            dimension,
            left_point_count: left.len(),
            right_point_count: right.len(),
            bottleneck_doubled,
            wasserstein_cost_power_sum_doubled,
            matches,
        });
    }

    if dimension_distances.is_empty() {
        obstructions.push(PersistenceDistanceObstruction {
            obstruction_type: EMPTY_DIAGRAM_PAIR_OBSTRUCTION_TYPE.to_owned(),
            dimension: None,
            severity: Severity::Low,
            reason: "both diagrams are empty in every dimension; no distance was computed"
                .to_owned(),
        });
    }

    Ok(PersistenceDistanceReport {
        stage_count,
        wasserstein_order: request.wasserstein_order,
        dimensions: dimension_distances,
        max_bottleneck_doubled,
        review_status: ReviewStatus::Candidate,
        obstructions,
    })
}

/// Normalized point plus the generator ids used for deterministic ordering.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct NormalizedPoint {
    birth_index: usize,
    death_index: usize,
    generator_cell_ids: Vec<Id>,
}

impl NormalizedPoint {
    fn point(&self) -> PersistencePoint {
        PersistencePoint {
            birth_index: self.birth_index,
            death_index: self.death_index,
        }
    }

    /// Doubled to-diagonal cost `2 * ((death - birth) / 2) = death - birth`.
    fn to_diagonal_doubled(&self) -> u64 {
        (self.death_index - self.birth_index) as u64
    }
}

fn group_points(
    field: &str,
    intervals: &[PersistenceInterval],
    stage_index_by_id: &BTreeMap<Id, usize>,
    stage_count: usize,
) -> Result<BTreeMap<Dimension, Vec<NormalizedPoint>>, CoreError> {
    let mut by_dimension: BTreeMap<Dimension, Vec<NormalizedPoint>> = BTreeMap::new();
    for interval in intervals {
        let death_index = match &interval.death_stage_id {
            Some(death_stage_id) => *stage_index_by_id.get(death_stage_id).ok_or_else(|| {
                malformed(
                    field,
                    format!("death stage id {death_stage_id} is not present in stage_ids"),
                )
            })?,
            None => stage_count,
        };
        let mut generator_cell_ids = interval.generator_cell_ids.clone();
        generator_cell_ids.sort();
        by_dimension
            .entry(interval.dimension)
            .or_default()
            .push(NormalizedPoint {
                birth_index: interval.birth_stage_index,
                death_index,
                generator_cell_ids,
            });
    }
    for points in by_dimension.values_mut() {
        points.sort();
    }
    Ok(by_dimension)
}

/// Doubled L-infinity distance between two diagram points.
fn linf_doubled(left: &NormalizedPoint, right: &NormalizedPoint) -> u64 {
    let birth_gap = left.birth_index.abs_diff(right.birth_index);
    let death_gap = left.death_index.abs_diff(right.death_index);
    2 * birth_gap.max(death_gap) as u64
}

/// Computes the exact doubled bottleneck distance between two diagrams.
fn bottleneck_doubled(left: &[NormalizedPoint], right: &[NormalizedPoint]) -> u64 {
    if left.is_empty() && right.is_empty() {
        return 0;
    }

    let mut candidates = BTreeSet::new();
    candidates.insert(0u64);
    for point in left.iter().chain(right.iter()) {
        candidates.insert(point.to_diagonal_doubled());
    }
    for left_point in left {
        for right_point in right {
            candidates.insert(linf_doubled(left_point, right_point));
        }
    }
    let candidates = candidates.into_iter().collect::<Vec<_>>();

    let mut low = 0usize;
    let mut high = candidates.len() - 1;
    while low < high {
        let mid = (low + high) / 2;
        if bottleneck_feasible(left, right, candidates[mid]) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    candidates[low]
}

/// Returns true when every point can be matched within the doubled threshold.
fn bottleneck_feasible(
    left: &[NormalizedPoint],
    right: &[NormalizedPoint],
    threshold_doubled: u64,
) -> bool {
    let left_count = left.len();
    let right_count = right.len();
    let size = left_count + right_count;

    // Feasibility uses the same augmented square assignment graph as the
    // Wasserstein solver, retaining only edges whose doubled cost is within
    // the threshold. Rows are left points followed by diagonal sources for
    // right points; columns are right points followed by diagonal sinks for
    // left points.
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); size];
    for (left_index, left_point) in left.iter().enumerate() {
        for (right_index, right_point) in right.iter().enumerate() {
            if linf_doubled(left_point, right_point) <= threshold_doubled {
                adjacency[left_index].push(right_index);
            }
        }
        if left_point.to_diagonal_doubled() <= threshold_doubled {
            adjacency[left_index].push(right_count + left_index);
        }
    }
    for (right_index, right_point) in right.iter().enumerate() {
        let row = left_count + right_index;
        if right_point.to_diagonal_doubled() <= threshold_doubled {
            adjacency[row].push(right_index);
        }
        for left_index in 0..left_count {
            adjacency[row].push(right_count + left_index);
        }
    }

    let mut match_row_for_column = vec![None::<usize>; size];
    for row in 0..size {
        let mut visited = vec![false; size];
        if !augment(row, &adjacency, &mut match_row_for_column, &mut visited) {
            return false;
        }
    }
    true
}

fn augment(
    left_index: usize,
    adjacency: &[Vec<usize>],
    match_left_for_right: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for &right_target in &adjacency[left_index] {
        if visited[right_target] {
            continue;
        }
        visited[right_target] = true;
        let free_or_reassignable = match match_left_for_right[right_target] {
            None => true,
            Some(other_left) => augment(other_left, adjacency, match_left_for_right, visited),
        };
        if free_or_reassignable {
            match_left_for_right[right_target] = Some(left_index);
            return true;
        }
    }
    false
}

const INFEASIBLE_COST: u128 = u128::MAX;

/// Computes the exact minimum `sum (doubled_cost)^p` and its matching.
fn wasserstein_matching(
    left: &[NormalizedPoint],
    right: &[NormalizedPoint],
    wasserstein_order: u32,
) -> (u128, Vec<PersistenceMatch>) {
    let left_count = left.len();
    let right_count = right.len();
    if left_count == 0 && right_count == 0 {
        return (0, Vec::new());
    }

    // Augmented square cost matrix of size (n + m) x (n + m).
    // Rows: 0..n left points, n..n+m diagonal sources for right points.
    // Cols: 0..m right points, m..m+n diagonal sinks for left points.
    let size = left_count + right_count;
    let mut cost = vec![vec![0u128; size]; size];
    for (left_index, left_point) in left.iter().enumerate() {
        let row = &mut cost[left_index];
        for (right_index, right_point) in right.iter().enumerate() {
            row[right_index] =
                pow_doubled(linf_doubled(left_point, right_point), wasserstein_order);
        }
        // Left point may only reach its own diagonal sink column; others infeasible.
        let diagonal_cost = pow_doubled(left_point.to_diagonal_doubled(), wasserstein_order);
        for (sink_index, slot) in row[right_count..].iter_mut().enumerate().take(left_count) {
            *slot = if sink_index == left_index {
                diagonal_cost
            } else {
                INFEASIBLE_COST
            };
        }
    }
    for (right_index, right_point) in right.iter().enumerate() {
        let row = &mut cost[left_count + right_index];
        // Right point may only reach its own diagonal source column; others infeasible.
        let diagonal_cost = pow_doubled(right_point.to_diagonal_doubled(), wasserstein_order);
        for (source_index, slot) in row.iter_mut().enumerate().take(right_count) {
            *slot = if source_index == right_index {
                diagonal_cost
            } else {
                INFEASIBLE_COST
            };
        }
        // Diagonal-to-diagonal block columns stay zero (already initialized).
    }

    let assignment = hungarian(&cost);

    let mut total = 0u128;
    let mut matches = Vec::new();
    for (row, &column) in assignment.iter().enumerate() {
        let entry = cost[row][column];
        if entry == INFEASIBLE_COST {
            // Unreachable for a valid augmented matrix; skip defensively.
            continue;
        }
        total = total.saturating_add(entry);

        let row_is_point = row < left_count;
        let column_is_point = column < right_count;
        match (row_is_point, column_is_point) {
            (true, true) => matches.push(PersistenceMatch {
                kind: PersistenceMatchKind::PointToPoint,
                left_point: Some(left[row].point()),
                right_point: Some(right[column].point()),
                cost_doubled: linf_doubled(&left[row], &right[column]),
            }),
            (true, false) => matches.push(PersistenceMatch {
                kind: PersistenceMatchKind::LeftToDiagonal,
                left_point: Some(left[row].point()),
                right_point: None,
                cost_doubled: left[row].to_diagonal_doubled(),
            }),
            (false, true) => matches.push(PersistenceMatch {
                kind: PersistenceMatchKind::RightToDiagonal,
                left_point: None,
                right_point: Some(right[column].point()),
                cost_doubled: right[column].to_diagonal_doubled(),
            }),
            (false, false) => {} // diagonal-to-diagonal: no reported pair.
        }
    }

    matches.sort();
    (total, matches)
}

fn pow_doubled(doubled_cost: u64, wasserstein_order: u32) -> u128 {
    let base = u128::from(doubled_cost);
    let mut value = 1u128;
    for _ in 0..wasserstein_order {
        value = value.saturating_mul(base);
    }
    value
}

/// Deterministic Kuhn-Munkres assignment minimizing total integer cost.
///
/// Returns, for each row, the assigned column. The implementation uses the
/// O(n^3) potential method with fixed iteration order, so equal cost matrices
/// always yield the same assignment.
fn hungarian(cost: &[Vec<u128>]) -> Vec<usize> {
    let n = cost.len();
    if n == 0 {
        return Vec::new();
    }

    // One-based potentials and matching arrays following the standard
    // Jonker-Volgenant style augmentation over a square matrix. The textbook
    // column potential is non-positive for minimization; store its negation so
    // all arithmetic can stay unsigned and saturating.
    let mut row_potential = vec![0u128; n + 1];
    let mut column_penalty = vec![0u128; n + 1];
    let mut column_match = vec![0usize; n + 1];
    let mut way = vec![0usize; n + 1];

    for row in 1..=n {
        column_match[0] = row;
        let mut current_column = 0usize;
        let mut min_slack = vec![INFEASIBLE_COST; n + 1];
        let mut used = vec![false; n + 1];

        loop {
            used[current_column] = true;
            let matched_row = column_match[current_column];
            let mut delta = INFEASIBLE_COST;
            let mut next_column = 0usize;

            for column in 1..=n {
                if used[column] {
                    continue;
                }
                let reduced = saturating_reduced_cost(
                    cost[matched_row - 1][column - 1],
                    row_potential[matched_row],
                    column_penalty[column],
                );
                if reduced < min_slack[column] {
                    min_slack[column] = reduced;
                    way[column] = current_column;
                }
                if min_slack[column] < delta {
                    delta = min_slack[column];
                    next_column = column;
                }
            }

            for column in 0..=n {
                if used[column] {
                    row_potential[column_match[column]] =
                        row_potential[column_match[column]].saturating_add(delta);
                    column_penalty[column] = column_penalty[column].saturating_add(delta);
                } else {
                    min_slack[column] = min_slack[column].saturating_sub(delta);
                }
            }

            current_column = next_column;
            if column_match[current_column] == 0 {
                break;
            }
        }

        loop {
            let previous_column = way[current_column];
            column_match[current_column] = column_match[previous_column];
            current_column = previous_column;
            if current_column == 0 {
                break;
            }
        }
    }

    let mut assignment = vec![0usize; n];
    for column in 1..=n {
        if column_match[column] != 0 {
            assignment[column_match[column] - 1] = column - 1;
        }
    }
    assignment
}

/// Computes `cost - row_potential - column_potential` without underflow.
///
/// The minimization algorithm's column potential is non-positive, and callers
/// pass its negation as `column_penalty`. The saturating form keeps the
/// [`INFEASIBLE_COST`] sentinel stable and avoids any panic path on integer
/// arithmetic.
fn saturating_reduced_cost(cost: u128, row_potential: u128, column_penalty: u128) -> u128 {
    if cost == INFEASIBLE_COST {
        return INFEASIBLE_COST;
    }
    cost.saturating_add(column_penalty)
        .saturating_sub(row_potential)
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
    use crate::space::Dimension as Dim;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn stages() -> Vec<Id> {
        vec![id("s0"), id("s1"), id("s2"), id("s3")]
    }

    fn interval(
        dimension: Dim,
        birth_index: usize,
        death_stage: Option<&str>,
        generator: &str,
    ) -> PersistenceInterval {
        PersistenceInterval {
            dimension,
            birth_stage_id: id(&format!("s{birth_index}")),
            birth_stage_index: birth_index,
            death_stage_id: death_stage.map(id),
            death_stage_index: death_stage.map(|stage| {
                stage
                    .trim_start_matches('s')
                    .parse::<usize>()
                    .expect("stage index")
            }),
            generator_cell_ids: vec![id(generator)],
        }
    }

    #[test]
    fn identical_diagrams_have_zero_distance() {
        let left = vec![
            interval(0, 0, Some("s2"), "g-a"),
            interval(1, 1, None, "g-b"),
        ];
        let request = PersistenceDistanceRequest::new(stages(), left.clone(), left);

        let report = persistence_distance(&request).expect("distance");

        assert_eq!(report.max_bottleneck_doubled, 0);
        for dimension in &report.dimensions {
            assert_eq!(dimension.bottleneck_doubled, 0);
            assert_eq!(dimension.wasserstein_cost_power_sum_doubled, 0);
            assert_eq!(dimension.bottleneck(), 0.0);
            assert_eq!(dimension.wasserstein(1), 0.0);
            assert_eq!(dimension.wasserstein(2), 0.0);
        }
        assert_eq!(report.review_status, ReviewStatus::Candidate);
        assert!(report.obstructions.is_empty());
    }

    /// Hand-computed example: one shifted feature and one diagonal-matched extra.
    ///
    /// Dimension 1, four stages so the open-interval sentinel is `4`.
    /// Left:  P = (birth 0, death 2)            -> doubled (0, 4).
    /// Right: Q = (birth 0, death 3)            -> doubled (0, 6); shifted P.
    ///        R = (birth 1, death 2)            -> doubled (2, 4); extra.
    /// L-inf(P, Q) = 1 (doubled 2); to-diagonal P = 1, Q = 1.5, R = 0.5.
    /// Bottleneck = 1 (match P-Q, R-diagonal). Doubled = 2.
    /// W1 = 1 + 0.5 = 1.5; doubled sum = 2 + 1 = 3.
    /// W2 = sqrt(1^2 + 0.5^2) = sqrt(1.25) = sqrt(5)/2; doubled power sum = 4 + 1 = 5.
    #[test]
    fn shifted_and_extra_feature_exact_values() {
        let left = vec![interval(1, 0, Some("s2"), "p")];
        let right = vec![
            interval(1, 0, Some("s3"), "q"),
            interval(1, 1, Some("s2"), "r"),
        ];

        let w1_report = persistence_distance(&PersistenceDistanceRequest::new(
            stages(),
            left.clone(),
            right.clone(),
        ))
        .expect("w1 distance");
        let w1 = &w1_report.dimensions[0];
        assert_eq!(w1.dimension, 1);
        assert_eq!(w1.left_point_count, 1);
        assert_eq!(w1.right_point_count, 2);
        assert_eq!(w1.bottleneck_doubled, 2);
        assert_eq!(w1.bottleneck(), 1.0);
        assert_eq!(w1.wasserstein_cost_power_sum_doubled, 3);
        assert_eq!(w1.wasserstein(1), 1.5);

        // Matching: P matched to Q (point-to-point, doubled cost 2),
        // R matched to its diagonal (doubled cost 1).
        let point_to_point = w1
            .matches
            .iter()
            .find(|m| m.kind == PersistenceMatchKind::PointToPoint)
            .expect("point-to-point match");
        assert_eq!(point_to_point.cost_doubled, 2);
        assert_eq!(
            point_to_point.left_point,
            Some(PersistencePoint {
                birth_index: 0,
                death_index: 2
            })
        );
        assert_eq!(
            point_to_point.right_point,
            Some(PersistencePoint {
                birth_index: 0,
                death_index: 3
            })
        );
        let right_to_diagonal = w1
            .matches
            .iter()
            .find(|m| m.kind == PersistenceMatchKind::RightToDiagonal)
            .expect("right-to-diagonal match");
        assert_eq!(right_to_diagonal.cost_doubled, 1);

        let w2_report = persistence_distance(
            &PersistenceDistanceRequest::new(stages(), left, right).with_wasserstein_order(2),
        )
        .expect("w2 distance");
        let w2 = &w2_report.dimensions[0];
        assert_eq!(w2.bottleneck_doubled, 2);
        assert_eq!(w2.wasserstein_cost_power_sum_doubled, 5);
        let expected_w2 = (5.0_f64).sqrt() / 2.0;
        assert!((w2.wasserstein(2) - expected_w2).abs() < 1e-12);
    }

    #[test]
    fn distance_is_symmetric() {
        let left = vec![
            interval(0, 0, Some("s2"), "a"),
            interval(1, 0, Some("s3"), "b"),
            interval(1, 1, None, "c"),
        ];
        let right = vec![
            interval(0, 0, Some("s1"), "d"),
            interval(1, 0, Some("s2"), "e"),
        ];

        for order in [1u32, 2, 3] {
            let forward = persistence_distance(
                &PersistenceDistanceRequest::new(stages(), left.clone(), right.clone())
                    .with_wasserstein_order(order),
            )
            .expect("forward");
            let backward = persistence_distance(
                &PersistenceDistanceRequest::new(stages(), right.clone(), left.clone())
                    .with_wasserstein_order(order),
            )
            .expect("backward");

            let forward_by_dim = forward
                .dimensions
                .iter()
                .map(|d| {
                    (
                        d.dimension,
                        d.bottleneck_doubled,
                        d.wasserstein_cost_power_sum_doubled,
                    )
                })
                .collect::<Vec<_>>();
            let backward_by_dim = backward
                .dimensions
                .iter()
                .map(|d| {
                    (
                        d.dimension,
                        d.bottleneck_doubled,
                        d.wasserstein_cost_power_sum_doubled,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(forward_by_dim, backward_by_dim);
        }
    }

    #[test]
    fn drift_threshold_emits_non_blocking_review_signal() {
        let left = vec![interval(1, 0, Some("s2"), "p")];
        let right = vec![
            interval(1, 0, Some("s3"), "q"),
            interval(1, 1, Some("s2"), "r"),
        ];
        let request =
            PersistenceDistanceRequest::new(stages(), left, right).with_drift_threshold_doubled(1);

        let report = persistence_distance(&request).expect("distance");

        assert_eq!(report.review_status, ReviewStatus::Candidate);
        let drift = report
            .obstructions
            .iter()
            .filter(|o| o.obstruction_type == STRUCTURAL_DRIFT_OBSTRUCTION_TYPE)
            .collect::<Vec<_>>();
        assert_eq!(drift.len(), 1);
        assert_eq!(drift[0].dimension, Some(1));
        assert_eq!(drift[0].severity, Severity::Medium);
    }

    #[test]
    fn empty_diagram_pair_is_reported_not_zero_distance() {
        let request = PersistenceDistanceRequest::new(stages(), Vec::new(), Vec::new());

        let report = persistence_distance(&request).expect("distance");

        assert!(report.dimensions.is_empty());
        assert_eq!(report.max_bottleneck_doubled, 0);
        assert_eq!(report.obstructions.len(), 1);
        assert_eq!(
            report.obstructions[0].obstruction_type,
            EMPTY_DIAGRAM_PAIR_OBSTRUCTION_TYPE
        );
        assert_eq!(report.review_status, ReviewStatus::Candidate);
    }

    #[test]
    fn one_sided_diagram_matches_everything_to_diagonal() {
        // Only the left diagram has points; each must match the diagonal.
        let left = vec![
            interval(0, 0, Some("s2"), "a"),
            interval(0, 1, Some("s3"), "b"),
        ];
        let request = PersistenceDistanceRequest::new(stages(), left, Vec::new());

        let report = persistence_distance(&request).expect("distance");
        let dimension = &report.dimensions[0];
        // to-diagonal doubled: a -> (2-0)=2, b -> (3-1)=2. Bottleneck = max = 2.
        assert_eq!(dimension.bottleneck_doubled, 2);
        // W1 doubled sum = 2 + 2 = 4.
        assert_eq!(dimension.wasserstein_cost_power_sum_doubled, 4);
        assert_eq!(dimension.matches.len(), 2);
        assert!(dimension
            .matches
            .iter()
            .all(|m| m.kind == PersistenceMatchKind::LeftToDiagonal));
    }

    /// Distant pair: matching both points to the diagonal is strictly optimal.
    ///
    /// Six stages (sentinel 6). Left A = (0, 5) -> doubled (0, 10), to-diagonal
    /// doubled 5. Right B = (4, 5) -> doubled (8, 10), to-diagonal doubled 1.
    /// L-inf(A, B) doubled = 8. Bottleneck prefers diagonal: max(5, 1) = 5 < 8.
    /// W1 prefers diagonal: 5 + 1 = 6 < 8.
    #[test]
    fn distant_pair_prefers_diagonal_in_both_metrics() {
        let six_stages = vec![id("s0"), id("s1"), id("s2"), id("s3"), id("s4"), id("s5")];
        let left = vec![interval(0, 0, Some("s5"), "a")];
        let right = vec![interval(0, 4, Some("s5"), "b")];
        let request = PersistenceDistanceRequest::new(six_stages, left, right);

        let report = persistence_distance(&request).expect("distance");
        let dimension = &report.dimensions[0];
        assert_eq!(dimension.bottleneck_doubled, 5);
        assert_eq!(dimension.wasserstein_cost_power_sum_doubled, 6);
        assert_eq!(dimension.matches.len(), 2);
        assert!(dimension.matches.iter().all(|m| matches!(
            m.kind,
            PersistenceMatchKind::LeftToDiagonal | PersistenceMatchKind::RightToDiagonal
        )));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct OraclePoint {
        birth_index: usize,
        death_index: usize,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct OracleResult {
        wasserstein_p1: u128,
        wasserstein_p2: u128,
        bottleneck: u64,
    }

    struct SplitMix64 {
        state: u64,
    }

    impl SplitMix64 {
        fn new(seed: u64) -> Self {
            Self { state: seed }
        }

        fn next_u64(&mut self) -> u64 {
            self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.state;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn usize_inclusive(&mut self, min: usize, max: usize) -> usize {
            let span = max - min + 1;
            min + (self.next_u64() % span as u64) as usize
        }
    }

    fn stages_for_count(stage_count: usize) -> Vec<Id> {
        (0..stage_count)
            .map(|index| id(&format!("s{index}")))
            .collect()
    }

    fn oracle_interval(
        dimension: Dim,
        stage_count: usize,
        point: OraclePoint,
        generator: &str,
    ) -> PersistenceInterval {
        let death_stage_id =
            (point.death_index < stage_count).then(|| id(&format!("s{}", point.death_index)));
        PersistenceInterval {
            dimension,
            birth_stage_id: id(&format!("s{}", point.birth_index)),
            birth_stage_index: point.birth_index,
            death_stage_id,
            death_stage_index: (point.death_index < stage_count).then_some(point.death_index),
            generator_cell_ids: vec![id(generator)],
        }
    }

    fn oracle_intervals(
        dimension: Dim,
        stage_count: usize,
        side: &str,
        points: &[OraclePoint],
    ) -> Vec<PersistenceInterval> {
        points
            .iter()
            .enumerate()
            .map(|(index, &point)| {
                oracle_interval(
                    dimension,
                    stage_count,
                    point,
                    &format!("oracle-{side}-{index}"),
                )
            })
            .collect()
    }

    fn random_oracle_point(rng: &mut SplitMix64, stage_count: usize) -> OraclePoint {
        let birth_index = rng.usize_inclusive(0, stage_count - 1);
        let death_index = rng.usize_inclusive(birth_index, stage_count);
        OraclePoint {
            birth_index,
            death_index,
        }
    }

    fn linf_oracle_doubled(left: OraclePoint, right: OraclePoint) -> u64 {
        2 * left
            .birth_index
            .abs_diff(right.birth_index)
            .max(left.death_index.abs_diff(right.death_index)) as u64
    }

    fn diagonal_oracle_doubled(point: OraclePoint) -> u64 {
        (point.death_index - point.birth_index) as u64
    }

    fn brute_force_oracle(left: &[OraclePoint], right: &[OraclePoint]) -> OracleResult {
        let left_count = left.len();
        let right_count = right.len();
        let size = left_count + right_count;
        if size == 0 {
            return OracleResult {
                wasserstein_p1: 0,
                wasserstein_p2: 0,
                bottleneck: 0,
            };
        }

        let mut cost = vec![vec![None; size]; size];
        for (left_index, &left_point) in left.iter().enumerate() {
            for (right_index, &right_point) in right.iter().enumerate() {
                cost[left_index][right_index] = Some(linf_oracle_doubled(left_point, right_point));
            }
            cost[left_index][right_count + left_index] = Some(diagonal_oracle_doubled(left_point));
        }
        for (right_index, &right_point) in right.iter().enumerate() {
            cost[left_count + right_index][right_index] =
                Some(diagonal_oracle_doubled(right_point));
            for left_index in 0..left_count {
                cost[left_count + right_index][right_count + left_index] = Some(0);
            }
        }

        let mut used = vec![false; size];
        let mut permutation = vec![0usize; size];
        let mut best = None;
        enumerate_oracle_permutations(0, &cost, &mut used, &mut permutation, &mut best);
        best.expect("augmented oracle matrix has at least one feasible permutation")
    }

    fn enumerate_oracle_permutations(
        row: usize,
        cost: &[Vec<Option<u64>>],
        used: &mut [bool],
        permutation: &mut [usize],
        best: &mut Option<OracleResult>,
    ) {
        if row == cost.len() {
            let mut wasserstein_p1 = 0u128;
            let mut wasserstein_p2 = 0u128;
            let mut bottleneck = 0u64;
            for (row_index, &column_index) in permutation.iter().enumerate() {
                let entry = cost[row_index][column_index].expect("permutation is feasible");
                wasserstein_p1 += u128::from(entry);
                wasserstein_p2 += u128::from(entry) * u128::from(entry);
                bottleneck = bottleneck.max(entry);
            }

            match best {
                Some(best) => {
                    best.wasserstein_p1 = best.wasserstein_p1.min(wasserstein_p1);
                    best.wasserstein_p2 = best.wasserstein_p2.min(wasserstein_p2);
                    best.bottleneck = best.bottleneck.min(bottleneck);
                }
                None => {
                    *best = Some(OracleResult {
                        wasserstein_p1,
                        wasserstein_p2,
                        bottleneck,
                    });
                }
            }
            return;
        }

        for column in 0..cost.len() {
            if used[column] || cost[row][column].is_none() {
                continue;
            }
            used[column] = true;
            permutation[row] = column;
            enumerate_oracle_permutations(row + 1, cost, used, permutation, best);
            used[column] = false;
        }
    }

    fn assert_oracle_case(
        label: &str,
        stage_count: usize,
        left_points: &[OraclePoint],
        right_points: &[OraclePoint],
    ) {
        let dimension = 0;
        let oracle = brute_force_oracle(left_points, right_points);
        let left = oracle_intervals(dimension, stage_count, "left", left_points);
        let right = oracle_intervals(dimension, stage_count, "right", right_points);
        let stage_ids = stages_for_count(stage_count);

        let p1_report = persistence_distance(&PersistenceDistanceRequest::new(
            stage_ids.clone(),
            left.clone(),
            right.clone(),
        ))
        .expect("p=1 distance");
        let p2_report = persistence_distance(
            &PersistenceDistanceRequest::new(stage_ids, left, right).with_wasserstein_order(2),
        )
        .expect("p=2 distance");

        let p1_dimension = p1_report
            .dimensions
            .iter()
            .find(|distance| distance.dimension == dimension);
        let p2_dimension = p2_report
            .dimensions
            .iter()
            .find(|distance| distance.dimension == dimension);

        let kernel_p1 =
            p1_dimension.map_or(0, |distance| distance.wasserstein_cost_power_sum_doubled);
        let kernel_p2 =
            p2_dimension.map_or(0, |distance| distance.wasserstein_cost_power_sum_doubled);
        let kernel_bottleneck = p1_dimension.map_or(0, |distance| distance.bottleneck_doubled);

        assert_metric_eq(
            label,
            stage_count,
            left_points,
            right_points,
            "wasserstein p=1 power sum",
            oracle.wasserstein_p1,
            kernel_p1,
        );
        assert_metric_eq(
            label,
            stage_count,
            left_points,
            right_points,
            "wasserstein p=2 power sum",
            oracle.wasserstein_p2,
            kernel_p2,
        );
        assert_metric_eq(
            label,
            stage_count,
            left_points,
            right_points,
            "bottleneck",
            u128::from(oracle.bottleneck),
            u128::from(kernel_bottleneck),
        );
    }

    fn assert_metric_eq(
        label: &str,
        stage_count: usize,
        left_points: &[OraclePoint],
        right_points: &[OraclePoint],
        metric: &str,
        expected: u128,
        actual: u128,
    ) {
        if expected != actual {
            panic!(
                "brute-force oracle mismatch in {label}: metric={metric}, stage_count={stage_count}, left={left_points:?}, right={right_points:?}, oracle_expected={expected}, kernel_actual={actual}"
            );
        }
    }

    #[test]
    fn brute_force_oracle_randomized() {
        const SEEDS: [u64; 8] = [
            0x0000_0000_0000_0000,
            0x0123_4567_89ab_cdef,
            0xfedc_ba98_7654_3210,
            0x9e37_79b9_7f4a_7c15,
            0x243f_6a88_85a3_08d3,
            0x1319_8a2e_0370_7344,
            0xa409_3822_299f_31d0,
            0x082e_fa98_ec4e_6c89,
        ];
        const CASES_PER_SEED: usize = 25;

        let mut case_count = 0usize;
        for seed in SEEDS {
            let mut rng = SplitMix64::new(seed);
            for case_index in 0..CASES_PER_SEED {
                let stage_count = rng.usize_inclusive(5, 8);
                let left_count = rng.usize_inclusive(0, 3);
                let mut right_count = rng.usize_inclusive(0, 3);
                if left_count == 0 && right_count == 0 {
                    right_count = 1;
                }

                let left_points = (0..left_count)
                    .map(|_| random_oracle_point(&mut rng, stage_count))
                    .collect::<Vec<_>>();
                let right_points = (0..right_count)
                    .map(|_| random_oracle_point(&mut rng, stage_count))
                    .collect::<Vec<_>>();
                assert_oracle_case(
                    &format!("random seed={seed:#018x} case={case_index}"),
                    stage_count,
                    &left_points,
                    &right_points,
                );
                case_count += 1;
            }
        }

        assert_eq!(case_count, SEEDS.len() * CASES_PER_SEED);
        assert!(case_count >= 200);
    }

    #[test]
    fn brute_force_oracle_regression_w2_69_case() {
        assert_oracle_case(
            "regression original w2 69 case",
            8,
            &[
                OraclePoint {
                    birth_index: 2,
                    death_index: 8,
                },
                OraclePoint {
                    birth_index: 4,
                    death_index: 4,
                },
            ],
            &[
                OraclePoint {
                    birth_index: 0,
                    death_index: 2,
                },
                OraclePoint {
                    birth_index: 0,
                    death_index: 8,
                },
                OraclePoint {
                    birth_index: 1,
                    death_index: 8,
                },
            ],
        );
    }

    #[test]
    fn brute_force_oracle_degenerate_cases() {
        assert_oracle_case(
            "degenerate empty left vs two-point right",
            6,
            &[],
            &[
                OraclePoint {
                    birth_index: 0,
                    death_index: 2,
                },
                OraclePoint {
                    birth_index: 3,
                    death_index: 6,
                },
            ],
        );

        let duplicated = [
            OraclePoint {
                birth_index: 2,
                death_index: 5,
            },
            OraclePoint {
                birth_index: 2,
                death_index: 5,
            },
        ];
        assert_oracle_case(
            "degenerate duplicate identical points on both sides",
            7,
            &duplicated,
            &duplicated,
        );

        assert_oracle_case(
            "degenerate far-apart points prefer diagonal",
            8,
            &[
                OraclePoint {
                    birth_index: 0,
                    death_index: 3,
                },
                OraclePoint {
                    birth_index: 0,
                    death_index: 2,
                },
            ],
            &[
                OraclePoint {
                    birth_index: 5,
                    death_index: 8,
                },
                OraclePoint {
                    birth_index: 6,
                    death_index: 8,
                },
            ],
        );
    }

    #[test]
    fn determinism_byte_identical_for_shuffled_input() {
        let ordered_left = vec![
            interval(0, 0, Some("s2"), "a"),
            interval(1, 0, Some("s3"), "b"),
            interval(1, 1, None, "c"),
        ];
        let ordered_right = vec![
            interval(0, 0, Some("s1"), "d"),
            interval(1, 0, Some("s2"), "e"),
            interval(1, 1, Some("s3"), "f"),
        ];
        let shuffled_left = vec![
            interval(1, 1, None, "c"),
            interval(0, 0, Some("s2"), "a"),
            interval(1, 0, Some("s3"), "b"),
        ];
        let shuffled_right = vec![
            interval(1, 0, Some("s2"), "e"),
            interval(1, 1, Some("s3"), "f"),
            interval(0, 0, Some("s1"), "d"),
        ];

        let ordered = persistence_distance(&PersistenceDistanceRequest::new(
            stages(),
            ordered_left,
            ordered_right,
        ))
        .expect("ordered");
        let shuffled = persistence_distance(&PersistenceDistanceRequest::new(
            stages(),
            shuffled_left,
            shuffled_right,
        ))
        .expect("shuffled");

        let ordered_json = serde_json::to_string(&ordered).expect("serialize ordered");
        let shuffled_json = serde_json::to_string(&shuffled).expect("serialize shuffled");
        assert_eq!(ordered_json, shuffled_json);
    }

    #[test]
    fn json_round_trip_reproduces_report() {
        let left = vec![interval(1, 0, Some("s2"), "p")];
        let right = vec![
            interval(1, 0, Some("s3"), "q"),
            interval(1, 1, Some("s2"), "r"),
        ];
        let request = PersistenceDistanceRequest::new(stages(), left, right)
            .with_drift_threshold_doubled(1)
            .with_wasserstein_order(2);

        let report = persistence_distance(&request).expect("distance");
        let json = serde_json::to_string(&report).expect("serialize");
        let roundtrip: PersistenceDistanceReport =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(roundtrip, report);

        // Request round-trip too.
        let request_json = serde_json::to_string(&request).expect("serialize request");
        let request_roundtrip: PersistenceDistanceRequest =
            serde_json::from_str(&request_json).expect("deserialize request");
        assert_eq!(request_roundtrip, request);
    }

    #[test]
    fn rejects_empty_stage_list() {
        let request = PersistenceDistanceRequest::new(Vec::new(), Vec::new(), Vec::new());
        let error = persistence_distance(&request).expect_err("empty stages rejected");
        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn rejects_zero_wasserstein_order() {
        let request = PersistenceDistanceRequest::new(stages(), Vec::new(), Vec::new())
            .with_wasserstein_order(0);
        let error = persistence_distance(&request).expect_err("zero order rejected");
        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn rejects_duplicate_stage_id() {
        let request =
            PersistenceDistanceRequest::new(vec![id("s0"), id("s0")], Vec::new(), Vec::new());
        let error = persistence_distance(&request).expect_err("duplicate stage rejected");
        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn rejects_unknown_death_stage_id() {
        let left = vec![interval(0, 0, Some("s9"), "a")];
        let request = PersistenceDistanceRequest::new(stages(), left, Vec::new());
        let error = persistence_distance(&request).expect_err("unknown death stage rejected");
        assert_eq!(error.code(), "malformed_field");
    }
}
