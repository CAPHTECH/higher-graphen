use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
pub const INPUT_LIFT_REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
pub const FEED_READER_REPORT_SCHEMA: &str = "highergraphen.feed.reader.report.v1";
pub const DDD_REVIEW_INPUT_SCHEMA: &str = "highergraphen.ddd_review.input.v1";
pub const DDD_REVIEW_REPORT_SCHEMA: &str = "highergraphen.ddd_review.report.v1";
pub const PR_REVIEW_TARGET_REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
pub const CORRESPONDENCE_DETECTION_INPUT_SCHEMA: &str =
    "highergraphen.correspondence.detection.input.v1";
pub const CORRESPONDENCE_EXPLANATION_SCHEMA: &str = "highergraphen.correspondence.explanation.v1";
pub const CORRESPONDENCE_PROJECTION_SCHEMA: &str = "highergraphen.correspondence.projection.v1";
pub const TEST_GAP_INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
pub const TEST_GAP_REPORT_SCHEMA: &str = "highergraphen.test_gap.report.v1";
pub const RUST_TEST_SEMANTICS_SCHEMA: &str = "highergraphen.rust_test_semantics.input.v1";
pub const TEST_SEMANTICS_INTERPRETATION_SCHEMA: &str =
    "highergraphen.test_semantics.interpretation.v1";
pub const TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA: &str =
    "highergraphen.test_semantics.interpretation_review.report.v1";
pub const TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA: &str =
    "highergraphen.test_semantics.verification.report.v1";
pub const TEST_SEMANTICS_GAP_REPORT_SCHEMA: &str = "highergraphen.test_semantics.gap.report.v1";
pub const SEMANTIC_PROOF_INPUT_SCHEMA: &str = "highergraphen.semantic_proof.input.v1";
pub const SEMANTIC_PROOF_REPORT_SCHEMA: &str = "highergraphen.semantic_proof.report.v1";
pub const COMPLETION_REVIEW_REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
pub const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
pub const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";
static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_highergraphen"))
        .args(args)
        .output()
        .expect("run highergraphen CLI")
}

pub fn run_cli_owned(args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_highergraphen"))
        .args(args)
        .output()
        .expect("run highergraphen CLI")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

pub fn unique_temp_dir() -> PathBuf {
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

pub fn input_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/architecture-lift.input.example.json")
}

pub fn resolved_input_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/architecture-lift.resolved.input.example.json")
}

pub fn feed_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/feed-lift.input.example.json")
}

pub fn pr_review_target_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/pr-review-target.input.example.json")
}

pub fn correspondence_detection_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/correspondence-detection.input.example.json")
}

pub fn correspondence_cell_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/highergraphen/correspondence.graph.example.json")
}

pub fn ddd_review_input_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/ddd-review.input.example.json")
}

pub fn ddd_case_space_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/casegraphen/ddd/domain-model-design/sales-billing-customer.case.space.json")
}

pub fn test_gap_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/test-gap.input.example.json")
}

pub fn semantic_proof_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/inputs/semantic-proof.input.example.json")
}

pub fn test_semantics_verification_report_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas/reports/test-semantics-verification.report.example.json")
}

pub fn semantic_artifact_command(artifact: &Path, output: Option<&Path>) -> Vec<String> {
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

pub fn write_smoke_report(directory: &Path) -> PathBuf {
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
