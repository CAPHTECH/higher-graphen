//! Command-line entry point for HigherGraphen workflows.

mod pr_review_git;
mod pr_review_structural;
mod rust_test_semantics;
mod semantic_proof_artifact;
mod semantic_proof_backend;
mod semantic_proof_reinput;
mod test_gap_evidence;
mod test_gap_git;

use higher_graphen_core::Id;
use higher_graphen_runtime::{
    run_architecture_direct_db_access_smoke, run_architecture_input_lift, run_completion_review,
    run_feed_reader, run_pr_review_target_recommend, run_semantic_proof_verify,
    run_test_gap_detect, ArchitectureInputLiftDocument, CompletionReviewDecision,
    CompletionReviewRequest, CompletionReviewSnapshot, CompletionReviewSourceReport,
    FeedReaderInputDocument, PrReviewTargetInputDocument, RuntimeError, SemanticProofInputDocument,
    SemanticProofReport, TestGapInputDocument,
};
use rust_test_semantics::RustTestSemanticsPathRequest;
use serde_json::Value;
use std::{
    env,
    ffi::OsString,
    fmt, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage:
  highergraphen version
  highergraphen --version
  highergraphen architecture smoke direct-db-access --format json [--output <path>]
  highergraphen architecture input lift --input <path> --format json [--output <path>]
  highergraphen feed reader run --input <path> --format json [--output <path>]
  highergraphen pr-review input from-git --base <ref> --head <ref> --format json [--repo <path>] [--output <path>]
  highergraphen pr-review targets recommend --input <path> --format json [--output <path>]
  highergraphen test-gap input from-git --base <ref> --head <ref> --format json [--repo <path>] [--output <path>]
  highergraphen test-gap input from-path --path <path> [--path <path> ...] [--include-tests] --format json [--repo <path>] [--output <path>]
  highergraphen test-gap evidence from-test-run --input <path> --test-run <path> --format json [--output <path>]
  highergraphen test-gap detect --input <path> --format json [--output <path>]
  highergraphen rust-test semantics from-path --path <path> [--path <path> ...] --format json [--repo <path>] [--output <path>]
  highergraphen semantic-proof backend run --backend <name> --backend-version <version> --command <path> [--arg <text> ...] [--input <path>] --format json [--output <path>]
  highergraphen semantic-proof input from-artifact --artifact <path> --backend <name> --backend-version <version> --theorem-id <id> --theorem-summary <text> --law-id <id> --law-summary <text> --morphism-id <id> --morphism-type <text> --base-cell <id> --base-label <text> --head-cell <id> --head-label <text> --format json [--output <path>]
  highergraphen semantic-proof input from-report --report <path> --format json [--output <path>]
  highergraphen semantic-proof verify --input <path> --format json [--output <path>]
  highergraphen completion review accept --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]
  highergraphen completion review reject --input <path> --candidate <id> --reviewer <id> --reason <text> --format json [--reviewed-at <text>] [--output <path>]";

fn main() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), CliError> {
    let command = Command::parse(args)?;
    if matches!(&command, Command::Version) {
        println!("highergraphen {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let output = command.output().cloned();
    let json = command.run_json()?;

    match output {
        Some(path) => fs::write(path, json).map_err(CliError::write_output),
        None => {
            println!("{json}");
            Ok(())
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Version,
    ArchitectureSmokeDirectDbAccess {
        output: Option<PathBuf>,
    },
    ArchitectureInputLift {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    FeedReaderRun {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    PrReviewInputFromGit {
        repo: PathBuf,
        base: String,
        head: String,
        output: Option<PathBuf>,
    },
    PrReviewTargetsRecommend {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    TestGapDetect {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    TestGapInputFromGit {
        repo: PathBuf,
        base: String,
        head: String,
        output: Option<PathBuf>,
    },
    TestGapInputFromPath {
        repo: PathBuf,
        paths: Vec<PathBuf>,
        include_tests: bool,
        output: Option<PathBuf>,
    },
    TestGapEvidenceFromTestRun {
        input: PathBuf,
        test_run: PathBuf,
        output: Option<PathBuf>,
    },
    RustTestSemanticsFromPath {
        repo: PathBuf,
        paths: Vec<PathBuf>,
        output: Option<PathBuf>,
    },
    SemanticProofVerify {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    SemanticProofBackendRun {
        backend: String,
        backend_version: String,
        command: PathBuf,
        args: Vec<String>,
        input: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    SemanticProofInputFromArtifact {
        artifact: PathBuf,
        backend: String,
        backend_version: String,
        theorem_id: String,
        theorem_summary: String,
        law_id: String,
        law_summary: String,
        morphism_id: String,
        morphism_type: String,
        base_cell: String,
        base_label: String,
        head_cell: String,
        head_label: String,
        output: Option<PathBuf>,
    },
    SemanticProofInputFromReport {
        report: PathBuf,
        output: Option<PathBuf>,
    },
    CompletionReview {
        decision: CompletionReviewDecision,
        input: PathBuf,
        candidate_id: String,
        reviewer_id: String,
        reason: String,
        reviewed_at: Option<String>,
        output: Option<PathBuf>,
    },
}

impl Command {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = args.into_iter();
        let root = required_segment(&mut args, "command")?;

        match root.to_str() {
            Some("version") | Some("--version") | Some("-V") => Self::parse_version(args),
            Some("architecture") => Self::parse_architecture(args),
            Some("feed") => Self::parse_feed(args),
            Some("pr-review") => Self::parse_pr_review(args),
            Some("test-gap") => Self::parse_test_gap(args),
            Some("rust-test") => Self::parse_rust_test(args),
            Some("semantic-proof") => Self::parse_semantic_proof(args),
            Some("completion") => Self::parse_completion(args),
            Some(_) | None => Err(CliError::usage("unsupported command segment")),
        }
    }

    fn parse_version(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        match args.next() {
            Some(arg) => Err(CliError::usage(format!(
                "unsupported argument {arg:?} for version"
            ))),
            None => Ok(Self::Version),
        }
    }

    fn parse_architecture(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "architecture command")?;
        match segment.to_str() {
            Some("smoke") => Self::parse_smoke(args),
            Some("input") => Self::parse_input(args),
            Some(_) | None => Err(CliError::usage("unsupported architecture command segment")),
        }
    }

    fn parse_smoke(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "direct-db-access")?;
        let options = ReportOptions::parse(args, false)?;
        Ok(Self::ArchitectureSmokeDirectDbAccess {
            output: options.output,
        })
    }

    fn parse_input(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "lift")?;
        let options = ReportOptions::parse(args, true)?;
        let input = options
            .input
            .ok_or_else(|| CliError::usage("--input <path> is required"))?;
        Ok(Self::ArchitectureInputLift {
            input,
            output: options.output,
        })
    }

    fn parse_feed(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "reader")?;
        require_token(&mut args, "run")?;
        let options = ReportOptions::parse(args, true)?;
        let input = options
            .input
            .ok_or_else(|| CliError::usage("--input <path> is required"))?;
        Ok(Self::FeedReaderRun {
            input,
            output: options.output,
        })
    }

    fn parse_pr_review(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "pr-review command")?;
        match segment.to_str() {
            Some("input") => Self::parse_pr_review_input(args),
            Some("targets") => Self::parse_pr_review_targets(args),
            Some(_) | None => Err(CliError::usage("unsupported pr-review command segment")),
        }
    }

    fn parse_pr_review_input(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "from-git")?;
        let options = GitInputOptions::parse(args)?;
        Ok(Self::PrReviewInputFromGit {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            base: options
                .base
                .ok_or_else(|| CliError::usage("--base <ref> is required"))?,
            head: options
                .head
                .ok_or_else(|| CliError::usage("--head <ref> is required"))?,
            output: options.output,
        })
    }

    fn parse_pr_review_targets(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "recommend")?;
        let options = ReportOptions::parse(args, true)?;
        let input = options
            .input
            .ok_or_else(|| CliError::usage("--input <path> is required"))?;
        Ok(Self::PrReviewTargetsRecommend {
            input,
            output: options.output,
        })
    }

    fn parse_test_gap(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "test-gap command")?;
        match segment.to_str() {
            Some("input") => Self::parse_test_gap_input(args),
            Some("evidence") => Self::parse_test_gap_evidence(args),
            Some("detect") => {
                let options = ReportOptions::parse(args, true)?;
                let input = options
                    .input
                    .ok_or_else(|| CliError::usage("--input <path> is required"))?;
                Ok(Self::TestGapDetect {
                    input,
                    output: options.output,
                })
            }
            Some(_) | None => Err(CliError::usage("unsupported test-gap command segment")),
        }
    }

    fn parse_test_gap_evidence(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "test-gap evidence command")?;
        match segment.to_str() {
            Some("from-test-run") => {
                let options = TestRunEvidenceOptions::parse(args)?;
                Ok(Self::TestGapEvidenceFromTestRun {
                    input: options
                        .input
                        .ok_or_else(|| CliError::usage("--input <path> is required"))?,
                    test_run: options
                        .test_run
                        .ok_or_else(|| CliError::usage("--test-run <path> is required"))?,
                    output: options.output,
                })
            }
            Some(_) | None => Err(CliError::usage(
                "unsupported test-gap evidence command segment",
            )),
        }
    }

    fn parse_test_gap_input(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "test-gap input command")?;
        match segment.to_str() {
            Some("from-git") => {
                let options = GitInputOptions::parse(args)?;
                Ok(Self::TestGapInputFromGit {
                    repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
                    base: options
                        .base
                        .ok_or_else(|| CliError::usage("--base <ref> is required"))?,
                    head: options
                        .head
                        .ok_or_else(|| CliError::usage("--head <ref> is required"))?,
                    output: options.output,
                })
            }
            Some("from-path") => {
                let options = PathInputOptions::parse(args)?;
                if options.paths.is_empty() {
                    return Err(CliError::usage("--path <path> is required"));
                }
                Ok(Self::TestGapInputFromPath {
                    repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
                    paths: options.paths,
                    include_tests: options.include_tests,
                    output: options.output,
                })
            }
            Some(_) | None => Err(CliError::usage(
                "unsupported test-gap input command segment",
            )),
        }
    }

    fn parse_rust_test(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "semantics")?;
        require_token(&mut args, "from-path")?;
        let options = PathInputOptions::parse(args)?;
        if options.include_tests {
            return Err(CliError::usage(
                "--include-tests is not supported for rust-test semantics from-path",
            ));
        }
        if options.paths.is_empty() {
            return Err(CliError::usage("--path <path> is required"));
        }
        Ok(Self::RustTestSemanticsFromPath {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            paths: options.paths,
            output: options.output,
        })
    }

    fn parse_semantic_proof(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "semantic-proof command")?;
        match segment.to_str() {
            Some("backend") => Self::parse_semantic_proof_backend(args),
            Some("input") => Self::parse_semantic_proof_input(args),
            Some("verify") => {
                let options = ReportOptions::parse(args, true)?;
                let input = options
                    .input
                    .ok_or_else(|| CliError::usage("--input <path> is required"))?;
                Ok(Self::SemanticProofVerify {
                    input,
                    output: options.output,
                })
            }
            Some(_) | None => Err(CliError::usage(
                "unsupported semantic-proof command segment",
            )),
        }
    }

    fn parse_semantic_proof_backend(
        mut args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        require_token(&mut args, "run")?;
        let options = SemanticProofBackendOptions::parse(args)?;
        Ok(Self::SemanticProofBackendRun {
            backend: options
                .backend
                .ok_or_else(|| CliError::usage("--backend <name> is required"))?,
            backend_version: options
                .backend_version
                .ok_or_else(|| CliError::usage("--backend-version <version> is required"))?,
            command: options
                .command
                .ok_or_else(|| CliError::usage("--command <path> is required"))?,
            args: options.args,
            input: options.input,
            output: options.output,
        })
    }

    fn parse_semantic_proof_input(
        mut args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "semantic-proof input command")?;
        match segment.to_str() {
            Some("from-artifact") => {
                let options = SemanticProofArtifactOptions::parse(args)?;
                Ok(Self::SemanticProofInputFromArtifact {
                    artifact: options
                        .artifact
                        .ok_or_else(|| CliError::usage("--artifact <path> is required"))?,
                    backend: options
                        .backend
                        .ok_or_else(|| CliError::usage("--backend <name> is required"))?,
                    backend_version: options.backend_version.ok_or_else(|| {
                        CliError::usage("--backend-version <version> is required")
                    })?,
                    theorem_id: options
                        .theorem_id
                        .ok_or_else(|| CliError::usage("--theorem-id <id> is required"))?,
                    theorem_summary: options
                        .theorem_summary
                        .ok_or_else(|| CliError::usage("--theorem-summary <text> is required"))?,
                    law_id: options
                        .law_id
                        .ok_or_else(|| CliError::usage("--law-id <id> is required"))?,
                    law_summary: options
                        .law_summary
                        .ok_or_else(|| CliError::usage("--law-summary <text> is required"))?,
                    morphism_id: options
                        .morphism_id
                        .ok_or_else(|| CliError::usage("--morphism-id <id> is required"))?,
                    morphism_type: options
                        .morphism_type
                        .ok_or_else(|| CliError::usage("--morphism-type <text> is required"))?,
                    base_cell: options
                        .base_cell
                        .ok_or_else(|| CliError::usage("--base-cell <id> is required"))?,
                    base_label: options
                        .base_label
                        .ok_or_else(|| CliError::usage("--base-label <text> is required"))?,
                    head_cell: options
                        .head_cell
                        .ok_or_else(|| CliError::usage("--head-cell <id> is required"))?,
                    head_label: options
                        .head_label
                        .ok_or_else(|| CliError::usage("--head-label <text> is required"))?,
                    output: options.output,
                })
            }
            Some("from-report") => {
                let options = SemanticProofReportInputOptions::parse(args)?;
                Ok(Self::SemanticProofInputFromReport {
                    report: options
                        .report
                        .ok_or_else(|| CliError::usage("--report <path> is required"))?,
                    output: options.output,
                })
            }
            Some(_) | None => Err(CliError::usage(
                "unsupported semantic-proof input command segment",
            )),
        }
    }

    fn parse_completion(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "review")?;
        let decision = match required_segment(&mut args, "completion review action")?.to_str() {
            Some("accept") => CompletionReviewDecision::Accepted,
            Some("reject") => CompletionReviewDecision::Rejected,
            Some(_) | None => return Err(CliError::usage("unsupported completion review action")),
        };
        let options = ReviewOptions::parse(args)?;

        Ok(Self::CompletionReview {
            decision,
            input: options
                .input
                .ok_or_else(|| CliError::usage("--input <path> is required"))?,
            candidate_id: options
                .candidate_id
                .ok_or_else(|| CliError::usage("--candidate <id> is required"))?,
            reviewer_id: options
                .reviewer_id
                .ok_or_else(|| CliError::usage("--reviewer <id> is required"))?,
            reason: options
                .reason
                .ok_or_else(|| CliError::usage("--reason <text> is required"))?,
            reviewed_at: options.reviewed_at,
            output: options.output,
        })
    }

    fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::Version => None,
            Self::ArchitectureSmokeDirectDbAccess { output }
            | Self::ArchitectureInputLift { output, .. }
            | Self::FeedReaderRun { output, .. }
            | Self::PrReviewInputFromGit { output, .. }
            | Self::PrReviewTargetsRecommend { output, .. }
            | Self::TestGapInputFromGit { output, .. }
            | Self::TestGapInputFromPath { output, .. }
            | Self::TestGapEvidenceFromTestRun { output, .. }
            | Self::TestGapDetect { output, .. }
            | Self::RustTestSemanticsFromPath { output, .. }
            | Self::SemanticProofBackendRun { output, .. }
            | Self::SemanticProofInputFromArtifact { output, .. }
            | Self::SemanticProofInputFromReport { output, .. }
            | Self::SemanticProofVerify { output, .. }
            | Self::CompletionReview { output, .. } => output.as_ref(),
        }
    }

    fn run_json(&self) -> Result<String, CliError> {
        match self {
            Self::Version => unreachable!("version command is handled before JSON execution"),
            Self::ArchitectureSmokeDirectDbAccess { .. } => {
                let report = run_architecture_direct_db_access_smoke()?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::ArchitectureInputLift { input, .. } => {
                let document = read_input_document(input)?;
                let report = run_architecture_input_lift(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::FeedReaderRun { input, .. } => {
                let document = read_feed_reader_input_document(input)?;
                let report = run_feed_reader(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::PrReviewInputFromGit {
                repo, base, head, ..
            } => {
                let document = pr_review_git::input_from_git(pr_review_git::GitInputRequest {
                    repo: repo.clone(),
                    base: base.clone(),
                    head: head.clone(),
                })
                .map_err(CliError::GitInput)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::PrReviewTargetsRecommend { input, .. } => {
                let document = read_pr_review_target_input_document(input)?;
                let report = run_pr_review_target_recommend(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::TestGapDetect { input, .. } => {
                let document = read_test_gap_input_document(input)?;
                let report = run_test_gap_detect(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::TestGapInputFromGit {
                repo, base, head, ..
            } => {
                let document = test_gap_git::input_from_git(test_gap_git::GitInputRequest {
                    repo: repo.clone(),
                    base: base.clone(),
                    head: head.clone(),
                })
                .map_err(CliError::GitInput)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::TestGapInputFromPath {
                repo,
                paths,
                include_tests,
                ..
            } => {
                let document = test_gap_git::input_from_path(test_gap_git::PathInputRequest {
                    repo: repo.clone(),
                    paths: paths.clone(),
                    include_tests: *include_tests,
                })
                .map_err(CliError::GitInput)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::TestGapEvidenceFromTestRun {
                input, test_run, ..
            } => {
                let input_document = read_test_gap_input_document(input)?;
                let document = test_gap_evidence::input_from_test_run(
                    test_gap_evidence::TestRunEvidenceRequest {
                        input: input_document,
                        test_run: test_run.clone(),
                    },
                )
                .map_err(CliError::TestGapEvidence)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::RustTestSemanticsFromPath { repo, paths, .. } => {
                let document =
                    rust_test_semantics::document_from_path(RustTestSemanticsPathRequest {
                        repo: repo.clone(),
                        paths: paths.clone(),
                    })
                    .map_err(CliError::RustTestSemantics)?;
                serde_json::to_string(&document.to_json_value())
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::SemanticProofVerify { input, .. } => {
                let document = read_semantic_proof_input_document(input)?;
                let report = run_semantic_proof_verify(document)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::SemanticProofBackendRun {
                backend,
                backend_version,
                command,
                args,
                input,
                ..
            } => {
                let artifact = semantic_proof_backend::run_backend(
                    semantic_proof_backend::BackendRunRequest {
                        backend: backend.clone(),
                        backend_version: backend_version.clone(),
                        command: command.clone(),
                        args: args.clone(),
                        input: input.clone(),
                    },
                )
                .map_err(CliError::SemanticProofArtifact)?;
                serde_json::to_string(&artifact)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::SemanticProofInputFromArtifact {
                artifact,
                backend,
                backend_version,
                theorem_id,
                theorem_summary,
                law_id,
                law_summary,
                morphism_id,
                morphism_type,
                base_cell,
                base_label,
                head_cell,
                head_label,
                ..
            } => {
                let document = semantic_proof_artifact::input_from_artifact(
                    semantic_proof_artifact::ArtifactInputRequest {
                        artifact: artifact.clone(),
                        backend: backend.clone(),
                        backend_version: backend_version.clone(),
                        theorem_id: theorem_id.clone(),
                        theorem_summary: theorem_summary.clone(),
                        law_id: law_id.clone(),
                        law_summary: law_summary.clone(),
                        morphism_id: morphism_id.clone(),
                        morphism_type: morphism_type.clone(),
                        base_cell: base_cell.clone(),
                        base_label: base_label.clone(),
                        head_cell: head_cell.clone(),
                        head_label: head_label.clone(),
                    },
                )
                .map_err(CliError::SemanticProofArtifact)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::SemanticProofInputFromReport { report, .. } => {
                let semantic_report = read_semantic_proof_report(report)?;
                let document = semantic_proof_reinput::input_from_report(semantic_report)
                    .map_err(CliError::SemanticProofArtifact)?;
                serde_json::to_string(&document)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
            Self::CompletionReview {
                decision,
                input,
                candidate_id,
                reviewer_id,
                reason,
                reviewed_at,
                ..
            } => {
                let snapshot = read_completion_review_snapshot(input)?;
                let mut request = CompletionReviewRequest::new(
                    Id::new(candidate_id.clone())?,
                    *decision,
                    Id::new(reviewer_id.clone())?,
                    reason.clone(),
                )?;
                if let Some(reviewed_at) = reviewed_at {
                    request = request.with_reviewed_at(reviewed_at.clone())?;
                }
                let report = run_completion_review(snapshot, request)?;
                serde_json::to_string(&report)
                    .map_err(|error| RuntimeError::serialization(error.to_string()).into())
            }
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct GitInputOptions {
    repo: Option<PathBuf>,
    base: Option<String>,
    head: Option<String>,
    output: Option<PathBuf>,
}

impl GitInputOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--repo" {
                options.repo = Some(require_path(&mut args, "--repo")?);
            } else if arg == "--base" {
                options.base = Some(require_string(&mut args, "--base")?);
            } else if arg == "--head" {
                options.head = Some(require_string(&mut args, "--head")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct PathInputOptions {
    repo: Option<PathBuf>,
    paths: Vec<PathBuf>,
    include_tests: bool,
    output: Option<PathBuf>,
}

impl PathInputOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--repo" {
                options.repo = Some(require_path(&mut args, "--repo")?);
            } else if arg == "--path" {
                options.paths.push(require_path(&mut args, "--path")?);
            } else if arg == "--include-tests" {
                options.include_tests = true;
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct TestRunEvidenceOptions {
    input: Option<PathBuf>,
    test_run: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl TestRunEvidenceOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--test-run" {
                options.test_run = Some(require_path(&mut args, "--test-run")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct SemanticProofBackendOptions {
    backend: Option<String>,
    backend_version: Option<String>,
    command: Option<PathBuf>,
    args: Vec<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl SemanticProofBackendOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--backend" {
                options.backend = Some(require_string(&mut args, "--backend")?);
            } else if arg == "--backend-version" {
                options.backend_version = Some(require_string(&mut args, "--backend-version")?);
            } else if arg == "--command" {
                options.command = Some(require_path(&mut args, "--command")?);
            } else if arg == "--arg" {
                options.args.push(require_string(&mut args, "--arg")?);
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct SemanticProofArtifactOptions {
    artifact: Option<PathBuf>,
    backend: Option<String>,
    backend_version: Option<String>,
    theorem_id: Option<String>,
    theorem_summary: Option<String>,
    law_id: Option<String>,
    law_summary: Option<String>,
    morphism_id: Option<String>,
    morphism_type: Option<String>,
    base_cell: Option<String>,
    base_label: Option<String>,
    head_cell: Option<String>,
    head_label: Option<String>,
    output: Option<PathBuf>,
}

impl SemanticProofArtifactOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--artifact" {
                options.artifact = Some(require_path(&mut args, "--artifact")?);
            } else if arg == "--backend" {
                options.backend = Some(require_string(&mut args, "--backend")?);
            } else if arg == "--backend-version" {
                options.backend_version = Some(require_string(&mut args, "--backend-version")?);
            } else if arg == "--theorem-id" {
                options.theorem_id = Some(require_string(&mut args, "--theorem-id")?);
            } else if arg == "--theorem-summary" {
                options.theorem_summary = Some(require_string(&mut args, "--theorem-summary")?);
            } else if arg == "--law-id" {
                options.law_id = Some(require_string(&mut args, "--law-id")?);
            } else if arg == "--law-summary" {
                options.law_summary = Some(require_string(&mut args, "--law-summary")?);
            } else if arg == "--morphism-id" {
                options.morphism_id = Some(require_string(&mut args, "--morphism-id")?);
            } else if arg == "--morphism-type" {
                options.morphism_type = Some(require_string(&mut args, "--morphism-type")?);
            } else if arg == "--base-cell" {
                options.base_cell = Some(require_string(&mut args, "--base-cell")?);
            } else if arg == "--base-label" {
                options.base_label = Some(require_string(&mut args, "--base-label")?);
            } else if arg == "--head-cell" {
                options.head_cell = Some(require_string(&mut args, "--head-cell")?);
            } else if arg == "--head-label" {
                options.head_label = Some(require_string(&mut args, "--head-label")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct SemanticProofReportInputOptions {
    report: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl SemanticProofReportInputOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--report" {
                options.report = Some(require_path(&mut args, "--report")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ReportOptions {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl ReportOptions {
    fn parse(args: impl Iterator<Item = OsString>, allow_input: bool) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else if arg == "--input" && allow_input {
                options.input = Some(require_path(&mut args, "--input")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct ReviewOptions {
    input: Option<PathBuf>,
    candidate_id: Option<String>,
    reviewer_id: Option<String>,
    reason: Option<String>,
    reviewed_at: Option<String>,
    output: Option<PathBuf>,
}

impl ReviewOptions {
    fn parse(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let mut format_seen = false;
        let mut options = Self::default();

        let mut args = args;
        while let Some(arg) = args.next() {
            if arg == "--format" {
                require_json_format(&mut args)?;
                format_seen = true;
            } else if arg == "--input" {
                options.input = Some(require_path(&mut args, "--input")?);
            } else if arg == "--candidate" {
                options.candidate_id = Some(require_string(&mut args, "--candidate")?);
            } else if arg == "--reviewer" {
                options.reviewer_id = Some(require_string(&mut args, "--reviewer")?);
            } else if arg == "--reason" {
                options.reason = Some(require_string(&mut args, "--reason")?);
            } else if arg == "--reviewed-at" {
                options.reviewed_at = Some(require_string(&mut args, "--reviewed-at")?);
            } else if arg == "--output" {
                options.output = Some(require_path(&mut args, "--output")?);
            } else {
                return Err(CliError::usage(format!("unsupported argument {arg:?}")));
            }
        }

        if !format_seen {
            return Err(CliError::usage("--format json is required"));
        }

        Ok(options)
    }
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Runtime(RuntimeError),
    InputRead {
        path: PathBuf,
        source: std::io::Error,
    },
    InputParse {
        path: PathBuf,
        source: serde_json::Error,
    },
    InputContract {
        path: PathBuf,
        reason: String,
    },
    GitInput(String),
    TestGapEvidence(String),
    RustTestSemantics(String),
    SemanticProofArtifact(String),
    Output(std::io::Error),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    fn write_output(error: std::io::Error) -> Self {
        Self::Output(error)
    }
}

impl From<RuntimeError> for CliError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<higher_graphen_core::CoreError> for CliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Runtime(RuntimeError::from(error))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::InputRead { path, source } => {
                write!(
                    formatter,
                    "failed to read input {}: {source}",
                    path.display()
                )
            }
            Self::InputParse { path, source } => {
                write!(
                    formatter,
                    "failed to parse input {}: {source}",
                    path.display()
                )
            }
            Self::InputContract { path, reason } => {
                write!(formatter, "invalid input {}: {reason}", path.display())
            }
            Self::GitInput(message) => write!(formatter, "failed to build git input: {message}"),
            Self::TestGapEvidence(message) => {
                write!(formatter, "failed to build test-gap evidence: {message}")
            }
            Self::RustTestSemantics(message) => {
                write!(formatter, "failed to build rust test semantics: {message}")
            }
            Self::SemanticProofArtifact(message) => {
                write!(formatter, "failed to build semantic proof input: {message}")
            }
            Self::Output(error) => write!(formatter, "failed to write output: {error}"),
        }
    }
}

impl std::error::Error for CliError {}

fn require_token(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<(), CliError> {
    match required_segment(args, expected)? {
        arg if arg == expected => Ok(()),
        arg => Err(CliError::usage(format!(
            "unsupported command segment {arg:?}; expected {expected:?}"
        ))),
    }
}

fn required_segment(
    args: &mut impl Iterator<Item = OsString>,
    expected: &'static str,
) -> Result<OsString, CliError> {
    match args.next() {
        Some(arg) => Ok(arg),
        None => Err(CliError::usage(format!(
            "missing command segment {expected:?}"
        ))),
    }
}

fn require_json_format(args: &mut impl Iterator<Item = OsString>) -> Result<(), CliError> {
    match args.next() {
        Some(arg) if arg == "json" => Ok(()),
        Some(arg) => Err(CliError::usage(format!(
            "unsupported format {arg:?}; only json is supported"
        ))),
        None => Err(CliError::usage("missing value for --format")),
    }
}

fn require_path(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<PathBuf, CliError> {
    match args.next() {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        Some(_) => Err(CliError::usage(format!("empty path for {option}"))),
        None => Err(CliError::usage(format!("missing value for {option}"))),
    }
}

fn require_string(
    args: &mut impl Iterator<Item = OsString>,
    option: &'static str,
) -> Result<String, CliError> {
    match args.next() {
        Some(value) if !value.is_empty() => value
            .into_string()
            .map_err(|value| CliError::usage(format!("non-utf8 value for {option}: {value:?}"))),
        Some(_) => Err(CliError::usage(format!("empty value for {option}"))),
        None => Err(CliError::usage(format!("missing value for {option}"))),
    }
}

fn read_input_document(path: &Path) -> Result<ArchitectureInputLiftDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_feed_reader_input_document(path: &Path) -> Result<FeedReaderInputDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_pr_review_target_input_document(
    path: &Path,
) -> Result<PrReviewTargetInputDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_test_gap_input_document(path: &Path) -> Result<TestGapInputDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_semantic_proof_input_document(path: &Path) -> Result<SemanticProofInputDocument, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_semantic_proof_report(path: &Path) -> Result<SemanticProofReport, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_completion_review_snapshot(path: &Path) -> Result<CompletionReviewSnapshot, CliError> {
    let value = read_json_value(path)?;
    if value.get("source_report").is_some() && value.get("completion_candidates").is_some() {
        return serde_json::from_value(value).map_err(|source| CliError::InputParse {
            path: path.to_owned(),
            source,
        });
    }

    snapshot_from_report_value(path, &value)
}

fn read_json_value(path: &Path) -> Result<Value, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn snapshot_from_report_value(
    path: &Path,
    value: &Value,
) -> Result<CompletionReviewSnapshot, CliError> {
    let candidates = dig_json(value, &["result", "completion_candidates"])
        .ok_or_else(|| input_contract(path, "missing result.completion_candidates"))?;
    let completion_candidates =
        serde_json::from_value(candidates.clone()).map_err(|source| CliError::InputParse {
            path: path.to_owned(),
            source,
        })?;

    Ok(CompletionReviewSnapshot {
        source_report: CompletionReviewSourceReport {
            schema: required_json_string(path, value, &["schema"])?,
            report_type: required_json_string(path, value, &["report_type"])?,
            report_version: required_json_u32(path, value, &["report_version"])?,
            command: required_json_string(path, value, &["metadata", "command"])?,
        },
        completion_candidates,
    })
}

fn dig_json<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn required_json_string(
    input_path: &Path,
    value: &Value,
    json_path: &[&str],
) -> Result<String, CliError> {
    match dig_json(value, json_path) {
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(text.clone()),
        Some(_) => Err(input_contract(
            input_path,
            format!("{} must be a non-empty string", json_path.join(".")),
        )),
        None => Err(input_contract(
            input_path,
            format!("missing {}", json_path.join(".")),
        )),
    }
}

fn required_json_u32(
    input_path: &Path,
    value: &Value,
    json_path: &[&str],
) -> Result<u32, CliError> {
    match dig_json(value, json_path).and_then(Value::as_u64) {
        Some(number) => u32::try_from(number).map_err(|_| {
            input_contract(
                input_path,
                format!("{} must fit in u32", json_path.join(".")),
            )
        }),
        None => Err(input_contract(
            input_path,
            format!("{} must be a non-negative integer", json_path.join(".")),
        )),
    }
}

fn input_contract(path: &Path, reason: impl Into<String>) -> CliError {
    CliError::InputContract {
        path: path.to_owned(),
        reason: reason.into(),
    }
}
