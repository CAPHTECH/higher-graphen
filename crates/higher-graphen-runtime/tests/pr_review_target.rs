//! Contract tests for the PR review target runtime workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_pr_review_target_recommend, AiProjectionRecordType, PrReviewTargetInputDocument,
    PrReviewTargetStatus, ProjectionAudience, ProjectionPurpose,
};
use serde_json::{json, Value};

const INPUT_SCHEMA: &str = "highergraphen.pr_review_target.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
const REPORT_TYPE: &str = "pr_review_target";
const RUNTIME_FILE: &str = "file:crates/runtime/src/workflows/architecture_input_lift.rs";
const LIFT_SYMBOL: &str = "symbol:architecture-input-lift:lift_input";
const SIGNAL_TEST_GAP: &str = "signal:architecture-lift-test-gap";

#[test]
fn runner_recommends_targets_from_bounded_pr_fixture() {
    let report = run_pr_review_target_recommend(fixture()).expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(
        report.metadata.command,
        "highergraphen pr-review targets recommend"
    );
    assert_eq!(
        report.result.status,
        PrReviewTargetStatus::TargetsRecommended
    );
    assert_eq!(report.result.review_targets.len(), 2);
    assert_eq!(report.result.obstructions.len(), 1);
    assert_eq!(report.result.completion_candidates.len(), 1);
}

#[test]
fn scenario_lifts_accepted_input_facts_without_accepting_proposals() {
    let report = run_pr_review_target_recommend(fixture()).expect("workflow should run");
    let scenario = report.scenario;

    assert_eq!(scenario.input_schema, INPUT_SCHEMA);
    assert_eq!(
        scenario.lifted_structure.space.id,
        id("space:pr-review-target:pr:higher-graphen:42")
    );
    assert!(scenario
        .lifted_structure
        .space
        .cell_ids
        .contains(&id(RUNTIME_FILE)));
    assert!(scenario
        .lifted_structure
        .space
        .cell_ids
        .contains(&id(SIGNAL_TEST_GAP)));

    let runtime_file = scenario
        .lifted_structure
        .cells
        .iter()
        .find(|cell| cell.id == id(RUNTIME_FILE))
        .expect("runtime file cell");
    assert_eq!(
        runtime_file.provenance.review_status,
        ReviewStatus::Accepted
    );
    assert_eq!(
        runtime_file.provenance.extraction_method.as_deref(),
        Some("pr_review_target_lift.v1")
    );
}

#[test]
fn result_and_projection_keep_ai_proposals_unreviewed() {
    let report = run_pr_review_target_recommend(fixture()).expect("workflow should run");
    let result = report.result;

    assert!(result.accepted_change_ids.contains(&id(RUNTIME_FILE)));
    assert!(result.accepted_change_ids.contains(&id(LIFT_SYMBOL)));
    assert!(result
        .review_targets
        .iter()
        .all(|target| target.review_status == ReviewStatus::Unreviewed));
    assert!(result
        .obstructions
        .iter()
        .all(|obstruction| obstruction.review_status == ReviewStatus::Unreviewed));
    assert!(result
        .completion_candidates
        .iter()
        .all(|candidate| candidate.review_status == ReviewStatus::Unreviewed));

    let projection = report.projection;
    assert_eq!(projection.audience, ProjectionAudience::Human);
    assert_eq!(projection.purpose, ProjectionPurpose::PrReviewTargeting);
    assert!(!projection.source_ids.is_empty());
    assert!(!projection.information_loss.is_empty());
    assert_eq!(projection.ai_view.audience, ProjectionAudience::AiAgent);
    assert!(projection
        .ai_view
        .records
        .iter()
        .any(
            |record| record.record_type == AiProjectionRecordType::ReviewTarget
                && record.review_status == Some(ReviewStatus::Unreviewed)
        ));
    assert_eq!(projection.audit_trace.audience, ProjectionAudience::Audit);
    assert!(!projection.audit_trace.traces.is_empty());
}

#[test]
fn no_signals_is_successful_no_targets_result() {
    let mut input = fixture();
    input.signals.clear();

    let report = run_pr_review_target_recommend(input).expect("workflow should run");

    assert_eq!(report.result.status, PrReviewTargetStatus::NoTargets);
    assert!(report.result.review_targets.is_empty());
    assert!(report.result.obstructions.is_empty());
    assert!(report.result.completion_candidates.is_empty());
    assert!(!report.result.source_ids.is_empty());
}

#[test]
fn rejects_unknown_symbol_file_reference() {
    let mut input = fixture();
    input.symbols[0].file_id = id("file:missing");

    let error = run_pr_review_target_recommend(input).expect_err("unknown file should fail");

    assert_eq!(error.code(), "workflow_construction");
    assert!(error
        .to_string()
        .contains("references unknown file file:missing"));
}

#[test]
fn rejects_duplicate_input_identifiers() {
    let mut input = fixture();
    input.signals[0].id = id(RUNTIME_FILE);

    let error = run_pr_review_target_recommend(input).expect_err("duplicate ids should fail");

    assert_eq!(error.code(), "workflow_construction");
    assert!(error
        .to_string()
        .contains("signal id file:crates/runtime/src/workflows/architecture_input_lift.rs duplicates existing changed_file id"));
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let report = run_pr_review_target_recommend(fixture()).expect("workflow should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("targets_recommended"));
    assert_eq!(
        value["result"]["review_targets"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["scenario"]["lifted_structure"]["cells"][0]["provenance"]["review_status"],
        json!("accepted")
    );
    assert_eq!(
        value["projection"]["ai_view"]["purpose"],
        json!("pr_review_targeting")
    );

    let json_text = serde_json::to_string(&report).expect("serialize report text");
    let round_tripped: Value = serde_json::from_str(&json_text).expect("parse report json");
    assert_eq!(round_tripped["schema"], json!(REPORT_SCHEMA));
}

fn fixture() -> PrReviewTargetInputDocument {
    serde_json::from_str(include_str!(
        "../../../schemas/inputs/pr-review-target.input.example.json"
    ))
    .expect("fixture should parse")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
