//! Command-level tests for the HigherGraphen CLI.

use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const INPUT_LIFT_REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const FEED_READER_REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const PR_REVIEW_TARGET_REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
const COMPLETION_REVIEW_REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";
static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

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

#[test]
fn pr_review_input_from_git_emits_bounded_snapshot() {
    let repository = write_git_fixture();
    let output = run_cli(&[
        "pr-review",
        "input",
        "from-git",
        "--repo",
        repository.to_str().expect("repo path should be utf-8"),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
        "--format",
        "json",
    ]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());

    let stdout = stdout(&output);
    assert_eq!(stdout.lines().count(), 1);
    let value: Value = serde_json::from_str(stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(
        value["schema"],
        json!("highergraphen.pr_review_target.input.v1")
    );
    assert_eq!(value["source"]["kind"], json!("code"));
    assert!(value["changed_files"]
        .as_array()
        .expect("changed files")
        .iter()
        .any(|file| file["path"]
            == json!("crates/higher-graphen-runtime/src/workflows/pr_review_target.rs")));
    assert!(value["changed_files"]
        .as_array()
        .expect("changed files")
        .iter()
        .any(|file| file["path"] == json!(".casegraphen/cases/generated/events.jsonl")));
    assert!(value["changed_files"]
        .as_array()
        .expect("changed files")
        .iter()
        .any(|file| file["path"] == json!(".github/workflows/ci.yml")));
    assert!(value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .any(|signal| signal["id"] == json!("signal:contract-coupling")));
    let signal_ids = value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .map(|signal| signal["id"].clone())
        .collect::<Vec<_>>();
    assert!(signal_ids.contains(&json!("signal:public-api-surface-change")));
    assert!(signal_ids.contains(&json!("signal:serde-contract-change")));
    assert!(signal_ids.contains(&json!("signal:panic-placeholder-added")));
    assert!(signal_ids.contains(&json!("signal:external-effect-surface-change")));
    assert!(signal_ids.contains(&json!("signal:test-assertion-weakened")));
    assert!(signal_ids.contains(&json!("signal:ai-review-boundary-change")));
    assert!(signal_ids.contains(&json!("signal:structural-boundary-change")));
    let public_api_signal = value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .find(|signal| signal["id"] == json!("signal:public-api-surface-change"))
        .expect("public API signal");
    let public_api_source_ids = public_api_signal["source_ids"]
        .as_array()
        .expect("public API source ids");
    assert!(public_api_source_ids.contains(&json!(
        "file:crates-higher-graphen-runtime-src-workflows-pr-review-target-lift-rs"
    )));
    assert!(public_api_source_ids.contains(&json!(
        "file:tools-highergraphen-cli-src-path-with-space-rs"
    )));
    let structural_signal = value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .find(|signal| signal["id"] == json!("signal:structural-boundary-change"))
        .expect("structural boundary signal");
    assert!(structural_signal["source_ids"]
        .as_array()
        .expect("structural source ids")
        .contains(&json!("file:tools-highergraphen-cli-src-main-rs")));
    let large_signal = value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .find(|signal| signal["id"] == json!("signal:large-git-change"))
        .expect("large git change signal");
    assert_eq!(
        large_signal["source_ids"],
        json!(["evidence:git-diff"]),
        "large changes should stay aggregate instead of expanding to every file"
    );
    let ownership_signal = value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .find(|signal| signal["id"] == json!("signal:ownership-boundary"))
        .expect("ownership boundary signal");
    assert_eq!(
        ownership_signal["source_ids"]
            .as_array()
            .expect("ownership source ids")
            .len(),
        5,
        "ownership boundary should use one representative file per reviewable owner"
    );
    assert!(value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .flat_map(|signal| signal["source_ids"].as_array().into_iter().flatten())
        .all(|source_id| !source_id
            .as_str()
            .expect("source id should be a string")
            .contains("casegraphen")));
    assert!(value["signals"]
        .as_array()
        .expect("signals")
        .iter()
        .flat_map(|signal| signal["source_ids"].as_array().into_iter().flatten())
        .any(|source_id| source_id
            .as_str()
            .expect("source id should be a string")
            .contains("github-workflows-ci-yml")));

    fs::remove_dir_all(repository).expect("remove temp test repository");
}

#[test]
fn pr_review_input_from_git_output_feeds_recommender() {
    let repository = write_git_fixture();
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let input_path = directory.join("pr-review.input.json");

    let input_output = run_cli(&[
        "pr-review",
        "input",
        "from-git",
        "--repo",
        repository.to_str().expect("repo path should be utf-8"),
        "--base",
        "HEAD~1",
        "--head",
        "HEAD",
        "--format",
        "json",
        "--output",
        input_path.to_str().expect("input path should be utf-8"),
    ]);
    assert!(
        input_output.status.success(),
        "stderr: {}",
        stderr(&input_output)
    );
    assert!(stdout(&input_output).is_empty());
    assert!(stderr(&input_output).is_empty());

    let report_output = run_cli(&[
        "pr-review",
        "targets",
        "recommend",
        "--input",
        input_path.to_str().expect("input path should be utf-8"),
        "--format",
        "json",
    ]);
    assert!(
        report_output.status.success(),
        "stderr: {}",
        stderr(&report_output)
    );
    assert!(stderr(&report_output).is_empty());
    let report_stdout = stdout(&report_output);
    let value: Value =
        serde_json::from_str(report_stdout.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(PR_REVIEW_TARGET_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("targets_recommended"));
    assert!(value["result"]["review_targets"]
        .as_array()
        .expect("review targets")
        .iter()
        .all(|target| target["review_status"] == json!("unreviewed")));
    let review_targets = value["result"]["review_targets"]
        .as_array()
        .expect("review targets");
    assert!(review_targets.iter().all(|target| !target["target_ref"]
        .as_str()
        .unwrap_or_default()
        .starts_with(".casegraphen/")));
    assert!(review_targets
        .iter()
        .any(|target| target["target_ref"] == json!(".github/workflows/ci.yml")));
    assert!(review_targets
        .iter()
        .any(|target| target["target_ref"] == json!("tools/highergraphen-cli/src/main.rs")));
    assert_eq!(
        review_targets
            .iter()
            .filter(|target| target["target_ref"] == json!("signal:large-git-change"))
            .count(),
        1
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
    fs::remove_dir_all(repository).expect("remove temp test repository");
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
fn pr_review_targets_recommend_requires_input_path() {
    let output = run_cli(&["pr-review", "targets", "recommend", "--format", "json"]);

    assert!(!output.status.success());
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).contains("--input <path> is required"));
}

#[test]
fn pr_review_input_from_git_requires_base_and_head() {
    let missing_base = run_cli(&[
        "pr-review",
        "input",
        "from-git",
        "--head",
        "HEAD",
        "--format",
        "json",
    ]);
    assert!(!missing_base.status.success());
    assert!(stdout(&missing_base).is_empty());
    assert!(stderr(&missing_base).contains("--base <ref> is required"));

    let missing_head = run_cli(&[
        "pr-review",
        "input",
        "from-git",
        "--base",
        "HEAD~1",
        "--format",
        "json",
    ]);
    assert!(!missing_head.status.success());
    assert!(stdout(&missing_head).is_empty());
    assert!(stderr(&missing_head).contains("--head <ref> is required"));
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
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "highergraphen-cli-test-{}-{nanos}-{counter}",
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

fn pr_review_target_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/pr-review-target.input.example.json")
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

fn write_git_fixture() -> PathBuf {
    let repository = unique_temp_dir();
    fs::create_dir_all(&repository).expect("create temp git repository");
    run_git(&repository, &["init"]);
    run_git(
        &repository,
        &["config", "user.email", "test@example.invalid"],
    );
    run_git(&repository, &["config", "user.name", "Test User"]);

    fs::write(repository.join("README.md"), "# fixture\n").expect("write base file");
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/tests/command.rs",
        "#[test]\nfn pr_review_targets_recommend() {\n    assert!(true);\n}\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(&repository, &["commit", "-m", "base"]);

    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/workflows/pr_review_target.rs",
        "pub fn run_pr_review_target_recommend() {\n    let _ = Some(1).unwrap();\n}\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/pr_review_reports.rs",
        "#[serde(rename_all = \"snake_case\")]\npub enum ReviewStatus {\n    Unreviewed,\n}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/pr_review_git.rs",
        "use std::process::Command as ProcessCommand;\nfn run_git() {\n    let _ = ProcessCommand::new(\"git\");\n}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/path with space.rs",
        "pub(super) fn spaced_api() {}\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/workflows/pr_review_target_lift.rs",
        "pub(super) struct LiftedPrReviewTarget {}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/main.rs",
        "use higher_graphen_runtime::{run_alpha, run_beta};\n\nenum Command {\n    Alpha,\n    Beta,\n}\n\nfn run(command: Command) {\n    match command {\n        Command::Alpha => run_alpha(),\n        Command::Beta => run_beta(),\n    }\n}\n",
    );
    write_repo_file(
        &repository,
        "schemas/reports/pr-review-target.report.schema.json",
        "{\"$schema\":\"https://json-schema.org/draft/2020-12/schema\"}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/tests/command.rs",
        "#[test]\nfn pr_review_targets_recommend() {}\n",
    );
    write_repo_file(
        &repository,
        "docs/cli/highergraphen.md",
        "# highergraphen\n\nAI proposal output must remain unreviewed until human review accepts it.\n",
    );
    write_repo_file(
        &repository,
        ".casegraphen/cases/generated/events.jsonl",
        "{\"event\":\"generated\"}\n",
    );
    write_repo_file(
        &repository,
        ".github/workflows/ci.yml",
        "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps: []\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(
        &repository,
        &["commit", "-m", "add pr review target workflow"],
    );

    repository
}

fn write_repo_file(repository: &std::path::Path, path: &str, contents: &str) {
    let path = repository.join(path);
    fs::create_dir_all(path.parent().expect("file has parent")).expect("create parent dirs");
    fs::write(path, contents).expect("write repo file");
}

fn run_git(repository: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}
