//! Command-level tests for the HigherGraphen CLI.

use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const INPUT_LIFT_REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const FEED_READER_REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const COMPLETION_REVIEW_REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

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

#[test]
fn input_lift_command_requires_input_path() {
    let output = run_cli(&["architecture", "input", "lift", "--format", "json"]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--input <path> is required"));
}

#[test]
fn feed_reader_run_requires_input_path() {
    let output = run_cli(&["feed", "reader", "run", "--format", "json"]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--input <path> is required"));
}

#[test]
fn unsupported_or_missing_format_exits_nonzero() {
    let missing = run_cli(&["architecture", "smoke", "direct-db-access"]);
    assert!(!missing.status.success());
    assert!(stdout(&missing).is_empty());
    assert!(stderr(&missing).contains("--format json is required"));

    let unsupported = run_cli(&[
        "architecture",
        "smoke",
        "direct-db-access",
        "--format",
        "human",
    ]);
    assert!(!unsupported.status.success());
    assert!(stdout(&unsupported).is_empty());
    assert!(stderr(&unsupported).contains("only json is supported"));
}

#[test]
fn unsupported_command_exits_nonzero() {
    let output = run_cli(&["architecture", "smoke", "unknown", "--format", "json"]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("unsupported command segment"));
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_highergraphen"))
        .args(args)
        .output()
        .expect("run highergraphen CLI")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "highergraphen-cli-test-{}-{nanos}",
        std::process::id()
    ))
}

fn input_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/architecture-lift.input.example.json")
}

fn feed_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/feed-lift.input.example.json")
}

fn write_smoke_report(directory: &std::path::Path) -> PathBuf {
    let source_report = directory.join("architecture-direct-db-access-smoke.report.json");
    let output = run_cli(&[
        "architecture",
        "smoke",
        "direct-db-access",
        "--format",
        "json",
        "--output",
        source_report.to_str().expect("source path should be utf-8"),
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
    source_report
}
