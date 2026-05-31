use super::*;

#[test]
fn correspondence_validation_enforces_reviewable_overlap_invariants() {
    let mut correspondence = correspondence_fixture(
        CorrespondenceKind::ConstraintOverlap,
        CorrespondencePolarity::Conflicting,
        ReviewStatus::Candidate,
    );

    correspondence.overlap_witnesses.clear();
    assert_eq!(
        correspondence
            .validate()
            .expect_err("conflict without overlap should fail")
            .code(),
        "malformed_field"
    );

    correspondence.overlap_witnesses = vec![overlap_witness()];
    correspondence.review_status = ReviewStatus::Accepted;
    correspondence.evidence.clear();
    assert_eq!(
        correspondence
            .validate()
            .expect_err("accepted without evidence should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn accepted_semantic_correspondence_requires_explicit_semantic_witness() {
    let mut correspondence = correspondence_fixture(
        CorrespondenceKind::SemanticOverlap,
        CorrespondencePolarity::Agreeing,
        ReviewStatus::Accepted,
    );
    correspondence.overlap_witnesses = vec![OverlapWitness {
        id: id("witness:context"),
        witness_kind: OverlapWitnessKind::ContextRestriction,
        shared_structure: SharedStructure::ContextRestriction(ContextRestriction::default()),
        participant_mappings: Vec::new(),
        scope: Scope::default(),
        context: id("ctx:architecture-review"),
        evidence: vec![id("evidence:architecture-doc")],
        confidence: Confidence::new(0.8).expect("confidence"),
        status: ReviewStatus::Candidate,
    }];

    assert_eq!(
        correspondence
            .validate()
            .expect_err("accepted semantic overlap needs semantic witness")
            .code(),
        "malformed_field"
    );

    correspondence.overlap_witnesses = vec![overlap_witness()];
    correspondence
        .validate()
        .expect("normalized claim witness should validate");
}

#[test]
fn accepted_semantic_correspondence_requires_evidence_and_witness_together() {
    let mut correspondence = correspondence_fixture(
        CorrespondenceKind::SemanticOverlap,
        CorrespondencePolarity::Agreeing,
        ReviewStatus::Accepted,
    );

    correspondence.evidence.clear();
    assert_eq!(
        correspondence
            .validate()
            .expect_err("accepted semantic overlap without evidence should fail")
            .code(),
        "malformed_field"
    );

    correspondence.evidence = vec![id("evidence:architecture-doc")];
    correspondence.overlap_witnesses.clear();
    assert_eq!(
        correspondence
            .validate()
            .expect_err("accepted semantic overlap without witness should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn gluing_success_requires_preservation_and_blocks_silent_merge() {
    let mut correspondence = correspondence_fixture(
        CorrespondenceKind::StructuralOverlap,
        CorrespondencePolarity::Agreeing,
        ReviewStatus::Candidate,
    );
    correspondence.difference_witnesses = vec![blocking_difference()];
    correspondence.gluing = Some(GluingAttempt {
        id: id("glue:order-service-billing-db-access"),
        participants: vec![
            ParticipantRef::Claim(id("claim:architecture-doc-order-billing-access")),
            ParticipantRef::Invariant(id("invariant:no-cross-context-db-access")),
        ],
        overlap_witnesses: vec![id("witness:shared-order-billing-access")],
        difference_witnesses: vec![id("diff:observed-vs-forbidden")],
        context: id("ctx:architecture-review"),
        invariant_checks: Vec::new(),
        preservation_report: PreservationReport::default(),
        result: GluingResult::Success {
            merged_complex: Some(id("complex:merged")),
            preservation_report: PreservationReport::default(),
        },
        evidence: vec![id("evidence:architecture-doc")],
        confidence: Confidence::new(0.8).expect("confidence"),
        status: ReviewStatus::Candidate,
        override_review: None,
    });

    assert_eq!(
        correspondence
            .validate()
            .expect_err("success without preservation should fail")
            .code(),
        "malformed_field"
    );

    if let Some(gluing) = &mut correspondence.gluing {
        gluing.result = GluingResult::Success {
            merged_complex: Some(id("complex:merged")),
            preservation_report: PreservationReport {
                preserved_invariants: vec![id("invariant:no-cross-context-db-access")],
                preserved_structures: Vec::new(),
                summary: Some("invariant checked".to_owned()),
            },
        };
    }

    assert_eq!(
        correspondence
            .validate()
            .expect_err("blocking difference should prevent silent success")
            .code(),
        "malformed_field"
    );

    let report = correspondence.validate_report();
    assert_eq!(report.findings.len(), 1);
    assert_eq!(
        report.findings[0].code,
        CorrespondenceValidationCode::BlockingDifferenceSilentMerge
    );
}

#[test]
fn correspondence_schema_fixture_loads_as_core_model_after_schema_envelope() {
    let mut value: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/highergraphen/correspondence.graph.example.json"
    ))
    .expect("fixture json is valid");

    assert_eq!(
        value["schema"],
        json!("highergraphen.correspondence.cell.v1")
    );
    assert_eq!(value["kind"], json!("CorrespondenceCell"));

    let object = value.as_object_mut().expect("fixture is object");
    object.remove("schema");
    object.remove("kind");

    let correspondence: CorrespondenceCell =
        serde_json::from_value(value).expect("fixture matches core model payload");

    assert_eq!(
        correspondence.correspondence_kind,
        CorrespondenceKind::ConstraintOverlap
    );
    assert_eq!(correspondence.polarity, CorrespondencePolarity::Conflicting);
    assert!(matches!(
        correspondence.participants[0].participant,
        ParticipantRef::Claim(_)
    ));
    assert!(correspondence.validate_report().is_valid());

    let gluing = correspondence.gluing.expect("fixture has gluing");
    assert!(matches!(
        gluing.result,
        GluingResult::Failure { ref obstruction }
            if obstruction == &id("obstruction:direct-db-access-violates-boundary")
    ));
}

fn correspondence_fixture(
    correspondence_kind: CorrespondenceKind,
    polarity: CorrespondencePolarity,
    review_status: ReviewStatus,
) -> CorrespondenceCell {
    CorrespondenceCell {
        id: id("corr:order-service-billing-db-access"),
        participants: vec![
            CorrespondenceParticipant::new(
                "observed_claim",
                ParticipantRef::Claim(id("claim:architecture-doc-order-billing-access")),
            )
            .expect("participant"),
            CorrespondenceParticipant::new(
                "constraint",
                ParticipantRef::Invariant(id("invariant:no-cross-context-db-access")),
            )
            .expect("participant"),
        ],
        correspondence_kind,
        polarity,
        overlap_witnesses: vec![overlap_witness()],
        difference_witnesses: Vec::new(),
        context: id("ctx:architecture-review"),
        evidence: vec![id("evidence:architecture-doc")],
        provenance: id("provenance:architecture-review"),
        confidence: Confidence::new(0.9).expect("confidence"),
        review_status,
        gluing: None,
    }
}

fn overlap_witness() -> OverlapWitness {
    OverlapWitness {
        id: id("witness:shared-order-billing-access"),
        witness_kind: OverlapWitnessKind::NormalizedClaim,
        shared_structure: SharedStructure::NormalizedClaim(NormalizedClaim {
            subject: "OrderService".to_owned(),
            relation: "accesses".to_owned(),
            object: "BillingDB".to_owned(),
            modality: None,
            temporal_scope: None,
        }),
        participant_mappings: Vec::new(),
        scope: Scope::default(),
        context: id("ctx:architecture-review"),
        evidence: vec![id("evidence:architecture-doc")],
        confidence: Confidence::new(0.91).expect("confidence"),
        status: ReviewStatus::Candidate,
    }
}

fn blocking_difference() -> DifferenceWitness {
    DifferenceWitness {
        id: id("diff:observed-vs-forbidden"),
        difference_kind: DifferenceKind::ModalityMismatch,
        differing_structure: DifferingStructure::ModalityMismatch(BTreeMap::from([
            ("observed_claim".to_owned(), "observed".to_owned()),
            ("constraint".to_owned(), "forbidden".to_owned()),
        ])),
        participant_mappings: Vec::new(),
        severity: DifferenceSeverity::Blocking,
        context: id("ctx:architecture-review"),
        evidence: vec![id("evidence:architecture-doc")],
        confidence: Confidence::new(0.93).expect("confidence"),
        status: ReviewStatus::Candidate,
    }
}
