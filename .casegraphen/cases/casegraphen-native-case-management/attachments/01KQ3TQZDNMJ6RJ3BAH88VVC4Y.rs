#![allow(missing_docs)]

use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn validate_command_emits_report_for_graph_fixture() {
    let output = run_cli(&[
        "validate",
        "--input",
        graph_fixture().to_str().expect("fixture path"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value = stdout_json(&output);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.validate.report.v1")
    );
    assert_eq!(value["result"]["valid"], json!(true));
    assert_eq!(
        value["metadata"]["tool_package"],
        json!("tools/casegraphen")
    );
}

#[test]
fn coverage_and_missing_are_successful_domain_reports() {
    let output = run_cli(&[
        "coverage",
        "--input",
        graph_fixture().to_str().expect("fixture path"),
        "--coverage",
        policy_fixture().to_str().expect("policy path"),
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));

    let coverage = stdout_json(&output);
    assert_eq!(coverage["result"]["coverage_status"], json!("partial"));
    assert_eq!(
        coverage["result"]["goals"][0]["uncovered_ids"],
        json!(["context:billing"])
    );

    let missing = run_cli(&[
        "missing",
        "--input",
        graph_fixture().to_str().expect("fixture path"),
        "--coverage",
        policy_fixture().to_str().expect("policy path"),
        "--format",
        "json",
    ]);
    assert!(missing.status.success(), "stderr: {}", stderr(&missing));

    let value = stdout_json(&missing);
    assert_eq!(
        value["result"]["missing_cases"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["missing_cases"][0]["target_ids"],
        json!(["context:billing"])
    );
}

#[test]
fn project_preserves_missing_cases_conflicts_and_sources() {
    let output = run_cli(&[
        "project",
        "--input",
        graph_fixture().to_str().expect("fixture path"),
        "--projection",
        projection_fixture().to_str().expect("projection path"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value = stdout_json(&output);

    assert_eq!(
        value["schema"],
        json!("highergraphen.case.project.report.v1")
    );
    assert_eq!(
        value["projection"]["ai_view"]["missing_cases"][0]["target_ids"],
        json!(["context:billing"])
    );
    assert_eq!(
        value["projection"]["ai_view"]["conflicts"][0]["source_ids"],
        json!(["source:architecture-input"])
    );
    assert!(value["projection"]["audit_trace"]["source_ids"]
        .as_array()
        .expect("source ids")
        .contains(&json!("source:architecture-input")));
}

#[test]
fn workflow_reason_emits_reasoning_report_for_workflow_fixture() {
    let output = run_cli(&[
        "workflow",
        "reason",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value = stdout_json(&output);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.report.v1")
    );
    assert_eq!(value["report_type"], json!("case_workflow_reasoning"));
    assert_eq!(
        value["metadata"]["command"],
        json!("casegraphen workflow reason")
    );
    assert_eq!(
        value["metadata"]["tool_package"],
        json!("tools/casegraphen")
    );
    assert_eq!(value["result"]["status"], json!("obstructions_detected"));
    assert_eq!(
        value["result"]["readiness"]["ready_item_ids"],
        json!(["task:define-workflow-reasoning-contract"])
    );
    assert_eq!(
        value["result"]["completion_candidates"][0]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["projection"]["ai_view"]["audience"],
        json!("ai_agent")
    );
}

#[test]
fn workflow_validate_reports_semantic_violations_as_json() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let bad_workflow_path = directory.join("bad.workflow.graph.json");
    let mut workflow = json_file(workflow_fixture());
    workflow["workflow_relations"][0]["from_id"] = json!("task:missing-work-item");
    fs::write(
        &bad_workflow_path,
        serde_json::to_string_pretty(&workflow).expect("serialize bad workflow"),
    )
    .expect("write bad workflow");

    let output = run_cli(&[
        "workflow",
        "validate",
        "--input",
        bad_workflow_path.to_str().expect("bad workflow path"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value = stdout_json(&output);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.validate.report.v1")
    );
    assert_eq!(value["report_type"], json!("case_workflow_validate"));
    assert_eq!(value["result"]["valid"], json!(false));
    assert!(value["result"]["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|violation| violation["code"] == json!("dangling_reference")));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn create_list_and_inspect_use_local_file_store() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");

    let create = run_cli(&[
        "create",
        "--case-graph-id",
        "case_graph:created",
        "--space-id",
        "space:created",
        "--store",
        directory.to_str().expect("temp path"),
        "--format",
        "json",
    ]);
    assert!(create.status.success(), "stderr: {}", stderr(&create));
    let created = stdout_json(&create);
    let graph_path = created["result"]["path"].as_str().expect("created path");

    let inspect = run_cli(&["inspect", "--input", graph_path, "--format", "json"]);
    assert!(inspect.status.success(), "stderr: {}", stderr(&inspect));
    assert_eq!(
        stdout_json(&inspect)["result"]["case_graph_id"],
        json!("case_graph:created")
    );

    let list = run_cli(&[
        "list",
        "--store",
        directory.to_str().expect("temp path"),
        "--format",
        "json",
    ]);
    assert!(list.status.success(), "stderr: {}", stderr(&list));
    assert_eq!(stdout_json(&list)["result"]["graph_count"], json!(1));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn workflow_reason_supports_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let output_path = directory.join("workflow.report.json");

    let output = run_cli(&[
        "workflow",
        "reason",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read report"))
            .expect("report JSON");
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.report.v1")
    );
    assert_eq!(
        value["input"]["workflow_graph_id"],
        json!("workflow_graph:casegraphen-rewrite-contract")
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn workflow_readiness_supports_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let output_path = directory.join("workflow.readiness.report.json");

    let output = run_cli(&[
        "workflow",
        "readiness",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read report"))
            .expect("report JSON");
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.readiness.report.v1")
    );
    assert_eq!(
        value["result"]["ready_item_ids"],
        json!(["task:define-workflow-reasoning-contract"])
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn focused_workflow_commands_emit_section_reports() {
    let readiness = run_cli(&[
        "workflow",
        "readiness",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--projection",
        projection_fixture().to_str().expect("projection path"),
        "--format",
        "json",
    ]);
    assert!(readiness.status.success(), "stderr: {}", stderr(&readiness));
    let value = stdout_json(&readiness);
    assert_eq!(value["report_type"], json!("case_workflow_readiness"));
    assert_eq!(
        value["input"]["projection"],
        json!(projection_fixture().display().to_string())
    );
    assert_eq!(
        value["projection"]["audit_trace"]["information_loss"],
        json!(["Focused report contains the requested section; use workflow reason for the aggregate projection."])
    );
    assert_eq!(
        value["result"]["not_ready_items"][0]["work_item_id"],
        json!("proof:workflow-schema-parse-check")
    );

    let obstructions = stdout_json(&successful_workflow_command("obstructions"));
    assert_eq!(
        obstructions["schema"],
        json!("highergraphen.case.workflow.obstructions.report.v1")
    );
    assert!(obstructions["result"]["obstructions"]
        .as_array()
        .expect("obstructions")
        .iter()
        .any(|record| record["obstruction_type"] == json!("missing_evidence")));

    let completions = stdout_json(&successful_workflow_command("completions"));
    assert!(completions["result"]["completion_candidates"]
        .as_array()
        .expect("completion candidates")
        .iter()
        .any(|record| record["candidate_type"] == json!("missing_proof")));

    let evidence = stdout_json(&successful_workflow_command("evidence"));
    assert_eq!(
        evidence["result"]["inference_record_ids"],
        json!(["evidence:workflow-gap-inference"])
    );

    let project = run_cli(&[
        "workflow",
        "project",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--projection",
        projection_fixture().to_str().expect("projection path"),
        "--format",
        "json",
    ]);
    assert!(project.status.success(), "stderr: {}", stderr(&project));
    assert_eq!(
        stdout_json(&project)["result"]["projection_profile_id"],
        json!("projection:workflow-ai-review")
    );

    let correspond = run_cli(&[
        "workflow",
        "correspond",
        "--left",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--right",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
    ]);
    assert!(
        correspond.status.success(),
        "stderr: {}",
        stderr(&correspond)
    );
    assert_eq!(
        stdout_json(&correspond)["result"]["combined_correspondence"][0]["correspondence_type"],
        json!("similar_with_loss")
    );

    let evolution = stdout_json(&successful_workflow_command("evolution"));
    assert_eq!(
        evolution["result"]["transition_ids"],
        json!(["transition:foundation-docs-to-workflow-contract"])
    );
}

#[test]
fn cg_bridge_workflow_workspace_commands_round_trip_store_history() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");

    let import = run_cli(&[
        "cg",
        "workflow",
        "import",
        "--store",
        directory.to_str().expect("temp path"),
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--revision-id",
        "revision:bridge-import",
        "--format",
        "json",
    ]);
    assert!(import.status.success(), "stderr: {}", stderr(&import));
    let imported = stdout_json(&import);
    assert_eq!(
        imported["schema"],
        json!("highergraphen.case.workflow.workspace_import.report.v1")
    );
    assert_eq!(
        imported["metadata"]["command"],
        json!("casegraphen cg workflow import")
    );
    assert_eq!(
        imported["result"]["current_revision_id"],
        json!("revision:bridge-import")
    );
    assert!(directory
        .join(
            imported["result"]["current_graph_path"]
                .as_str()
                .expect("current graph path")
        )
        .exists());

    let list = run_cli(&[
        "cg",
        "workflow",
        "list",
        "--store",
        directory.to_str().expect("temp path"),
        "--format",
        "json",
    ]);
    assert!(list.status.success(), "stderr: {}", stderr(&list));
    assert_eq!(
        stdout_json(&list)["result"]["workflow_graph_count"],
        json!(1)
    );

    let inspect = run_bridge_store_command(&directory, "inspect");
    assert_eq!(
        stdout_json(&inspect)["result"]["history_entry_count"],
        json!(1)
    );

    let history = run_bridge_store_command(&directory, "history");
    let history_json = stdout_json(&history);
    assert_eq!(
        history_json["result"]["entries"][0]["event_type"],
        json!("imported")
    );

    let replay = run_bridge_store_command(&directory, "replay");
    assert_eq!(
        stdout_json(&replay)["result"]["graph"]["workflow_graph_id"],
        json!("workflow_graph:casegraphen-rewrite-contract")
    );

    let output_path = directory.join("bridge.validate.report.json");
    let validate = run_cli(&[
        "cg",
        "workflow",
        "validate",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path"),
    ]);
    assert!(validate.status.success(), "stderr: {}", stderr(&validate));
    assert!(stdout(&validate).is_empty());
    let validation = json_file(output_path);
    assert_eq!(validation["result"]["valid"], json!(true));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_readiness_supports_file_and_stored_workflow_graphs() {
    let file_based = run_cli(&[
        "cg",
        "workflow",
        "readiness",
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
    ]);
    assert!(
        file_based.status.success(),
        "stderr: {}",
        stderr(&file_based)
    );
    let file_json = stdout_json(&file_based);
    assert_eq!(
        file_json["metadata"]["command"],
        json!("casegraphen cg workflow readiness")
    );
    assert_eq!(file_json["input"]["source"], json!("file"));
    assert_eq!(
        file_json["projection"]["audit_trace"]["information_loss"],
        json!(["Focused report contains the requested section; use workflow reason for the aggregate projection."])
    );
    assert_eq!(
        file_json["result"]["ready_item_ids"],
        json!(["task:define-workflow-reasoning-contract"])
    );

    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);
    let stored = run_cli(&[
        "cg",
        "workflow",
        "readiness",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--format",
        "json",
    ]);
    assert!(stored.status.success(), "stderr: {}", stderr(&stored));
    let stored_json = stdout_json(&stored);
    assert_eq!(stored_json["input"]["source"], json!("workspace_store"));
    assert_eq!(
        stored_json["projection"]["audit_trace"]["information_loss"],
        json!(["Focused report contains the requested section; use workflow reason for the aggregate projection."])
    );
    assert_eq!(
        stored_json["result"]["not_ready_items"][0]["work_item_id"],
        json!("proof:workflow-schema-parse-check")
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_completion_accept_records_review_without_promoting_inference() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);

    let output = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_evidence_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Reviewed the proposed evidence gap",
        "--revision-id",
        "revision:completion-accept",
        "--evidence-id",
        "evidence:workflow-target-doc",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value = stdout_json(&output);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.completion_accept.report.v1")
    );
    assert_eq!(
        value["result"]["candidate_before_review"]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["candidate_after_review"]["review_status"],
        json!("accepted")
    );
    assert_eq!(
        value["result"]["review_record"]["evidence_ids"],
        json!(["evidence:workflow-target-doc"])
    );
    assert_eq!(
        value["result"]["transition_record"]["transition_type"],
        json!("review_transition")
    );

    let replay = run_bridge_store_command(&directory, "replay");
    let graph = stdout_json(&replay)["result"]["graph"].clone();
    assert_eq!(
        graph["completion_reviews"][0]["candidate_snapshot"]["review_status"],
        json!("unreviewed")
    );
    assert!(!graph["evidence_records"]
        .as_array()
        .expect("evidence records")
        .iter()
        .any(|record| record["id"] == json!("evidence:json-parse-check-output")));

    let readiness = run_bridge_store_command(&directory, "readiness");
    let readiness_json = stdout_json(&readiness);
    assert!(readiness_json["result"]["not_ready_items"]
        .as_array()
        .expect("not ready items")
        .iter()
        .any(|item| item["obstruction_ids"]
            .as_array()
            .expect("obstruction ids")
            .contains(&json!(
                "obstruction:missing-evidence:proof-workflow-schema-parse-check:evidence-json-parse-check-output"
            ))));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_completion_reject_supports_output_file_and_invalid_target_errors() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);
    let output_path = directory.join("completion.reject.report.json");

    let reject = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "reject",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_proof_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Duplicate of existing proof task",
        "--revision-id",
        "revision:completion-reject",
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path"),
    ]);

    assert!(reject.status.success(), "stderr: {}", stderr(&reject));
    assert!(stdout(&reject).is_empty());
    let value = json_file(output_path);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.completion_reject.report.v1")
    );
    assert_eq!(
        value["result"]["candidate_after_review"]["review_status"],
        json!("rejected")
    );
    assert_eq!(
        value["result"]["review_record"]["outcome_review_status"],
        json!("rejected")
    );

    let invalid = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        "candidate:does-not-exist",
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Invalid target smoke",
        "--revision-id",
        "revision:completion-invalid",
        "--format",
        "json",
    ]);

    assert!(!invalid.status.success());
    assert!(stdout(&invalid).is_empty());
    assert!(stderr(&invalid).contains("unknown completion candidate candidate:does-not-exist"));

    let invalid_evidence = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_proof_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Invalid linked evidence smoke",
        "--revision-id",
        "revision:completion-invalid-evidence",
        "--evidence-id",
        "evidence:does-not-exist",
        "--format",
        "json",
    ]);

    assert!(!invalid_evidence.status.success());
    assert!(stdout(&invalid_evidence).is_empty());
    assert!(stderr(&invalid_evidence)
        .contains("unknown linked evidence record evidence:does-not-exist"));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_completion_reopen_restores_unreviewed_candidate_state() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);

    let accept = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_evidence_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Reviewed the proposed evidence gap",
        "--revision-id",
        "revision:completion-accept",
        "--evidence-id",
        "evidence:workflow-target-doc",
        "--format",
        "json",
    ]);
    assert!(accept.status.success(), "stderr: {}", stderr(&accept));

    let reopen = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "reopen",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_evidence_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Reopen after missing implementation evidence",
        "--revision-id",
        "revision:completion-reopen",
        "--format",
        "json",
    ]);

    assert!(reopen.status.success(), "stderr: {}", stderr(&reopen));
    let value = stdout_json(&reopen);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.completion_reopen.report.v1")
    );
    assert_eq!(value["result"]["action"], json!("reopen"));
    assert_eq!(
        value["result"]["candidate_before_review"]["review_status"],
        json!("accepted")
    );
    assert_eq!(
        value["result"]["candidate_after_review"]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["review_record"]["outcome_review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["workspace_record"]["history_entry_count"],
        json!(3)
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_completion_patch_check_and_apply_flow() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);

    let accept = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_task_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Task candidate is a valid patch source",
        "--revision-id",
        "revision:patch-source-accepted",
        "--format",
        "json",
    ]);
    assert!(accept.status.success(), "stderr: {}", stderr(&accept));

    let patch = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "patch",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_task_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Convert accepted candidate into a reviewable patch transition",
        "--revision-id",
        "revision:completion-patch",
        "--transition-id",
        "transition:patch:test-missing-task",
        "--format",
        "json",
    ]);
    assert!(patch.status.success(), "stderr: {}", stderr(&patch));
    let patch_json = stdout_json(&patch);
    assert_eq!(
        patch_json["schema"],
        json!("highergraphen.case.workflow.completion_patch.report.v1")
    );
    assert_eq!(patch_json["result"]["applied"], json!(false));
    assert_eq!(
        patch_json["result"]["transition_record"]["provenance"]["review_status"],
        json!("unreviewed")
    );

    let check = run_cli(&[
        "cg",
        "workflow",
        "patch",
        "check",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--transition-id",
        "transition:patch:test-missing-task",
        "--format",
        "json",
    ]);
    assert!(check.status.success(), "stderr: {}", stderr(&check));
    let check_json = stdout_json(&check);
    assert_eq!(check_json["result"]["valid"], json!(true));
    assert_eq!(check_json["result"]["applicable"], json!(true));

    let apply = run_cli(&[
        "cg",
        "workflow",
        "patch",
        "apply",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--transition-id",
        "transition:patch:test-missing-task",
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Apply reviewed patch transition",
        "--revision-id",
        "revision:patch-applied",
        "--format",
        "json",
    ]);
    assert!(apply.status.success(), "stderr: {}", stderr(&apply));
    let apply_json = stdout_json(&apply);
    assert_eq!(
        apply_json["schema"],
        json!("highergraphen.case.workflow.patch_apply.report.v1")
    );
    assert_eq!(
        apply_json["result"]["transition_after_review"]["provenance"]["review_status"],
        json!("accepted")
    );
    assert_eq!(apply_json["result"]["materialized_record_count"], json!(0));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn cg_bridge_patch_reject_records_review_without_materializing_patch() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_bridge_workflow(&directory);

    let accept = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "accept",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_task_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Task candidate is a valid patch source",
        "--revision-id",
        "revision:patch-source-accepted",
        "--format",
        "json",
    ]);
    assert!(accept.status.success(), "stderr: {}", stderr(&accept));

    let patch = run_cli(&[
        "cg",
        "workflow",
        "completion",
        "patch",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--candidate-id",
        missing_task_candidate_id(),
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Convert accepted candidate into a reviewable patch transition",
        "--revision-id",
        "revision:completion-patch",
        "--transition-id",
        "transition:patch:test-rejected-missing-task",
        "--format",
        "json",
    ]);
    assert!(patch.status.success(), "stderr: {}", stderr(&patch));

    let reject = run_cli(&[
        "cg",
        "workflow",
        "patch",
        "reject",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--transition-id",
        "transition:patch:test-rejected-missing-task",
        "--reviewer-id",
        "reviewer:workflow-lead",
        "--reason",
        "Reject patch until source proof is attached",
        "--revision-id",
        "revision:patch-rejected",
        "--format",
        "json",
    ]);

    assert!(reject.status.success(), "stderr: {}", stderr(&reject));
    let value = stdout_json(&reject);
    assert_eq!(
        value["schema"],
        json!("highergraphen.case.workflow.patch_reject.report.v1")
    );
    assert_eq!(value["result"]["action"], json!("reject"));
    assert_eq!(value["result"]["materialized_record_count"], json!(0));
    assert_eq!(
        value["result"]["transition_before_review"]["provenance"]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["transition_after_review"]["provenance"]["review_status"],
        json!("rejected")
    );

    let check = run_cli(&[
        "cg",
        "workflow",
        "patch",
        "check",
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--transition-id",
        "transition:patch:test-rejected-missing-task",
        "--format",
        "json",
    ]);
    assert!(check.status.success(), "stderr: {}", stderr(&check));
    let check_json = stdout_json(&check);
    assert_eq!(check_json["result"]["valid"], json!(true));
    assert_eq!(check_json["result"]["applicable"], json!(false));
    assert_eq!(
        check_json["result"]["reason"],
        json!("Patch transition is rejected.")
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn native_case_commands_create_import_list_inspect_history_and_replay() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");

    let created = run_cli(&[
        "case",
        "new",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        "case_space:native-cli-created",
        "--space-id",
        "space:native-cli",
        "--title",
        "Native CLI created case",
        "--revision-id",
        "revision:native-cli-created",
        "--format",
        "json",
    ]);
    assert!(created.status.success(), "stderr: {}", stderr(&created));
    assert_eq!(
        stdout_json(&created)["result"]["record"]["case_space_id"],
        json!("case_space:native-cli-created")
    );

    let imported = import_native_case_space(&directory, "revision:native-cli-imported");
    assert_eq!(
        stdout_json(&imported)["metadata"]["command"],
        json!("casegraphen case import")
    );

    let list = run_cli(&[
        "case",
        "list",
        "--store",
        directory.to_str().expect("temp path"),
        "--format",
        "json",
    ]);
    assert!(list.status.success(), "stderr: {}", stderr(&list));
    assert_eq!(
        stdout_json(&list)["result"]["case_spaces"]
            .as_array()
            .expect("case spaces")
            .len(),
        2
    );

    let inspect = run_native_case_store_command(&directory, "inspect");
    assert_eq!(
        stdout_json(&inspect)["result"]["record"]["current_revision_id"],
        json!("revision:native-cli-imported")
    );

    let history = run_native_case_store_command(&directory, "history");
    assert_eq!(
        stdout_json(&history)["result"]["entries"][0]["entry_id"],
        json!("morphism_log_entry:genesis-native-contract")
    );

    let replay = run_native_case_store_command(&directory, "replay");
    assert_eq!(
        stdout_json(&replay)["result"]["replay"]["case_space"]["case_space_id"],
        json!("case_space:native-case-management-contract")
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn native_reasoning_commands_emit_domain_reports_and_output_file() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_native_case_space(&directory, "revision:native-cli-imported");

    let reason = run_native_case_store_command(&directory, "reason");
    assert!(reason.status.success(), "stderr: {}", stderr(&reason));
    let reason_json = stdout_json(&reason);
    assert_eq!(
        reason_json["result"]["evaluation"]["status"],
        json!("review_required")
    );
    assert!(reason_json["result"]["evaluation"]["completion_candidates"]
        .as_array()
        .expect("completion candidates")
        .is_empty());

    let frontier_output = directory.join("native.frontier.report.json");
    let frontier = run_cli(&[
        "case",
        "frontier",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--format",
        "json",
        "--output",
        frontier_output.to_str().expect("frontier output path"),
    ]);
    assert!(frontier.status.success(), "stderr: {}", stderr(&frontier));
    assert!(stdout(&frontier).is_empty());
    assert!(json_file(frontier_output)["result"]["frontier_cell_ids"]
        .as_array()
        .expect("frontier ids")
        .contains(&json!("goal:native-case-contract")));

    let close_check = run_cli(&[
        "case",
        "close-check",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--base-revision-id",
        "revision:native-cli-imported",
        "--validation-evidence-id",
        "evidence:native-schema-json-valid",
        "--format",
        "json",
    ]);
    assert!(
        close_check.status.success(),
        "stderr: {}",
        stderr(&close_check)
    );
    assert_eq!(
        stdout_json(&close_check)["result"]["close_check"]["case_space_id"],
        json!(native_case_space_id())
    );

    for command in ["obstructions", "completions", "evidence", "project"] {
        let output = run_native_case_store_command(&directory, command);
        assert!(
            output.status.success(),
            "{command} stderr: {}",
            stderr(&output)
        );
        assert_eq!(
            stdout_json(&output)["metadata"]["command"],
            json!(format!("casegraphen case {command}"))
        );
    }

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn native_morphism_propose_check_apply_and_reject_flow() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_native_case_space(&directory, "revision:native-cli-imported");

    let apply_morphism_path = directory.join("apply.case_morphism.json");
    write_native_metadata_morphism(
        &apply_morphism_path,
        "morphism:native-cli-apply",
        "revision:native-cli-imported",
        "revision:native-cli-applied",
    );

    let propose = run_cli(&[
        "morphism",
        "propose",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--input",
        apply_morphism_path.to_str().expect("morphism path"),
        "--format",
        "json",
    ]);
    assert!(propose.status.success(), "stderr: {}", stderr(&propose));
    assert_eq!(
        stdout_json(&propose)["result"]["morphism"]["review_status"],
        json!("reviewed")
    );

    let check = run_cli(&[
        "morphism",
        "check",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--morphism-id",
        "morphism:native-cli-apply",
        "--format",
        "json",
    ]);
    assert!(check.status.success(), "stderr: {}", stderr(&check));
    assert_eq!(stdout_json(&check)["result"]["applicable"], json!(true));

    let apply = run_cli(&[
        "morphism",
        "apply",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--morphism-id",
        "morphism:native-cli-apply",
        "--base-revision-id",
        "revision:native-cli-imported",
        "--reviewer-id",
        "reviewer:native-cli",
        "--reason",
        "Accept metadata-only CLI morphism",
        "--format",
        "json",
    ]);
    assert!(apply.status.success(), "stderr: {}", stderr(&apply));
    assert_eq!(
        stdout_json(&apply)["result"]["record"]["current_revision_id"],
        json!("revision:native-cli-applied")
    );

    let reject_morphism_path = directory.join("reject.case_morphism.json");
    write_native_metadata_morphism(
        &reject_morphism_path,
        "morphism:native-cli-reject",
        "revision:native-cli-applied",
        "revision:native-cli-reject-candidate",
    );
    let propose_reject = run_cli(&[
        "morphism",
        "propose",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--input",
        reject_morphism_path.to_str().expect("morphism path"),
        "--format",
        "json",
    ]);
    assert!(
        propose_reject.status.success(),
        "stderr: {}",
        stderr(&propose_reject)
    );

    let reject = run_cli(&[
        "morphism",
        "reject",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--morphism-id",
        "morphism:native-cli-reject",
        "--reviewer-id",
        "reviewer:native-cli",
        "--reason",
        "Reject native CLI proposal",
        "--revision-id",
        "revision:native-cli-rejected",
        "--format",
        "json",
    ]);
    assert!(reject.status.success(), "stderr: {}", stderr(&reject));
    let reject_json = stdout_json(&reject);
    assert_eq!(
        reject_json["result"]["entry"]["morphism"]["metadata"]["outcome_review_status"],
        json!("rejected")
    );
    assert_eq!(
        reject_json["result"]["record"]["current_revision_id"],
        json!("revision:native-cli-rejected")
    );

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn native_cli_invalid_targets_exit_nonzero() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    import_native_case_space(&directory, "revision:native-cli-imported");

    let missing_case = run_cli(&[
        "case",
        "inspect",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        "case_space:does-not-exist",
        "--format",
        "json",
    ]);
    assert!(!missing_case.status.success());
    assert!(stdout(&missing_case).is_empty());
    assert!(stderr(&missing_case).contains("missing native case space"));

    let missing_morphism = run_cli(&[
        "morphism",
        "check",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--morphism-id",
        "morphism:does-not-exist",
        "--format",
        "json",
    ]);
    assert!(!missing_morphism.status.success());
    assert!(stdout(&missing_morphism).is_empty());
    assert!(stderr(&missing_morphism).contains("No such file"));

    let stale_morphism_path = directory.join("stale.case_morphism.json");
    write_native_metadata_morphism(
        &stale_morphism_path,
        "morphism:native-cli-stale",
        "revision:stale",
        "revision:native-cli-stale-target",
    );
    let stale = run_cli(&[
        "morphism",
        "propose",
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--input",
        stale_morphism_path.to_str().expect("morphism path"),
        "--format",
        "json",
    ]);
    assert!(!stale.status.success());
    assert!(stderr(&stale).contains("does not match current revision"));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn reference_workflow_reasoning_matches_checked_in_report() {
    let output = run_cli(&[
        "workflow",
        "reason",
        "--input",
        reference_workflow_fixture()
            .to_str()
            .expect("reference workflow path"),
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let value = stdout_json(&output);
    let reference = json_file(reference_workflow_report_fixture());
    assert_eq!(value, reference);

    assert_eq!(
        value["result"]["readiness"]["ready_item_ids"],
        json!(["task:define-workflow-reasoning-contract"])
    );
    assert_eq!(
        value["result"]["readiness"]["not_ready_items"][0]["work_item_id"],
        json!("proof:workflow-schema-parse-check")
    );

    let obstructions = value["result"]["obstructions"]
        .as_array()
        .expect("obstructions");
    assert!(obstructions
        .iter()
        .any(|record| record["obstruction_type"] == json!("missing_evidence")));
    assert!(obstructions
        .iter()
        .any(|record| record["obstruction_type"] == json!("missing_proof")));
    assert!(obstructions
        .iter()
        .any(|record| record["obstruction_type"] == json!("unresolved_dependency")));
    assert!(obstructions
        .iter()
        .any(|record| record["obstruction_type"] == json!("review_required")));

    let completion_candidates = value["result"]["completion_candidates"]
        .as_array()
        .expect("completion candidates");
    assert!(completion_candidates
        .iter()
        .any(|record| record["candidate_type"] == json!("missing_evidence")));
    assert!(completion_candidates
        .iter()
        .any(|record| record["candidate_type"] == json!("missing_proof")));
    assert!(completion_candidates
        .iter()
        .any(|record| record["candidate_type"] == json!("missing_task")));

    assert_eq!(
        value["result"]["evidence_findings"]["accepted_evidence_ids"],
        json!(["evidence:workflow-target-doc"])
    );
    assert_eq!(
        value["result"]["evidence_findings"]["inference_record_ids"],
        json!(["evidence:workflow-gap-inference"])
    );
    assert!(value["result"]["evidence_findings"]["findings"]
        .as_array()
        .expect("evidence findings")
        .iter()
        .any(|record| record["finding_type"] == json!("evidence_missing")));

    assert_eq!(
        value["result"]["projection"]["projection_profile_id"],
        json!("projection:workflow-ai-review")
    );
    assert_eq!(
        value["projection"]["ai_view"]["audience"],
        json!("ai_agent")
    );
    assert_eq!(
        value["projection"]["ai_view"]["information_loss"][0]["omitted_ids"],
        json!(["docs/specs/intermediate-tools/casegraphen-workflow-reasoning-engine.md"])
    );
    let ai_records = value["projection"]["ai_view"]["records"]
        .as_array()
        .expect("ai records");
    for record_type in [
        "readiness",
        "obstruction",
        "completion_candidate",
        "evidence_finding",
        "projection",
        "correspondence",
        "evolution",
    ] {
        assert!(
            ai_records
                .iter()
                .any(|record| record["record_type"] == json!(record_type)),
            "missing AI projection record type {record_type}"
        );
    }

    assert_eq!(
        value["result"]["correspondence"][0]["correspondence_type"],
        json!("similar_with_loss")
    );
    assert_eq!(
        value["result"]["evolution"]["transition_ids"],
        json!(["transition:foundation-docs-to-workflow-contract"])
    );
    assert_eq!(
        value["result"]["evolution"]["persisted_shape_ids"],
        json!([
            "schemas/casegraphen/case.graph.schema.json",
            "schemas/casegraphen/case.report.schema.json"
        ])
    );
}

#[test]
fn compare_supports_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let output_path = directory.join("compare.report.json");

    let output = run_cli(&[
        "compare",
        "--left",
        graph_fixture().to_str().expect("fixture path"),
        "--right",
        graph_fixture().to_str().expect("fixture path"),
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let value: Value =
        serde_json::from_str(&fs::read_to_string(&output_path).expect("read report"))
            .expect("report JSON");
    assert_eq!(value["result"]["equivalent"], json!(true));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn invalid_input_errors_exit_nonzero() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let bad_graph_path = directory.join("bad-schema.case.graph.json");
    let mut graph: Value =
        serde_json::from_str(&fs::read_to_string(graph_fixture()).expect("read graph fixture"))
            .expect("graph fixture JSON");
    graph["schema"] = json!("highergraphen.case.graph.v0");
    fs::write(
        &bad_graph_path,
        serde_json::to_string_pretty(&graph).expect("serialize bad graph"),
    )
    .expect("write bad graph");

    let output = run_cli(&[
        "validate",
        "--input",
        bad_graph_path.to_str().expect("bad graph path"),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("unsupported schema"));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn invalid_workflow_reference_errors_before_reasoning_report() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp directory");
    let bad_workflow_path = directory.join("bad.workflow.graph.json");
    let mut workflow = json_file(workflow_fixture());
    workflow["workflow_relations"][0]["from_id"] = json!("task:missing-work-item");
    fs::write(
        &bad_workflow_path,
        serde_json::to_string_pretty(&workflow).expect("serialize bad workflow"),
    )
    .expect("write bad workflow");

    let output = run_cli(&[
        "workflow",
        "reason",
        "--input",
        bad_workflow_path.to_str().expect("bad workflow path"),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("workflow validation failed"));
    assert!(stderr(&output).contains("dangling_reference"));

    fs::remove_dir_all(directory).expect("remove temp directory");
}

#[test]
fn schema_and_fixture_files_are_valid_json() {
    for path in schema_fixture_paths() {
        let text = fs::read_to_string(&path).expect("read JSON file");
        serde_json::from_str::<Value>(&text).unwrap_or_else(|error| {
            panic!("{} should be valid JSON: {error}", path.display());
        });
    }
}

#[test]
fn native_schema_examples_validate_against_json_schemas() {
    for (schema, example) in native_schema_example_pairs() {
        let output = Command::new("python3")
            .args([
                "-m",
                "jsonschema",
                schema.to_str().expect("schema path"),
                "--instance",
                example.to_str().expect("example path"),
            ])
            .output()
            .expect("run python jsonschema validator");

        assert!(
            output.status.success(),
            "{} should validate against {}\nstdout: {}\nstderr: {}",
            example.display(),
            schema.display(),
            stdout(&output),
            stderr(&output)
        );
    }
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_casegraphen"))
        .args(args)
        .output()
        .expect("run casegraphen CLI")
}

fn successful_workflow_command(command: &str) -> Output {
    let output = run_cli(&[
        "workflow",
        command,
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    output
}

fn import_bridge_workflow(directory: &Path) {
    let output = run_cli(&[
        "cg",
        "workflow",
        "import",
        "--store",
        directory.to_str().expect("temp path"),
        "--input",
        workflow_fixture().to_str().expect("workflow fixture path"),
        "--revision-id",
        "revision:bridge-import",
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

fn run_bridge_store_command(directory: &Path, command: &str) -> Output {
    let output = run_cli(&[
        "cg",
        "workflow",
        command,
        "--store",
        directory.to_str().expect("temp path"),
        "--workflow-graph-id",
        "workflow_graph:casegraphen-rewrite-contract",
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    output
}

fn import_native_case_space(directory: &Path, revision_id: &str) -> Output {
    let output = run_cli(&[
        "case",
        "import",
        "--store",
        directory.to_str().expect("temp path"),
        "--input",
        native_case_fixture().to_str().expect("native fixture path"),
        "--revision-id",
        revision_id,
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    output
}

fn run_native_case_store_command(directory: &Path, command: &str) -> Output {
    let output = run_cli(&[
        "case",
        command,
        "--store",
        directory.to_str().expect("temp path"),
        "--case-space-id",
        native_case_space_id(),
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    output
}

fn write_native_metadata_morphism(
    path: &Path,
    morphism_id: &str,
    source_revision_id: &str,
    target_revision_id: &str,
) {
    let morphism = json!({
        "morphism_id": morphism_id,
        "morphism_type": "review",
        "source_revision_id": source_revision_id,
        "target_revision_id": target_revision_id,
        "added_ids": [],
        "updated_ids": [],
        "retired_ids": [],
        "preserved_ids": ["goal:native-case-contract"],
        "violated_invariant_ids": [],
        "review_status": "unreviewed",
        "evidence_ids": [],
        "source_ids": ["source:native-cli-test"],
        "metadata": {}
    });
    fs::write(
        path,
        serde_json::to_string_pretty(&morphism).expect("serialize morphism"),
    )
    .expect("write native morphism");
}

fn stdout_json(output: &Output) -> Value {
    let stdout = stdout(output);
    assert_eq!(stdout.lines().count(), 1);
    serde_json::from_str(stdout.trim_end()).expect("stdout JSON")
}

fn json_file(path: PathBuf) -> Value {
    serde_json::from_str(&fs::read_to_string(&path).expect("read JSON file"))
        .unwrap_or_else(|error| panic!("{} should be valid JSON: {error}", path.display()))
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf8")
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "casegraphen-cli-test-{}-{nanos}-{counter}",
        std::process::id()
    ))
}

fn graph_fixture() -> PathBuf {
    repo_path("schemas/casegraphen/case.graph.example.json")
}

fn policy_fixture() -> PathBuf {
    repo_path("schemas/casegraphen/coverage.policy.example.json")
}

fn projection_fixture() -> PathBuf {
    repo_path("schemas/casegraphen/projection.example.json")
}

fn workflow_fixture() -> PathBuf {
    repo_path("schemas/casegraphen/workflow.graph.example.json")
}

fn native_case_fixture() -> PathBuf {
    repo_path("schemas/casegraphen/native.case.space.example.json")
}

fn reference_workflow_fixture() -> PathBuf {
    repo_path("examples/casegraphen/reference/workflow.graph.json")
}

fn reference_workflow_report_fixture() -> PathBuf {
    repo_path("examples/casegraphen/reference/reports/workflow.reason.report.json")
}

fn missing_evidence_candidate_id() -> &'static str {
    "candidate:missing-evidence:obstruction-missing-evidence-proof-workflow-schema-parse-check-evidence-json-parse-check-output"
}

fn missing_proof_candidate_id() -> &'static str {
    "candidate:missing-proof:obstruction-missing-proof-task-implement-workflow-engine-proof-workflow-schema-parse-check"
}

fn missing_task_candidate_id() -> &'static str {
    "candidate:missing-task:obstruction-unresolved-dependency-task-implement-workflow-engine-task-define-workflow-reasoning-contract"
}

fn native_case_space_id() -> &'static str {
    "case_space:native-case-management-contract"
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn schema_fixture_paths() -> Vec<PathBuf> {
    [
        "schemas/casegraphen/case.graph.example.json",
        "schemas/casegraphen/coverage.policy.example.json",
        "schemas/casegraphen/projection.example.json",
        "schemas/casegraphen/workflow.graph.example.json",
        "schemas/casegraphen/workflow.report.example.json",
        "schemas/casegraphen/native.case.space.example.json",
        "schemas/casegraphen/native.case.report.example.json",
        "schemas/casegraphen/case.graph.schema.json",
        "schemas/casegraphen/coverage.policy.schema.json",
        "schemas/casegraphen/projection.schema.json",
        "schemas/casegraphen/case.report.schema.json",
        "schemas/casegraphen/workflow.graph.schema.json",
        "schemas/casegraphen/workflow.report.schema.json",
        "schemas/casegraphen/native.case.space.schema.json",
        "schemas/casegraphen/native.case.report.schema.json",
        "examples/casegraphen/reference/workflow.graph.json",
        "examples/casegraphen/reference/reports/workflow.reason.report.json",
    ]
    .iter()
    .map(|path| repo_path(path))
    .collect()
}

fn native_schema_example_pairs() -> Vec<(PathBuf, PathBuf)> {
    [
        (
            "schemas/casegraphen/native.case.space.schema.json",
            "schemas/casegraphen/native.case.space.example.json",
        ),
        (
            "schemas/casegraphen/native.case.report.schema.json",
            "schemas/casegraphen/native.case.report.example.json",
        ),
    ]
    .iter()
    .map(|(schema, example)| (repo_path(schema), repo_path(example)))
    .collect()
}
