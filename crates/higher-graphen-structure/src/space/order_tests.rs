use super::{
    check_order_monotonicity, check_relation_order_monotonicity, FiniteOrderRelationSet,
    OrderBoundCandidates, OrderCheckReport, OrderCheckStatus, OrderMonotonicityReport,
    OrderObstructionType, OrderPair, OrderRelation,
};
use crate::morphism::{Morphism, MorphismType};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceKind, SourceRef};
use std::collections::BTreeMap;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

#[test]
fn partial_order_reports_incomparables_and_unique_bounds() {
    let set = order_set(
        "order/lattice",
        "space/order",
        ["bottom", "a", "b", "top"],
        [
            ("bottom-to-a", "bottom", "a"),
            ("bottom-to-b", "bottom", "b"),
            ("a-to-top", "a", "top"),
            ("b-to-top", "b", "top"),
        ],
    );

    let report = set.analyze().expect("analyze order");

    assert_eq!(report.status, OrderCheckStatus::PartialOrder);
    assert_eq!(
        report.incomparable_pairs,
        vec![OrderPair {
            left_id: id("a"),
            right_id: id("b"),
        }]
    );
    assert!(report.obstructions.is_empty());
    assert!(set.implies(&id("bottom"), &id("top")).expect("closure"));
    assert_eq!(
        bound_for(&report.least_upper_bound_candidates, "a", "b"),
        vec![id("top")]
    );
    assert_eq!(
        bound_for(&report.greatest_lower_bound_candidates, "a", "b"),
        vec![id("bottom")]
    );

    let roundtrip: OrderCheckReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn antisymmetry_violation_marks_order_invalid() {
    let set = order_set(
        "order/cycle",
        "space/order",
        ["a", "b"],
        [("a-to-b", "a", "b"), ("b-to-a", "b", "a")],
    );

    let report = set.analyze().expect("analyze order");

    assert_eq!(report.status, OrderCheckStatus::Invalid);
    assert_eq!(report.cycle_witness, Some(vec![id("a"), id("b"), id("a")]));
    assert_eq!(
        report.obstructions[0].obstruction_type,
        OrderObstructionType::AntisymmetryViolation
    );
}

#[test]
fn accepted_relation_filter_limits_order_analysis_input() {
    let accepted = OrderRelation::new(
        id("accepted"),
        id("space/order"),
        "refines",
        id("a"),
        id("b"),
    )
    .expect("relation")
    .with_review_status(ReviewStatus::Accepted);
    let unreviewed = OrderRelation::new(
        id("unreviewed"),
        id("space/order"),
        "refines",
        id("b"),
        id("a"),
    )
    .expect("relation");
    let set = FiniteOrderRelationSet::new(
        id("order/reviewed"),
        id("space/order"),
        "refines",
        [id("a"), id("b")],
    )
    .expect("set")
    .with_relations([accepted, unreviewed]);

    let report = set.accepted_relations().analyze().expect("analyze");

    assert_eq!(report.status, OrderCheckStatus::PartialOrder);
    assert_eq!(report.selected_relation_ids, vec![id("accepted")]);
    assert!(report.antisymmetry_violations.is_empty());
}

#[test]
fn non_unique_join_and_meet_are_reported_as_obstructions() {
    let set = order_set(
        "order/non-unique",
        "space/order",
        ["left", "right", "upper-a", "upper-b", "lower-a", "lower-b"],
        [
            ("left-to-upper-a", "left", "upper-a"),
            ("right-to-upper-a", "right", "upper-a"),
            ("left-to-upper-b", "left", "upper-b"),
            ("right-to-upper-b", "right", "upper-b"),
            ("lower-a-to-left", "lower-a", "left"),
            ("lower-a-to-right", "lower-a", "right"),
            ("lower-b-to-left", "lower-b", "left"),
            ("lower-b-to-right", "lower-b", "right"),
        ],
    );

    let report = set.analyze().expect("analyze order");

    assert_eq!(
        bound_for(&report.least_upper_bound_candidates, "left", "right"),
        vec![id("upper-a"), id("upper-b")]
    );
    assert_eq!(
        bound_for(&report.greatest_lower_bound_candidates, "left", "right"),
        vec![id("lower-a"), id("lower-b")]
    );
    assert!(report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type == OrderObstructionType::JoinNotUnique));
    assert!(report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type == OrderObstructionType::MeetNotUnique));
}

#[test]
fn monotonicity_check_reports_unpreserved_order_relation() {
    let source = order_set(
        "order/source",
        "space/source",
        ["a", "b"],
        [("a-to-b", "a", "b")],
    );
    let target = order_set("order/target", "space/target", ["x", "y"], []);
    let morphism = morphism([("a", "x"), ("b", "y")]);

    let report = check_order_monotonicity(&source, &target, &morphism).expect("check monotonicity");

    assert!(!report.is_monotone());
    assert_eq!(report.violations.len(), 1);
    assert_eq!(
        report.obstructions[0].obstruction_type,
        OrderObstructionType::MonotonicityViolation
    );

    let ordered_target = order_set(
        "order/target",
        "space/target",
        ["x", "y"],
        [("x-to-y", "x", "y")],
    );
    let monotone =
        check_order_monotonicity(&source, &ordered_target, &morphism).expect("check monotone");
    assert!(monotone.is_monotone());
    assert!(monotone.obstructions.is_empty());

    let roundtrip: OrderMonotonicityReport =
        serde_json::from_str(&serde_json::to_string(&monotone).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, monotone);
}

#[test]
fn relation_mapping_monotonicity_uses_relation_mapping() {
    let source = order_set(
        "order/source-rel",
        "space/source",
        ["rel/a", "rel/b"],
        [("rel-a-to-b", "rel/a", "rel/b")],
    );
    let target = order_set(
        "order/target-rel",
        "space/target",
        ["rel/x", "rel/y"],
        [("rel-x-to-y", "rel/x", "rel/y")],
    );
    let mut morphism = morphism([]);
    morphism.relation_mapping = [(id("rel/a"), id("rel/x")), (id("rel/b"), id("rel/y"))]
        .into_iter()
        .collect();

    let report = check_relation_order_monotonicity(&source, &target, &morphism).expect("monotone");

    assert!(report.is_monotone());
    assert!(report.unmapped_source_ids.is_empty());
}

fn order_set<const E: usize, const R: usize>(
    set_id: &str,
    space_id: &str,
    elements: [&str; E],
    relations: [(&str, &str, &str); R],
) -> FiniteOrderRelationSet {
    FiniteOrderRelationSet::new(
        id(set_id),
        id(space_id),
        "refines",
        elements.into_iter().map(id),
    )
    .expect("order set")
    .with_relations(
        relations
            .into_iter()
            .map(|(relation_id, lesser_id, greater_id)| {
                OrderRelation::new(
                    id(relation_id),
                    id(space_id),
                    "refines",
                    id(lesser_id),
                    id(greater_id),
                )
                .expect("relation")
                .with_review_status(ReviewStatus::Accepted)
            }),
    )
}

fn bound_for(candidates: &[OrderBoundCandidates], left: &str, right: &str) -> Vec<Id> {
    candidates
        .iter()
        .find(|candidate| candidate.left_id == id(left) && candidate.right_id == id(right))
        .map(|candidate| candidate.candidate_ids.clone())
        .unwrap_or_default()
}

fn morphism<const N: usize>(cell_pairs: [(&str, &str); N]) -> Morphism {
    Morphism {
        id: id("morphism/source-target"),
        source_space_id: id("space/source"),
        target_space_id: id("space/target"),
        name: "source to target".to_owned(),
        morphism_type: MorphismType::Translation,
        cell_mapping: cell_pairs
            .into_iter()
            .map(|(source, target)| (id(source), id(target)))
            .collect::<BTreeMap<_, _>>(),
        relation_mapping: BTreeMap::new(),
        preserved_invariant_ids: Vec::new(),
        lost_structure: Vec::new(),
        distortion: Vec::new(),
        composable_with: Vec::new(),
        provenance: Provenance::new(
            SourceRef::new(SourceKind::custom("order-test").expect("source kind")),
            Confidence::ONE,
        )
        .with_review_status(ReviewStatus::Accepted),
    }
}
