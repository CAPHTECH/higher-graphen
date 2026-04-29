//! Runtime workflow orchestration APIs for HigherGraphen.

pub mod error;
mod feed_reports;
mod pr_review_reports;
pub mod reports;
mod test_gap_reports;
pub mod workflows;

pub use error::{RuntimeError, RuntimeResult};
pub use feed_reports::{
    FeedAuditTrace, FeedAuditTraceRecord, FeedCompletionCandidate, FeedCompletionHint,
    FeedCorrespondence, FeedCorrespondenceHint, FeedCorrespondenceType, FeedEntry, FeedEntryCell,
    FeedEventCell, FeedInputSource, FeedInputSpace, FeedMissingType, FeedObstruction,
    FeedObstructionHint, FeedObstructionSeverity, FeedObstructionType, FeedProjectionAudience,
    FeedProjectionPurpose, FeedProjectionRecord, FeedProjectionRecordType, FeedProjectionRequest,
    FeedProjectionView, FeedReaderInputDocument, FeedReaderProjection, FeedReaderReport,
    FeedReaderResult, FeedReaderScenario, FeedReaderStatus, FeedReportSourceFeed, FeedReportSpace,
    FeedSourceFeed, FeedSourceFeedKind, FeedSuggestedStructure, FeedTimeAxis, FeedTopicCell,
    FeedTrustLevel,
};
pub use higher_graphen_completion::{
    CompletionReviewDecision, CompletionReviewRecord, CompletionReviewRequest,
};
pub use pr_review_reports::{
    PrReviewTarget, PrReviewTargetChangeType, PrReviewTargetChangedFile, PrReviewTargetContext,
    PrReviewTargetContextType, PrReviewTargetDependencyEdge, PrReviewTargetDependencyRelationType,
    PrReviewTargetEvidence, PrReviewTargetEvidenceType, PrReviewTargetInputChangedFile,
    PrReviewTargetInputContext, PrReviewTargetInputDependencyEdge, PrReviewTargetInputDocument,
    PrReviewTargetInputEvidence, PrReviewTargetInputOwner, PrReviewTargetInputRiskSignal,
    PrReviewTargetInputSymbol, PrReviewTargetInputTest, PrReviewTargetLiftedCell,
    PrReviewTargetLiftedContext, PrReviewTargetLiftedIncidence, PrReviewTargetLiftedSpace,
    PrReviewTargetLiftedStructure, PrReviewTargetLocation, PrReviewTargetObstruction,
    PrReviewTargetObstructionType, PrReviewTargetOwner, PrReviewTargetOwnerType,
    PrReviewTargetProjection, PrReviewTargetPullRequest, PrReviewTargetReport,
    PrReviewTargetRepository, PrReviewTargetResult, PrReviewTargetReviewerContext,
    PrReviewTargetRiskSignal, PrReviewTargetRiskSignalType, PrReviewTargetScenario,
    PrReviewTargetSource, PrReviewTargetStatus, PrReviewTargetSymbol, PrReviewTargetSymbolKind,
    PrReviewTargetTest, PrReviewTargetTestType, PrReviewTargetType,
};
pub use reports::{
    AiProjectionRecord, AiProjectionRecordType, AiProjectionView,
    ArchitectureDirectDbAccessSmokeProjection, ArchitectureDirectDbAccessSmokeReport,
    ArchitectureDirectDbAccessSmokeResult, ArchitectureDirectDbAccessSmokeScenario,
    ArchitectureInputComponent, ArchitectureInputContext, ArchitectureInputInference,
    ArchitectureInputLiftDocument, ArchitectureInputLiftProjection, ArchitectureInputLiftReport,
    ArchitectureInputLiftResult, ArchitectureInputLiftScenario, ArchitectureInputLiftStatus,
    ArchitectureInputRelation, ArchitectureInputSource, ArchitectureInputSpace,
    ArchitectureSmokeStatus, AuditProjectionView, CompletionReviewProjection,
    CompletionReviewReport, CompletionReviewResult, CompletionReviewScenario,
    CompletionReviewSnapshot, CompletionReviewSourceReport, CompletionReviewStatus,
    HumanReviewProjectionView, ProjectionAudience, ProjectionPurpose, ProjectionTrace,
    ProjectionViewSet, ReportEnvelope, ReportMetadata,
};
pub use test_gap_reports::{
    TestGapBranchType, TestGapCandidateProvenance, TestGapChangeSet, TestGapChangeType,
    TestGapCompletionCandidate, TestGapContextType, TestGapCoverageStatus, TestGapCoverageType,
    TestGapDependencyRelationType, TestGapDetectorContext, TestGapEvidenceType, TestGapFactSource,
    TestGapInputBranch, TestGapInputChangedFile, TestGapInputContext, TestGapInputCoverage,
    TestGapInputDependencyEdge, TestGapInputDocument, TestGapInputEvidence,
    TestGapInputRequirement, TestGapInputRiskSignal, TestGapInputSymbol, TestGapInputTest,
    TestGapLiftedCell, TestGapLiftedContext, TestGapLiftedIncidence, TestGapLiftedSpace,
    TestGapLiftedStructure, TestGapMissingType, TestGapMorphismSummary, TestGapMorphismType,
    TestGapObservedBranch, TestGapObservedChangedFile, TestGapObservedContext,
    TestGapObservedCoverage, TestGapObservedDependencyEdge, TestGapObservedEvidence,
    TestGapObservedRequirement, TestGapObservedRiskSignal, TestGapObservedSymbol,
    TestGapObservedTest, TestGapObstruction, TestGapObstructionType, TestGapPreservationStatus,
    TestGapProjection, TestGapReport, TestGapRepository, TestGapRequirementType, TestGapResult,
    TestGapRiskSignalType, TestGapScenario, TestGapSource, TestGapSourceBoundary, TestGapStatus,
    TestGapStructuralSummary, TestGapSuggestedTestShape, TestGapSymbolKind, TestGapTestType,
    TestGapVisibility,
};
pub use workflows::{
    architecture::{run_architecture_direct_db_access_smoke, run_architecture_input_lift},
    run_completion_review, run_feed_reader, run_pr_review_target_recommend, run_test_gap_detect,
};
