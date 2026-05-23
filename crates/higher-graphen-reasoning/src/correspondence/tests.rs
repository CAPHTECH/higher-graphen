use super::*;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

#[test]
fn deterministic_detection_finds_label_evidence_invariant_and_relation_overlap() {
    let relation =
        TypedRelation::new("OrderService", "accesses", "BillingDB").expect("valid relation");
    let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:observed")))
        .with_role("observed_claim")
        .expect("role")
        .with_normalized_label("order service billing access")
        .expect("label")
        .with_evidence(vec![id("evidence:scan")])
        .with_invariants(vec![id("invariant:no-cross-context-db-access")])
        .with_typed_relations(vec![relation.clone()]);
    let right = CorrespondenceSubject::new(ParticipantRef::Invariant(id(
        "invariant:no-cross-context-db-access",
    )))
    .with_role("constraint")
    .expect("role")
    .with_normalized_label("order service billing access")
    .expect("label")
    .with_evidence(vec![id("evidence:scan")])
    .with_invariants(vec![id("invariant:no-cross-context-db-access")])
    .with_typed_relations(vec![relation]);

    let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
        id("ctx:architecture-review"),
        id("provenance:deterministic-overlap"),
        vec![left, right],
    ))
    .expect("detection succeeds");

    assert_eq!(result.candidates.len(), 1);
    let candidate = &result.candidates[0];
    assert_eq!(
        candidate.correspondence_kind,
        CorrespondenceKind::StructuralOverlap
    );
    assert_eq!(candidate.review_status, ReviewStatus::Candidate);
    assert_eq!(candidate.participants[0].role, "observed_claim");
    assert!(candidate
        .overlap_witnesses
        .iter()
        .any(|witness| witness.witness_kind == OverlapWitnessKind::EvidenceSet));
    assert!(candidate
        .overlap_witnesses
        .iter()
        .any(|witness| witness.witness_kind == OverlapWitnessKind::ConstraintSet));
    assert!(candidate
        .overlap_witnesses
        .iter()
        .any(|witness| witness.witness_kind == OverlapWitnessKind::PredicateSet));
    assert!(candidate.validate_report().is_valid());
}

#[test]
fn deterministic_detection_respects_scope() {
    let claim = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:one")))
        .with_normalized_label("same")
        .expect("label");
    let evidence = CorrespondenceSubject::new(ParticipantRef::Evidence(id("evidence:one")))
        .with_normalized_label("same")
        .expect("label");

    let result = derive_correspondence_candidates(
        CorrespondenceDetectionInput::new(
            id("ctx:test"),
            id("provenance:test"),
            vec![claim, evidence],
        )
        .with_scope(CorrespondenceScope::Claims),
    )
    .expect("detection succeeds");

    assert!(result.candidates.is_empty());
}

#[test]
fn deterministic_detection_extracts_phase3_difference_witnesses() {
    let relation =
        TypedRelation::new("OrderService", "accesses", "BillingDB").expect("valid relation");
    let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:observed")))
        .with_role("observed_claim")
        .expect("role")
        .with_normalized_label("order service billing access")
        .expect("label")
        .with_modality("observed")
        .expect("modality")
        .with_contexts(vec![id("ctx:legacy-migration")])
        .with_evidence(vec![id("evidence:scan"), id("evidence:architecture-doc")])
        .with_invariants(vec![id("invariant:no-cross-context-db-access")])
        .with_invariant_states(vec![InvariantState::new(
            id("invariant:no-cross-context-db-access"),
            InvariantSatisfaction::Failed,
        )])
        .with_typed_relations(vec![relation.clone()]);
    let right = CorrespondenceSubject::new(ParticipantRef::Invariant(id(
        "invariant:no-cross-context-db-access",
    )))
    .with_role("constraint")
    .expect("role")
    .with_normalized_label("order service billing access")
    .expect("label")
    .with_modality("forbidden")
    .expect("modality")
    .with_contexts(vec![id("ctx:production-architecture")])
    .with_evidence(vec![id("evidence:architecture-doc")])
    .with_invariants(vec![id("invariant:no-cross-context-db-access")])
    .with_invariant_states(vec![InvariantState::new(
        id("invariant:no-cross-context-db-access"),
        InvariantSatisfaction::Satisfied,
    )])
    .with_typed_relations(vec![relation]);

    let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
        id("ctx:architecture-review"),
        id("provenance:deterministic-overlap"),
        vec![left, right],
    ))
    .expect("detection succeeds");

    let candidate = &result.candidates[0];
    assert!(candidate
        .difference_witnesses
        .iter()
        .any(
            |difference| difference.difference_kind == DifferenceKind::ModalityMismatch
                && difference.severity == DifferenceSeverity::Blocking
        ));
    assert!(candidate
        .difference_witnesses
        .iter()
        .any(
            |difference| difference.difference_kind == DifferenceKind::EvidenceMismatch
                && difference.severity == DifferenceSeverity::Minor
        ));
    assert!(candidate
        .difference_witnesses
        .iter()
        .any(
            |difference| difference.difference_kind == DifferenceKind::InvariantMismatch
                && difference.severity == DifferenceSeverity::Blocking
        ));
    assert!(candidate
        .difference_witnesses
        .iter()
        .any(
            |difference| difference.difference_kind == DifferenceKind::ContextMismatch
                && difference.severity == DifferenceSeverity::Major
        ));
    assert!(candidate.validate_report().is_valid());
}

#[test]
fn compact_participant_ids_deserialize_by_prefix() {
    let participant: ParticipantRef =
        serde_json::from_str("\"claim:architecture\"").expect("participant ref");

    assert!(matches!(participant, ParticipantRef::Claim(_)));
}

#[test]
fn detection_result_roundtrips() {
    let left = CorrespondenceSubject::new(ParticipantRef::Cell(id("cell:a")))
        .with_normalized_label("same")
        .expect("label");
    let right = CorrespondenceSubject::new(ParticipantRef::Cell(id("cell:b")))
        .with_normalized_label("same")
        .expect("label");
    let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
        id("ctx:test"),
        id("provenance:test"),
        vec![left, right],
    ))
    .expect("detection succeeds");

    let value = serde_json::to_value(&result).expect("serialize");
    let roundtrip: CorrespondenceDetectionResult =
        serde_json::from_value(value).expect("deserialize");

    assert_eq!(roundtrip, result);
}

#[test]
fn semantic_signals_generate_candidate_overlap_with_calibrated_confidence() {
    let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:doc")))
        .with_role("doc_claim")
        .expect("role")
        .with_modality("observed")
        .expect("modality")
        .with_evidence(vec![id("evidence:doc")]);
    let right = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:scan")))
        .with_role("scan_claim")
        .expect("role")
        .with_modality("forbidden")
        .expect("modality")
        .with_evidence(vec![id("evidence:scan")]);
    let mut signal = SemanticCorrespondenceSignal::new(
        id("semantic-signal:order-billing-access"),
        vec![
            ParticipantRef::Claim(id("claim:doc")),
            ParticipantRef::Claim(id("claim:scan")),
        ],
        SemanticSignalSource::Llm,
        Confidence::new(0.97).expect("confidence"),
    )
    .expect("semantic signal");
    signal.normalized_claim = Some(NormalizedClaim {
        subject: "OrderService".to_owned(),
        relation: "accesses".to_owned(),
        object: "BillingDB".to_owned(),
        modality: None,
        temporal_scope: None,
    });
    signal.embedding_score = Some(Confidence::new(0.91).expect("confidence"));
    signal.evidence = vec![id("evidence:semantic-adapter")];
    signal.rationale = Some("same normalized access claim".to_owned());

    let mut input = CorrespondenceDetectionInput::new(
        id("ctx:architecture-review"),
        id("provenance:semantic-adapter"),
        vec![left, right],
    );
    input.semantic_signals.push(signal);

    let result = derive_correspondence_candidates(input).expect("detection succeeds");
    let semantic = result
        .candidates
        .iter()
        .find(|candidate| candidate.correspondence_kind == CorrespondenceKind::SemanticOverlap)
        .expect("semantic candidate");

    assert_eq!(semantic.review_status, ReviewStatus::Candidate);
    assert_eq!(semantic.confidence, Confidence::new(0.86).expect("cap"));
    assert!(semantic
        .overlap_witnesses
        .iter()
        .any(|witness| witness.witness_kind == OverlapWitnessKind::NormalizedClaim));
    assert!(semantic
        .difference_witnesses
        .iter()
        .any(|difference| difference.difference_kind == DifferenceKind::ModalityMismatch));
}

#[test]
fn semantic_review_accepts_only_explicit_reviewed_candidate() {
    let mut signal = SemanticCorrespondenceSignal::new(
        id("semantic-signal:accepted"),
        vec![
            ParticipantRef::Claim(id("claim:left")),
            ParticipantRef::Claim(id("claim:right")),
        ],
        SemanticSignalSource::Human,
        Confidence::new(0.88).expect("confidence"),
    )
    .expect("semantic signal");
    signal.normalized_claim = Some(NormalizedClaim {
        subject: "OrderService".to_owned(),
        relation: "accesses".to_owned(),
        object: "BillingDB".to_owned(),
        modality: None,
        temporal_scope: None,
    });
    signal.evidence = vec![id("evidence:review")];
    let mut input = CorrespondenceDetectionInput::new(
        id("ctx:review"),
        id("provenance:semantic"),
        vec![
            CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:left"))),
            CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:right"))),
        ],
    );
    input.semantic_signals.push(signal);
    let candidate = derive_correspondence_candidates(input)
        .expect("detection succeeds")
        .candidates
        .into_iter()
        .next()
        .expect("candidate");

    let reviewed = review_semantic_correspondence(
        &candidate,
        SemanticCorrespondenceReviewRequest::new(
            candidate.id.clone(),
            id("reviewer:human"),
            SemanticReviewDecision::Accept,
            "normalized claim and evidence reviewed",
        )
        .expect("review request"),
    )
    .expect("review succeeds");

    assert_eq!(reviewed.review_status, ReviewStatus::Accepted);
}
