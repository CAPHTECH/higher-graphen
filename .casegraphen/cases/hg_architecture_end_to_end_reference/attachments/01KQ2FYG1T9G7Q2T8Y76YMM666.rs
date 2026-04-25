//! End-to-end reference checks for the Architecture Product workflow.

use casegraphen::{
    eval::{projection_result, validate_case_graph},
    model::{CaseGraph, ProjectionDefinition, CASE_GRAPH_SCHEMA, PROJECTION_SCHEMA},
};
use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, run_architecture_input_lift, run_completion_review,
    AiProjectionRecordType, ArchitectureInputLiftDocument, ArchitectureInputLiftStatus,
    ArchitectureSmokeStatus, CompletionReviewDecision, CompletionReviewRequest,
    CompletionReviewSnapshot, CompletionReviewSourceReport, CompletionReviewStatus,
    ProjectionAudience,
};

const INPUT_REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const SMOKE_REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const REVIEW_REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
const CANDIDATE_ID: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

#[test]
fn reference_fixture_lifts_complete_architecture_vocabulary() {
    let report = run_architecture_input_lift(reference_input()).expect("reference lift");

    assert_eq!(report.schema, INPUT_REPORT_SCHEMA);
    assert_eq!(report.result.status, ArchitectureInputLiftStatus::Lifted);
    assert_eq!(report.scenario.cells.len(), 6);
    assert_eq!(report.scenario.incidences.len(), 5);
    assert!(report
        .result
        .accepted_fact_ids
        .contains(&id("cell:order-status-api")));
    assert!(report
        .result
        .accepted_fact_ids
        .contains(&id("cell:req-billing-status-visible")));
    assert!(report
        .result
        .accepted_fact_ids
        .contains(&id("cell:test-order-status-contract")));
    assert!(!report
        .result
        .accepted_fact_ids
        .contains(&id(BILLING_STATUS_API_CELL)));

    let candidate = report
        .result
        .completion_candidates
        .first()
        .expect("reference candidate");
    assert_eq!(report.result.completion_candidates.len(), 1);
    assert_eq!(candidate.id, id(CANDIDATE_ID));
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(
        candidate.suggested_structure.structure_id,
        Some(id(BILLING_STATUS_API_CELL))
    );
    assert_eq!(
        report.projection.human_review.audience,
        ProjectionAudience::Human
    );
    assert_eq!(
        report.projection.ai_view.audience,
        ProjectionAudience::AiAgent
    );
    assert!(!report.projection.audit_trace.traces.is_empty());
}

#[test]
fn reference_pipeline_detects_obstruction_then_requires_explicit_review() {
    let smoke_report = run_architecture_direct_db_access_smoke().expect("direct DB smoke workflow");

    assert_eq!(smoke_report.schema, SMOKE_REPORT_SCHEMA);
    assert_eq!(
        smoke_report.result.status,
        ArchitectureSmokeStatus::ViolationDetected
    );
    assert_eq!(smoke_report.result.obstructions.len(), 1);
    assert_eq!(smoke_report.result.completion_candidates.len(), 1);
    assert_eq!(
        smoke_report.result.completion_candidates[0].review_status,
        ReviewStatus::Unreviewed
    );
    assert!(smoke_report
        .projection
        .ai_view
        .records
        .iter()
        .any(|record| {
            record.record_type == AiProjectionRecordType::Obstruction
                && record.review_status == Some(ReviewStatus::Unreviewed)
        }));

    let review_report = run_completion_review(
        CompletionReviewSnapshot {
            source_report: CompletionReviewSourceReport {
                schema: smoke_report.schema.clone(),
                report_type: smoke_report.report_type.clone(),
                report_version: smoke_report.report_version,
                command: smoke_report.metadata.command.clone(),
            },
            completion_candidates: smoke_report.result.completion_candidates.clone(),
        },
        CompletionReviewRequest::new(
            id(CANDIDATE_ID),
            CompletionReviewDecision::Accepted,
            id("reviewer:architecture-lead"),
            "Billing owns the API boundary for status reads.",
        )
        .expect("review request")
        .with_reviewed_at("2026-04-25T00:00:00Z")
        .expect("review timestamp"),
    )
    .expect("completion review");

    assert_eq!(review_report.schema, REVIEW_REPORT_SCHEMA);
    assert_eq!(
        review_report.result.status,
        CompletionReviewStatus::Accepted
    );
    assert_eq!(
        review_report.result.review_record.candidate.review_status,
        ReviewStatus::Unreviewed
    );
    assert_eq!(
        review_report
            .result
            .review_record
            .accepted_completion
            .as_ref()
            .expect("accepted completion")
            .review_status,
        ReviewStatus::Accepted
    );
    assert!(review_report
        .projection
        .audit_trace
        .traces
        .iter()
        .any(|trace| {
            trace.source_id == id("reviewer:architecture-lead")
                && trace.represented_in.contains(&"audit_trace".to_owned())
        }));
}

#[test]
fn reference_casegraphen_fixture_validates_and_projects_sources() {
    let graph: CaseGraph = serde_json::from_str(include_str!(
        "../reference/casegraphen-reference.case.graph.json"
    ))
    .expect("reference case graph");
    let projection: ProjectionDefinition = serde_json::from_str(include_str!(
        "../reference/casegraphen-reference.projection.json"
    ))
    .expect("reference projection");

    assert_eq!(graph.schema, CASE_GRAPH_SCHEMA);
    assert_eq!(projection.schema, PROJECTION_SCHEMA);

    let validation = validate_case_graph(&graph);
    assert!(validation.valid, "errors: {:?}", validation.errors);
    assert_eq!(validation.counts.cases, 3);
    assert_eq!(validation.counts.review_records, 1);

    let projected = projection_result(&graph);
    assert_eq!(projected.projection_result, "projected");
    assert!(projected
        .selected_source_ids
        .contains(&id("source:architecture-reference-input")));
    assert!(projected
        .selected_source_ids
        .contains(&id("source:completion-review-report")));
    assert!(!projected.information_loss.is_empty());
}

fn reference_input() -> ArchitectureInputLiftDocument {
    serde_json::from_str(include_str!(
        "../reference/architecture-reference.input.json"
    ))
    .expect("reference input")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id")
}
