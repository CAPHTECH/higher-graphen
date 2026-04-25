//! Runtime workflow orchestration APIs for HigherGraphen.

pub mod error;
mod feed_reports;
pub mod reports;
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
pub use workflows::{
    architecture::{run_architecture_direct_db_access_smoke, run_architecture_input_lift},
    run_completion_review, run_feed_reader,
};
