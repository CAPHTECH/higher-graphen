use super::*;
use crate::space::Dimension as Dim;
use crate::topology::PersistenceInterval;
use higher_graphen_core::{Id, ReviewStatus, Severity};

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
        cost[left_count + right_index][right_index] = Some(diagonal_oracle_doubled(right_point));
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

    let kernel_p1 = p1_dimension.map_or(0, |distance| distance.wasserstein_cost_power_sum_doubled);
    let kernel_p2 = p2_dimension.map_or(0, |distance| distance.wasserstein_cost_power_sum_doubled);
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
    let roundtrip: PersistenceDistanceReport = serde_json::from_str(&json).expect("deserialize");
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
    let request =
        PersistenceDistanceRequest::new(stages(), Vec::new(), Vec::new()).with_wasserstein_order(0);
    let error = persistence_distance(&request).expect_err("zero order rejected");
    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn rejects_duplicate_stage_id() {
    let request = PersistenceDistanceRequest::new(vec![id("s0"), id("s0")], Vec::new(), Vec::new());
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
