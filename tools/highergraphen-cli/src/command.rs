use crate::test_semantics_review;
use higher_graphen_runtime::CompletionReviewDecision;
use std::path::PathBuf;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Command {
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
    DddInputFromCaseSpace {
        case_space: PathBuf,
        output: Option<PathBuf>,
    },
    DddReview {
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
        binding_rules: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    TestGapInputFromPath {
        repo: PathBuf,
        paths: Vec<PathBuf>,
        include_tests: bool,
        binding_rules: Option<PathBuf>,
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
        test_run: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    TestSemanticsInterpret {
        input: PathBuf,
        interpreter: String,
        output: Option<PathBuf>,
    },
    TestSemanticsReview {
        decision: test_semantics_review::TestSemanticsReviewDecision,
        input: PathBuf,
        candidate_id: String,
        reviewer_id: String,
        reason: String,
        output: Option<PathBuf>,
    },
    TestSemanticsVerify {
        interpretation: PathBuf,
        review: PathBuf,
        test_run: Option<PathBuf>,
        output: Option<PathBuf>,
    },
    TestSemanticsGap {
        expected: PathBuf,
        verified: Vec<PathBuf>,
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
    SemanticProofInputAttachArtifact {
        input: PathBuf,
        artifact: PathBuf,
        backend: String,
        backend_version: String,
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
    pub(crate) fn output(&self) -> Option<&PathBuf> {
        match self {
            Self::Version => None,
            Self::ArchitectureSmokeDirectDbAccess { output }
            | Self::ArchitectureInputLift { output, .. }
            | Self::FeedReaderRun { output, .. }
            | Self::DddInputFromCaseSpace { output, .. }
            | Self::DddReview { output, .. }
            | Self::PrReviewInputFromGit { output, .. }
            | Self::PrReviewTargetsRecommend { output, .. }
            | Self::TestGapInputFromGit { output, .. }
            | Self::TestGapInputFromPath { output, .. }
            | Self::TestGapEvidenceFromTestRun { output, .. }
            | Self::TestGapDetect { output, .. }
            | Self::RustTestSemanticsFromPath { output, .. }
            | Self::TestSemanticsInterpret { output, .. }
            | Self::TestSemanticsReview { output, .. }
            | Self::TestSemanticsVerify { output, .. }
            | Self::TestSemanticsGap { output, .. }
            | Self::SemanticProofBackendRun { output, .. }
            | Self::SemanticProofInputFromArtifact { output, .. }
            | Self::SemanticProofInputFromReport { output, .. }
            | Self::SemanticProofInputAttachArtifact { output, .. }
            | Self::SemanticProofVerify { output, .. }
            | Self::CompletionReview { output, .. } => output.as_ref(),
        }
    }
}
