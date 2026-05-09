use super::*;
use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};
use serde_json::json;

#[test]
fn composition_succeeds_for_compatible_spaces() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/a", "invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared", "invariant/c"],
    );

    let result = compose_morphisms(
        &first,
        &second,
        id("first-then-second"),
        "first then second",
        MorphismType::Translation,
        provenance(),
    );

    let CompositionResult::Composed { morphism } = result else {
        panic!("expected compatible morphisms to compose");
    };

    assert_eq!(morphism.source_space_id, id("space/a"));
    assert_eq!(morphism.target_space_id, id("space/c"));
    assert_eq!(morphism.cell_mapping[&id("cell/a1")], id("cell/c1"));
    assert_eq!(morphism.relation_mapping[&id("rel/a1")], id("rel/c1"));
    assert_eq!(
        morphism.preserved_invariant_ids,
        vec![id("invariant/shared")]
    );
    assert_eq!(morphism.lost_structure.len(), 2);
    assert_eq!(morphism.distortion.len(), 2);
}

#[test]
fn composition_rejects_incompatible_spaces() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/a"],
    );
    let second = fixture_morphism(
        "second",
        "space/x",
        "space/c",
        [("cell/x1", "cell/c1")],
        [("rel/x1", "rel/c1")],
        ["invariant/x"],
    );

    let result = first.compose_with(
        &second,
        id("invalid"),
        "invalid composition",
        MorphismType::Translation,
        provenance(),
    );

    assert_eq!(
        result,
        CompositionResult::IncompatibleSpace {
            first_morphism_id: id("first"),
            second_morphism_id: id("second"),
            first_target_space_id: id("space/b"),
            second_source_space_id: id("space/x"),
        }
    );
}

#[test]
fn composition_does_not_infer_unmatched_intermediate_mappings() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
        [("rel/a1", "rel/b1"), ("rel/a2", "rel/b2")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let CompositionResult::Composed { morphism } = compose_morphisms(
        &first,
        &second,
        id("composed"),
        "composed",
        MorphismType::Projection,
        provenance(),
    ) else {
        panic!("expected compatible morphisms to compose");
    };

    assert_eq!(morphism.cell_mapping.len(), 1);
    assert_eq!(morphism.relation_mapping.len(), 1);
    assert!(!morphism.cell_mapping.contains_key(&id("cell/a2")));
    assert!(!morphism.relation_mapping.contains_key(&id("rel/a2")));
}

#[test]
fn composition_coverage_reports_unmatched_intermediate_mappings() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
        [("rel/a1", "rel/b1"), ("rel/a2", "rel/b2")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let coverage = first.composition_coverage_with(&second);

    assert!(!coverage.is_complete());
    assert_eq!(coverage.unmapped_cell_intermediate_ids, vec![id("cell/b2")]);
    assert_eq!(
        coverage.unmapped_relation_intermediate_ids,
        vec![id("rel/b2")]
    );
}

#[test]
fn checked_composition_succeeds_for_complete_mappings() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/a", "invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared", "invariant/c"],
    );

    let result = first.compose_checked_with(
        &second,
        id("checked"),
        "checked composition",
        MorphismType::Translation,
        provenance(),
    );

    let CheckedCompositionResult::Composed { morphism } = result else {
        panic!("expected checked composition to succeed");
    };

    assert_eq!(morphism.cell_mapping[&id("cell/a1")], id("cell/c1"));
    assert_eq!(morphism.relation_mapping[&id("rel/a1")], id("rel/c1"));
    assert_eq!(
        morphism.preserved_invariant_ids,
        vec![id("invariant/shared")]
    );
}

#[test]
fn checked_composition_fails_with_first_class_findings_for_unmapped_intermediates() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
        [("rel/a1", "rel/b1"), ("rel/a2", "rel/b2")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let result = compose_morphisms_checked(
        &first,
        &second,
        id("strict"),
        "strict composition",
        MorphismType::Projection,
        provenance(),
    );

    let CheckedCompositionResult::FailedComposition {
        obstruction_type,
        coverage,
        findings,
    } = result
    else {
        panic!("expected checked composition to fail with findings");
    };

    assert_eq!(obstruction_type, FAILED_COMPOSITION_OBSTRUCTION_TYPE);
    assert_eq!(coverage.unmapped_cell_intermediate_ids, vec![id("cell/b2")]);
    assert_eq!(
        coverage.unmapped_relation_intermediate_ids,
        vec![id("rel/b2")]
    );
    assert_eq!(
        findings,
        vec![
            FailedCompositionFinding {
                obstruction_type: FAILED_COMPOSITION_OBSTRUCTION_TYPE.to_owned(),
                finding_type: FailedCompositionFindingKind::UnmappedIntermediateCell,
                first_morphism_id: id("first"),
                second_morphism_id: id("second"),
                source_element_id: id("cell/a2"),
                intermediate_element_id: id("cell/b2"),
            },
            FailedCompositionFinding {
                obstruction_type: FAILED_COMPOSITION_OBSTRUCTION_TYPE.to_owned(),
                finding_type: FailedCompositionFindingKind::UnmappedIntermediateRelation,
                first_morphism_id: id("first"),
                second_morphism_id: id("second"),
                source_element_id: id("rel/a2"),
                intermediate_element_id: id("rel/b2"),
            },
        ]
    );
    assert_eq!(findings, first.failed_composition_findings_with(&second));
}

#[test]
fn checked_composition_rejects_incompatible_spaces_before_gap_findings() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/a"],
    );
    let second = fixture_morphism("second", "space/x", "space/c", [], [], ["invariant/x"]);

    let result = compose_morphisms_checked(
        &first,
        &second,
        id("invalid"),
        "invalid composition",
        MorphismType::Translation,
        provenance(),
    );

    assert_eq!(
        result,
        CheckedCompositionResult::IncompatibleSpace {
            first_morphism_id: id("first"),
            second_morphism_id: id("second"),
            first_target_space_id: id("space/b"),
            second_source_space_id: id("space/x"),
        }
    );
}

#[test]
fn checked_composition_failure_serializes_failed_composition_obstruction_type() {
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [],
        ["invariant/shared"],
    );
    let second = fixture_morphism("second", "space/b", "space/c", [], [], ["invariant/shared"]);

    let result = compose_morphisms_checked(
        &first,
        &second,
        id("strict"),
        "strict composition",
        MorphismType::Projection,
        provenance(),
    );
    let value = serde_json::to_value(result).expect("serialize checked composition failure");

    assert_eq!(value["status"], json!("failed_composition"));
    assert_eq!(
        value["obstruction_type"],
        json!(FAILED_COMPOSITION_OBSTRUCTION_TYPE)
    );
    assert_eq!(
        value["findings"][0]["finding_type"],
        json!("unmapped_intermediate_cell")
    );
    assert_eq!(
        value["findings"][0]["obstruction_type"],
        json!(FAILED_COMPOSITION_OBSTRUCTION_TYPE)
    );
}

#[test]
fn serde_defaults_empty_morphism_collections() {
    let value = json!({
        "id": "morphism/minimal",
        "source_space_id": "space/a",
        "target_space_id": "space/b",
        "name": "minimal",
        "morphism_type": "translation",
        "provenance": provenance()
    });

    let morphism: Morphism = serde_json::from_value(value).expect("deserialize morphism");

    assert!(morphism.cell_mapping.is_empty());
    assert!(morphism.relation_mapping.is_empty());
    assert!(morphism.preserved_invariant_ids.is_empty());
    assert!(morphism.lost_structure.is_empty());
    assert!(morphism.distortion.is_empty());
    assert!(morphism.composable_with.is_empty());
}

#[test]
fn preservation_check_sorts_and_deduplicates_selected_invariants() {
    let morphism = fixture_morphism(
        "morphism",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/b", "invariant/a"],
    );

    let report = morphism.check_preservation([
        id("invariant/c"),
        id("invariant/a"),
        id("invariant/a"),
        id("invariant/b"),
    ]);

    assert_eq!(report.preserved, vec![id("invariant/a"), id("invariant/b")]);
    assert_eq!(report.violated, vec![id("invariant/c")]);
    assert_eq!(report.lost_structure, morphism.lost_structure);
    assert_eq!(report.distortion, morphism.distortion);
}

#[test]
fn explicit_pullback_candidate_extracts_common_mapped_substructure() {
    let left = fixture_morphism(
        "left",
        "space/left",
        "space/target",
        [
            ("cell/left-a", "cell/shared-a"),
            ("cell/left-b", "cell/shared-b"),
            ("cell/left-only", "cell/left-only-target"),
        ],
        [
            ("rel/left-a", "rel/shared-a"),
            ("rel/left-only", "rel/left-only-target"),
        ],
        ["invariant/shared"],
    );
    let right = fixture_morphism(
        "right",
        "space/right",
        "space/target",
        [
            ("cell/right-a", "cell/shared-a"),
            ("cell/right-b", "cell/shared-b"),
            ("cell/right-only", "cell/right-only-target"),
        ],
        [
            ("rel/right-a", "rel/shared-a"),
            ("rel/right-only", "rel/right-only-target"),
        ],
        ["invariant/shared"],
    );

    let report = left.explicit_pullback_with(&right);

    assert_eq!(report.target_space_id, Some(id("space/target")));
    assert_eq!(
        report.cell_matches,
        vec![
            PullbackCellMatch {
                left_cell_id: id("cell/left-a"),
                right_cell_id: id("cell/right-a"),
                target_cell_id: id("cell/shared-a"),
            },
            PullbackCellMatch {
                left_cell_id: id("cell/left-b"),
                right_cell_id: id("cell/right-b"),
                target_cell_id: id("cell/shared-b"),
            },
        ]
    );
    assert_eq!(
        report.relation_matches,
        vec![PullbackRelationMatch {
            left_relation_id: id("rel/left-a"),
            right_relation_id: id("rel/right-a"),
            target_relation_id: id("rel/shared-a"),
        }]
    );
    assert_eq!(report.unmatched_left_cell_ids, vec![id("cell/left-only")]);
    assert_eq!(report.unmatched_right_cell_ids, vec![id("cell/right-only")]);
    assert_eq!(
        report.unmatched_left_relation_ids,
        vec![id("rel/left-only")]
    );
    assert_eq!(
        report.unmatched_right_relation_ids,
        vec![id("rel/right-only")]
    );
    assert_eq!(
        report.obstructions[0].obstruction_type,
        PullbackObstructionType::PullbackIncomplete
    );
    assert!(!report.is_complete());

    let roundtrip: ExplicitPullbackReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn explicit_pullback_candidate_reports_incompatible_targets() {
    let left = fixture_morphism(
        "left",
        "space/left",
        "space/target-a",
        [("cell/left-a", "cell/shared")],
        [],
        ["invariant/shared"],
    );
    let right = fixture_morphism(
        "right",
        "space/right",
        "space/target-b",
        [("cell/right-a", "cell/shared")],
        [],
        ["invariant/shared"],
    );

    let report = explicit_pullback_candidate(&left, &right);

    assert_eq!(report.target_space_id, None);
    assert_eq!(
        report.obstructions[0].obstruction_type,
        PullbackObstructionType::IncompatibleTargetSpace
    );
    assert!(!report.is_complete());
}

#[test]
fn explicit_pushout_candidate_identifies_targets_from_shared_sources() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [
            ("cell/source-a", "cell/left-a"),
            ("cell/source-only-left", "cell/left-only"),
        ],
        [("rel/source-a", "rel/left-a")],
        ["invariant/shared"],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [
            ("cell/source-a", "cell/right-a"),
            ("cell/source-only-right", "cell/right-only"),
        ],
        [("rel/source-a", "rel/right-a")],
        ["invariant/shared"],
    );

    let report = explicit_pushout_candidate(&left, &right, id("space/pushout-candidate"));

    assert_eq!(report.source_space_id, Some(id("space/source")));
    assert_eq!(
        report.identified_cell_groups,
        vec![IdentifiedSourceGroup {
            source_element_id: id("cell/source-a"),
            left_target_id: id("cell/left-a"),
            right_target_id: id("cell/right-a"),
        }]
    );
    assert_eq!(
        report.identified_relation_groups,
        vec![IdentifiedSourceGroup {
            source_element_id: id("rel/source-a"),
            left_target_id: id("rel/left-a"),
            right_target_id: id("rel/right-a"),
        }]
    );
    assert_eq!(
        report.unmatched_left_cell_source_ids,
        vec![id("cell/source-only-left")]
    );
    assert_eq!(
        report.unmatched_right_cell_source_ids,
        vec![id("cell/source-only-right")]
    );
    assert_eq!(
        report.obstructions[0].obstruction_type,
        PushoutObstructionType::PushoutIncomplete
    );

    let roundtrip: ExplicitPushoutReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn explicit_pushout_candidate_reports_incompatible_sources() {
    let left = fixture_morphism(
        "left",
        "space/source-a",
        "space/left",
        [("cell/source-a", "cell/left-a")],
        [],
        ["invariant/shared"],
    );
    let right = fixture_morphism(
        "right",
        "space/source-b",
        "space/right",
        [("cell/source-a", "cell/right-a")],
        [],
        ["invariant/shared"],
    );

    let report = left.explicit_pushout_with(&right, id("space/pushout-candidate"));

    assert_eq!(report.source_space_id, None);
    assert_eq!(
        report.obstructions[0].obstruction_type,
        PushoutObstructionType::IncompatibleSourceSpace
    );
    assert!(!report.is_complete());
}

#[test]
fn diagram_commutativity_accepts_equivalent_explicit_paths() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c1")],
        [("rel/a1", "rel/c1")],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, second]);

    assert!(report.commutes);
    assert!(report.non_commutative_witnesses.is_empty());
    assert!(report.obstructions.is_empty());

    let roundtrip: DiagramCommutativityReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn diagram_commutativity_reports_mapping_disagreement() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c2")],
        [("rel/a1", "rel/c2")],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, second]);

    assert!(!report.commutes);
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::NonCommutativeDiagram
    }));
    assert_eq!(
        report.non_commutative_witnesses,
        vec![
            NonCommutativeWitness {
                element_kind: DiagramElementKind::Cell,
                source_element_id: id("cell/a1"),
                left_target_id: Some(id("cell/c2")),
                right_target_id: Some(id("cell/c1")),
            },
            NonCommutativeWitness {
                element_kind: DiagramElementKind::Relation,
                source_element_id: id("rel/a1"),
                left_target_id: Some(id("rel/c2")),
                right_target_id: Some(id("rel/c1")),
            },
        ]
    );
}

#[test]
fn diagram_commutativity_reports_incomplete_and_incompatible_paths() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c1")],
        [],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
        [],
        ["invariant/shared"],
    );
    let incompatible_second = fixture_morphism(
        "second",
        "space/x",
        "space/c",
        [("cell/b1", "cell/c1")],
        [],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, incompatible_second]);

    assert!(!report.commutes);
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::IncompatiblePath
    }));
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::IncompletePath
    }));
    assert_eq!(
        report.right_path.coverage.unmapped_cell_intermediate_ids,
        vec![id("cell/b2")]
    );
}

fn fixture_morphism<const C: usize, const R: usize, const I: usize>(
    morphism_id: &str,
    source_space_id: &str,
    target_space_id: &str,
    cell_pairs: [(&str, &str); C],
    relation_pairs: [(&str, &str); R],
    invariant_ids: [&str; I],
) -> Morphism {
    Morphism {
        id: id(morphism_id),
        source_space_id: id(source_space_id),
        target_space_id: id(target_space_id),
        name: morphism_id.to_owned(),
        morphism_type: MorphismType::Translation,
        cell_mapping: mapping(cell_pairs),
        relation_mapping: mapping(relation_pairs),
        preserved_invariant_ids: invariant_ids.into_iter().map(id).collect(),
        lost_structure: vec![LostStructure {
            source_element_id: id(format!("{morphism_id}/lost")),
            reason: "fixture loss".to_owned(),
            severity: Severity::Low,
        }],
        distortion: vec![Distortion {
            source_element_id: id(format!("{morphism_id}/source")),
            target_element_id: id(format!("{morphism_id}/target")),
            description: "fixture distortion".to_owned(),
            severity: Severity::Medium,
        }],
        composable_with: Vec::new(),
        provenance: provenance(),
    }
}

fn mapping<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<Id, Id> {
    pairs
        .into_iter()
        .map(|(source_id, target_id)| (id(source_id), id(target_id)))
        .collect()
}

fn provenance() -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::custom("morphism-test").expect("valid custom source kind")),
        Confidence::new(1.0).expect("valid confidence"),
    )
    .with_review_status(ReviewStatus::Accepted)
}

fn id(value: impl AsRef<str>) -> Id {
    Id::new(value.as_ref()).expect("valid id")
}
