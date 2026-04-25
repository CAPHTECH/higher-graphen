//! Contract tests for the explicit completion review runtime workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, run_completion_review, AiProjectionRecordType,
    CompletionReviewDecision, CompletionReviewRequest, CompletionReviewSnapshot,
    CompletionReviewSourceReport, CompletionReviewStatus, ProjectionAudience, ProjectionPurpose,
};
use serde_json::{json, Value};

const REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
const REPORT_TYPE: &str = "completion_review";
const SOURCE_REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const SOURCE_REPORT_TYPE: &str = "architecture_direct_db_access_smoke";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

#[test]
fn accepts_candidate_from_source_report_without_promoting_source_candidate() {
    let snapshot = smoke_snapshot();
    let request = CompletionReviewRequest::accepted(
        id(BILLING_STATUS_API_CANDIDATE),
        id("reviewer:architecture-lead"),
        "Billing service owns this API boundary.",
    )
    .expect("valid request")
    .with_reviewed_at("2026-04-25T00:00:00Z")
    .expect("valid reviewed_at");

    let report = run_completion_review(snapshot, request).expect("review should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(
        report.metadata.command,
        "highergraphen completion review accept"
    );
    assert_eq!(report.scenario.source_report.schema, SOURCE_REPORT_SCHEMA);
    assert_eq!(
        report.scenario.source_report.report_type,
        SOURCE_REPORT_TYPE
    );
    assert_eq!(
        report.scenario.candidate.review_status,
        ReviewStatus::Unreviewed
    );
    assert_eq!(report.result.status, CompletionReviewStatus::Accepted);

    let record = report.result.review_record;
    assert_eq!(record.request.decision, CompletionReviewDecision::Accepted);
    assert_eq!(
        record.request.reviewed_at.as_deref(),
        Some("2026-04-25T00:00:00Z")
    );
    assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(record.outcome_review_status, ReviewStatus::Accepted);
    assert!(record.rejected_completion.is_none());
    let accepted = record
        .accepted_completion
        .expect("accepted completion payload");
    assert_eq!(accepted.review_status, ReviewStatus::Accepted);
    assert_eq!(
        accepted.accepted_structure.structure_id,
        Some(id(BILLING_STATUS_API_CELL))
    );
}

#[test]
fn rejects_candidate_from_source_report_with_audit_record() {
    let snapshot = smoke_snapshot();
    let request = CompletionReviewRequest::rejected(
        id(BILLING_STATUS_API_CANDIDATE),
        id("reviewer:architecture-lead"),
        "The service will expose an event instead.",
    )
    .expect("valid request");

    let report = run_completion_review(snapshot, request).expect("review should run");

    assert_eq!(
        report.metadata.command,
        "highergraphen completion review reject"
    );
    assert_eq!(report.result.status, CompletionReviewStatus::Rejected);
    assert_eq!(report.projection.audience, ProjectionAudience::Human);
    assert_eq!(
        report.projection.purpose,
        ProjectionPurpose::CompletionReview
    );
    assert!(report
        .projection
        .source_ids
        .contains(&id(BILLING_STATUS_API_CANDIDATE)));
    assert!(report
        .projection
        .source_ids
        .contains(&id(BILLING_STATUS_API_CELL)));
    assert!(!report.projection.information_loss.is_empty());
    assert_eq!(
        report.projection.ai_view.audience,
        ProjectionAudience::AiAgent
    );
    assert!(!report.projection.ai_view.information_loss.is_empty());
    assert!(report
        .projection
        .ai_view
        .records
        .iter()
        .any(
            |record| record.record_type == AiProjectionRecordType::CompletionReview
                && record.review_status == Some(ReviewStatus::Rejected)
        ));
    assert_eq!(
        report.projection.audit_trace.audience,
        ProjectionAudience::Audit
    );
    assert!(!report.projection.audit_trace.information_loss.is_empty());
    assert!(report
        .projection
        .audit_trace
        .traces
        .iter()
        .any(|trace| trace.source_id == id("reviewer:architecture-lead")));

    let record = report.result.review_record;
    assert_eq!(record.request.decision, CompletionReviewDecision::Rejected);
    assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
    assert!(record.accepted_completion.is_none());
    let rejected = record
        .rejected_completion
        .expect("rejected completion payload");
    assert_eq!(rejected.review_status, ReviewStatus::Rejected);
    assert_eq!(
        rejected.rejected_structure.structure_id,
        Some(id(BILLING_STATUS_API_CELL))
    );
}

#[test]
fn refuses_unknown_or_already_reviewed_candidates() {
    let unknown_request = CompletionReviewRequest::accepted(
        id("candidate:missing"),
        id("reviewer:architecture-lead"),
        "Reviewed",
    )
    .expect("valid request");
    let missing_error =
        run_completion_review(smoke_snapshot(), unknown_request).expect_err("missing candidate");
    assert_eq!(missing_error.code(), "workflow_construction");
    assert!(missing_error.to_string().contains("was not found"));

    let mut snapshot = smoke_snapshot();
    snapshot.completion_candidates[0].review_status = ReviewStatus::Accepted;
    let reviewed_request = CompletionReviewRequest::rejected(
        id(BILLING_STATUS_API_CANDIDATE),
        id("reviewer:architecture-lead"),
        "Reviewed",
    )
    .expect("valid request");
    let reviewed_error =
        run_completion_review(snapshot, reviewed_request).expect_err("reviewed candidate");
    assert_eq!(reviewed_error.code(), "workflow_construction");
    assert!(reviewed_error
        .to_string()
        .contains("only unreviewed candidates can be reviewed"));
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let request = CompletionReviewRequest::accepted(
        id(BILLING_STATUS_API_CANDIDATE),
        id("reviewer:architecture-lead"),
        "Reviewed",
    )
    .expect("valid request");
    let report = run_completion_review(smoke_snapshot(), request).expect("review should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("accepted"));
    assert_eq!(
        value["result"]["review_record"]["request"]["decision"],
        json!("accepted")
    );
    assert_eq!(
        value["result"]["review_record"]["candidate"]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["review_record"]["accepted_completion"]["review_status"],
        json!("accepted")
    );
    assert_eq!(
        value["projection"]["ai_view"]["audience"],
        json!("ai_agent")
    );
    assert_eq!(
        value["projection"]["audit_trace"]["purpose"],
        json!("audit_trace")
    );

    let json_text = serde_json::to_string(&report).expect("serialize report text");
    let round_tripped: Value = serde_json::from_str(&json_text).expect("parse report json");
    assert_eq!(round_tripped["schema"], json!(REPORT_SCHEMA));
    assert_eq!(
        round_tripped["scenario"]["source_report"]["schema"],
        json!(SOURCE_REPORT_SCHEMA)
    );
}

fn smoke_snapshot() -> CompletionReviewSnapshot {
    let report = run_architecture_direct_db_access_smoke().expect("smoke report");
    CompletionReviewSnapshot {
        source_report: CompletionReviewSourceReport {
            schema: report.schema,
            report_type: report.report_type,
            report_version: report.report_version,
            command: report.metadata.command,
        },
        completion_candidates: report.result.completion_candidates,
    }
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
