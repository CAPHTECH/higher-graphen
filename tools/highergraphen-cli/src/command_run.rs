use crate::{
    cli_error::CliError, command::Command, pr_review_git, rust_test_semantics,
    semantic_proof_artifact, semantic_proof_attach_artifact, semantic_proof_backend,
    semantic_proof_reinput, test_gap_evidence, test_gap_git, test_semantics_gap,
    test_semantics_interpretation, test_semantics_review, test_semantics_verification,
};
use higher_graphen_core::Id;
use higher_graphen_runtime::{
    ddd_input_from_case_space, run_architecture_direct_db_access_smoke,
    run_architecture_input_lift, run_completion_review, run_ddd_review, run_feed_reader,
    run_pr_review_target_recommend, run_semantic_proof_verify, run_test_gap_detect,
    ArchitectureInputLiftDocument, CompletionReviewDecision, CompletionReviewRequest,
    CompletionReviewSnapshot, CompletionReviewSourceReport, FeedReaderInputDocument,
    PrReviewTargetInputDocument, RuntimeError, SemanticProofInputDocument, TestGapInputDocument,
};
use rust_test_semantics::RustTestSemanticsPathRequest;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

macro_rules! serialize_json {
    ($value:expr) => {
        serde_json::to_string($value)
            .map_err(|error| RuntimeError::serialization(error.to_string()).into())
    };
}

impl Command {
    pub(crate) fn run_json(&self) -> Result<String, CliError> {
        match self {
            Self::Version => unreachable!("version command is handled before JSON execution"),
            Self::ArchitectureSmokeDirectDbAccess { .. }
            | Self::ArchitectureInputLift { .. }
            | Self::FeedReaderRun { .. }
            | Self::DddInputFromCaseSpace { .. }
            | Self::DddReview { .. }
            | Self::PrReviewInputFromGit { .. }
            | Self::PrReviewTargetsRecommend { .. } => self.run_primary_json(),
            Self::TestGapDetect { .. }
            | Self::TestGapInputFromGit { .. }
            | Self::TestGapInputFromPath { .. }
            | Self::TestGapEvidenceFromTestRun { .. }
            | Self::RustTestSemanticsFromPath { .. } => self.run_test_input_json(),
            Self::TestSemanticsInterpret { .. }
            | Self::TestSemanticsReview { .. }
            | Self::TestSemanticsVerify { .. }
            | Self::TestSemanticsGap { .. } => self.run_test_semantics_json(),
            Self::SemanticProofVerify { .. }
            | Self::SemanticProofBackendRun { .. }
            | Self::SemanticProofInputFromArtifact { .. }
            | Self::SemanticProofInputFromReport { .. }
            | Self::SemanticProofInputAttachArtifact { .. } => self.run_semantic_proof_json(),
            Self::CompletionReview { .. } => self.run_completion_review_json(),
        }
    }

    fn run_primary_json(&self) -> Result<String, CliError> {
        match self {
            Self::ArchitectureSmokeDirectDbAccess { .. } => architecture_smoke_json(),
            Self::ArchitectureInputLift { input, .. } => architecture_input_lift_json(input),
            Self::FeedReaderRun { input, .. } => feed_reader_json(input),
            Self::DddInputFromCaseSpace { case_space, .. } => {
                ddd_input_from_case_space_json(case_space)
            }
            Self::DddReview { input, .. } => ddd_review_json(input),
            Self::PrReviewInputFromGit {
                repo, base, head, ..
            } => pr_review_input_from_git_json(repo, base, head),
            Self::PrReviewTargetsRecommend { input, .. } => pr_review_targets_json(input),
            _ => unreachable!("primary dispatch helper received another variant"),
        }
    }

    fn run_test_input_json(&self) -> Result<String, CliError> {
        match self {
            Self::TestGapDetect { input, .. } => test_gap_detect_json(input),
            Self::TestGapInputFromGit {
                repo,
                base,
                head,
                binding_rules,
                ..
            } => test_gap_input_from_git_json(repo, base, head, binding_rules),
            Self::TestGapInputFromPath {
                repo,
                paths,
                include_tests,
                binding_rules,
                ..
            } => test_gap_input_from_path_json(repo, paths, *include_tests, binding_rules),
            Self::TestGapEvidenceFromTestRun {
                input, test_run, ..
            } => test_gap_evidence_json(input, test_run),
            Self::RustTestSemanticsFromPath {
                repo,
                paths,
                test_run,
                ..
            } => rust_test_semantics_json(repo, paths, test_run),
            _ => unreachable!("test input dispatch helper received another variant"),
        }
    }

    fn run_test_semantics_json(&self) -> Result<String, CliError> {
        match self {
            Self::TestSemanticsInterpret {
                input, interpreter, ..
            } => test_semantics_interpret_json(input, interpreter),
            Self::TestSemanticsReview {
                decision,
                input,
                candidate_id,
                reviewer_id,
                reason,
                ..
            } => test_semantics_review_json(decision, input, candidate_id, reviewer_id, reason),
            Self::TestSemanticsVerify {
                interpretation,
                review,
                test_run,
                ..
            } => test_semantics_verify_json(interpretation, review, test_run),
            Self::TestSemanticsGap {
                expected, verified, ..
            } => test_semantics_gap_json(expected, verified),
            _ => unreachable!("test semantics dispatch helper received another variant"),
        }
    }

    fn run_semantic_proof_json(&self) -> Result<String, CliError> {
        match self {
            Self::SemanticProofVerify { input, .. } => semantic_proof_verify_json(input),
            Self::SemanticProofBackendRun {
                backend,
                backend_version,
                command,
                args,
                input,
                ..
            } => semantic_proof_backend_json(backend, backend_version, command, args, input),
            Self::SemanticProofInputFromArtifact { .. } => semantic_proof_from_artifact_json(self),
            Self::SemanticProofInputFromReport { report, .. } => {
                semantic_proof_from_report_json(report)
            }
            Self::SemanticProofInputAttachArtifact {
                input,
                artifact,
                backend,
                backend_version,
                ..
            } => semantic_proof_attach_artifact_json(input, artifact, backend, backend_version),
            _ => unreachable!("semantic proof dispatch helper received another variant"),
        }
    }

    fn run_completion_review_json(&self) -> Result<String, CliError> {
        match self {
            Self::CompletionReview {
                decision,
                input,
                candidate_id,
                reviewer_id,
                reason,
                reviewed_at,
                ..
            } => completion_review_json(
                decision,
                input,
                candidate_id,
                reviewer_id,
                reason,
                reviewed_at,
            ),
            _ => unreachable!("completion review dispatch helper received another variant"),
        }
    }
}

fn architecture_smoke_json() -> Result<String, CliError> {
    let report = run_architecture_direct_db_access_smoke()?;
    serialize_json!(&report)
}

fn architecture_input_lift_json(input: &Path) -> Result<String, CliError> {
    let document = read_input_document(input)?;
    let report = run_architecture_input_lift(document)?;
    serialize_json!(&report)
}

fn feed_reader_json(input: &Path) -> Result<String, CliError> {
    let document = read_feed_reader_input_document(input)?;
    let report = run_feed_reader(document)?;
    serialize_json!(&report)
}

fn ddd_input_from_case_space_json(case_space: &Path) -> Result<String, CliError> {
    let case_space_value = read_json_value(case_space)?;
    let document = ddd_input_from_case_space(case_space_value, &case_space.to_string_lossy())?;
    serialize_json!(&document)
}

fn ddd_review_json(input: &Path) -> Result<String, CliError> {
    let document = read_json_value(input)?;
    let report = run_ddd_review(document)?;
    serialize_json!(&report)
}

fn pr_review_input_from_git_json(repo: &Path, base: &str, head: &str) -> Result<String, CliError> {
    let document = pr_review_git::input_from_git(pr_review_git::GitInputRequest {
        repo: repo.to_owned(),
        base: base.to_owned(),
        head: head.to_owned(),
    })
    .map_err(CliError::GitInput)?;
    serialize_json!(&document)
}

fn pr_review_targets_json(input: &Path) -> Result<String, CliError> {
    let document = read_pr_review_target_input_document(input)?;
    let report = run_pr_review_target_recommend(document)?;
    serialize_json!(&report)
}

fn test_gap_detect_json(input: &Path) -> Result<String, CliError> {
    let document = read_test_gap_input_document(input)?;
    let report = run_test_gap_detect(document)?;
    serialize_json!(&report)
}

fn test_gap_input_from_git_json(
    repo: &Path,
    base: &str,
    head: &str,
    binding_rules: &Option<PathBuf>,
) -> Result<String, CliError> {
    let document = test_gap_git::input_from_git(test_gap_git::GitInputRequest {
        repo: repo.to_owned(),
        base: base.to_owned(),
        head: head.to_owned(),
        binding_rules: binding_rules.clone(),
    })
    .map_err(CliError::GitInput)?;
    serialize_json!(&document)
}

fn test_gap_input_from_path_json(
    repo: &Path,
    paths: &[PathBuf],
    include_tests: bool,
    binding_rules: &Option<PathBuf>,
) -> Result<String, CliError> {
    let document = test_gap_git::input_from_path(test_gap_git::PathInputRequest {
        repo: repo.to_owned(),
        paths: paths.to_owned(),
        include_tests,
        binding_rules: binding_rules.clone(),
    })
    .map_err(CliError::GitInput)?;
    serialize_json!(&document)
}

fn test_gap_evidence_json(input: &Path, test_run: &Path) -> Result<String, CliError> {
    let input_document = read_test_gap_input_document(input)?;
    let document =
        test_gap_evidence::input_from_test_run(test_gap_evidence::TestRunEvidenceRequest {
            input: input_document,
            test_run: test_run.to_owned(),
        })
        .map_err(CliError::TestGapEvidence)?;
    serialize_json!(&document)
}

fn rust_test_semantics_json(
    repo: &Path,
    paths: &[PathBuf],
    test_run: &Option<PathBuf>,
) -> Result<String, CliError> {
    let document = rust_test_semantics::document_from_path(RustTestSemanticsPathRequest {
        repo: repo.to_owned(),
        paths: paths.to_owned(),
        test_run: test_run.clone(),
    })
    .map_err(CliError::RustTestSemantics)?;
    serialize_json!(&document.to_json_value())
}

fn test_semantics_interpret_json(input: &Path, interpreter: &str) -> Result<String, CliError> {
    let input = read_json_value(input)?;
    let document =
        test_semantics_interpretation::interpret(test_semantics_interpretation::InterpretRequest {
            input,
            interpreter: interpreter.to_owned(),
        })
        .map_err(CliError::TestSemanticsInterpretation)?;
    serialize_json!(&document)
}

fn test_semantics_review_json(
    decision: &test_semantics_review::TestSemanticsReviewDecision,
    input: &Path,
    candidate_id: &str,
    reviewer_id: &str,
    reason: &str,
) -> Result<String, CliError> {
    let interpretation = read_json_value(input)?;
    let report = test_semantics_review::review(test_semantics_review::ReviewRequest {
        interpretation,
        decision: *decision,
        candidate_id: candidate_id.to_owned(),
        reviewer_id: reviewer_id.to_owned(),
        reason: reason.to_owned(),
    })
    .map_err(CliError::TestSemanticsReview)?;
    serialize_json!(&report)
}

fn test_semantics_verify_json(
    interpretation: &Path,
    review: &Path,
    test_run: &Option<PathBuf>,
) -> Result<String, CliError> {
    let interpretation = read_json_value(interpretation)?;
    let review = read_json_value(review)?;
    let report = test_semantics_verification::verify(test_semantics_verification::VerifyRequest {
        interpretation,
        review,
        test_run_path: test_run
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
    })
    .map_err(CliError::TestSemanticsVerification)?;
    serialize_json!(&report)
}

fn test_semantics_gap_json(expected: &Path, verified: &[PathBuf]) -> Result<String, CliError> {
    let expected = read_json_value(expected)?;
    let verified_reports = verified
        .iter()
        .map(|path| read_json_value(path))
        .collect::<Result<Vec<_>, _>>()?;
    let report = test_semantics_gap::detect(test_semantics_gap::GapRequest {
        expected,
        verified_reports,
    })
    .map_err(CliError::TestSemanticsGap)?;
    serialize_json!(&report)
}

fn semantic_proof_verify_json(input: &Path) -> Result<String, CliError> {
    let document = read_semantic_proof_input_document(input)?;
    let report = run_semantic_proof_verify(document)?;
    serialize_json!(&report)
}

fn semantic_proof_backend_json(
    backend: &str,
    backend_version: &str,
    command: &Path,
    args: &[String],
    input: &Option<PathBuf>,
) -> Result<String, CliError> {
    let artifact = semantic_proof_backend::run_backend(semantic_proof_backend::BackendRunRequest {
        backend: backend.to_owned(),
        backend_version: backend_version.to_owned(),
        command: command.to_owned(),
        args: args.to_owned(),
        input: input.clone(),
    })
    .map_err(CliError::SemanticProofArtifact)?;
    serialize_json!(&artifact)
}

fn semantic_proof_from_artifact_json(command: &Command) -> Result<String, CliError> {
    let Command::SemanticProofInputFromArtifact {
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
    } = command
    else {
        unreachable!("semantic proof artifact command helper received another variant");
    };

    let document = semantic_proof_artifact::input_from_artifact(
        semantic_proof_artifact::ArtifactInputRequest {
            artifact: artifact.to_owned(),
            backend: backend.to_owned(),
            backend_version: backend_version.to_owned(),
            theorem_id: theorem_id.to_owned(),
            theorem_summary: theorem_summary.to_owned(),
            law_id: law_id.to_owned(),
            law_summary: law_summary.to_owned(),
            morphism_id: morphism_id.to_owned(),
            morphism_type: morphism_type.to_owned(),
            base_cell: base_cell.to_owned(),
            base_label: base_label.to_owned(),
            head_cell: head_cell.to_owned(),
            head_label: head_label.to_owned(),
        },
    )
    .map_err(CliError::SemanticProofArtifact)?;
    serialize_json!(&document)
}

fn semantic_proof_from_report_json(report: &Path) -> Result<String, CliError> {
    let report = read_json_value(report)?;
    let document = semantic_proof_reinput::input_from_report_value(report)
        .map_err(CliError::SemanticProofArtifact)?;
    serialize_json!(&document)
}

fn semantic_proof_attach_artifact_json(
    input: &Path,
    artifact: &Path,
    backend: &str,
    backend_version: &str,
) -> Result<String, CliError> {
    let input = read_semantic_proof_input_document(input)?;
    let document = semantic_proof_attach_artifact::attach_artifact(
        semantic_proof_attach_artifact::AttachArtifactRequest {
            input,
            artifact: artifact.to_owned(),
            backend: backend.to_owned(),
            backend_version: backend_version.to_owned(),
        },
    )
    .map_err(CliError::SemanticProofArtifact)?;
    serialize_json!(&document)
}

fn completion_review_json(
    decision: &CompletionReviewDecision,
    input: &Path,
    candidate_id: &str,
    reviewer_id: &str,
    reason: &str,
    reviewed_at: &Option<String>,
) -> Result<String, CliError> {
    let snapshot = read_completion_review_snapshot(input)?;
    let mut request = CompletionReviewRequest::new(
        Id::new(candidate_id.to_owned())?,
        *decision,
        Id::new(reviewer_id.to_owned())?,
        reason.to_owned(),
    )?;
    if let Some(reviewed_at) = reviewed_at {
        request = request.with_reviewed_at(reviewed_at.clone())?;
    }
    let report = run_completion_review(snapshot, request)?;
    serialize_json!(&report)
}

fn read_json_document<T: DeserializeOwned>(path: &Path) -> Result<T, CliError> {
    let text = fs::read_to_string(path).map_err(|source| CliError::InputRead {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| CliError::InputParse {
        path: path.to_owned(),
        source,
    })
}

fn read_input_document(path: &Path) -> Result<ArchitectureInputLiftDocument, CliError> {
    read_json_document(path)
}

fn read_feed_reader_input_document(path: &Path) -> Result<FeedReaderInputDocument, CliError> {
    read_json_document(path)
}

fn read_pr_review_target_input_document(
    path: &Path,
) -> Result<PrReviewTargetInputDocument, CliError> {
    read_json_document(path)
}

fn read_test_gap_input_document(path: &Path) -> Result<TestGapInputDocument, CliError> {
    read_json_document(path)
}

fn read_semantic_proof_input_document(path: &Path) -> Result<SemanticProofInputDocument, CliError> {
    read_json_document(path)
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
