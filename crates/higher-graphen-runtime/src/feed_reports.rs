//! Feed Product input and report shapes.

use crate::reports::{ReportEnvelope, ReportMetadata};
use higher_graphen_core::{Confidence, Id, ReviewStatus, SourceKind};
use higher_graphen_projection::InformationLoss;
use serde::{Deserialize, Serialize};

/// Feed reader workflow report envelope.
pub type FeedReaderReport =
    ReportEnvelope<FeedReaderScenario, FeedReaderResult, FeedReaderProjection>;

impl ReportMetadata {
    /// Creates metadata for the bounded Feed Product reader workflow.
    #[must_use]
    pub fn feed_reader() -> Self {
        Self {
            command: "highergraphen feed reader run".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }
}

/// Bounded v1 feed JSON document accepted by the reader workflow.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReaderInputDocument {
    /// Stable input schema identifier.
    pub schema: String,
    /// Source metadata for the bounded fixture.
    pub source: FeedInputSource,
    /// Observation space declaration.
    pub space: FeedInputSpace,
    /// Source-indexed feed contexts.
    pub source_feeds: Vec<FeedSourceFeed>,
    /// Observed feed entries.
    pub entries: Vec<FeedEntry>,
    /// Deterministic correspondence hints.
    #[serde(default)]
    pub correspondence_hints: Vec<FeedCorrespondenceHint>,
    /// Deterministic completion hints.
    #[serde(default)]
    pub completion_hints: Vec<FeedCompletionHint>,
    /// Deterministic obstruction hints.
    #[serde(default)]
    pub obstruction_hints: Vec<FeedObstructionHint>,
    /// Requested projections.
    #[serde(default)]
    pub projection_requests: Vec<FeedProjectionRequest>,
}

/// Source metadata for the bounded feed input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedInputSource {
    /// Source category.
    pub kind: SourceKind,
    /// Optional stable URI for the source document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional human-readable source title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional stable text capture time, such as RFC 3339.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    /// Optional local identifier inside the source document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
    /// Confidence applied to facts that do not override it.
    pub confidence: Confidence,
}

/// Feed observation space declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedInputSpace {
    /// Stable space identifier.
    pub id: Id,
    /// Human-readable space name.
    pub name: String,
    /// Optional space description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Timestamp axis used by timeline projections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_axis: Option<FeedTimeAxis>,
}

/// Timestamp axis for a feed space.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedTimeAxis {
    /// Use entry publication time.
    PublishedAt,
    /// Use entry update time.
    UpdatedAt,
}

/// Source feed context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedSourceFeed {
    /// Stable feed identifier.
    pub id: Id,
    /// Context identifier for this feed.
    pub context_id: Id,
    /// Human-readable title.
    pub title: String,
    /// Feed source kind.
    pub kind: FeedSourceFeedKind,
    /// Optional source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Optional publisher name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    /// Optional trust classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<FeedTrustLevel>,
    /// Optional expected update cadence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_update_cadence: Option<String>,
    /// Optional source tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional feed-specific confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Source feed kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedSourceFeedKind {
    /// Official source.
    Official,
    /// Community source.
    Community,
    /// Counterpoint source.
    Counterpoint,
    /// Aggregator source.
    Aggregator,
    /// Blog source.
    Blog,
    /// News source.
    News,
    /// External source.
    External,
}

/// Source trust classification.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedTrustLevel {
    /// Official source.
    Official,
    /// Trusted source.
    Trusted,
    /// Community source.
    Community,
    /// Unverified source.
    Unverified,
}

/// Observed feed entry.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedEntry {
    /// Stable entry identifier.
    pub id: Id,
    /// Source feed identifier.
    pub source_id: Id,
    /// Entry title.
    pub title: String,
    /// Optional entry URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Publication timestamp.
    pub published_at: String,
    /// Optional update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Entry summary.
    pub summary: String,
    /// Optional content text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_text: Option<String>,
    /// Optional author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Topic identifiers attached to the entry.
    pub topic_ids: Vec<Id>,
    /// Optional event identifier attached to the entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Id>,
    /// Optional source-local identifier for provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
    /// Optional entry confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Deterministic correspondence hint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedCorrespondenceHint {
    /// Stable correspondence identifier.
    pub id: Id,
    /// Correspondence kind.
    pub hint_type: FeedCorrespondenceType,
    /// Entries participating in the correspondence.
    pub entry_ids: Vec<Id>,
    /// Optional topic identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<Id>,
    /// Optional event identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Rationale for the hint.
    pub rationale: String,
    /// Confidence in the correspondence.
    pub confidence: Confidence,
}

/// Feed correspondence kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedCorrespondenceType {
    /// Entries discuss the same topic.
    SameTopic,
    /// Entries are duplicates.
    Duplicate,
    /// One entry follows up another.
    FollowUp,
    /// One entry is a counterpoint to another.
    Counterpoint,
}

/// Deterministic completion hint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedCompletionHint {
    /// Stable completion identifier.
    pub id: Id,
    /// Missing structure kind.
    pub missing_type: FeedMissingType,
    /// Optional subject identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_id: Option<Id>,
    /// Optional topic identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<Id>,
    /// Optional expected source kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_source_kind: Option<FeedSourceFeedKind>,
    /// Human-readable summary.
    pub summary: String,
    /// Rationale for the hint.
    pub rationale: String,
    /// Source identifiers used to infer this missing structure.
    pub inferred_from: Vec<Id>,
    /// Confidence in the completion.
    pub confidence: Confidence,
}

/// Feed-specific missing structure kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedMissingType {
    /// Missing official source.
    OfficialSource,
    /// Missing counterpoint.
    Counterpoint,
    /// Missing entry metadata.
    EntryMetadata,
    /// Missing timeline context.
    TimelineContext,
    /// Missing source feed.
    SourceFeed,
}

/// Deterministic obstruction hint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedObstructionHint {
    /// Stable obstruction identifier.
    pub id: Id,
    /// Feed obstruction kind.
    pub obstruction_type: FeedObstructionType,
    /// Entries involved in the obstruction.
    pub entry_ids: Vec<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Rationale for the obstruction.
    pub rationale: String,
    /// Feed-specific severity.
    pub severity: FeedObstructionSeverity,
    /// Confidence in the obstruction.
    pub confidence: Confidence,
}

/// Feed-specific obstruction kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedObstructionType {
    /// Two or more entries carry incompatible claims.
    FeedContentConflict,
    /// Entry timestamps conflict.
    TimestampConflict,
    /// An entry or hint references a broken source.
    BrokenSourceReference,
    /// Topic assignments conflict.
    TopicMismatch,
}

/// Feed-specific obstruction severity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedObstructionSeverity {
    /// Informational obstruction.
    Info,
    /// Warning obstruction.
    Warning,
    /// Error obstruction.
    Error,
    /// Critical obstruction.
    Critical,
}

/// Feed projection request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedProjectionRequest {
    /// Stable projection request identifier.
    pub id: Id,
    /// Projection audience.
    pub audience: FeedProjectionAudience,
    /// Projection purpose.
    pub purpose: FeedProjectionPurpose,
    /// Source identifiers represented by the projection.
    pub source_ids: Vec<Id>,
    /// Optional declared information-loss policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_loss_policy: Option<String>,
}

/// Feed projection audience.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedProjectionAudience {
    /// Human reader.
    Human,
    /// AI agent consumer.
    AiAgent,
    /// Audit consumer.
    Audit,
}

/// Feed projection purpose.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedProjectionPurpose {
    /// Timeline projection.
    Timeline,
    /// Topic digest projection.
    TopicDigest,
    /// Audit trace projection.
    AuditTrace,
}

/// Report view of the bounded feed scenario.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReaderScenario {
    /// Input schema accepted by the workflow.
    pub input_schema: String,
    /// Source metadata shared by accepted observations.
    pub source: FeedInputSource,
    /// Lifted source-indexed observation space summary.
    pub space: FeedReportSpace,
    /// Source feeds represented in the scenario.
    pub source_feeds: Vec<FeedReportSourceFeed>,
    /// Observed entry cells.
    pub entries: Vec<FeedEntryCell>,
    /// Inferred topic cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<FeedTopicCell>,
    /// Inferred event cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<FeedEventCell>,
}

/// Report-level feed space summary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReportSpace {
    /// Stable space identifier.
    pub id: Id,
    /// Human-readable space name.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Source feed identifiers.
    pub source_feed_ids: Vec<Id>,
    /// Entry identifiers.
    pub entry_ids: Vec<Id>,
    /// Topic identifiers.
    pub topic_ids: Vec<Id>,
    /// Event identifiers.
    pub event_ids: Vec<Id>,
    /// Context identifiers.
    pub context_ids: Vec<Id>,
}

/// Report-level source feed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReportSourceFeed {
    /// Stable feed identifier.
    pub id: Id,
    /// Context identifier for this feed.
    pub context_id: Id,
    /// Human-readable title.
    pub title: String,
    /// Feed source kind.
    pub kind: FeedSourceFeedKind,
    /// Optional source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Review state of the source feed fact.
    pub review_status: ReviewStatus,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Report-level feed entry cell.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedEntryCell {
    /// Stable entry cell identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Source feed identifier.
    pub source_id: Id,
    /// Cell dimension.
    pub dimension: u32,
    /// Cell type.
    pub cell_type: String,
    /// Entry label.
    pub label: String,
    /// Publication timestamp.
    pub published_at: String,
    /// Optional update timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Entry summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Topic identifiers.
    pub topic_ids: Vec<Id>,
    /// Optional event identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Id>,
    /// Review state.
    pub review_status: ReviewStatus,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Report-level topic cell.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedTopicCell {
    /// Stable topic cell identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Cell type.
    pub cell_type: String,
    /// Topic label.
    pub label: String,
    /// Entries grouped by this topic.
    pub entry_ids: Vec<Id>,
    /// Review state.
    pub review_status: ReviewStatus,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Report-level event cell.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedEventCell {
    /// Stable event cell identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Cell type.
    pub cell_type: String,
    /// Event label.
    pub label: String,
    /// Entries grouped by this event.
    pub entry_ids: Vec<Id>,
    /// Review state.
    pub review_status: ReviewStatus,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Machine-checkable feed reader workflow outcome.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReaderResult {
    /// Workflow status.
    pub status: FeedReaderStatus,
    /// Observed entry identifiers.
    pub observed_entry_ids: Vec<Id>,
    /// Inferred topic identifiers.
    pub inferred_topic_ids: Vec<Id>,
    /// Inferred event identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_event_ids: Vec<Id>,
    /// Correspondences preserved by the workflow.
    pub correspondences: Vec<FeedCorrespondence>,
    /// Completion candidates emitted by the workflow.
    pub completion_candidates: Vec<FeedCompletionCandidate>,
    /// Obstructions emitted by the workflow.
    pub obstructions: Vec<FeedObstruction>,
}

/// Feed reader workflow status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedReaderStatus {
    /// The input was analyzed without obstructions.
    Analyzed,
    /// The input was analyzed and obstructions were detected.
    ObstructionsDetected,
}

/// Report-level correspondence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedCorrespondence {
    /// Stable correspondence identifier.
    pub id: Id,
    /// Correspondence kind.
    pub correspondence_type: FeedCorrespondenceType,
    /// Entries participating in the correspondence.
    pub entry_ids: Vec<Id>,
    /// Optional topic identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_id: Option<Id>,
    /// Optional event identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence in the correspondence.
    pub confidence: Confidence,
}

/// Report-level completion candidate.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedCompletionCandidate {
    /// Stable candidate identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Feed-specific missing kind.
    pub missing_type: FeedMissingType,
    /// Suggested structure payload.
    pub suggested_structure: FeedSuggestedStructure,
    /// Source identifiers used to infer this candidate.
    pub inferred_from: Vec<Id>,
    /// Rationale for the candidate.
    pub rationale: String,
    /// Confidence in the candidate.
    pub confidence: Confidence,
    /// Review state.
    pub review_status: ReviewStatus,
}

/// Report-level suggested structure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedSuggestedStructure {
    /// Optional identifier for the structure that would be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_id: Option<Id>,
    /// Stable suggested structure type.
    pub structure_type: String,
    /// Human-readable summary.
    pub summary: String,
    /// Related source identifiers.
    pub related_ids: Vec<Id>,
}

/// Report-level obstruction.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedObstruction {
    /// Stable obstruction identifier.
    pub id: Id,
    /// Feed-specific obstruction kind.
    pub obstruction_type: FeedObstructionType,
    /// Entries involved in the obstruction.
    pub entry_ids: Vec<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Feed-specific severity.
    pub severity: FeedObstructionSeverity,
    /// Source identifiers represented by the obstruction.
    pub source_ids: Vec<Id>,
    /// Confidence in the obstruction.
    pub confidence: Confidence,
}

/// Audience-specific feed projections.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedReaderProjection {
    /// Human timeline projection.
    pub timeline: FeedProjectionView,
    /// AI-agent topic digest projection.
    pub topic_digest: FeedProjectionView,
    /// Audit trace projection.
    pub audit_trace: FeedAuditTrace,
}

/// Feed projection view.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedProjectionView {
    /// Projection audience.
    pub audience: FeedProjectionAudience,
    /// Projection purpose.
    pub purpose: FeedProjectionPurpose,
    /// Human-readable summary.
    pub summary: String,
    /// Projection records.
    pub records: Vec<FeedProjectionRecord>,
    /// Source identifiers represented in this view.
    pub source_ids: Vec<Id>,
    /// Declared projection information loss.
    pub information_loss: Vec<InformationLoss>,
}

/// One record in a feed projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedProjectionRecord {
    /// Stable record identifier.
    pub id: Id,
    /// Record kind.
    pub record_type: FeedProjectionRecordType,
    /// Record summary.
    pub summary: String,
    /// Source identifiers represented by this record.
    pub source_ids: Vec<Id>,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    /// Optional review state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_status: Option<ReviewStatus>,
}

/// Feed projection record kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedProjectionRecordType {
    /// Source feed record.
    SourceFeed,
    /// Entry record.
    Entry,
    /// Topic record.
    Topic,
    /// Event record.
    Event,
    /// Correspondence record.
    Correspondence,
    /// Completion candidate record.
    CompletionCandidate,
    /// Obstruction record.
    Obstruction,
}

/// Feed audit trace projection.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedAuditTrace {
    /// Audit audience.
    pub audience: FeedProjectionAudience,
    /// Audit purpose.
    pub purpose: FeedProjectionPurpose,
    /// Source identifiers represented in the audit trace.
    pub source_ids: Vec<Id>,
    /// Declared audit information loss.
    pub information_loss: Vec<InformationLoss>,
    /// Per-source traces.
    pub traces: Vec<FeedAuditTraceRecord>,
}

/// One feed audit trace record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeedAuditTraceRecord {
    /// Source identifier represented by the trace.
    pub source_id: Id,
    /// Source role.
    pub role: String,
    /// Projection views containing this source.
    pub represented_in: Vec<String>,
}
