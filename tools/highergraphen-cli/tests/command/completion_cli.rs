use crate::common::*;
use serde_json::{json, Value};
use std::fs;

#[test]
fn completion_review_accept_reads_report_and_writes_one_json_report_to_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let source_report = write_smoke_report(&directory);

    let output = run_cli(&[
        "completion",
        "review",
        "accept",
        "--input",
        source_report.to_str().expect("source path should be utf-8"),
        "--candidate",
        BILLING_STATUS_API_CANDIDATE,
        "--reviewer",
        "reviewer:architecture-lead",
        "--reason",
        "Billing owns the API boundary.",
        "--reviewed-at",
        "2026-04-25T00:00:00Z",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);

    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(COMPLETION_REVIEW_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen completion review accept")
    );
    assert_eq!(value["result"]["status"], json!("accepted"));
    assert_eq!(
        value["result"]["review_record"]["candidate"]["review_status"],
        json!("unreviewed")
    );
    assert_eq!(
        value["result"]["review_record"]["accepted_completion"]["review_status"],
        json!("accepted")
    );
    assert_eq!(
        value["result"]["review_record"]["accepted_completion"]["accepted_structure"]
            ["structure_id"],
        json!(BILLING_STATUS_API_CELL)
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn completion_review_reject_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let source_report = write_smoke_report(&directory);
    let output_path = directory.join("completion-review.report.json");

    let output = run_cli(&[
        "completion",
        "review",
        "reject",
        "--input",
        source_report.to_str().expect("source path should be utf-8"),
        "--candidate",
        BILLING_STATUS_API_CANDIDATE,
        "--reviewer",
        "reviewer:architecture-lead",
        "--reason",
        "Use an event instead.",
        "--format",
        "json",
        "--output",
        output_path.to_str().expect("output path should be utf-8"),
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());

    let text = fs::read_to_string(&output_path).expect("read JSON review file");
    let value: Value = serde_json::from_str(&text).expect("file should be JSON");
    assert_eq!(value["schema"], json!(COMPLETION_REVIEW_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen completion review reject")
    );
    assert_eq!(value["result"]["status"], json!("rejected"));
    assert_eq!(
        value["result"]["review_record"]["rejected_completion"]["review_status"],
        json!("rejected")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn completion_review_requires_explicit_candidate_reviewer_and_reason() {
    let output = run_cli(&[
        "completion",
        "review",
        "accept",
        "--input",
        "missing.report.json",
        "--candidate",
        BILLING_STATUS_API_CANDIDATE,
        "--reviewer",
        "reviewer:architecture-lead",
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--reason <text> is required"));
}

#[test]
fn completion_review_refuses_unknown_candidate() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let source_report = write_smoke_report(&directory);

    let output = run_cli(&[
        "completion",
        "review",
        "accept",
        "--input",
        source_report.to_str().expect("source path should be utf-8"),
        "--candidate",
        "candidate:missing",
        "--reviewer",
        "reviewer:architecture-lead",
        "--reason",
        "Reviewed",
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("was not found"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}
