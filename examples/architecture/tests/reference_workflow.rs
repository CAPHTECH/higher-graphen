//! End-to-end reference checks for the Architecture Product workflow.

use casegraphen::{
    eval::{evaluate_coverage, projection_result, validate_case_graph},
    model::{
        CaseGraph, CoveragePolicy, ProjectionDefinition, CASE_GRAPH_SCHEMA, PROJECTION_SCHEMA,
    },
    report::{coverage_report, project_report, validate_report},
};
use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, run_architecture_input_lift, run_completion_review,
    AiProjectionRecordType, ArchitectureDirectDbAccessSmokeReport, ArchitectureInputLiftDocument,
    ArchitectureInputLiftReport, ArchitectureInputLiftStatus, ArchitectureSmokeStatus,
    CompletionReviewDecision, CompletionReviewReport, CompletionReviewRequest,
    CompletionReviewSnapshot, CompletionReviewSourceReport, CompletionReviewStatus,
    ProjectionAudience,
};
use std::path::Path;

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
    let coverage_policy: CoveragePolicy = serde_json::from_str(include_str!(
        "../reference/casegraphen-reference.coverage.policy.json"
    ))
    .expect("reference coverage policy");

    assert_eq!(graph.schema, CASE_GRAPH_SCHEMA);
    assert_eq!(projection.schema, PROJECTION_SCHEMA);

    let validation = validate_case_graph(&graph);
    assert!(validation.valid, "errors: {:?}", validation.errors);
    assert_eq!(validation.counts.cases, 4);
    assert_eq!(validation.counts.review_records, 1);

    let coverage = evaluate_coverage(&graph, &coverage_policy);
    assert_eq!(coverage.coverage_status, "covered");

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

#[test]
fn checked_in_runtime_reports_match_current_contracts() {
    let generated_lift = run_architecture_input_lift(reference_input()).expect("reference lift");
    let checked_in_lift: ArchitectureInputLiftReport = serde_json::from_str(include_str!(
        "../reference/reports/architecture-input-lift.report.json"
    ))
    .expect("checked-in input lift report parses");
    assert_eq!(
        report_value(&checked_in_lift),
        report_value(&generated_lift)
    );

    let generated_smoke =
        run_architecture_direct_db_access_smoke().expect("direct DB smoke workflow");
    let checked_in_smoke: ArchitectureDirectDbAccessSmokeReport = serde_json::from_str(
        include_str!("../reference/reports/architecture-direct-db-access-smoke.report.json"),
    )
    .expect("checked-in direct DB smoke report parses");
    assert_eq!(
        report_value(&checked_in_smoke),
        report_value(&generated_smoke)
    );

    let generated_review = run_completion_review(
        CompletionReviewSnapshot {
            source_report: CompletionReviewSourceReport {
                schema: checked_in_smoke.schema.clone(),
                report_type: checked_in_smoke.report_type.clone(),
                report_version: checked_in_smoke.report_version,
                command: checked_in_smoke.metadata.command.clone(),
            },
            completion_candidates: checked_in_smoke.result.completion_candidates.clone(),
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
    let checked_in_review: CompletionReviewReport = serde_json::from_str(include_str!(
        "../reference/reports/completion-review-accepted.report.json"
    ))
    .expect("checked-in completion review report parses");
    assert_eq!(
        report_value(&checked_in_review),
        report_value(&generated_review)
    );
}

#[test]
fn checked_in_casegraphen_reports_match_current_contracts() {
    let graph_path =
        Path::new("examples/architecture/reference/casegraphen-reference.case.graph.json");
    let coverage_path =
        Path::new("examples/architecture/reference/casegraphen-reference.coverage.policy.json");
    let projection_path =
        Path::new("examples/architecture/reference/casegraphen-reference.projection.json");
    let graph: CaseGraph = serde_json::from_str(include_str!(
        "../reference/casegraphen-reference.case.graph.json"
    ))
    .expect("reference case graph");
    let coverage_policy: CoveragePolicy = serde_json::from_str(include_str!(
        "../reference/casegraphen-reference.coverage.policy.json"
    ))
    .expect("reference coverage policy");

    let generated_validate = validate_report(
        "casegraphen validate",
        graph_path,
        &graph,
        validate_case_graph(&graph),
    );
    let checked_in_validate: serde_json::Value = serde_json::from_str(include_str!(
        "../reference/reports/casegraphen-validate.report.json"
    ))
    .expect("checked-in validate report parses");
    assert_eq!(
        checked_in_validate,
        report_value(&generated_validate),
        "casegraphen validate report drifted"
    );

    let generated_coverage = coverage_report(
        "casegraphen coverage",
        graph_path,
        coverage_path,
        &graph,
        evaluate_coverage(&graph, &coverage_policy),
    );
    let checked_in_coverage: serde_json::Value = serde_json::from_str(include_str!(
        "../reference/reports/casegraphen-coverage.report.json"
    ))
    .expect("checked-in coverage report parses");
    assert_eq!(
        checked_in_coverage,
        report_value(&generated_coverage),
        "casegraphen coverage report drifted"
    );

    let generated_project = project_report(
        "casegraphen project",
        graph_path,
        projection_path,
        &graph,
        projection_result(&graph),
    );
    let checked_in_project: serde_json::Value = serde_json::from_str(include_str!(
        "../reference/reports/casegraphen-project.report.json"
    ))
    .expect("checked-in project report parses");
    assert_eq!(
        checked_in_project,
        report_value(&generated_project),
        "casegraphen project report drifted"
    );
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

fn report_value(report: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(report).expect("report serializes")
}
