//! Contract tests for the architecture direct DB access smoke runtime workflow.

use higher_graphen_core::{Id, ReviewStatus, Severity};
use higher_graphen_obstruction::ObstructionType;
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, AiProjectionRecordType, ArchitectureSmokeStatus,
    ProjectionAudience, ProjectionPurpose,
};
use serde_json::{json, Value};

const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const REPORT_TYPE: &str = "architecture_direct_db_access_smoke";
const ARCHITECTURE_SPACE: &str = "space:architecture-product-smoke";
const ARCHITECTURE_CONTEXT: &str = "context:architecture-review";
const ORDER_CONTEXT: &str = "context:orders";
const BILLING_CONTEXT: &str = "context:billing";
const ORDER_SERVICE: &str = "cell:order-service";
const BILLING_SERVICE: &str = "cell:billing-service";
const BILLING_DB: &str = "cell:billing-db";
const ORDER_READS_BILLING_DB: &str = "incidence:order-service-reads-billing-db";
const BILLING_OWNS_BILLING_DB: &str = "incidence:billing-service-owns-billing-db";
const NO_CROSS_CONTEXT_DB_ACCESS: &str = "invariant:no-cross-context-direct-database-access";
const DIRECT_DB_ACCESS_OBSTRUCTION: &str = "obstruction:order-service-direct-billing-db-access";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

#[test]
fn runner_returns_violation_report_as_success() {
    let report = run_architecture_direct_db_access_smoke().expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(
        report.metadata.command,
        "highergraphen architecture smoke direct-db-access"
    );
    assert_eq!(report.metadata.runtime_package, "higher-graphen-runtime");
    assert_eq!(report.metadata.runtime_crate, "higher_graphen_runtime");
    assert_eq!(report.metadata.cli_package, "highergraphen-cli");
    assert_eq!(
        report.result.status,
        ArchitectureSmokeStatus::ViolationDetected
    );
}

#[test]
fn scenario_preserves_stable_ids_without_accepting_suggested_api() {
    let report = run_architecture_direct_db_access_smoke().expect("workflow should run");
    let scenario = report.scenario;

    assert_eq!(scenario.space_id, id(ARCHITECTURE_SPACE));
    assert_eq!(scenario.workflow_context_id, id(ARCHITECTURE_CONTEXT));
    assert_eq!(
        scenario.context_ids,
        vec![id(ORDER_CONTEXT), id(BILLING_CONTEXT)]
    );
    assert_eq!(
        scenario
            .cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect::<Vec<_>>(),
        vec![id(ORDER_SERVICE), id(BILLING_SERVICE), id(BILLING_DB)]
    );
    assert!(!scenario
        .cells
        .iter()
        .any(|cell| cell.id == id(BILLING_STATUS_API_CELL)));
    assert_eq!(
        scenario
            .incidences
            .iter()
            .map(|incidence| incidence.id.clone())
            .collect::<Vec<_>>(),
        vec![id(ORDER_READS_BILLING_DB), id(BILLING_OWNS_BILLING_DB)]
    );
    assert_eq!(scenario.invariant_id, id(NO_CROSS_CONTEXT_DB_ACCESS));
    assert_eq!(
        scenario.invariant_name,
        "No cross-context direct database access"
    );
}

#[test]
fn result_contains_check_obstruction_and_unreviewed_candidate_contract() {
    let report = run_architecture_direct_db_access_smoke().expect("workflow should run");
    let result = report.result;

    assert_eq!(result.violated_invariant_id, id(NO_CROSS_CONTEXT_DB_ACCESS));
    assert_eq!(
        result.check_result.target_id(),
        &id(NO_CROSS_CONTEXT_DB_ACCESS)
    );
    assert!(result.check_result.is_violated());

    let obstruction = result.obstructions.first().expect("one obstruction");
    assert_eq!(result.obstructions.len(), 1);
    assert_eq!(obstruction.id, id(DIRECT_DB_ACCESS_OBSTRUCTION));
    assert_eq!(
        obstruction.obstruction_type,
        ObstructionType::InvariantViolation
    );
    assert_eq!(
        obstruction.location_cell_ids,
        vec![id(ORDER_SERVICE), id(BILLING_DB)]
    );
    assert_eq!(
        obstruction.location_context_ids,
        vec![id(ORDER_CONTEXT), id(BILLING_CONTEXT)]
    );
    assert_eq!(obstruction.severity, Severity::Critical);
    assert!(obstruction.counterexample.is_some());
    assert!(obstruction.required_resolution.is_some());

    let candidate = result
        .completion_candidates
        .first()
        .expect("one completion candidate");
    assert_eq!(result.completion_candidates.len(), 1);
    assert_eq!(candidate.id, id(BILLING_STATUS_API_CANDIDATE));
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert_eq!(
        candidate.suggested_structure.structure_id,
        Some(id(BILLING_STATUS_API_CELL))
    );
    assert_eq!(candidate.suggested_structure.structure_type, "api");
    assert_eq!(
        candidate.inferred_from,
        vec![id(DIRECT_DB_ACCESS_OBSTRUCTION), id(ORDER_READS_BILLING_DB)]
    );
    assert_eq!(candidate.confidence.value(), 0.9);
}

#[test]
fn projection_is_traceable_human_architecture_review() {
    let report = run_architecture_direct_db_access_smoke().expect("workflow should run");
    let projection = report.projection;

    assert_eq!(projection.audience, ProjectionAudience::Human);
    assert_eq!(projection.purpose, ProjectionPurpose::ArchitectureReview);
    assert!(projection
        .summary
        .contains("Order Service directly reads Billing DB"));
    assert!(!projection.recommended_actions.is_empty());
    assert!(!projection.information_loss.is_empty());
    assert!(projection.source_ids.contains(&id(ORDER_READS_BILLING_DB)));
    assert!(projection
        .source_ids
        .contains(&id(DIRECT_DB_ACCESS_OBSTRUCTION)));
    assert!(projection
        .source_ids
        .contains(&id(BILLING_STATUS_API_CANDIDATE)));
    assert!(projection.source_ids.contains(&id(BILLING_STATUS_API_CELL)));
    assert_eq!(projection.human_review.audience, ProjectionAudience::Human);
    assert!(!projection.human_review.information_loss.is_empty());
    assert_eq!(projection.ai_view.audience, ProjectionAudience::AiAgent);
    assert!(!projection.ai_view.information_loss.is_empty());
    assert!(projection.ai_view.source_ids.contains(&id(BILLING_DB)));
    let obstruction_record = projection
        .ai_view
        .records
        .iter()
        .find(|record| record.id == id(DIRECT_DB_ACCESS_OBSTRUCTION))
        .expect("obstruction AI record");
    assert_eq!(
        obstruction_record.record_type,
        AiProjectionRecordType::Obstruction
    );
    assert_eq!(
        obstruction_record.review_status,
        Some(ReviewStatus::Unreviewed)
    );
    assert_eq!(
        obstruction_record.confidence.expect("confidence").value(),
        1.0
    );
    assert!(obstruction_record.provenance.is_some());
    let candidate_record = projection
        .ai_view
        .records
        .iter()
        .find(|record| record.id == id(BILLING_STATUS_API_CANDIDATE))
        .expect("candidate AI record");
    assert_eq!(
        candidate_record.confidence.expect("confidence").value(),
        0.9
    );
    assert_eq!(
        candidate_record.review_status,
        Some(ReviewStatus::Unreviewed)
    );
    assert_eq!(projection.audit_trace.audience, ProjectionAudience::Audit);
    assert!(!projection.audit_trace.information_loss.is_empty());
    assert!(projection
        .audit_trace
        .traces
        .iter()
        .any(|trace| trace.source_id == id(BILLING_STATUS_API_CELL)));
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let report = run_architecture_direct_db_access_smoke().expect("workflow should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("violation_detected"));
    assert_eq!(
        value["result"]["obstructions"][0]["obstruction_type"],
        json!("invariant_violation")
    );
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(value["projection"]["audience"], json!("human"));
    assert_eq!(value["projection"]["purpose"], json!("architecture_review"));
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
        round_tripped["result"]["violated_invariant_id"],
        json!(NO_CROSS_CONTEXT_DB_ACCESS)
    );
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
