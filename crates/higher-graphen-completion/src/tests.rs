use super::*;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("valid confidence")
}

fn candidate() -> CompletionCandidate {
    let suggestion = SuggestedStructure::new("cell", "Add a missing API contract cell")
        .expect("valid suggestion")
        .with_structure_id(id("cell.contract"))
        .with_related_ids(vec![id("cell.api")]);

    CompletionCandidate::new(
        id("candidate.contract"),
        id("space.architecture"),
        MissingType::Cell,
        suggestion,
        vec![id("cell.api")],
        "The API has behavior but no contract cell.",
        confidence(0.82),
    )
    .expect("valid candidate")
}

fn rule() -> CompletionRule {
    let suggestion = SuggestedStructure::new("cell", "Add a missing API contract cell")
        .expect("valid suggestion")
        .with_structure_id(id("cell.contract"))
        .with_related_ids(vec![id("cell.api")]);

    CompletionRule::new(
        id("rule.contract"),
        id("candidate.contract"),
        MissingType::Cell,
        suggestion,
        "The API has behavior but no contract cell.",
        confidence(0.82),
    )
    .expect("valid rule")
    .with_context_ids(vec![id("context.api")])
    .with_inferred_from(vec![id("cell.api")])
}

#[test]
fn new_candidate_defaults_to_unreviewed() {
    let candidate = candidate();

    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(
        candidate.rationale,
        "The API has behavior but no contract cell."
    );
}

#[test]
fn detection_materializes_matching_rules_as_unreviewed_candidates() {
    let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
        .with_context_ids(vec![id("context.api"), id("context.review")]);

    let result = detect_completion_candidates(input).expect("completion detection should succeed");

    assert_eq!(result.candidates().len(), 1);
    let candidate = &result.candidates()[0];
    assert_eq!(candidate.id, id("candidate.contract"));
    assert_eq!(candidate.space_id, id("space.architecture"));
    assert_eq!(candidate.inferred_from, vec![id("cell.api")]);
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
}

#[test]
fn detection_skips_rules_for_missing_contexts() {
    let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
        .with_context_ids(vec![id("context.other")]);

    let result = detect_completion_candidates(input).expect("completion detection should succeed");

    assert!(result.candidates().is_empty());
}

#[test]
fn detection_result_rejects_reviewed_candidates() {
    let mut candidate = candidate();
    candidate.review_status = ReviewStatus::Accepted;

    let error =
        CompletionDetectionResult::new(id("space.architecture"), Vec::new(), vec![candidate])
            .expect_err("reviewed candidates should be rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn detection_result_rejects_duplicate_candidate_ids() {
    let candidate = candidate();

    let error = CompletionDetectionResult::new(
        id("space.architecture"),
        Vec::new(),
        vec![candidate.clone(), candidate],
    )
    .expect_err("duplicate candidate ids should be rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn deserialization_rejects_invalid_detection_results() {
    let reviewed = {
        let mut candidate = candidate();
        candidate.review_status = ReviewStatus::Accepted;
        candidate
    };
    let reviewed_value = serde_json::json!({
        "space_id": "space.architecture",
        "candidates": [reviewed]
    });
    let duplicate = candidate();
    let duplicate_value = serde_json::json!({
        "space_id": "space.architecture",
        "candidates": [duplicate, duplicate]
    });

    assert!(serde_json::from_value::<CompletionDetectionResult>(reviewed_value).is_err());
    assert!(serde_json::from_value::<CompletionDetectionResult>(duplicate_value).is_err());
}

#[test]
fn simple_engine_wraps_detection_and_review_helpers() {
    let engine = SimpleCompletionEngine;
    let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
        .with_context_ids(vec![id("context.api")]);

    let result = engine
        .detect_candidates(input)
        .expect("completion detection should succeed");
    let accepted = engine
        .accept_candidate(
            &result.candidates()[0],
            id("reviewer.architect"),
            "Reviewed",
        )
        .expect("accepted completion");

    assert_eq!(
        result.candidates()[0].review_status,
        ReviewStatus::Unreviewed
    );
    assert_eq!(accepted.review_status, ReviewStatus::Accepted);
}

#[test]
fn review_request_records_explicit_acceptance_without_mutating_candidate() {
    let candidate = candidate();
    let request = CompletionReviewRequest::accepted(
        candidate.id.clone(),
        id("reviewer.architect"),
        "Reviewed plan",
    )
    .expect("valid request")
    .with_reviewed_at("2026-04-25T00:00:00Z")
    .expect("valid review time");

    let record = review_completion(&candidate, request).expect("review record");

    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(record.decision(), CompletionReviewDecision::Accepted);
    assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(record.outcome_review_status, ReviewStatus::Accepted);
    assert_eq!(record.request.reason, "Reviewed plan");
    assert_eq!(
        record.request.reviewed_at.as_deref(),
        Some("2026-04-25T00:00:00Z")
    );
    let accepted = record
        .accepted_completion
        .expect("accepted completion payload");
    assert_eq!(accepted.review_status, ReviewStatus::Accepted);
    assert_eq!(
        accepted.accepted_structure.structure_id,
        Some(id("cell.contract"))
    );
    assert!(record.rejected_completion.is_none());
}

#[test]
fn review_request_records_explicit_rejection() {
    let candidate = candidate();
    let request = CompletionReviewRequest::rejected(
        candidate.id.clone(),
        id("reviewer.architect"),
        "Duplicate of an existing contract",
    )
    .expect("valid request");

    let record = SimpleCompletionEngine
        .review_candidate(&candidate, request)
        .expect("review record");

    assert_eq!(record.decision(), CompletionReviewDecision::Rejected);
    assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(record.outcome_review_status, ReviewStatus::Rejected);
    let rejected = record
        .rejected_completion
        .expect("rejected completion payload");
    assert_eq!(rejected.review_status, ReviewStatus::Rejected);
    assert_eq!(
        rejected.rejected_structure.structure_id,
        Some(id("cell.contract"))
    );
    assert!(record.accepted_completion.is_none());
}

#[test]
fn review_request_rejects_candidate_mismatch_and_empty_metadata() {
    let candidate = candidate();
    let mismatch = CompletionReviewRequest::accepted(
        id("candidate.other"),
        id("reviewer.architect"),
        "Reviewed",
    )
    .expect("valid request");

    let error = review_completion(&candidate, mismatch).expect_err("candidate mismatch");

    assert_eq!(error.code(), "malformed_field");
    assert!(CompletionReviewRequest::accepted(candidate.id.clone(), id("reviewer"), "  ").is_err());
    assert!(
        CompletionReviewRequest::rejected(candidate.id, id("reviewer"), "Rejected")
            .expect("valid request")
            .with_reviewed_at(" ")
            .is_err()
    );
}

#[test]
fn accept_returns_separate_accepted_completion() {
    let candidate = candidate();

    let accepted = accept_completion(&candidate, id("reviewer.architect"), "Reviewed plan")
        .expect("accepted completion");

    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(accepted.candidate_id, candidate.id);
    assert_eq!(accepted.review_status, ReviewStatus::Accepted);
    assert_eq!(accepted.reason, "Reviewed plan");
    assert_eq!(
        accepted.accepted_structure.structure_id,
        Some(id("cell.contract"))
    );
}

#[test]
fn reject_returns_rejection_record() {
    let candidate = candidate();

    let rejected = candidate
        .reject(
            id("reviewer.architect"),
            "Duplicate of an existing contract",
        )
        .expect("rejected completion");

    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(rejected.candidate_id, candidate.id);
    assert_eq!(rejected.review_status, ReviewStatus::Rejected);
    assert_eq!(rejected.reason, "Duplicate of an existing contract");
}

#[test]
fn review_helpers_require_non_empty_reason() {
    let candidate = candidate();

    let error = candidate
        .accept(id("reviewer.architect"), "   ")
        .expect_err("empty reason rejected");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn accepted_and_rejected_statuses_cannot_be_cross_reviewed() {
    let mut rejected_candidate = candidate();
    rejected_candidate.review_status = ReviewStatus::Rejected;
    let accept_error = rejected_candidate
        .accept(id("reviewer.architect"), "Reconsidered")
        .expect_err("cannot accept rejected candidate");
    assert_eq!(accept_error.code(), "malformed_field");

    let mut accepted_candidate = candidate();
    accepted_candidate.review_status = ReviewStatus::Accepted;
    let reject_error = accepted_candidate
        .reject(id("reviewer.architect"), "Too late")
        .expect_err("cannot reject accepted candidate");
    assert_eq!(reject_error.code(), "malformed_field");
}

#[test]
fn constructors_trim_and_validate_required_text() {
    let suggestion = SuggestedStructure::new("  invariant  ", "  Add a stability invariant  ")
        .expect("valid suggestion");

    assert_eq!(suggestion.structure_type, "invariant");
    assert_eq!(suggestion.summary, "Add a stability invariant");
    assert!(SuggestedStructure::new(" ", "summary").is_err());
}
