use crate::{
    cli_args::{
        require_token, required_segment, GitInputOptions, PathInputOptions, ReportOptions,
        ReviewOptions, SemanticProofArtifactOptions, SemanticProofAttachArtifactOptions,
        SemanticProofBackendOptions, SemanticProofReportInputOptions, TestRunEvidenceOptions,
        TestSemanticsGapOptions, TestSemanticsInterpretOptions, TestSemanticsReviewOptions,
        TestSemanticsVerifyOptions,
    },
    cli_error::CliError,
    command::Command,
    test_semantics_review,
};
use higher_graphen_runtime::CompletionReviewDecision;
use std::{ffi::OsString, path::PathBuf};

impl Command {
    pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, CliError> {
        let mut args = args.into_iter();
        let root = required_segment(&mut args, "command")?;

        match root.to_str() {
            Some("version") | Some("--version") | Some("-V") => Self::parse_version(args),
            Some("architecture") => Self::parse_architecture(args),
            Some("feed") => Self::parse_feed(args),
            Some("pr-review") => Self::parse_pr_review(args),
            Some("test-gap") => Self::parse_test_gap(args),
            Some("test-semantics") => Self::parse_test_semantics(args),
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
        let input = required_option(options.input, "--input <path> is required")?;
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
        if options.binding_rules.is_some() {
            return Err(CliError::usage(
                "--binding-rules is not supported for pr-review input from-git",
            ));
        }
        Ok(Self::PrReviewInputFromGit {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            base: required_option(options.base, "--base <ref> is required")?,
            head: required_option(options.head, "--head <ref> is required")?,
            output: options.output,
        })
    }

    fn parse_pr_review_targets(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        require_token(&mut args, "recommend")?;
        let options = ReportOptions::parse(args, true)?;
        let input = required_option(options.input, "--input <path> is required")?;
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
            Some("detect") => Self::parse_test_gap_detect(args),
            Some(_) | None => Err(CliError::usage("unsupported test-gap command segment")),
        }
    }

    fn parse_test_gap_detect(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = ReportOptions::parse(args, true)?;
        let input = required_option(options.input, "--input <path> is required")?;
        Ok(Self::TestGapDetect {
            input,
            output: options.output,
        })
    }

    fn parse_test_gap_evidence(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "test-gap evidence command")?;
        match segment.to_str() {
            Some("from-test-run") => {
                let options = TestRunEvidenceOptions::parse(args)?;
                Ok(Self::TestGapEvidenceFromTestRun {
                    input: required_option(options.input, "--input <path> is required")?,
                    test_run: required_option(options.test_run, "--test-run <path> is required")?,
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
            Some("from-git") => Self::parse_test_gap_input_from_git(args),
            Some("from-path") => Self::parse_test_gap_input_from_path(args),
            Some(_) | None => Err(CliError::usage(
                "unsupported test-gap input command segment",
            )),
        }
    }

    fn parse_test_gap_input_from_git(
        args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let options = GitInputOptions::parse(args)?;
        Ok(Self::TestGapInputFromGit {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            base: required_option(options.base, "--base <ref> is required")?,
            head: required_option(options.head, "--head <ref> is required")?,
            binding_rules: options.binding_rules,
            output: options.output,
        })
    }

    fn parse_test_gap_input_from_path(
        args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let options = PathInputOptions::parse(args)?;
        if options.test_run.is_some() {
            return Err(CliError::usage(
                "--test-run is not supported for test-gap input from-path; use test-gap evidence from-test-run",
            ));
        }
        if options.paths.is_empty() {
            return Err(CliError::usage("--path <path> is required"));
        }
        Ok(Self::TestGapInputFromPath {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            paths: options.paths,
            include_tests: options.include_tests,
            binding_rules: options.binding_rules,
            output: options.output,
        })
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
        if options.binding_rules.is_some() {
            return Err(CliError::usage(
                "--binding-rules is not supported for rust-test semantics from-path",
            ));
        }
        if options.paths.is_empty() {
            return Err(CliError::usage("--path <path> is required"));
        }
        Ok(Self::RustTestSemanticsFromPath {
            repo: options.repo.unwrap_or_else(|| PathBuf::from(".")),
            paths: options.paths,
            test_run: options.test_run,
            output: options.output,
        })
    }

    fn parse_test_semantics(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "test-semantics command")?;
        match segment.to_str() {
            Some("interpret") => Self::parse_test_semantics_interpret(args),
            Some("review") => Self::parse_test_semantics_review(args),
            Some("verify") => Self::parse_test_semantics_verify(args),
            Some("gap") => Self::parse_test_semantics_gap(args),
            Some(_) | None => Err(CliError::usage(
                "unsupported test-semantics command segment",
            )),
        }
    }
}

impl Command {
    fn parse_test_semantics_interpret(
        args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let options = TestSemanticsInterpretOptions::parse(args)?;
        Ok(Self::TestSemanticsInterpret {
            input: required_option(options.input, "--input <path> is required")?,
            interpreter: options.interpreter.unwrap_or_else(|| "ai-agent".to_owned()),
            output: options.output,
        })
    }

    fn parse_test_semantics_review(
        mut args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let decision = match required_segment(&mut args, "test-semantics review action")?.to_str() {
            Some("accept") => test_semantics_review::TestSemanticsReviewDecision::Accepted,
            Some("reject") => test_semantics_review::TestSemanticsReviewDecision::Rejected,
            Some(_) | None => {
                return Err(CliError::usage("unsupported test-semantics review action"));
            }
        };
        let options = TestSemanticsReviewOptions::parse(args)?;

        Ok(Self::TestSemanticsReview {
            decision,
            input: required_option(options.input, "--input <path> is required")?,
            candidate_id: required_option(options.candidate_id, "--candidate <id> is required")?,
            reviewer_id: required_option(options.reviewer_id, "--reviewer <id> is required")?,
            reason: required_option(options.reason, "--reason <text> is required")?,
            output: options.output,
        })
    }

    fn parse_test_semantics_verify(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = TestSemanticsVerifyOptions::parse(args)?;
        Ok(Self::TestSemanticsVerify {
            interpretation: required_option(
                options.interpretation,
                "--interpretation <path> is required",
            )?,
            review: required_option(options.review, "--review <path> is required")?,
            test_run: options.test_run,
            output: options.output,
        })
    }

    fn parse_test_semantics_gap(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = TestSemanticsGapOptions::parse(args)?;
        if options.verified.is_empty() {
            return Err(CliError::usage("--verified <path> is required"));
        }
        Ok(Self::TestSemanticsGap {
            expected: required_option(options.expected, "--expected <path> is required")?,
            verified: options.verified,
            output: options.output,
        })
    }

    fn parse_semantic_proof(mut args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "semantic-proof command")?;
        match segment.to_str() {
            Some("backend") => Self::parse_semantic_proof_backend(args),
            Some("input") => Self::parse_semantic_proof_input(args),
            Some("verify") => Self::parse_semantic_proof_verify(args),
            Some(_) | None => Err(CliError::usage(
                "unsupported semantic-proof command segment",
            )),
        }
    }

    fn parse_semantic_proof_verify(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = ReportOptions::parse(args, true)?;
        let input = required_option(options.input, "--input <path> is required")?;
        Ok(Self::SemanticProofVerify {
            input,
            output: options.output,
        })
    }

    fn parse_semantic_proof_backend(
        mut args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        require_token(&mut args, "run")?;
        let options = SemanticProofBackendOptions::parse(args)?;
        Ok(Self::SemanticProofBackendRun {
            backend: required_option(options.backend, "--backend <name> is required")?,
            backend_version: required_option(
                options.backend_version,
                "--backend-version <version> is required",
            )?,
            command: required_option(options.command, "--command <path> is required")?,
            args: options.args,
            input: options.input,
            output: options.output,
        })
    }
}

impl Command {
    fn parse_semantic_proof_input(
        mut args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let segment = required_segment(&mut args, "semantic-proof input command")?;
        match segment.to_str() {
            Some("from-artifact") => Self::semantic_proof_from_artifact(args),
            Some("from-report") => Self::semantic_proof_from_report(args),
            Some("attach-artifact") => Self::semantic_proof_attach_artifact(args),
            Some(_) | None => Err(CliError::usage(
                "unsupported semantic-proof input command segment",
            )),
        }
    }

    fn semantic_proof_from_artifact(
        args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let options = SemanticProofArtifactOptions::parse(args)?;
        Ok(Self::SemanticProofInputFromArtifact {
            artifact: required_option(options.artifact, "--artifact <path> is required")?,
            backend: required_option(options.backend, "--backend <name> is required")?,
            backend_version: required_option(
                options.backend_version,
                "--backend-version <version> is required",
            )?,
            theorem_id: required_option(options.theorem_id, "--theorem-id <id> is required")?,
            theorem_summary: required_option(
                options.theorem_summary,
                "--theorem-summary <text> is required",
            )?,
            law_id: required_option(options.law_id, "--law-id <id> is required")?,
            law_summary: required_option(options.law_summary, "--law-summary <text> is required")?,
            morphism_id: required_option(options.morphism_id, "--morphism-id <id> is required")?,
            morphism_type: required_option(
                options.morphism_type,
                "--morphism-type <text> is required",
            )?,
            base_cell: required_option(options.base_cell, "--base-cell <id> is required")?,
            base_label: required_option(options.base_label, "--base-label <text> is required")?,
            head_cell: required_option(options.head_cell, "--head-cell <id> is required")?,
            head_label: required_option(options.head_label, "--head-label <text> is required")?,
            output: options.output,
        })
    }

    fn semantic_proof_from_report(args: impl Iterator<Item = OsString>) -> Result<Self, CliError> {
        let options = SemanticProofReportInputOptions::parse(args)?;
        Ok(Self::SemanticProofInputFromReport {
            report: required_option(options.report, "--report <path> is required")?,
            output: options.output,
        })
    }

    fn semantic_proof_attach_artifact(
        args: impl Iterator<Item = OsString>,
    ) -> Result<Self, CliError> {
        let options = SemanticProofAttachArtifactOptions::parse(args)?;
        Ok(Self::SemanticProofInputAttachArtifact {
            input: required_option(options.input, "--input <path> is required")?,
            artifact: required_option(options.artifact, "--artifact <path> is required")?,
            backend: required_option(options.backend, "--backend <name> is required")?,
            backend_version: required_option(
                options.backend_version,
                "--backend-version <version> is required",
            )?,
            output: options.output,
        })
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
            input: required_option(options.input, "--input <path> is required")?,
            candidate_id: required_option(options.candidate_id, "--candidate <id> is required")?,
            reviewer_id: required_option(options.reviewer_id, "--reviewer <id> is required")?,
            reason: required_option(options.reason, "--reason <text> is required")?,
            reviewed_at: options.reviewed_at,
            output: options.output,
        })
    }
}

fn required_option<T>(option: Option<T>, message: &'static str) -> Result<T, CliError> {
    option.ok_or_else(|| CliError::usage(message))
}
