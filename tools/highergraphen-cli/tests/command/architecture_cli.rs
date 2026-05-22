use crate::common::*;
use serde_json::{json, Value};
use std::fs;

#[test]
fn version_command_reports_package_version() {
    for args in [["version"], ["--version"], ["-V"]] {
        let output = run_cli(&args);

        assert!(output.status.success(), "stderr: {}", stderr(&output));
        assert_eq!(
            stdout(&output).trim_end(),
            format!("highergraphen {}", env!("CARGO_PKG_VERSION"))
        );
        assert!(stderr(&output).is_empty());
    }
}

#[test]
fn smoke_command_writes_one_json_report_to_stdout() {
    let output = run_cli(&[
        "architecture",
        "smoke",
        "direct-db-access",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);

    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(
        value["result"]["status"],
        json!("violation_detected"),
        "domain violations are successful reports"
    );
}

#[test]
fn smoke_command_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("architecture-direct-db-access-smoke.report.json");

    let output = run_cli(&[
        "architecture",
        "smoke",
        "direct-db-access",
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("temp path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON report file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(REPORT_SCHEMA));
    assert_eq!(value["metadata"]["cli_package"], json!("highergraphen-cli"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn covers_obligation_test_semantics_architecture_smoke_human_format_error() {
    let output = run_cli(&[
        "architecture",
        "smoke",
        "direct-db-access",
        "--format",
        "human",
    ]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("only json is supported"));
}

#[test]
fn input_lift_command_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = input_fixture();
    let output = run_cli(&[
        "architecture",
        "input",
        "lift",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);

    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(INPUT_LIFT_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("lifted"));
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert!(!value["result"]["accepted_fact_ids"]
        .as_array()
        .expect("accepted facts")
        .contains(&json!("cell:billing-status-api")));
}

#[test]
fn input_lift_command_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("architecture-input-lift.report.json");
    let fixture = input_fixture();

    let output = run_cli(&[
        "architecture",
        "input",
        "lift",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("temp path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON report file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(INPUT_LIFT_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen architecture input lift")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn input_lift_command_reads_resolved_billing_boundary_fixture() {
    let fixture = resolved_input_fixture();
    let output = run_cli(&[
        "architecture",
        "input",
        "lift",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(INPUT_LIFT_REPORT_SCHEMA));
    assert!(value["result"]["accepted_fact_ids"]
        .as_array()
        .expect("accepted facts")
        .contains(&json!(BILLING_STATUS_API_CELL)));
    assert!(value["result"]["accepted_fact_ids"]
        .as_array()
        .expect("accepted facts")
        .contains(&json!("incidence:order-service-calls-billing-status-api")));
    assert!(value["result"]["accepted_fact_ids"]
        .as_array()
        .expect("accepted facts")
        .contains(&json!("incidence:billing-service-owns-billing-status-api")));
    assert!(!value["result"]["accepted_fact_ids"]
        .as_array()
        .expect("accepted facts")
        .contains(&json!("incidence:order-service-reads-billing-db")));
    assert_eq!(value["result"]["completion_candidates"], json!([]));
}

#[test]
fn feed_reader_run_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = feed_fixture();
    let output = run_cli(&[
        "feed",
        "reader",
        "run",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);

    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(FEED_READER_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("obstructions_detected"));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen feed reader run")
    );
}

#[test]
fn feed_reader_run_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("feed-reader.report.json");
    let fixture = feed_fixture();

    let output = run_cli(&[
        "feed",
        "reader",
        "run",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("temp path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON report file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(FEED_READER_REPORT_SCHEMA));
    assert_eq!(value["projection"]["timeline"]["audience"], json!("human"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn overlap_candidates_reads_bounded_input_and_emits_candidate_cells() {
    let fixture = correspondence_detection_fixture();
    let fixture_value: Value =
        serde_json::from_str(&fs::read_to_string(&fixture).expect("read fixture"))
            .expect("fixture should be JSON");
    assert_eq!(
        fixture_value["schema"],
        json!(CORRESPONDENCE_DETECTION_INPUT_SCHEMA)
    );

    let output = run_cli(&[
        "overlap",
        "candidates",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["context"], json!("ctx:architecture-review"));
    assert_eq!(value["scope"], json!("all"));
    assert_eq!(value["candidates"][0]["reviewStatus"], json!("candidate"));
    assert_eq!(
        value["candidates"][0]["correspondenceKind"],
        json!("StructuralOverlap")
    );
    assert!(value["candidates"][0]["overlapWitnesses"]
        .as_array()
        .expect("overlap witnesses")
        .iter()
        .any(|witness| witness["witnessKind"] == json!("EvidenceSet")));
    assert!(value["candidates"][0]["differenceWitnesses"]
        .as_array()
        .expect("difference witnesses")
        .iter()
        .any(
            |difference| difference["differenceKind"] == json!("ModalityMismatch")
                && difference["severity"] == json!("blocking")
        ));
    assert!(value["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .any(
            |candidate| candidate["correspondenceKind"] == json!("SemanticOverlap")
                && candidate["reviewStatus"] == json!("candidate")
                && candidate["confidence"] == json!(0.86)
        ));
}

#[test]
fn correspondence_validate_reads_cell_fixture_and_emits_report() {
    let fixture = correspondence_cell_fixture();
    let output = run_cli(&[
        "correspondence",
        "validate",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(
        value["correspondenceId"],
        json!("corr:order-service-billing-db-access")
    );
    let findings_empty = match value.get("findings") {
        Some(findings) => findings.as_array().is_some_and(Vec::is_empty),
        None => true,
    };
    assert!(findings_empty, "fixture should satisfy Phase 1 invariants");
}

#[test]
fn overlap_explain_reads_cell_fixture_and_emits_structured_projection() {
    let fixture = correspondence_cell_fixture();
    let output = run_cli(&[
        "overlap",
        "explain",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(CORRESPONDENCE_EXPLANATION_SCHEMA));
    assert_eq!(
        value["correspondenceId"],
        json!("corr:order-service-billing-db-access")
    );
    assert_eq!(value["reviewStatus"], json!("candidate"));
    assert_eq!(
        value["gluing"]["obstruction"],
        json!("obstruction:direct-db-access-violates-boundary")
    );
    assert_eq!(
        value["overlapWitnesses"][0]["id"],
        json!("witness:shared-order-billing-access")
    );
    assert_eq!(
        value["differenceWitnesses"][0]["id"],
        json!("diff:observed-vs-forbidden")
    );
    assert!(value["projectionLoss"]
        .as_object()
        .expect("projection loss object")
        .is_empty());
}

#[test]
fn correspondence_project_emits_human_markdown_projection() {
    let fixture = correspondence_cell_fixture();
    let output = run_cli(&[
        "correspondence",
        "project",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--audience",
        "human-reviewer",
        "--format",
        "markdown",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let text = stdout(&output);
    assert!(text.contains("# Correspondence corr:order-service-billing-db-access"));
    assert!(text.contains("- Audience: human"));
    assert!(text.contains("- Review status: candidate"));
    assert!(text.contains("witness:shared-order-billing-access"));
    assert!(text.contains("diff:observed-vs-forbidden"));
    assert!(text.contains("- Result: failure"));
    assert!(text.contains("- Obstruction: obstruction:direct-db-access-violates-boundary"));
    assert!(text.contains("## Projection Loss"));
}

#[test]
fn correspondence_project_emits_ai_agent_json_projection() {
    let fixture = correspondence_cell_fixture();
    let output = run_cli(&[
        "correspondence",
        "project",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--audience",
        "ai-agent",
        "--purpose",
        "review",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(CORRESPONDENCE_PROJECTION_SCHEMA));
    assert_eq!(value["audience"], json!("ai_agent"));
    assert_eq!(value["purpose"], json!("review"));
    assert_eq!(value["renderer"], json!("structured"));
    assert_eq!(value["reviewStatus"], json!("candidate"));
    assert_eq!(value["projectionLoss"], json!({}));
}

#[test]
fn correspondence_review_accepts_semantic_candidate_without_silent_promotion() {
    let fixture = correspondence_detection_fixture();
    let candidates_output = run_cli(&[
        "overlap",
        "candidates",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);
    assert!(
        candidates_output.status.success(),
        "stderr: {}",
        stderr(&candidates_output)
    );
    let candidates_value: Value =
        serde_json::from_str(stdout(&candidates_output).trim_end()).expect("stdout JSON");
    let semantic_candidate = candidates_value["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .find(|candidate| candidate["correspondenceKind"] == json!("SemanticOverlap"))
        .expect("semantic candidate")
        .clone();

    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let candidate_path = directory.join("semantic-candidate.json");
    fs::write(
        &candidate_path,
        serde_json::to_string(&semantic_candidate).expect("candidate JSON"),
    )
    .expect("write candidate");

    let output = run_cli(&[
        "correspondence",
        "review",
        "accept",
        "--input",
        candidate_path.to_str().expect("temp path should be utf-8"),
        "--candidate",
        semantic_candidate["id"].as_str().expect("candidate id"),
        "--reviewer",
        "reviewer:human",
        "--reason",
        "semantic normalized claim reviewed",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["correspondenceKind"], json!("SemanticOverlap"));
    assert_eq!(value["reviewStatus"], json!("accepted"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn correspondence_end_to_end_semantic_review_project_and_gluing_flow() {
    let fixture = correspondence_detection_fixture();
    let candidates_output = run_cli(&[
        "overlap",
        "candidates",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);
    assert!(
        candidates_output.status.success(),
        "stderr: {}",
        stderr(&candidates_output)
    );
    let candidates_value: Value =
        serde_json::from_str(stdout(&candidates_output).trim_end()).expect("stdout JSON");
    let semantic_candidate = candidates_value["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .find(|candidate| candidate["correspondenceKind"] == json!("SemanticOverlap"))
        .expect("semantic candidate")
        .clone();

    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let candidate_path = directory.join("semantic-candidate.json");
    let accepted_path = directory.join("semantic-accepted.json");
    fs::write(
        &candidate_path,
        serde_json::to_string(&semantic_candidate).expect("candidate JSON"),
    )
    .expect("write candidate");

    let review_output = run_cli(&[
        "correspondence",
        "review",
        "accept",
        "--input",
        candidate_path.to_str().expect("temp path should be utf-8"),
        "--candidate",
        semantic_candidate["id"].as_str().expect("candidate id"),
        "--reviewer",
        "reviewer:human",
        "--reason",
        "semantic normalized claim reviewed",
        "--format",
        "json",
        "--output",
        accepted_path.to_str().expect("temp path should be utf-8"),
    ]);
    assert!(
        review_output.status.success(),
        "stderr: {}",
        stderr(&review_output)
    );
    assert!(stdout(&review_output).is_empty());

    let project_output = run_cli(&[
        "correspondence",
        "project",
        "--input",
        accepted_path.to_str().expect("temp path should be utf-8"),
        "--audience",
        "human-reviewer",
        "--format",
        "markdown",
    ]);
    assert!(
        project_output.status.success(),
        "stderr: {}",
        stderr(&project_output)
    );
    let markdown = stdout(&project_output);
    assert!(markdown.contains("- Review status: accepted"));
    assert!(markdown.contains("witness:semantic:normalized-claim"));
    assert!(markdown.contains("## Projection Loss"));

    let gluing_output = run_cli(&[
        "gluing",
        "check",
        "--input",
        accepted_path.to_str().expect("temp path should be utf-8"),
        "--format",
        "json",
    ]);
    assert!(
        gluing_output.status.success(),
        "stderr: {}",
        stderr(&gluing_output)
    );
    let gluing: Value =
        serde_json::from_str(stdout(&gluing_output).trim_end()).expect("stdout should be JSON");
    assert_eq!(gluing["result"]["kind"], json!("failure"));
    assert_eq!(
        gluing["result"]["obstruction"],
        json!("obstruction:gluing:corr-semantic-semantic-signal-order-service-billing-db-access:blocking-difference")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn gluing_check_reads_cell_fixture_and_reports_failure() {
    let fixture = correspondence_cell_fixture();
    let output = run_cli(&[
        "gluing",
        "check",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(
        value["id"],
        json!("glue:check:corr-order-service-billing-db-access")
    );
    assert_eq!(value["result"]["kind"], json!("failure"));
    assert_eq!(
        value["result"]["obstruction"],
        json!("obstruction:direct-db-access-violates-boundary")
    );
    assert_eq!(
        value["invariantChecks"][0]["invariant"],
        json!("invariant:no-cross-context-db-access")
    );
    assert_eq!(value["invariantChecks"][0]["result"], json!("failed"));
    assert_eq!(
        value["differenceWitnesses"][0],
        json!("diff:observed-vs-forbidden")
    );
}

#[test]
fn ddd_input_from_case_space_emits_lift_contract() {
    let fixture = ddd_case_space_fixture();
    let output = run_cli(&[
        "ddd",
        "input",
        "from-case-space",
        "--case-space",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(DDD_REVIEW_INPUT_SCHEMA));
    assert_eq!(
        value["source_boundary"]["id"],
        json!("source_boundary:ddd-sales-billing-demo")
    );
    assert_eq!(
        value["lift_morphism"]["source_boundary_id"],
        value["source_boundary"]["id"]
    );
    assert_eq!(
        value["operation_gate"]["source_boundary_id"],
        value["source_boundary"]["id"]
    );
    assert!(value["inferred_claims"]
        .as_array()
        .expect("inferred claims")
        .iter()
        .any(|claim| claim["id"] == json!("semantic_case:customer-identity-loss")));
}

#[test]
fn ddd_review_reads_fixture_and_reports_operation_gate_effects() {
    let fixture = ddd_review_input_fixture();
    let output = run_cli(&[
        "ddd",
        "review",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(DDD_REVIEW_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("issues_detected"));
    assert_eq!(value["result"]["closeability"]["closeable"], json!(false));
    assert!(value["result"]["interpretation_mapping_ids"]
        .as_array()
        .expect("interpretation mappings")
        .contains(&json!("mapping:ddd-bounded-context-to-context-cell")));
    assert!(value["result"]["completion_morphisms"]
        .as_array()
        .expect("completion morphisms")
        .contains(&json!({
            "id": "morphism:complete-missing-sales-billing-acl",
            "morphism_type": "completion_candidate_to_casegraphen_patch",
            "completion_candidate_id": "completion:missing-sales-billing-acl",
            "source_ids": ["semantic_case:customer-identity-loss", "evidence:workshop-notes"],
            "target_ids": ["context:sales", "context:billing", "decision:unified-customer-model"],
            "operation": {
                "op": "upsert_ontology_record",
                "record_kind": "transformation",
                "review_required": true
            },
            "review_status": "unreviewed"
        })));
    assert!(value["projection"]["audit_trace"]["represented_ids"]
        .as_array()
        .expect("audit represented ids")
        .contains(&json!("morphism:lift-ddd-sales-billing-demo")));
}

#[test]
fn ddd_review_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("ddd-review.report.json");
    let fixture = ddd_review_input_fixture();

    let output = run_cli(&[
        "ddd",
        "review",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("temp path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON report file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(DDD_REVIEW_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen ddd review")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn pr_review_targets_recommend_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = pr_review_target_fixture();
    let output = run_cli(&[
        "pr-review",
        "targets",
        "recommend",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);

    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(PR_REVIEW_TARGET_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen pr-review targets recommend")
    );
    assert_eq!(value["result"]["status"], json!("targets_recommended"));
    assert_eq!(
        value["result"]["review_targets"][0]["review_status"],
        json!("unreviewed")
    );
}

#[test]
fn pr_review_targets_recommend_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("pr-review-target.report.json");
    let fixture = pr_review_target_fixture();

    let output = run_cli(&[
        "pr-review",
        "targets",
        "recommend",
        "--input",
        fixture.to_str().expect("fixture path should be utf-8"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("temp path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON report file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(PR_REVIEW_TARGET_REPORT_SCHEMA));
    assert_eq!(value["projection"]["purpose"], json!("pr_review_targeting"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}
