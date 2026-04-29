//! Contract tests for the missing unit test detector runtime workflow.

use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_runtime::{
    run_test_gap_detect, TestGapDetectorContext, TestGapHigherOrderCell, TestGapInputDocument,
    TestGapInputLaw, TestGapInputMorphism, TestGapInputTest, TestGapObstructionType, TestGapStatus,
    TestGapTestType, TestGapVerificationCell,
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
fn detector_context_allows_integration_tests_as_verification_policy() {
    let mut input = fixture();
    input.tests.push(TestGapInputTest {
        id: id("test:pricing:zero-discount-integration"),
        name: "pricing zero discount integration".to_owned(),
        test_type: TestGapTestType::Integration,
        file_id: None,
        target_ids: vec![id("function:pricing:calculate_discount")],
        branch_ids: vec![id(ZERO_BRANCH)],
        requirement_ids: vec![id("requirement:pricing:zero-discount-regression")],
        is_regression: true,
        context_ids: Vec::new(),
        source_ids: vec![id("evidence:pricing:test-metadata")],
    });
    let prior_context = input.detector_context.take().unwrap_or_default();
    input.detector_context = Some(TestGapDetectorContext {
        test_kinds: vec![TestGapTestType::Unit, TestGapTestType::Integration],
        ..prior_context
    });

    let report = run_test_gap_detect(input).expect("workflow should run");

    assert_eq!(report.result.status, TestGapStatus::NoGapsInSnapshot);
    assert!(report.result.obstructions.is_empty());
    assert!(report.result.completion_candidates.is_empty());
}

#[test]
fn semantic_delta_without_verification_is_counterexample_and_ai_record() {
    let mut input = fixture();
    add_semantic_delta(&mut input, false);

    let report = run_test_gap_detect(input).expect("workflow should run");

    assert!(report.result.counterexamples.iter().any(|counterexample| {
        counterexample
            .morphism_ids
            .contains(&id("morphism:test-gap:semantic-preservation:pricing"))
    }));
    assert!(report.projection.ai_view.records.iter().any(|record| {
        record.id
            == id(
                "counterexample:test-gap:morphism:morphism-test-gap-semantic-preservation-pricing",
            )
            && record.review_status == Some(ReviewStatus::Unreviewed)
    }));
    assert!(report
        .scenario
        .source_boundary
        .information_loss
        .iter()
        .any(|loss| loss.contains("typed MIR-level equivalence")));
}

#[test]
fn semantic_delta_with_verification_emits_proof_object() {
    let mut input = fixture();
    add_semantic_delta(&mut input, true);

    let report = run_test_gap_detect(input).expect("workflow should run");

    assert!(report.result.proof_objects.iter().any(|proof| {
        proof
            .morphism_ids
            .contains(&id("morphism:test-gap:semantic-preservation:pricing"))
            && proof
                .verified_by_ids
                .contains(&id("verification:test-gap:semantic-pricing"))
    }));
    assert!(!report.result.counterexamples.iter().any(|counterexample| {
        counterexample
            .morphism_ids
            .contains(&id("morphism:test-gap:semantic-preservation:pricing"))
    }));
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

fn add_semantic_delta(input: &mut TestGapInputDocument, verified: bool) {
    input.higher_order_cells.push(TestGapHigherOrderCell {
        id: id("semantic:rust:function:pricing:base:calculate-discount"),
        cell_type: "rust_function".to_owned(),
        label: "Rust function calculate_discount at base".to_owned(),
        dimension: 0,
        context_ids: Vec::new(),
        source_ids: Vec::new(),
        confidence: None,
    });
    input.higher_order_cells.push(TestGapHigherOrderCell {
        id: id("semantic:rust:function:pricing:head:calculate-discount"),
        cell_type: "rust_function".to_owned(),
        label: "Rust function calculate_discount at head".to_owned(),
        dimension: 0,
        context_ids: Vec::new(),
        source_ids: Vec::new(),
        confidence: None,
    });
    input.laws.push(TestGapInputLaw {
        id: id("law:test-gap:semantic-delta-has-verification"),
        summary: "changed semantic delta morphisms require accepted verification cells".to_owned(),
        applies_to_ids: vec![id("morphism:test-gap:semantic-preservation:pricing")],
        requirement_ids: Vec::new(),
        source_ids: Vec::new(),
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: None,
    });
    input.morphisms.push(TestGapInputMorphism {
        id: id("morphism:test-gap:semantic-preservation:pricing"),
        morphism_type: "semantic_preservation".to_owned(),
        source_ids: vec![id("semantic:rust:function:pricing:base:calculate-discount")],
        target_ids: vec![id("semantic:rust:function:pricing:head:calculate-discount")],
        law_ids: vec![id("law:test-gap:semantic-delta-has-verification")],
        requirement_ids: Vec::new(),
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: None,
    });
    if verified {
        input.verification_cells.push(TestGapVerificationCell {
            id: id("verification:test-gap:semantic-pricing"),
            name: "Semantic pricing verification".to_owned(),
            verification_type: "automated_test".to_owned(),
            test_type: TestGapTestType::Unit,
            target_ids: Vec::new(),
            requirement_ids: Vec::new(),
            law_ids: vec![id("law:test-gap:semantic-delta-has-verification")],
            morphism_ids: vec![id("morphism:test-gap:semantic-preservation:pricing")],
            source_ids: Vec::new(),
            confidence: None,
        });
    }
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}
