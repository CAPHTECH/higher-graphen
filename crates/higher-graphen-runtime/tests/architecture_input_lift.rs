//! Contract tests for the architecture input lift runtime workflow.

use higher_graphen_core::{Id, ReviewStatus, SourceKind};
use higher_graphen_runtime::{
    run_architecture_input_lift, AiProjectionRecordType, ArchitectureInputLiftDocument,
    ArchitectureInputLiftStatus, ProjectionAudience, ProjectionPurpose,
};
use serde_json::{json, Value};

const INPUT_SCHEMA: &str = "highergraphen.architecture.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const REPORT_TYPE: &str = "architecture_input_lift";
const ARCHITECTURE_SPACE: &str = "space:architecture-product-input";
const ORDER_CONTEXT: &str = "context:orders";
const BILLING_CONTEXT: &str = "context:billing";
const ORDER_SERVICE: &str = "cell:order-service";
const BILLING_DB: &str = "cell:billing-db";
const ORDER_READS_BILLING_DB: &str = "incidence:order-service-reads-billing-db";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api-input";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

#[test]
fn runner_lifts_bounded_json_fixture() {
    let report = run_architecture_input_lift(fixture()).expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(
        report.metadata.command,
        "highergraphen architecture input lift"
    );
    assert_eq!(report.result.status, ArchitectureInputLiftStatus::Lifted);
}

#[test]
fn scenario_contains_accepted_cells_incidences_and_source_provenance() {
    let report = run_architecture_input_lift(fixture()).expect("workflow should run");
    let scenario = report.scenario;

    assert_eq!(scenario.input_schema, INPUT_SCHEMA);
    assert_eq!(scenario.source.kind, SourceKind::Document);
    assert_eq!(scenario.space.id, id(ARCHITECTURE_SPACE));
    assert!(scenario.space.cell_ids.contains(&id(ORDER_SERVICE)));
    assert!(scenario
        .space
        .incidence_ids
        .contains(&id(ORDER_READS_BILLING_DB)));
    assert!(scenario.space.context_ids.contains(&id(ORDER_CONTEXT)));
    assert!(scenario.space.context_ids.contains(&id(BILLING_CONTEXT)));

    let order_service = scenario
        .cells
        .iter()
        .find(|cell| cell.id == id(ORDER_SERVICE))
        .expect("order service cell");
    assert_eq!(order_service.dimension, 0);
    assert_eq!(order_service.cell_type, "service");
    assert_eq!(order_service.context_ids, vec![id(ORDER_CONTEXT)]);
    let provenance = order_service.provenance.as_ref().expect("cell provenance");
    assert_eq!(provenance.review_status, ReviewStatus::Accepted);
    assert_eq!(provenance.confidence.value(), 1.0);
    assert_eq!(
        provenance.source.source_local_id.as_deref(),
        Some("components.order_service")
    );
}

#[test]
fn result_keeps_accepted_facts_and_inferences_separate() {
    let report = run_architecture_input_lift(fixture()).expect("workflow should run");
    let result = report.result;

    assert!(result.accepted_fact_ids.contains(&id(ORDER_SERVICE)));
    assert!(result
        .accepted_fact_ids
        .contains(&id(ORDER_READS_BILLING_DB)));
    assert!(!result
        .accepted_fact_ids
        .contains(&id(BILLING_STATUS_API_CELL)));
    assert_eq!(
        result.inferred_structure_ids,
        vec![id(BILLING_STATUS_API_CANDIDATE)]
    );

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
    assert_eq!(candidate.inferred_from, vec![id(ORDER_READS_BILLING_DB)]);
}

#[test]
fn projection_summarizes_lift_without_promoting_candidates() {
    let report = run_architecture_input_lift(fixture()).expect("workflow should run");
    let projection = report.projection;

    assert_eq!(projection.audience, ProjectionAudience::Human);
    assert_eq!(projection.purpose, ProjectionPurpose::ArchitectureReview);
    assert!(projection.summary.contains("Lifted 5 accepted facts"));
    assert!(projection
        .summary
        .contains("1 inferred structures as unreviewed candidates"));
    assert!(projection
        .source_ids
        .contains(&id(BILLING_STATUS_API_CANDIDATE)));
    assert!(projection.source_ids.contains(&id(ORDER_READS_BILLING_DB)));
    assert!(!projection.information_loss.is_empty());
    assert_eq!(projection.human_review.audience, ProjectionAudience::Human);
    assert!(!projection.human_review.information_loss.is_empty());
    assert_eq!(projection.ai_view.audience, ProjectionAudience::AiAgent);
    assert!(!projection.ai_view.information_loss.is_empty());
    let cell_record = projection
        .ai_view
        .records
        .iter()
        .find(|record| record.id == id(ORDER_SERVICE))
        .expect("order service AI record");
    assert_eq!(cell_record.record_type, AiProjectionRecordType::Cell);
    assert_eq!(cell_record.review_status, Some(ReviewStatus::Accepted));
    assert_eq!(cell_record.confidence.expect("confidence").value(), 1.0);
    assert!(cell_record.provenance.is_some());
    let candidate_record = projection
        .ai_view
        .records
        .iter()
        .find(|record| record.id == id(BILLING_STATUS_API_CANDIDATE))
        .expect("candidate AI record");
    assert_eq!(
        candidate_record.record_type,
        AiProjectionRecordType::CompletionCandidate
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
        .any(|trace| trace.source_id == id(ORDER_READS_BILLING_DB)));
}

#[test]
fn runner_reuses_interpretation_package_for_non_database_fixture() {
    let report = run_architecture_input_lift(reuse_fixture()).expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.result.status, ArchitectureInputLiftStatus::Lifted);
    assert_eq!(report.result.accepted_fact_ids.len(), 9);
    assert!(report.result.inferred_structure_ids.is_empty());
    assert!(report.result.completion_candidates.is_empty());

    let cell_types = report
        .scenario
        .cells
        .iter()
        .map(|cell| cell.cell_type.as_str())
        .collect::<Vec<_>>();
    assert!(cell_types.contains(&"api"));
    assert!(cell_types.contains(&"event"));
    assert!(cell_types.contains(&"requirement"));
    assert!(cell_types.contains(&"test"));

    let relation_types = report
        .scenario
        .incidences
        .iter()
        .map(|incidence| incidence.relation_type.as_str())
        .collect::<Vec<_>>();
    assert!(relation_types.contains(&"calls_api"));
    assert!(relation_types.contains(&"publishes_event"));
    assert!(relation_types.contains(&"realizes_requirement"));
    assert!(relation_types.contains(&"covered_by_test"));
    assert!(report
        .projection
        .summary
        .contains("Lifted 9 accepted facts"));
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let report = run_architecture_input_lift(fixture()).expect("workflow should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("lifted"));
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["scenario"]["cells"][0]["provenance"]["review_status"],
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
        round_tripped["scenario"]["space"]["id"],
        json!(ARCHITECTURE_SPACE)
    );
}

#[test]
fn rejects_inference_that_reuses_an_accepted_cell_id() {
    let mut input = fixture();
    input.inferred_structures[0].structure_id = Some(id(BILLING_DB));

    let error = run_architecture_input_lift(input).expect_err("boundary should be rejected");

    assert_eq!(error.code(), "workflow_construction");
    assert!(error
        .to_string()
        .contains("proposes an already accepted cell"));
}

fn fixture() -> ArchitectureInputLiftDocument {
    serde_json::from_str(include_str!(
        "../../../schemas/inputs/architecture-lift.input.example.json"
    ))
    .expect("fixture should parse")
}

fn reuse_fixture() -> ArchitectureInputLiftDocument {
    serde_json::from_str(include_str!(
        "../../../schemas/inputs/architecture-lift.reuse.input.example.json"
    ))
    .expect("reuse fixture should parse")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
