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
