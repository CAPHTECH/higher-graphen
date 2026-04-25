#![allow(missing_docs)]

use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

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
fn schema_and_fixture_files_are_valid_json() {
    for path in schema_fixture_paths() {
        let text = fs::read_to_string(&path).expect("read JSON file");
        serde_json::from_str::<Value>(&text).unwrap_or_else(|error| {
            panic!("{} should be valid JSON: {error}", path.display());
        });
    }
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_casegraphen"))
        .args(args)
        .output()
        .expect("run casegraphen CLI")
}

fn stdout_json(output: &Output) -> Value {
    let stdout = stdout(output);
    assert_eq!(stdout.lines().count(), 1);
    serde_json::from_str(stdout.trim_end()).expect("stdout JSON")
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
    std::env::temp_dir().join(format!(
        "casegraphen-cli-test-{}-{nanos}",
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
        "schemas/casegraphen/case.graph.schema.json",
        "schemas/casegraphen/coverage.policy.schema.json",
        "schemas/casegraphen/projection.schema.json",
        "schemas/casegraphen/case.report.schema.json",
        "schemas/casegraphen/workflow.graph.schema.json",
        "schemas/casegraphen/workflow.report.schema.json",
    ]
    .iter()
    .map(|path| repo_path(path))
    .collect()
}
