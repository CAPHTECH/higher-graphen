use super::super::PersistenceInterval;
use super::*;
use crate::space::Dimension;
use higher_graphen_core::{CoreError, Id, ReviewStatus, Severity};
use std::collections::{BTreeMap, BTreeSet};

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
    validate_request(request)?;
    let stage_index_by_id = stage_index_by_id(&request.stage_ids)?;
    let stage_count = request.stage_ids.len();
    let left_by_dimension = group_points("left", &request.left, &stage_index_by_id, stage_count)?;
    let right_by_dimension =
        group_points("right", &request.right, &stage_index_by_id, stage_count)?;
    let (dimension_distances, max_bottleneck_doubled, mut obstructions) =
        dimension_distances(request, &left_by_dimension, &right_by_dimension);

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

fn validate_request(request: &PersistenceDistanceRequest) -> Result<(), CoreError> {
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
    Ok(())
}

fn stage_index_by_id(stage_ids: &[Id]) -> Result<BTreeMap<Id, usize>, CoreError> {
    let mut stage_index_by_id = BTreeMap::new();
    for (index, stage_id) in stage_ids.iter().enumerate() {
        if stage_index_by_id.insert(stage_id.clone(), index).is_some() {
            return Err(malformed(
                "stage_ids",
                format!("stage id {stage_id} appears more than once"),
            ));
        }
    }
    Ok(stage_index_by_id)
}

fn dimension_distances(
    request: &PersistenceDistanceRequest,
    left_by_dimension: &BTreeMap<Dimension, Vec<NormalizedPoint>>,
    right_by_dimension: &BTreeMap<Dimension, Vec<NormalizedPoint>>,
) -> (
    Vec<PersistenceDimensionDistance>,
    u64,
    Vec<PersistenceDistanceObstruction>,
) {
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
    (dimension_distances, max_bottleneck_doubled, obstructions)
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
    if left.is_empty() && right.is_empty() {
        return (0, Vec::new());
    }

    let cost = wasserstein_cost_matrix(left, right, wasserstein_order);
    let assignment = hungarian(&cost);
    matching_from_assignment(left, right, &cost, &assignment)
}

fn wasserstein_cost_matrix(
    left: &[NormalizedPoint],
    right: &[NormalizedPoint],
    wasserstein_order: u32,
) -> Vec<Vec<u128>> {
    let size = left.len() + right.len();
    let mut cost = vec![vec![0u128; size]; size];
    for (left_index, left_point) in left.iter().enumerate() {
        let row = &mut cost[left_index];
        for (right_index, right_point) in right.iter().enumerate() {
            row[right_index] =
                pow_doubled(linf_doubled(left_point, right_point), wasserstein_order);
        }
        // Left point may only reach its own diagonal sink column; others infeasible.
        let diagonal_cost = pow_doubled(left_point.to_diagonal_doubled(), wasserstein_order);
        for (sink_index, slot) in row[right.len()..].iter_mut().enumerate().take(left.len()) {
            *slot = if sink_index == left_index {
                diagonal_cost
            } else {
                INFEASIBLE_COST
            };
        }
    }
    for (right_index, right_point) in right.iter().enumerate() {
        let row = &mut cost[left.len() + right_index];
        // Right point may only reach its own diagonal source column; others infeasible.
        let diagonal_cost = pow_doubled(right_point.to_diagonal_doubled(), wasserstein_order);
        for (source_index, slot) in row.iter_mut().enumerate().take(right.len()) {
            *slot = if source_index == right_index {
                diagonal_cost
            } else {
                INFEASIBLE_COST
            };
        }
        // Diagonal-to-diagonal block columns stay zero (already initialized).
    }
    cost
}

fn matching_from_assignment(
    left: &[NormalizedPoint],
    right: &[NormalizedPoint],
    cost: &[Vec<u128>],
    assignment: &[usize],
) -> (u128, Vec<PersistenceMatch>) {
    let mut total = 0u128;
    let mut matches = Vec::new();
    for (row, &column) in assignment.iter().enumerate() {
        let entry = cost[row][column];
        if entry == INFEASIBLE_COST {
            // Unreachable for a valid augmented matrix; skip defensively.
            continue;
        }
        total = total.saturating_add(entry);

        if let Some(matched) = assignment_match(left, right, row, column) {
            matches.push(matched);
        }
    }

    matches.sort();
    (total, matches)
}

fn assignment_match(
    left: &[NormalizedPoint],
    right: &[NormalizedPoint],
    row: usize,
    column: usize,
) -> Option<PersistenceMatch> {
    let row_is_point = row < left.len();
    let column_is_point = column < right.len();
    match (row_is_point, column_is_point) {
        (true, true) => Some(PersistenceMatch {
            kind: PersistenceMatchKind::PointToPoint,
            left_point: Some(left[row].point()),
            right_point: Some(right[column].point()),
            cost_doubled: linf_doubled(&left[row], &right[column]),
        }),
        (true, false) => Some(PersistenceMatch {
            kind: PersistenceMatchKind::LeftToDiagonal,
            left_point: Some(left[row].point()),
            right_point: None,
            cost_doubled: left[row].to_diagonal_doubled(),
        }),
        (false, true) => Some(PersistenceMatch {
            kind: PersistenceMatchKind::RightToDiagonal,
            left_point: None,
            right_point: Some(right[column].point()),
            cost_doubled: right[column].to_diagonal_doubled(),
        }),
        (false, false) => None,
    }
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
    let mut work = HungarianWork::new(n);

    for row in 1..=n {
        augment_hungarian_row(row, cost, &mut work);
    }

    assignment_from_column_match(&work.column_match)
}

struct HungarianWork {
    row_potential: Vec<u128>,
    column_penalty: Vec<u128>,
    column_match: Vec<usize>,
    way: Vec<usize>,
}

impl HungarianWork {
    fn new(n: usize) -> Self {
        Self {
            row_potential: vec![0u128; n + 1],
            column_penalty: vec![0u128; n + 1],
            column_match: vec![0usize; n + 1],
            way: vec![0usize; n + 1],
        }
    }
}

fn augment_hungarian_row(row: usize, cost: &[Vec<u128>], work: &mut HungarianWork) {
    work.column_match[0] = row;
    let mut current_column = 0usize;
    let mut min_slack = vec![INFEASIBLE_COST; cost.len() + 1];
    let mut used = vec![false; cost.len() + 1];

    loop {
        let next_column =
            advance_hungarian_column(current_column, cost, work, &mut min_slack, &mut used);
        current_column = next_column;
        if work.column_match[current_column] == 0 {
            break;
        }
    }

    augment_column_path(current_column, work);
}

fn advance_hungarian_column(
    current_column: usize,
    cost: &[Vec<u128>],
    work: &mut HungarianWork,
    min_slack: &mut [u128],
    used: &mut [bool],
) -> usize {
    used[current_column] = true;
    let matched_row = work.column_match[current_column];
    let (delta, next_column) =
        best_next_column(current_column, matched_row, cost, work, min_slack, used);
    apply_hungarian_delta(delta, work, min_slack, used);
    next_column
}

fn best_next_column(
    current_column: usize,
    matched_row: usize,
    cost: &[Vec<u128>],
    work: &mut HungarianWork,
    min_slack: &mut [u128],
    used: &[bool],
) -> (u128, usize) {
    let mut delta = INFEASIBLE_COST;
    let mut next_column = 0usize;
    for column in 1..=cost.len() {
        if used[column] {
            continue;
        }
        let reduced = saturating_reduced_cost(
            cost[matched_row - 1][column - 1],
            work.row_potential[matched_row],
            work.column_penalty[column],
        );
        if reduced < min_slack[column] {
            min_slack[column] = reduced;
            work.way[column] = current_column;
        }
        if min_slack[column] < delta {
            delta = min_slack[column];
            next_column = column;
        }
    }
    (delta, next_column)
}

fn apply_hungarian_delta(
    delta: u128,
    work: &mut HungarianWork,
    min_slack: &mut [u128],
    used: &[bool],
) {
    for column in 0..used.len() {
        if used[column] {
            work.row_potential[work.column_match[column]] =
                work.row_potential[work.column_match[column]].saturating_add(delta);
            work.column_penalty[column] = work.column_penalty[column].saturating_add(delta);
        } else {
            min_slack[column] = min_slack[column].saturating_sub(delta);
        }
    }
}

fn augment_column_path(mut current_column: usize, work: &mut HungarianWork) {
    loop {
        let previous_column = work.way[current_column];
        work.column_match[current_column] = work.column_match[previous_column];
        current_column = previous_column;
        if current_column == 0 {
            break;
        }
    }
}

fn assignment_from_column_match(column_match: &[usize]) -> Vec<usize> {
    let mut assignment = vec![0usize; column_match.len() - 1];
    for column in 1..column_match.len() {
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
