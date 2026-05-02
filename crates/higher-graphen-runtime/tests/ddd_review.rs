//! Contract tests for the bounded DDD review runtime workflow.

use higher_graphen_runtime::{ddd_input_from_case_space, run_ddd_review};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

const INPUT_SCHEMA: &str = "highergraphen.ddd_review.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.ddd_review.report.v1";

#[test]
fn case_space_adapter_emits_lift_morphism_and_operation_gate() {
    let case_space = read_json(ddd_case_space_fixture());
    let document = ddd_input_from_case_space(case_space, "fixture.case.space.json")
        .expect("adapter should emit DDD input");

    assert_eq!(document["schema"], json!(INPUT_SCHEMA));
    assert_eq!(
        document["source_boundary"]["id"],
        json!("source_boundary:ddd-sales-billing-demo")
    );
    assert_eq!(
        document["lift_morphism"]["source_boundary_id"],
        document["source_boundary"]["id"]
    );
    assert_eq!(
        document["operation_gate"]["source_boundary_id"],
        document["source_boundary"]["id"]
    );
    assert!(document["inferred_claims"]
        .as_array()
        .expect("inferred claims")
        .iter()
        .any(|claim| claim["id"] == json!("semantic_case:customer-identity-loss")));
    assert!(document["accepted_facts"]
        .as_array()
        .expect("accepted facts")
        .iter()
        .any(
            |record| record["id"] == json!("relation:decision-requires-equivalence-proof")
                && record["record_type"] == json!("relation")
        ));
}

#[test]
fn review_report_preserves_interpretation_mappings_and_closeability_blockers() {
    let input = read_json(ddd_review_input_fixture());
    let report = run_ddd_review(input).expect("review should run");

    assert_eq!(report["schema"], json!(REPORT_SCHEMA));
    assert_eq!(
        report["metadata"]["command"],
        json!("highergraphen ddd review")
    );
    assert_eq!(report["result"]["status"], json!("issues_detected"));
    assert_eq!(report["result"]["closeability"]["closeable"], json!(false));
    assert!(report["result"]["interpretation_mapping_ids"]
        .as_array()
        .expect("interpretation mappings")
        .contains(&json!("mapping:ddd-bounded-context-to-context-cell")));
    assert!(report["projection"]["audit_trace"]["represented_ids"]
        .as_array()
        .expect("audit trace represented ids")
        .contains(&json!("morphism:lift-ddd-sales-billing-demo")));
    assert!(report["result"]["completion_morphisms"]
        .as_array()
        .expect("completion morphisms")
        .iter()
        .any(|morphism| morphism["id"] == json!("morphism:complete-missing-sales-billing-acl")));
}

#[test]
fn review_rejects_mismatched_source_boundary_gate() {
    let mut input = read_json(ddd_review_input_fixture());
    input["operation_gate"]["source_boundary_id"] = json!("source_boundary:other");

    let error = run_ddd_review(input).expect_err("mismatched boundary should fail");
    assert!(error
        .to_string()
        .contains("source_boundary.id must match lift_morphism.source_boundary_id"));
}

#[test]
fn case_space_review_detects_language_conflict_and_missing_mapping() {
    let case_space = read_json(ddd_case_space_fixture());
    let document = ddd_input_from_case_space(case_space, "fixture.case.space.json")
        .expect("adapter should emit DDD input");
    let report = run_ddd_review(document).expect("review should run");

    let obstruction_ids: Vec<_> = report["result"]["obstructions"]
        .as_array()
        .expect("obstructions")
        .iter()
        .filter_map(|obstruction| obstruction["id"].as_str())
        .collect();
    assert!(obstruction_ids.contains(&"obstruction:ddd-cross-context-language-conflict"));
    assert!(obstruction_ids.contains(&"obstruction:ddd-boundary-mapping-missing"));
    assert!(obstruction_ids.contains(&"obstruction:ddd-inferred-boundary-risk-unaccepted"));
}

fn read_json(path: PathBuf) -> Value {
    let text = fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&text).expect("fixture should be JSON")
}

fn ddd_review_input_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/ddd-review.input.example.json")
}

fn ddd_case_space_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json")
}
