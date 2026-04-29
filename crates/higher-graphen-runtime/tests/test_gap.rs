//! Contract tests for the missing unit test detector runtime workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_test_gap_detect, TestGapInputDocument, TestGapObstructionType, TestGapStatus,
};
use serde_json::{json, Value};

const INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.test_gap.report.v1";
const REPORT_TYPE: &str = "test_gap";
const ZERO_BRANCH: &str = "branch:pricing:calculate_discount:discount_percent_zero";

#[test]
fn runner_detects_missing_unit_test_from_bounded_fixture() {
    let report = run_test_gap_detect(fixture()).expect("workflow should run");

    assert_eq!(report.schema, REPORT_SCHEMA);
    assert_eq!(report.report_type, REPORT_TYPE);
    assert_eq!(report.report_version, 1);
    assert_eq!(report.metadata.command, "highergraphen test-gap detect");
    assert_eq!(report.result.status, TestGapStatus::GapsDetected);
    assert!(report.result.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == TestGapObstructionType::MissingBoundaryCaseUnitTest
            && obstruction.target_ids.contains(&id(ZERO_BRANCH))
    }));
    assert!(report
        .result
        .completion_candidates
        .iter()
        .any(|candidate| candidate.candidate_type == "missing_test"
            && candidate.target_ids.contains(&id(ZERO_BRANCH))));
}

#[test]
fn scenario_preserves_accepted_input_facts_and_source_boundary() {
    let report = run_test_gap_detect(fixture()).expect("workflow should run");
    let scenario = report.scenario;

    assert_eq!(scenario.input_schema, INPUT_SCHEMA);
    assert_eq!(
        scenario.lifted_structure.space.id,
        id("space:test-gap:repo:higher-graphen:change:pricing-zero-discount")
    );
    assert!(!scenario.source_boundary.information_loss.is_empty());
    assert_eq!(
        scenario.source_boundary.coverage_dimensions,
        vec![higher_graphen_runtime::TestGapCoverageType::Branch]
    );
    assert!(scenario
        .lifted_structure
        .space
        .cell_ids
        .contains(&id(ZERO_BRANCH)));
    assert!(scenario
        .lifted_structure
        .cells
        .iter()
        .all(|cell| cell.provenance.review_status == ReviewStatus::Accepted));
}

#[test]
fn result_and_projection_keep_detector_inference_unreviewed() {
    let report = run_test_gap_detect(fixture()).expect("workflow should run");

    assert!(report
        .result
        .accepted_fact_ids
        .contains(&id("test:pricing:normal-discount")));
    assert!(report
        .result
        .obstructions
        .iter()
        .all(|obstruction| obstruction.review_status == ReviewStatus::Unreviewed));
    assert!(report
        .result
        .completion_candidates
        .iter()
        .all(|candidate| candidate.review_status == ReviewStatus::Unreviewed));
    assert!(!report.projection.source_ids.is_empty());
    assert!(!report.projection.information_loss.is_empty());
    assert!(!report.projection.human_review.information_loss.is_empty());
    assert!(!report.projection.ai_view.information_loss.is_empty());
    assert!(!report.projection.audit_trace.information_loss.is_empty());
}

#[test]
fn report_serializes_lower_snake_case_values_and_round_trips() {
    let report = run_test_gap_detect(fixture()).expect("workflow should run");
    let value = serde_json::to_value(&report).expect("serialize report");

    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["report_type"], json!(REPORT_TYPE));
    assert_eq!(value["result"]["status"], json!("gaps_detected"));
    assert_eq!(
        value["result"]["completion_candidates"][0]["candidate_type"],
        json!("missing_test")
    );
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["scenario"]["lifted_structure"]["cells"][0]["provenance"]["review_status"],
        json!("accepted")
    );
    assert_eq!(value["projection"]["purpose"], json!("test_gap_detection"));

    let json_text = serde_json::to_string(&report).expect("serialize report text");
    let round_tripped: Value = serde_json::from_str(&json_text).expect("parse report json");
    assert_eq!(round_tripped["schema"], json!(REPORT_SCHEMA));
}

fn fixture() -> TestGapInputDocument {
    serde_json::from_str(include_str!(
        "../../../schemas/inputs/test-gap.input.example.json"
    ))
    .expect("fixture should parse")
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
