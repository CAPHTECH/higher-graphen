use super::structural_gluing::success_honesty_invariant;
use super::*;
use higher_graphen_core::{
    Confidence, CorrespondenceCell, CorrespondenceKind, CorrespondenceParticipant,
    CorrespondencePolarity, DifferenceKind, DifferenceSeverity, DifferenceWitness,
    DifferingStructure, Feature, GluingResult, Id, OverlapWitness, OverlapWitnessKind,
    ParticipantMapping, ParticipantRef, PreservationReport, Provenance, ReviewStatus, Scope,
    SharedStructure, SourceKind, SourceRef,
};
use higher_graphen_structure::{
    morphism::{Morphism, MorphismType},
    space::{Cell, ComplexType, InMemorySpaceStore, Incidence, IncidenceOrientation, Space},
};
use std::collections::BTreeMap;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("valid confidence")
}

fn base_correspondence() -> CorrespondenceCell {
    CorrespondenceCell {
        id: id("corr:gluing-test"),
        participants: vec![
            CorrespondenceParticipant::new("left", ParticipantRef::Claim(id("claim:observed")))
                .expect("participant"),
            CorrespondenceParticipant::new("right", ParticipantRef::Claim(id("claim:required")))
                .expect("participant"),
        ],
        correspondence_kind: CorrespondenceKind::SurfaceOverlap,
        polarity: CorrespondencePolarity::Agreeing,
        overlap_witnesses: vec![OverlapWitness {
            id: id("witness:shared-feature"),
            witness_kind: OverlapWitnessKind::FeatureSet,
            shared_structure: SharedStructure::FeatureSet(vec![Feature {
                key: "label".to_owned(),
                value: "OrderService".to_owned(),
            }]),
            participant_mappings: vec![ParticipantMapping {
                participant: id("claim:observed"),
                path: "$.label".to_owned(),
            }],
            scope: Scope::default(),
            context: id("ctx:test"),
            evidence: vec![id("evidence:test")],
            confidence: confidence(0.9),
            status: ReviewStatus::Candidate,
        }],
        difference_witnesses: Vec::new(),
        context: id("ctx:test"),
        evidence: vec![id("evidence:test")],
        provenance: id("provenance:test"),
        confidence: confidence(0.9),
        review_status: ReviewStatus::Candidate,
        gluing: None,
    }
}

#[test]
fn blocking_difference_fails_gluing() {
    let mut correspondence = base_correspondence();
    correspondence.difference_witnesses.push(DifferenceWitness {
        id: id("diff:blocking"),
        difference_kind: DifferenceKind::ModalityMismatch,
        differing_structure: DifferingStructure::ModalityMismatch(BTreeMap::from([
            ("left".to_owned(), "observed".to_owned()),
            ("right".to_owned(), "forbidden".to_owned()),
        ])),
        participant_mappings: Vec::new(),
        severity: DifferenceSeverity::Blocking,
        context: id("ctx:test"),
        evidence: vec![id("evidence:test")],
        confidence: confidence(0.93),
        status: ReviewStatus::Candidate,
    });

    let attempt = attempt_gluing(&correspondence).expect("gluing check");

    assert!(matches!(attempt.result, GluingResult::Failure { .. }));
    assert!(attempt
        .validate_with_differences(&correspondence.difference_witnesses)
        .is_ok());
}

#[test]
fn major_difference_yields_review_candidate() {
    let mut correspondence = base_correspondence();
    correspondence.difference_witnesses.push(DifferenceWitness {
        id: id("diff:context"),
        difference_kind: DifferenceKind::ContextMismatch,
        differing_structure: DifferingStructure::ContextMismatch(BTreeMap::from([
            ("left".to_owned(), "ctx:legacy".to_owned()),
            ("right".to_owned(), "ctx:production".to_owned()),
        ])),
        participant_mappings: Vec::new(),
        severity: DifferenceSeverity::Major,
        context: id("ctx:test"),
        evidence: vec![id("evidence:test")],
        confidence: confidence(0.84),
        status: ReviewStatus::Candidate,
    });

    let attempt = attempt_gluing(&correspondence).expect("gluing check");

    match attempt.result {
        GluingResult::Candidate {
            required_review, ..
        } => {
            assert!(required_review.required);
            assert!(required_review
                .decision_reason
                .expect("decision reason")
                .contains("major differences"));
        }
        other => panic!("expected candidate, got {other:?}"),
    }
}

#[test]
fn supported_overlap_succeeds_with_preservation_report() {
    let mut correspondence = base_correspondence();
    correspondence.review_status = ReviewStatus::Reviewed;

    let attempt = attempt_gluing(&correspondence).expect("gluing check");

    match &attempt.result {
        GluingResult::Success {
            merged_complex,
            preservation_report,
        } => {
            assert!(merged_complex.is_none());
            assert!(!preservation_report.preserved_structures.is_empty());
            assert!(preservation_report.summary.is_some());
        }
        other => panic!("expected success, got {other:?}"),
    }
    let serialized = serde_json::to_string(&attempt).expect("serialize attempt");
    let fabricated_prefix = ["complex", "glued"].join(":");
    assert!(!serialized.contains("mergedComplex"));
    assert!(!serialized.contains(&fabricated_prefix));
    assert!(attempt.preservation_report.summary.is_some());
}

#[test]
fn evidence_absence_yields_review_candidate() {
    let mut correspondence = base_correspondence();
    correspondence.evidence.clear();
    correspondence.overlap_witnesses[0].evidence.clear();

    let attempt = attempt_gluing(&correspondence).expect("gluing check");

    assert!(matches!(attempt.result, GluingResult::Candidate { .. }));
}

#[test]
fn gluing_result_option_roundtrips_for_absent_and_present_complex() {
    let absent = GluingResult::Success {
        merged_complex: None,
        preservation_report: PreservationReport::default(),
    };
    let absent_value = serde_json::to_value(&absent).expect("serialize absent success");
    assert!(absent_value.get("mergedComplex").is_none());
    let absent_roundtrip: GluingResult =
        serde_json::from_value(absent_value).expect("deserialize absent success");
    assert_eq!(absent_roundtrip, absent);

    let present = GluingResult::Success {
        merged_complex: Some(id("complex:materialized")),
        preservation_report: PreservationReport::default(),
    };
    let present_value = serde_json::to_value(&present).expect("serialize present success");
    assert_eq!(
        present_value
            .get("mergedComplex")
            .and_then(|value| value.as_str()),
        Some("complex:materialized")
    );
    let present_roundtrip: GluingResult =
        serde_json::from_value(present_value).expect("deserialize present success");
    assert_eq!(present_roundtrip, present);
}

#[test]
fn structural_gluing_success_returns_real_constructed_complex_id() {
    let (store, left, right) = clean_pushout_store();

    let structural = attempt_structural_gluing(
        &left,
        &right,
        &store,
        id("space/pushout"),
        "Pushout".to_owned(),
        ComplexType::CellComplex,
    )
    .expect("structural gluing");

    match (&structural.result, &structural.construction) {
        (
            GluingResult::Success {
                merged_complex: Some(merged_complex),
                preservation_report,
            },
            Some(construction),
        ) => {
            assert_eq!(merged_complex, &construction.complex.id);
            assert!(!preservation_report.preserved_structures.is_empty());
        }
        other => panic!("expected successful structural gluing, got {other:?}"),
    }
    assert!(success_honesty_invariant(&structural));
}

#[test]
fn structural_gluing_blocked_returns_failure_without_construction() {
    let (store, left, mut right) = clean_pushout_store();
    right.source_space_id = id("space/other-source");

    let structural = attempt_structural_gluing(
        &left,
        &right,
        &store,
        id("space/pushout"),
        "Pushout".to_owned(),
        ComplexType::CellComplex,
    )
    .expect("structural gluing");

    assert!(matches!(structural.result, GluingResult::Failure { .. }));
    assert!(structural.construction.is_none());
}

#[test]
fn structural_gluing_ambiguous_returns_candidate_with_construction() {
    let (store, mut left, right) = clean_pushout_store();
    left.cell_mapping
        .insert(id("cell/source-b"), id("cell/left-a"));

    let structural = attempt_structural_gluing(
        &left,
        &right,
        &store,
        id("space/pushout"),
        "Pushout".to_owned(),
        ComplexType::CellComplex,
    )
    .expect("structural gluing");

    match structural.result {
        GluingResult::Candidate {
            required_review, ..
        } => {
            assert!(required_review.required);
            assert_eq!(
                required_review.decision_reason.as_deref(),
                Some("ambiguous identification requires review")
            );
        }
        other => panic!("expected candidate, got {other:?}"),
    }
    assert!(structural.construction.is_some());
}

fn clean_pushout_store() -> (InMemorySpaceStore, Morphism, Morphism) {
    let left = fixture_morphism(
        "morphism:left",
        "space/source",
        "space/left",
        [
            ("cell/source-a", "cell/left-a"),
            ("cell/source-b", "cell/left-b"),
        ],
        [],
    );
    let right = fixture_morphism(
        "morphism:right",
        "space/source",
        "space/right",
        [
            ("cell/source-a", "cell/right-a"),
            ("cell/source-b", "cell/right-b"),
        ],
        [],
    );
    let mut store = InMemorySpaceStore::new();
    for (space_id, name) in [
        ("space/source", "Source"),
        ("space/left", "Left"),
        ("space/right", "Right"),
    ] {
        store
            .insert_space(Space::new(id(space_id), name))
            .expect("insert space");
    }
    store
        .insert_cell(Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex"))
        .expect("insert left cell a");
    store
        .insert_cell(Cell::new(id("cell/left-b"), id("space/left"), 0, "vertex"))
        .expect("insert left cell b");
    store
        .insert_cell(Cell::new(
            id("cell/right-a"),
            id("space/right"),
            0,
            "vertex",
        ))
        .expect("insert right cell a");
    store
        .insert_cell(Cell::new(
            id("cell/right-b"),
            id("space/right"),
            0,
            "vertex",
        ))
        .expect("insert right cell b");
    store
        .insert_incidence(Incidence::new(
            id("incidence/left-a"),
            id("space/left"),
            id("cell/left-a"),
            id("cell/left-b"),
            "edge",
            IncidenceOrientation::Directed,
        ))
        .expect("insert left incidence");
    store
        .insert_incidence(Incidence::new(
            id("incidence/right-a"),
            id("space/right"),
            id("cell/right-a"),
            id("cell/right-b"),
            "edge",
            IncidenceOrientation::Directed,
        ))
        .expect("insert right incidence");

    (store, left, right)
}

fn fixture_morphism<const C: usize, const R: usize>(
    morphism_id: &str,
    source_space_id: &str,
    target_space_id: &str,
    cell_pairs: [(&str, &str); C],
    relation_pairs: [(&str, &str); R],
) -> Morphism {
    Morphism {
        id: id(morphism_id),
        source_space_id: id(source_space_id),
        target_space_id: id(target_space_id),
        name: morphism_id.to_owned(),
        morphism_type: MorphismType::Translation,
        cell_mapping: mapping(cell_pairs),
        relation_mapping: mapping(relation_pairs),
        preserved_invariant_ids: Vec::new(),
        lost_structure: Vec::new(),
        distortion: Vec::new(),
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
        SourceRef::new(SourceKind::custom("gluing-test").expect("valid source kind")),
        Confidence::ONE,
    )
    .with_review_status(ReviewStatus::Accepted)
}
