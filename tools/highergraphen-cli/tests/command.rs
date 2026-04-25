//! Command-level tests for the HigherGraphen CLI.

use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";

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
