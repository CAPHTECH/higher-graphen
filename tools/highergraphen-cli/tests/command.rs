//! Command-level tests for the HigherGraphen CLI.

use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const INPUT_LIFT_REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const FEED_READER_REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
const PR_REVIEW_TARGET_REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
const TEST_GAP_INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
const TEST_GAP_REPORT_SCHEMA: &str = "highergraphen.test_gap.report.v1";
const SEMANTIC_PROOF_INPUT_SCHEMA: &str = "highergraphen.semantic_proof.input.v1";
const SEMANTIC_PROOF_REPORT_SCHEMA: &str = "highergraphen.semantic_proof.report.v1";
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
fn test_gap_detect_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = test_gap_fixture();
    let output = run_cli(&[
        "test-gap",
        "detect",
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
    assert_eq!(value["schema"], json!(TEST_GAP_REPORT_SCHEMA));
    assert_eq!(
        value["metadata"]["command"],
        json!("highergraphen test-gap detect")
    );
    assert_eq!(value["result"]["status"], json!("gaps_detected"));
    assert!(value["result"]["obstructions"]
        .as_array()
        .expect("obstructions")
        .iter()
        .any(|obstruction| obstruction["obstruction_type"]
            == json!("missing_boundary_case_unit_test")));
    assert!(value["result"]["completion_candidates"]
        .as_array()
        .expect("completion candidates")
        .iter()
        .all(
            |candidate| candidate["candidate_type"] == json!("missing_test")
                && candidate["review_status"] == json!("unreviewed")
        ));
}

#[test]
fn test_gap_detect_writes_output_file_without_stdout() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let output_path = directory.join("test-gap.report.json");
    let fixture = test_gap_fixture();

    let output = run_cli(&[
        "test-gap",
        "detect",
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
    assert_eq!(value["schema"], json!(TEST_GAP_REPORT_SCHEMA));
    assert_eq!(value["projection"]["purpose"], json!("test_gap_detection"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_verify_reads_fixture_and_writes_one_json_report_to_stdout() {
    let fixture = semantic_proof_fixture();
    let output = run_cli(&[
        "semantic-proof",
        "verify",
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
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("proved"));
    assert!(value["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["certificate_ids"]
            .as_array()
            .is_some_and(
                |certificate_ids| certificate_ids.contains(&json!("certificate:semantic:pricing"))
            )));
}

#[test]
fn semantic_proof_input_from_artifact_emits_proved_input_and_verifies() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("kani-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "proved",
            "input_hash": "sha256:input",
            "proof_hash": "sha256:proof",
            "witness_ids": [
                "cell:semantic:pricing:base",
                "cell:semantic:pricing:head"
            ],
            "review_status": "accepted",
            "confidence": 0.91
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let output = run_cli_owned(&semantic_artifact_command(&artifact, None));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let generated_input = stdout(&output);
    assert_eq!(generated_input.lines().count(), 1);
    let value: Value =
        serde_json::from_str(generated_input.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert_eq!(value["source"]["kind"], json!("code"));
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("source adapters")
        .contains(&json!("semantic-proof-from-artifact.v1")));
    assert_eq!(
        value["verification_policy"]["accepted_backends"],
        json!(["kani"])
    );
    assert_eq!(value["proof_certificates"][0]["backend"], json!("kani"));

    let input = directory.join("semantic-proof.input.json");
    fs::write(&input, generated_input).expect("write generated semantic proof input");
    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("proved"));
    assert!(report["result"]["accepted_certificate_ids"]
        .as_array()
        .expect("accepted certificate ids")
        .iter()
        .any(|id| id == &json!("certificate:semantic:kani:theorem-semantic-pricing")));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_emits_counterexample_input_and_verifies() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("smt-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "counterexample",
            "path_ids": [
                "cell:semantic:pricing:base",
                "cell:semantic:pricing:head"
            ],
            "summary": "symbolic execution found a mismatch",
            "severity": "critical",
            "review_status": "accepted",
            "confidence": 0.93
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let input = directory.join("semantic-proof.input.json");
    let output = run_cli_owned(&semantic_artifact_command(&artifact, Some(&input)));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).is_empty());
    assert!(stderr(&output).is_empty());
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&input).expect("read generated semantic proof input"),
    )
    .expect("generated input should be JSON");
    assert_eq!(value["schema"], json!(SEMANTIC_PROOF_INPUT_SCHEMA));
    assert_eq!(value["counterexamples"][0]["severity"], json!("critical"));

    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("counterexample_found"));
    assert_eq!(
        report["result"]["counterexamples"][0]["id"],
        json!("counterexample:semantic:kani:theorem-semantic-pricing")
    );

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_rejects_invalid_artifacts() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");

    for (name, artifact, expected_error) in [
        (
            "missing-status",
            json!({
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof"
            }),
            "missing status",
        ),
        (
            "unknown-status",
            json!({
                "status": "unknown",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof"
            }),
            "unsupported semantic proof artifact status",
        ),
        (
            "bad-confidence",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "confidence": 1.1
            }),
            "confidence must be between 0.0 and 1.0 inclusive",
        ),
        (
            "bad-witness-ids",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "witness_ids": "not-an-array"
            }),
            "witness_ids must be an array of strings",
        ),
        (
            "bad-path-ids-entry",
            json!({
                "status": "counterexample",
                "path_ids": ["cell:semantic:pricing:base", 1]
            }),
            "path_ids entries must be strings",
        ),
    ] {
        let artifact_path = directory.join(format!("{name}.json"));
        fs::write(&artifact_path, artifact.to_string()).expect("write invalid artifact");

        let output = run_cli_owned(&semantic_artifact_command(&artifact_path, None));

        assert!(
            !output.status.success(),
            "{name} unexpectedly succeeded with stdout: {}",
            stdout(&output)
        );
        assert!(stdout(&output).is_empty());
        assert!(
            stderr(&output).contains(expected_error),
            "{name} stderr did not contain {expected_error:?}: {}",
            stderr(&output)
        );
    }

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_rejected_or_unhashed_certificates_are_insufficient() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");

    for (name, artifact) in [
        (
            "rejected-review",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "proof_hash": "sha256:proof",
                "review_status": "rejected",
                "confidence": 0.91
            }),
        ),
        (
            "missing-input-hash",
            json!({
                "status": "proved",
                "proof_hash": "sha256:proof",
                "review_status": "accepted",
                "confidence": 0.91
            }),
        ),
        (
            "missing-proof-hash",
            json!({
                "status": "proved",
                "input_hash": "sha256:input",
                "review_status": "accepted",
                "confidence": 0.91
            }),
        ),
    ] {
        let artifact_path = directory.join(format!("{name}.json"));
        let input_path = directory.join(format!("{name}.input.json"));
        fs::write(&artifact_path, artifact.to_string()).expect("write policy artifact");

        let input_output = run_cli_owned(&semantic_artifact_command(
            &artifact_path,
            Some(&input_path),
        ));
        assert!(
            input_output.status.success(),
            "{name} input stderr: {}",
            stderr(&input_output)
        );
        assert!(stdout(&input_output).is_empty());

        let verify = run_cli_owned(&[
            "semantic-proof".to_owned(),
            "verify".to_owned(),
            "--input".to_owned(),
            input_path
                .to_str()
                .expect("input path should be utf-8")
                .to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
        ]);
        assert!(
            verify.status.success(),
            "{name} stderr: {}",
            stderr(&verify)
        );
        let report: Value =
            serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
        assert_eq!(
            report["result"]["status"],
            json!("insufficient_proof"),
            "{name} should not prove the theorem"
        );
        assert!(report["result"]["proof_objects"]
            .as_array()
            .map(|proof_objects| proof_objects.is_empty())
            .unwrap_or(true));
    }

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn semantic_proof_input_from_artifact_preserves_counterexample_found_defaults() {
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let artifact = directory.join("counterexample-found-artifact.json");
    fs::write(
        &artifact,
        json!({
            "status": "counterexample_found",
            "confidence": 0.88
        })
        .to_string(),
    )
    .expect("write semantic proof artifact");

    let output = run_cli_owned(&semantic_artifact_command(&artifact, None));

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stderr(&output).is_empty());
    let generated_input = stdout(&output);
    let value: Value =
        serde_json::from_str(generated_input.trim_end()).expect("stdout should be JSON");
    assert_eq!(value["counterexamples"][0]["severity"], json!("high"));
    assert_eq!(
        value["counterexamples"][0]["path_ids"],
        json!(["cell:semantic:pricing:base", "cell:semantic:pricing:head"])
    );
    assert_eq!(
        value["counterexamples"][0]["summary"],
        json!("Backend artifact supplied a counterexample.")
    );
    assert_eq!(
        value["counterexamples"][0]["review_status"],
        json!("accepted")
    );

    let input = directory.join("semantic-proof.input.json");
    fs::write(&input, generated_input).expect("write generated semantic proof input");
    let verify = run_cli_owned(&[
        "semantic-proof".to_owned(),
        "verify".to_owned(),
        "--input".to_owned(),
        input
            .to_str()
            .expect("input path should be utf-8")
            .to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
    ]);
    assert!(verify.status.success(), "stderr: {}", stderr(&verify));
    let report: Value =
        serde_json::from_str(stdout(&verify).trim_end()).expect("verify stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("counterexample_found"));

    fs::remove_dir_all(directory).expect("remove temp test directory");
}

#[test]
fn test_gap_input_from_git_emits_bounded_snapshot() {
    let repository = write_git_fixture();
    let output = run_cli(&[
        "test-gap",
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
    assert_eq!(value["schema"], json!(TEST_GAP_INPUT_SCHEMA));
    assert_eq!(value["source"]["kind"], json!("code"));
    assert!(value["source"]["adapters"]
        .as_array()
        .expect("source adapters")
        .contains(&json!("test-gap-from-git.v1")));
    assert!(value["changed_files"]
        .as_array()
        .expect("changed files")
        .iter()
        .any(|file| file["path"]
            == json!("crates/higher-graphen-runtime/src/workflows/pr_review_target.rs")));
    assert!(!value["symbols"].as_array().expect("symbols").is_empty());
    assert!(!value["requirements"]
        .as_array()
        .expect("requirements")
        .is_empty());
    assert!(value["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .all(
            |requirement| requirement["expected_verification"] == json!("unit_or_integration_test")
        ));
    assert!(value["signals"].as_array().expect("signals").iter().any(
        |signal| signal["id"] == json!("signal:test-gap:changed-source-without-accepted-test")
    ));
    assert_eq!(
        value["detector_context"]["test_kinds"],
        json!(["unit", "integration"])
    );
    assert!(!value["detector_context"]["declared_obligation_ids"]
        .as_array()
        .expect("declared obligations")
        .is_empty());

    fs::remove_dir_all(repository).expect("remove temp test repository");
}

#[test]
fn test_gap_input_from_git_output_feeds_detector() {
    let repository = write_git_fixture();
    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let input_path = directory.join("test-gap.input.json");

    let input_output = run_cli(&[
        "test-gap",
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
        "test-gap",
        "detect",
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
    assert_eq!(value["schema"], json!(TEST_GAP_REPORT_SCHEMA));
    assert_eq!(value["result"]["status"], json!("gaps_detected"));
    assert!(value["result"]["obstructions"]
        .as_array()
        .expect("obstructions")
        .iter()
        .any(|obstruction| obstruction["obstruction_type"]
            == json!("missing_requirement_verification")));

    fs::remove_dir_all(directory).expect("remove temp test directory");
    fs::remove_dir_all(repository).expect("remove temp test repository");
}

#[test]
fn test_gap_input_from_git_lifts_higher_order_test_gap_structure() {
    let repository = write_test_gap_structural_git_fixture();
    let output = run_cli(&[
        "test-gap",
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
    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");

    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("command:highergraphen:test-gap:detect")));
    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("runner:test-gap:detect")));
    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("adapter:test-gap:git-input")));
    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("law:test-gap:json-format-required")));
    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("law:test-gap:fixtures-validate-against-schema")));
    assert!(value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .any(|symbol| symbol["id"] == json!("validator:test-gap:json-contracts")));
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["id"] == json!("adapter:test-gap:git-input")));
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["cell_type"] == json!("rust_function")
            && cell["id"]
                == json!(
                    "semantic:rust:function:tools-highergraphen-cli-src-main-rs:base:parse-test-gap-detect"
                )));
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["cell_type"] == json!("rust_function")
            && cell["id"]
                .as_str()
                .expect("semantic rust function id")
                .contains("run-test-gap-detect")));
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["cell_type"] == json!("json_schema_property")
            && cell["label"]
                .as_str()
                .expect("semantic json property label")
                .contains("schema")));
    assert!(value["higher_order_incidences"]
        .as_array()
        .expect("higher order incidences")
        .iter()
        .any(|incidence| incidence["relation_type"] == json!("contains_function")));
    assert!(value["laws"]
        .as_array()
        .expect("laws")
        .iter()
        .any(|law| law["id"] == json!("law:test-gap:json-format-required")));
    assert!(value["laws"]
        .as_array()
        .expect("laws")
        .iter()
        .any(|law| law["id"] == json!("law:test-gap:semantic-delta-has-verification")));
    assert!(value["morphisms"]
        .as_array()
        .expect("morphisms")
        .iter()
        .any(
            |morphism| morphism["id"] == json!("morphism:test-gap:input-from-git-to-input-schema")
        ));
    assert!(value["morphisms"]
        .as_array()
        .expect("morphisms")
        .iter()
        .any(|morphism| morphism["morphism_type"] == json!("semantic_preservation")));
    assert!(value["morphisms"]
        .as_array()
        .expect("morphisms")
        .iter()
        .any(|morphism| morphism["morphism_type"] == json!("semantic_addition")));
    assert!(value["verification_cells"]
        .as_array()
        .expect("verification cells")
        .iter()
        .any(|verification| verification["law_ids"]
            .as_array()
            .expect("law ids")
            .contains(&json!("law:test-gap:json-format-required"))));
    assert!(value["dependency_edges"]
        .as_array()
        .expect("dependency edges")
        .iter()
        .any(|edge| edge["id"] == json!("edge:test-gap:command-detect-to-runner")));
    assert!(value["dependency_edges"]
        .as_array()
        .expect("dependency edges")
        .iter()
        .any(|edge| edge["id"] == json!("edge:test-gap:git-adapter-to-input-schema")));
    assert!(value["dependency_edges"]
        .as_array()
        .expect("dependency edges")
        .iter()
        .any(|edge| edge["id"] == json!("edge:test-gap:validator-to-input-fixture")));
    assert!(value["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|requirement| requirement["id"]
            == json!("requirement:morphism:test-gap:command-detect-to-runner")));
    assert!(value["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|requirement| requirement["id"]
            == json!("requirement:law:test-gap:json-format-required")));
    assert!(value["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|requirement| requirement["id"]
            == json!("requirement:law:test-gap:fixtures-validate-against-schema")));
    assert!(value["tests"]
        .as_array()
        .expect("tests")
        .iter()
        .any(|test| test["target_ids"]
            .as_array()
            .expect("target ids")
            .contains(&json!("command:highergraphen:test-gap:detect"))));
    assert!(value["tests"]
        .as_array()
        .expect("tests")
        .iter()
        .any(|test| test["target_ids"]
            .as_array()
            .expect("target ids")
            .contains(&json!("law:test-gap:json-format-required"))));
    assert!(value["tests"]
        .as_array()
        .expect("tests")
        .iter()
        .any(
            |test| test["id"] == json!("test:validator:test-gap-json-contracts")
                && test["target_ids"]
                    .as_array()
                    .expect("target ids")
                    .contains(&json!("law:test-gap:fixtures-validate-against-schema"))
        ));
    assert!(value["detector_context"]["test_kinds"]
        .as_array()
        .expect("test kinds")
        .contains(&json!("smoke")));

    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let input_path = directory.join("test-gap-structural.input.json");
    fs::write(
        &input_path,
        serde_json::to_string(&value).expect("serialize structural input"),
    )
    .expect("write structural input");
    let report_output = run_cli(&[
        "test-gap",
        "detect",
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
    let report: Value =
        serde_json::from_str(stdout(&report_output).trim_end()).expect("stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("no_gaps_in_snapshot"));
    assert!(report["scenario"]["laws"]
        .as_array()
        .expect("scenario laws")
        .iter()
        .any(|law| law["id"] == json!("law:test-gap:json-format-required")));
    assert!(
        report["scenario"]["lifted_structure"]["structural_summary"]["law_count"]
            .as_u64()
            .expect("law count")
            > 0
    );
    assert!(report["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["law_ids"]
            .as_array()
            .expect("law ids")
            .contains(&json!("law:test-gap:json-format-required"))));
    assert!(report["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["morphism_ids"]
            .as_array()
            .is_some_and(|morphism_ids| {
                morphism_ids.iter().any(|morphism_id| {
                    morphism_id
                        .as_str()
                        .expect("morphism id")
                        .contains("semantic_addition")
                })
            })));
    assert!(report["result"]["counterexamples"]
        .as_array()
        .map(|counterexamples| counterexamples.is_empty())
        .unwrap_or(true));

    fs::remove_dir_all(directory).expect("remove temp test directory");
    fs::remove_dir_all(repository).expect("remove temp test repository");
}

#[test]
fn test_gap_input_from_git_lifts_semantic_proof_adapter_theorems() {
    let repository = write_semantic_proof_structural_git_fixture();
    let output = run_cli(&[
        "test-gap",
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
    let value: Value =
        serde_json::from_str(stdout(&output).trim_end()).expect("stdout should be JSON");
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["id"] == json!("adapter:semantic-proof:artifact-input")));
    assert!(value["higher_order_cells"]
        .as_array()
        .expect("higher order cells")
        .iter()
        .any(|cell| cell["id"] == json!("theorem:semantic-proof:artifact-adapter-correctness")));
    assert!(value["laws"]
        .as_array()
        .expect("laws")
        .iter()
        .any(|law| law["id"] == json!("law:semantic-proof:artifact-status-totality")));
    assert!(value["morphisms"]
        .as_array()
        .expect("morphisms")
        .iter()
        .any(|morphism| morphism["id"]
            == json!("morphism:semantic-proof:artifact-to-input-document")));
    assert!(value["morphisms"]
        .as_array()
        .expect("morphisms")
        .iter()
        .filter(|morphism| morphism["id"]
            .as_str()
            .expect("morphism id")
            .contains("tools-highergraphen-cli-src-semantic-proof-artifact-rs"))
        .all(|morphism| morphism.get("expected_verification").is_none()));
    assert!(value["verification_cells"]
        .as_array()
        .expect("verification cells")
        .iter()
        .any(|verification| verification["morphism_ids"]
            .as_array()
            .expect("morphism ids")
            .contains(&json!(
                "morphism:semantic-proof:certificate-to-proof-object"
            ))));

    let directory = unique_temp_dir();
    fs::create_dir_all(&directory).expect("create temp test directory");
    let input_path = directory.join("semantic-proof-structural.input.json");
    fs::write(
        &input_path,
        serde_json::to_string(&value).expect("serialize semantic proof structural input"),
    )
    .expect("write semantic proof structural input");
    let report_output = run_cli(&[
        "test-gap",
        "detect",
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
    let report: Value =
        serde_json::from_str(stdout(&report_output).trim_end()).expect("stdout should be JSON");
    assert_eq!(report["result"]["status"], json!("no_gaps_in_snapshot"));
    assert!(report["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["morphism_ids"]
            .as_array()
            .is_some_and(|morphism_ids| morphism_ids
                .contains(&json!("morphism:semantic-proof:artifact-to-input-document")))));
    assert!(!report["result"]["proof_objects"]
        .as_array()
        .expect("proof objects")
        .iter()
        .any(|proof| proof["morphism_ids"]
            .as_array()
            .is_some_and(
                |morphism_ids| morphism_ids.iter().any(|morphism_id| morphism_id
                    .as_str()
                    .expect("morphism id")
                    .contains("tools-highergraphen-cli-src-semantic-proof-artifact-rs"))
            )));

    fs::remove_dir_all(directory).expect("remove temp test directory");
    fs::remove_dir_all(repository).expect("remove temp test repository");
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
fn test_gap_detect_requires_input_path() {
    let output = run_cli(&["test-gap", "detect", "--format", "json"]);

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
fn test_gap_input_from_git_requires_base_and_head() {
    let missing_base = run_cli(&[
        "test-gap", "input", "from-git", "--head", "HEAD", "--format", "json",
    ]);
    assert!(!missing_base.status.success());
    assert!(stdout(&missing_base).is_empty());
    assert!(stderr(&missing_base).contains("--base <ref> is required"));

    let missing_head = run_cli(&[
        "test-gap", "input", "from-git", "--base", "HEAD~1", "--format", "json",
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

    let missing_test_gap_format = run_cli(&["test-gap", "detect"]);
    assert!(!missing_test_gap_format.status.success());
    assert!(stdout(&missing_test_gap_format).is_empty());
    assert!(stderr(&missing_test_gap_format).contains("--format json is required"));

    let unsupported_test_gap_format = run_cli(&["test-gap", "detect", "--format", "human"]);
    assert!(!unsupported_test_gap_format.status.success());
    assert!(stdout(&unsupported_test_gap_format).is_empty());
    assert!(stderr(&unsupported_test_gap_format).contains("only json is supported"));
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

fn run_cli_owned(args: &[String]) -> Output {
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

fn test_gap_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/test-gap.input.example.json")
}

fn semantic_proof_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/semantic-proof.input.example.json")
}

fn semantic_artifact_command(artifact: &Path, output: Option<&Path>) -> Vec<String> {
    let mut args = vec![
        "semantic-proof",
        "input",
        "from-artifact",
        "--artifact",
        artifact.to_str().expect("artifact path should be utf-8"),
        "--backend",
        "kani",
        "--backend-version",
        "1.0.0",
        "--theorem-id",
        "theorem:semantic:pricing",
        "--theorem-summary",
        "Pricing typed signature is preserved.",
        "--law-id",
        "law:semantic:signature-preserved",
        "--law-summary",
        "Public typed signature is preserved.",
        "--morphism-id",
        "morphism:semantic:pricing-signature",
        "--morphism-type",
        "typed_signature_preservation",
        "--base-cell",
        "cell:semantic:pricing:base",
        "--base-label",
        "base calculate_discount MIR",
        "--head-cell",
        "cell:semantic:pricing:head",
        "--head-label",
        "head calculate_discount MIR",
        "--format",
        "json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    if let Some(output) = output {
        args.push("--output".to_owned());
        args.push(
            output
                .to_str()
                .expect("output path should be utf-8")
                .to_owned(),
        );
    }
    args
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

fn write_test_gap_structural_git_fixture() -> PathBuf {
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
        "tools/highergraphen-cli/src/main.rs",
        "fn parse_test_gap_detect() {}\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(&repository, &["commit", "-m", "base"]);

    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/main.rs",
        "fn parse_test_gap_detect() {}\nfn run_test_gap_detect_command() {}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/test_gap_git.rs",
        "pub(crate) fn input_from_git() {}\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/lib.rs",
        "pub use workflows::test_gap::run_test_gap_detect;\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/workflows/mod.rs",
        "pub mod test_gap;\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/workflows/test_gap.rs",
        "pub fn run_test_gap_detect() {}\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/test_gap_reports.rs",
        "pub struct TestGapInputDocument;\npub struct TestGapReport;\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/src/reports.rs",
        "pub struct ReportEnvelope<T, U, V>(T, U, V);\n",
    );
    write_repo_file(
        &repository,
        "schemas/inputs/test-gap.input.schema.json",
        "{\"$id\":\"highergraphen.test_gap.input.v1\",\"type\":\"object\",\"properties\":{\"schema\":{\"const\":\"highergraphen.test_gap.input.v1\"}}}\n",
    );
    write_repo_file(
        &repository,
        "schemas/reports/test-gap.report.schema.json",
        "{\"$id\":\"highergraphen.test_gap.report.v1\",\"type\":\"object\",\"properties\":{\"schema\":{\"const\":\"highergraphen.test_gap.report.v1\"}}}\n",
    );
    write_repo_file(
        &repository,
        "schemas/inputs/test-gap.input.example.json",
        "{\"schema\":\"highergraphen.test_gap.input.v1\"}\n",
    );
    write_repo_file(
        &repository,
        "schemas/reports/test-gap.report.example.json",
        "{\"schema\":\"highergraphen.test_gap.report.v1\"}\n",
    );
    write_repo_file(
        &repository,
        "scripts/validate-json-contracts.py",
        "def main():\n    return 0\n",
    );
    write_repo_file(
        &repository,
        "crates/higher-graphen-runtime/tests/test_gap.rs",
        "#[test]\nfn test_gap_runtime_contract() {}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/tests/command.rs",
        "#[test]\nfn test_gap_detect_command() {}\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(
        &repository,
        &["commit", "-m", "add structural test-gap surface"],
    );

    repository
}

fn write_semantic_proof_structural_git_fixture() -> PathBuf {
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
        "tools/highergraphen-cli/src/main.rs",
        "fn parse_semantic_proof_verify() {}\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(&repository, &["commit", "-m", "base"]);

    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/main.rs",
        "fn parse_semantic_proof_verify() {}\nfn parse_semantic_proof_input_from_artifact() {}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/src/semantic_proof_artifact.rs",
        "pub(crate) struct ArtifactInputRequest;\npub(crate) fn input_from_artifact() {}\nfn required_string() {}\n",
    );
    write_repo_file(
        &repository,
        "tools/highergraphen-cli/tests/command.rs",
        "#[test]\nfn semantic_proof_input_from_artifact_emits_proved_input_and_verifies() {}\n#[test]\nfn semantic_proof_input_from_artifact_emits_counterexample_input_and_verifies() {}\n",
    );
    write_repo_file(
        &repository,
        "docs/cli/highergraphen.md",
        "highergraphen semantic-proof input from-artifact\n",
    );
    run_git(&repository, &["add", "."]);
    run_git(
        &repository,
        &["commit", "-m", "add semantic proof artifact adapter"],
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
