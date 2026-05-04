use super::*;
use crate::{
    native_eval::{evaluate_native_case, NativeReviewGapType},
    native_model::{
        CaseCell, CaseCellLifecycle, CaseMorphismType, CaseRelation, CaseRelationType,
        MorphismLogEntry, Projection, ProjectionAudience, RelationStrength, Revision,
        NATIVE_CASE_SPACE_SCHEMA, NATIVE_CASE_SPACE_SCHEMA_VERSION,
        NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
    },
};
use higher_graphen_core::{Confidence, Provenance, ReviewStatus, SourceKind, SourceRef};
use serde_json::{json, Map};

mod close;

#[test]
fn builds_review_morphisms_for_all_explicit_outcomes() {
    let space = fixture_space_with_completion();

    let accepted = accept_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Completion,
            "completion:source-backed-evidence",
            ReviewAction::Reject,
            "revision:accept",
        ),
    )
    .expect("accept completion");
    let rejected = reject_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Evidence,
            "evidence:source-backed",
            ReviewAction::Accept,
            "revision:reject",
        ),
    )
    .expect("reject evidence");
    let reopened = reopen_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Morphism,
            "morphism:generated",
            ReviewAction::Accept,
            "revision:reopen",
        ),
    )
    .expect("reopen morphism");
    let deferred = defer_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Waiver,
            "relation:needs-evidence",
            ReviewAction::Accept,
            "revision:defer",
        ),
    )
    .expect("defer waiver");

    assert_eq!(accepted.morphism_type, CaseMorphismType::CompletionAccept);
    assert_eq!(accepted.metadata["action"], json!("accept"));
    assert_eq!(
        rejected.metadata["outcome_review_status"],
        json!("rejected")
    );
    assert_eq!(
        reopened.metadata["outcome_review_status"],
        json!("unreviewed")
    );
    assert_eq!(deferred.metadata["action"], json!("defer"));
    assert_eq!(accepted.added_ids, Vec::<Id>::new());
}

#[test]
fn invalid_review_target_is_rejected() {
    let space = fixture_space_with_completion();
    let err = accept_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Evidence,
            "evidence:not-present",
            ReviewAction::Accept,
            "revision:bad",
        ),
    )
    .expect_err("invalid target");

    assert!(err.message.contains("unknown evidence target"));
}

#[test]
fn generated_completion_review_does_not_preserve_virtual_target_id() {
    let mut space = fixture_space();
    space.case_cells.push(cell(
        "work:needs-generated-evidence",
        CaseCellType::Work,
        CaseCellLifecycle::Active,
        SourceKind::Human,
        ReviewStatus::Reviewed,
    ));
    space.case_cells.push(cell(
        "evidence:generated-placeholder",
        CaseCellType::Evidence,
        CaseCellLifecycle::Proposed,
        SourceKind::Human,
        ReviewStatus::Reviewed,
    ));
    space
        .case_cells
        .last_mut()
        .expect("placeholder evidence")
        .source_ids
        .clear();
    space.case_relations.push(relation(
        "relation:needs-generated-evidence",
        CaseRelationType::RequiresEvidence,
        "work:needs-generated-evidence",
        "evidence:generated-placeholder",
    ));
    refresh_added_ids(&mut space);
    let candidate_id = evaluate_native_case(&space)
        .expect("evaluation")
        .completion_candidates
        .into_iter()
        .find(|candidate| {
            candidate
                .target_ids
                .contains(&id("work:needs-generated-evidence"))
        })
        .expect("generated completion")
        .id;

    let review = defer_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Completion,
            candidate_id.as_str(),
            ReviewAction::Defer,
            "revision:defer-generated-completion",
        ),
    )
    .expect("review generated completion");

    assert!(review.preserved_ids.is_empty());
    assert_eq!(review.metadata["target_id"], json!(candidate_id));
}

#[test]
fn inferred_evidence_cannot_satisfy_close_until_reviewed_or_waived() {
    let mut space = fixture_space();
    space.case_cells.push(cell(
        "work:needs-inference",
        CaseCellType::Work,
        CaseCellLifecycle::Active,
        SourceKind::Human,
        ReviewStatus::Reviewed,
    ));
    let mut evidence = cell(
        "evidence:ai-inference",
        CaseCellType::Evidence,
        CaseCellLifecycle::Active,
        SourceKind::Ai,
        ReviewStatus::Unreviewed,
    );
    evidence
        .metadata
        .insert("evidence_boundary".to_owned(), json!("inferred"));
    space.case_cells.push(evidence);
    space.case_relations.push(relation(
        "relation:needs-inference",
        CaseRelationType::RequiresEvidence,
        "work:needs-inference",
        "evidence:ai-inference",
    ));
    refresh_added_ids(&mut space);

    let blocked = check_native_close(&space, close_request()).expect("close check");
    assert!(!blocked.closeable);
    assert!(blocked.blocker_ids.contains(&id("evidence:ai-inference")));

    let review = accept_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Evidence,
            "evidence:ai-inference",
            ReviewAction::Accept,
            "revision:promote-inference",
        ),
    )
    .expect("review inferred evidence");
    append_review_for_test(&mut space, review, "entry:promote-inference");

    let reviewed = check_native_close(&space, close_request_for(&space)).expect("close check");
    assert!(!reviewed
        .invariant_results
        .iter()
        .find(|result| result.invariant_id == id("close:native-evidence-accepted-or-waived"))
        .expect("evidence invariant")
        .witness_ids
        .contains(&id("evidence:ai-inference")));
}

#[test]
fn reopen_review_morphism_reopens_completion_for_close() {
    let mut space = fixture_space_with_completion();
    let completion_review = defer_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Completion,
            "completion:source-backed-evidence",
            ReviewAction::Defer,
            "revision:completion-deferred",
        ),
    )
    .expect("defer completion");
    append_review_for_test(&mut space, completion_review, "entry:completion-deferred");
    let morphism_review = accept_review_morphism(
        &space,
        request_for_space(
            &space,
            NativeReviewTargetKind::Morphism,
            "morphism:generated",
            ReviewAction::Accept,
            "revision:morphism-reviewed",
        ),
    )
    .expect("accept generated morphism");
    append_review_for_test(&mut space, morphism_review, "entry:morphism-reviewed");
    let reopened = reopen_review_morphism(
        &space,
        request_for_space(
            &space,
            NativeReviewTargetKind::Completion,
            "completion:source-backed-evidence",
            ReviewAction::Reopen,
            "revision:completion-reopened",
        ),
    )
    .expect("reopen completion");
    append_review_for_test(&mut space, reopened, "entry:completion-reopened");

    let close = check_native_close(
        &space,
        NativeCloseCheckRequest {
            declared_projection_loss_ids: vec![id("projection:lossy")],
            ..close_request_for(&space)
        },
    )
    .expect("close check");

    assert!(!close.closeable);
    assert!(close
        .blocker_ids
        .contains(&id("completion:source-backed-evidence")));
}

#[test]
fn unreviewed_review_morphism_does_not_satisfy_close() {
    let mut space = fixture_space_with_completion();
    let mut projection_review = accept_review_morphism(
        &space,
        request(
            NativeReviewTargetKind::Waiver,
            "projection:lossy",
            ReviewAction::Accept,
            "revision:projection-unreviewed",
        ),
    )
    .expect("accept projection loss");
    projection_review.review_status = ReviewStatus::Unreviewed;
    append_review_for_test(&mut space, projection_review, "entry:projection-unreviewed");

    let close = check_native_close(&space, close_request_for(&space)).expect("close check");

    assert!(!close.closeable);
    assert!(close.blocker_ids.contains(&id("projection:lossy")));
}

#[test]
fn generated_morphism_remains_reviewable_until_explicit_review() {
    let space = fixture_space_with_completion();
    let evaluation = evaluate_native_case(&space).expect("evaluation");

    assert!(evaluation.review_gaps.iter().any(|gap| {
        gap.gap_type == NativeReviewGapType::UnreviewedMorphism
            && gap.target_id == id("morphism:generated")
    }));
    assert!(
        !check_native_close(&space, close_request())
            .expect("close check")
            .closeable
    );
}

fn fixture_space_with_completion() -> CaseSpace {
    let mut space = fixture_space();
    space.case_cells.push(cell(
        "work:needs-evidence",
        CaseCellType::Work,
        CaseCellLifecycle::Active,
        SourceKind::Human,
        ReviewStatus::Reviewed,
    ));
    space.case_cells.push(cell(
        "evidence:source-backed",
        CaseCellType::Evidence,
        CaseCellLifecycle::Active,
        SourceKind::Document,
        ReviewStatus::Reviewed,
    ));
    space.case_cells.push(cell(
        "completion:source-backed-evidence",
        CaseCellType::Completion,
        CaseCellLifecycle::Proposed,
        SourceKind::Ai,
        ReviewStatus::Unreviewed,
    ));
    space.case_relations.push(relation(
        "relation:needs-evidence",
        CaseRelationType::RequiresEvidence,
        "work:needs-evidence",
        "evidence:source-backed",
    ));
    space.projections.push(Projection {
        projection_id: id("projection:lossy"),
        audience: ProjectionAudience::AiAgent,
        revision_id: space.revision.revision_id.clone(),
        represented_cell_ids: vec![id("work:needs-evidence")],
        represented_relation_ids: Vec::new(),
        omitted_cell_ids: vec![id("completion:source-backed-evidence")],
        omitted_relation_ids: Vec::new(),
        information_loss: vec![crate::native_model::ProjectionLoss {
            description: "Completion candidate omitted from AI projection.".to_owned(),
            represented_ids: vec![id("work:needs-evidence")],
            omitted_ids: vec![id("completion:source-backed-evidence")],
        }],
        allowed_operations: Vec::new(),
        source_ids: vec![id("source:test")],
        warnings: vec![crate::native_model::ProjectionWarning::InformationLoss],
        metadata: Map::new(),
    });
    refresh_added_ids(&mut space);
    space.morphism_log.push(generated_morphism(&space, 2));
    space.revision.revision_id = id("revision:generated");
    space.revision.parent_revision_id = Some(id("revision:fixture"));
    space.revision.applied_morphism_ids = vec![id("morphism:generated")];
    space.revision.checksum = "fixture-generated".to_owned();
    space.morphism_log[1].target_revision_id = space.revision.revision_id.clone();
    space.morphism_log[1].morphism.target_revision_id = space.revision.revision_id.clone();
    space.morphism_log[1].source_revision_id = Some(id("revision:fixture"));
    space.morphism_log[1].morphism.source_revision_id = Some(id("revision:fixture"));
    for projection in &mut space.projections {
        projection.revision_id = space.revision.revision_id.clone();
    }
    space
}

fn fixture_space() -> CaseSpace {
    let source_boundary = source_boundary_metadata();
    let revision = Revision {
        revision_id: id("revision:fixture"),
        case_space_id: id("case_space:review-fixture"),
        applied_entry_ids: vec![id("entry:genesis")],
        applied_morphism_ids: vec![id("morphism:genesis")],
        checksum: "fixture".to_owned(),
        parent_revision_id: None,
        created_at: "2026-04-26T00:00:00Z".to_owned(),
        source_ids: vec![id("source:test")],
        metadata: Map::new(),
    };
    let mut morphism_metadata = Map::new();
    morphism_metadata.insert("lift_semantics".to_owned(), json!("fixture_to_case_space"));
    morphism_metadata.insert(
        "source_boundary_id".to_owned(),
        json!("source_boundary:review-fixture"),
    );
    morphism_metadata.insert("source_boundary".to_owned(), source_boundary.clone());
    let morphism = CaseMorphism {
        morphism_id: id("morphism:genesis"),
        morphism_type: CaseMorphismType::Create,
        source_revision_id: None,
        target_revision_id: revision.revision_id.clone(),
        added_ids: Vec::new(),
        updated_ids: Vec::new(),
        retired_ids: Vec::new(),
        preserved_ids: Vec::new(),
        violated_invariant_ids: Vec::new(),
        review_status: ReviewStatus::Accepted,
        evidence_ids: Vec::new(),
        source_ids: vec![id("source:test")],
        metadata: morphism_metadata,
    };
    let mut metadata = Map::new();
    metadata.insert("source_boundary".to_owned(), source_boundary);
    CaseSpace {
        schema: NATIVE_CASE_SPACE_SCHEMA.to_owned(),
        schema_version: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        case_space_id: id("case_space:review-fixture"),
        space_id: id("space:review-fixture"),
        case_cells: Vec::new(),
        case_relations: Vec::new(),
        morphism_log: vec![MorphismLogEntry {
            schema: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA.to_owned(),
            schema_version: 1,
            case_space_id: id("case_space:review-fixture"),
            sequence: 1,
            entry_id: id("entry:genesis"),
            morphism_id: id("morphism:genesis"),
            source_revision_id: None,
            target_revision_id: revision.revision_id.clone(),
            morphism,
            actor_id: id("actor:test"),
            recorded_at: "2026-04-26T00:00:00Z".to_owned(),
            provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
            source_ids: vec![id("source:test")],
            previous_entry_hash: None,
            replay_checksum: "fixture".to_owned(),
        }],
        projections: Vec::new(),
        revision,
        close_policy_id: Some(id("close_policy:native-default")),
        metadata,
    }
}

fn source_boundary_metadata() -> serde_json::Value {
    json!({
        "id": "source_boundary:review-fixture",
        "included_sources": ["source:test"],
        "excluded_sources": [],
        "adapters": ["native.review.fixture.v1"],
        "accepted_fact_policy": "fixture facts are accepted test input",
        "inference_policy": "fixture makes no inferred claims",
        "information_loss": []
    })
}

fn generated_morphism(space: &CaseSpace, sequence: u64) -> MorphismLogEntry {
    let morphism = CaseMorphism {
        morphism_id: id("morphism:generated"),
        morphism_type: CaseMorphismType::Review,
        source_revision_id: Some(space.revision.revision_id.clone()),
        target_revision_id: id("revision:generated"),
        added_ids: Vec::new(),
        updated_ids: Vec::new(),
        retired_ids: Vec::new(),
        preserved_ids: vec![id("work:needs-evidence")],
        violated_invariant_ids: Vec::new(),
        review_status: ReviewStatus::Unreviewed,
        evidence_ids: Vec::new(),
        source_ids: vec![id("source:test")],
        metadata: Map::new(),
    };
    MorphismLogEntry {
        schema: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA.to_owned(),
        schema_version: 1,
        case_space_id: space.case_space_id.clone(),
        sequence,
        entry_id: id("entry:generated"),
        morphism_id: morphism.morphism_id.clone(),
        source_revision_id: morphism.source_revision_id.clone(),
        target_revision_id: morphism.target_revision_id.clone(),
        morphism,
        actor_id: id("actor:ai"),
        recorded_at: "2026-04-26T00:10:00Z".to_owned(),
        provenance: provenance(SourceKind::Ai, ReviewStatus::Unreviewed),
        source_ids: vec![id("source:test")],
        previous_entry_hash: None,
        replay_checksum: "fixture-generated".to_owned(),
    }
}

fn append_review_for_test(space: &mut CaseSpace, morphism: CaseMorphism, entry_id: &str) {
    let previous_revision_id = space.revision.revision_id.clone();
    let target_revision_id = morphism.target_revision_id.clone();
    space.morphism_log.push(MorphismLogEntry {
        schema: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA.to_owned(),
        schema_version: 1,
        case_space_id: space.case_space_id.clone(),
        sequence: space.morphism_log.len() as u64 + 1,
        entry_id: id(entry_id),
        morphism_id: morphism.morphism_id.clone(),
        source_revision_id: Some(previous_revision_id.clone()),
        target_revision_id: target_revision_id.clone(),
        morphism,
        actor_id: id("actor:reviewer"),
        recorded_at: "2026-04-26T00:20:00Z".to_owned(),
        provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
        source_ids: vec![id("source:test")],
        previous_entry_hash: None,
        replay_checksum: "fixture-review".to_owned(),
    });
    space.revision.revision_id = target_revision_id;
    space.revision.parent_revision_id = Some(previous_revision_id);
    space.revision.checksum = "fixture-review".to_owned();
    for projection in &mut space.projections {
        projection.revision_id = space.revision.revision_id.clone();
    }
}

fn refresh_added_ids(space: &mut CaseSpace) {
    space.morphism_log[0].morphism.added_ids = space
        .case_cells
        .iter()
        .map(|cell| cell.id.clone())
        .chain(
            space
                .case_relations
                .iter()
                .map(|relation| relation.id.clone()),
        )
        .collect();
}

fn cell(
    id_value: &str,
    cell_type: CaseCellType,
    lifecycle: CaseCellLifecycle,
    source_kind: SourceKind,
    review_status: ReviewStatus,
) -> CaseCell {
    CaseCell {
        id: id(id_value),
        cell_type,
        space_id: id("space:review-fixture"),
        title: id_value.to_owned(),
        summary: None,
        lifecycle,
        source_ids: vec![id("source:test")],
        structure_ids: Vec::new(),
        provenance: provenance(source_kind, review_status),
        metadata: Map::new(),
    }
}

fn relation(
    id_value: &str,
    relation_type: CaseRelationType,
    from_id: &str,
    to_id: &str,
) -> CaseRelation {
    CaseRelation {
        id: id(id_value),
        relation_type,
        relation_strength: RelationStrength::Hard,
        from_id: id(from_id),
        to_id: id(to_id),
        evidence_ids: Vec::new(),
        source_ids: vec![id("source:test")],
        provenance: provenance(SourceKind::Human, ReviewStatus::Reviewed),
        metadata: Map::new(),
    }
}

fn request(
    target_kind: NativeReviewTargetKind,
    target_id: &str,
    action: ReviewAction,
    target_revision_id: &str,
) -> NativeReviewRequest {
    NativeReviewRequest {
        target_kind,
        target_id: id(target_id),
        action,
        reviewer_id: id("reviewer:native"),
        reviewed_at: "2026-04-26T00:30:00Z".to_owned(),
        reason: "Reviewed during native review API test.".to_owned(),
        evidence_ids: Vec::new(),
        source_ids: vec![id("source:test")],
        target_revision_id: id(target_revision_id),
    }
}

fn request_for_space(
    space: &CaseSpace,
    target_kind: NativeReviewTargetKind,
    target_id: &str,
    action: ReviewAction,
    target_revision_id: &str,
) -> NativeReviewRequest {
    let request = request(target_kind, target_id, action, target_revision_id);
    assert_ne!(request.target_revision_id, space.revision.revision_id);
    request
}

fn close_request() -> NativeCloseCheckRequest {
    NativeCloseCheckRequest {
        close_policy_id: Some(id("close_policy:native-default")),
        base_revision_id: id("revision:fixture"),
        declared_projection_loss_ids: Vec::new(),
        validation_evidence_ids: vec![id("source:test")],
        source_ids: vec![id("source:test")],
        operation_gate: Some(NativeOperationGate {
            actor_id: id("actor:native-review-test"),
            operation: "close-check".to_owned(),
            operation_scope_id: id("case_space:review-fixture"),
            audience: ProjectionAudience::Audit,
            capability_ids: vec![id("capability:native-review-test:close-check")],
            source_boundary_id: id("source_boundary:review-fixture"),
        }),
    }
}

fn close_request_for(space: &CaseSpace) -> NativeCloseCheckRequest {
    let validation_evidence_ids = space
        .case_cells
        .iter()
        .find(|cell| cell.cell_type == CaseCellType::Evidence)
        .map(|cell| vec![cell.id.clone()])
        .unwrap_or_else(|| close_request().validation_evidence_ids);
    NativeCloseCheckRequest {
        base_revision_id: space.revision.revision_id.clone(),
        validation_evidence_ids,
        ..close_request()
    }
}

fn provenance(kind: SourceKind, status: ReviewStatus) -> Provenance {
    Provenance::new(
        SourceRef::new(kind),
        Confidence::new(1.0).expect("confidence"),
    )
    .with_review_status(status)
}
