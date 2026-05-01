//! PR review target recommender report shapes.
#![allow(missing_docs)]

use crate::reports::{ProjectionViewSet, ReportEnvelope};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind};
use higher_graphen_reasoning::completion::CompletionCandidate;
use higher_graphen_structure::space::IncidenceOrientation;
use serde::{Deserialize, Serialize};

/// PR review target recommender report envelope.
pub type PrReviewTargetReport =
    ReportEnvelope<PrReviewTargetScenario, PrReviewTargetResult, PrReviewTargetProjection>;

/// Stable PR review target projection.
pub type PrReviewTargetProjection = ProjectionViewSet;

/// Bounded v1 PR snapshot accepted by the review target recommender.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputDocument {
    /// Stable input schema identifier.
    pub schema: String,
    /// Source metadata for the bounded snapshot.
    pub source: PrReviewTargetSource,
    /// Repository identity.
    pub repository: PrReviewTargetRepository,
    /// Pull request identity.
    pub pull_request: PrReviewTargetPullRequest,
    /// Changed files in the bounded snapshot.
    pub changed_files: Vec<PrReviewTargetInputChangedFile>,
    /// Optional changed symbols.
    #[serde(default)]
    pub symbols: Vec<PrReviewTargetInputSymbol>,
    /// Optional owners.
    #[serde(default)]
    pub owners: Vec<PrReviewTargetInputOwner>,
    /// Optional review contexts.
    #[serde(default)]
    pub contexts: Vec<PrReviewTargetInputContext>,
    /// Optional tests.
    #[serde(default)]
    pub tests: Vec<PrReviewTargetInputTest>,
    /// Optional dependency edges.
    #[serde(default)]
    pub dependency_edges: Vec<PrReviewTargetInputDependencyEdge>,
    /// Optional evidence records.
    #[serde(default)]
    pub evidence: Vec<PrReviewTargetInputEvidence>,
    /// Optional risk signals.
    #[serde(default)]
    pub signals: Vec<PrReviewTargetInputRiskSignal>,
    /// Optional reviewer-supplied context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_context: Option<PrReviewTargetReviewerContext>,
}

/// Source metadata for a bounded PR review target input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetSource {
    /// Source category.
    pub kind: SourceKind,
    /// Optional stable URI for the source document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional human-readable title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional capture timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    /// Snapshot confidence.
    pub confidence: Confidence,
}

/// Repository identity in the bounded snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetRepository {
    /// Stable repository identifier.
    pub id: Id,
    /// Repository name.
    pub name: String,
    /// Optional repository URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional default branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

/// Pull request identity in the bounded snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetPullRequest {
    /// Stable pull request identifier.
    pub id: Id,
    /// Pull request number.
    pub number: u32,
    /// Pull request title.
    pub title: String,
    /// Source branch.
    pub source_branch: String,
    /// Target branch.
    pub target_branch: String,
    /// Optional author identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<Id>,
    /// Optional pull request URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// Changed file input fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputChangedFile {
    /// Stable file identifier.
    pub id: Id,
    /// Current path.
    pub path: String,
    /// Change type.
    pub change_type: PrReviewTargetChangeType,
    /// Optional previous path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    /// Optional language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Added lines.
    pub additions: u32,
    /// Deleted lines.
    pub deletions: u32,
    /// Symbols declared in this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_ids: Vec<Id>,
    /// Owners declared for this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_ids: Vec<Id>,
    /// Contexts declared for this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Source IDs that support this file fact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

/// Changed symbol input fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputSymbol {
    /// Stable symbol identifier.
    pub id: Id,
    /// File containing the symbol.
    pub file_id: Id,
    /// Symbol name.
    pub name: String,
    /// Symbol kind.
    pub kind: PrReviewTargetSymbolKind,
    /// Optional path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional start line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    /// Optional end line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    /// Owners declared for this symbol.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_ids: Vec<Id>,
    /// Contexts declared for this symbol.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
}

/// Owner input fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputOwner {
    /// Stable owner identifier.
    pub id: Id,
    /// Owner kind.
    pub owner_type: PrReviewTargetOwnerType,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Source IDs supporting this owner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

/// Context input fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputContext {
    /// Stable context identifier.
    pub id: Id,
    /// Context name.
    pub name: String,
    /// Context kind.
    pub context_type: PrReviewTargetContextType,
    /// Source IDs supporting this context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

/// Test input fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputTest {
    /// Stable test identifier.
    pub id: Id,
    /// Test name.
    pub name: String,
    /// Test kind.
    pub test_type: PrReviewTargetTestType,
    /// Optional covered file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<Id>,
    /// Optional covered symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_ids: Vec<Id>,
    /// Optional contexts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Source IDs supporting this test fact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

/// Dependency edge input fact.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputDependencyEdge {
    /// Stable dependency identifier.
    pub id: Id,
    /// Source endpoint.
    pub from_id: Id,
    /// Target endpoint.
    pub to_id: Id,
    /// Relation kind.
    pub relation_type: PrReviewTargetDependencyRelationType,
    /// Optional orientation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<IncidenceOrientation>,
    /// Source IDs supporting this dependency.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    /// Optional dependency confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Evidence input fact.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputEvidence {
    /// Stable evidence identifier.
    pub id: Id,
    /// Evidence kind.
    pub evidence_type: PrReviewTargetEvidenceType,
    /// Evidence summary.
    pub summary: String,
    /// Source IDs supported by this evidence.
    pub source_ids: Vec<Id>,
    /// Optional evidence confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Risk signal input fact.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetInputRiskSignal {
    /// Stable signal identifier.
    pub id: Id,
    /// Signal kind.
    pub signal_type: PrReviewTargetRiskSignalType,
    /// Signal summary.
    pub summary: String,
    /// Source IDs supporting the signal.
    pub source_ids: Vec<Id>,
    /// Impact if the signal hides a real problem.
    pub severity: Severity,
    /// Signal confidence.
    pub confidence: Confidence,
}

/// Reviewer-supplied context.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetReviewerContext {
    /// Required reviewer expertise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_expertise: Vec<String>,
    /// Declared review focus.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_focus: Vec<String>,
    /// Paths to exclude from target recommendation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_paths: Vec<String>,
}

/// Report scenario preserving accepted observations and the lifted structure.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetScenario {
    /// Accepted input schema.
    pub input_schema: String,
    /// Source metadata copied from the input.
    pub source: PrReviewTargetSource,
    /// Repository identity.
    pub repository: PrReviewTargetRepository,
    /// Pull request identity.
    pub pull_request: PrReviewTargetPullRequest,
    /// Accepted changed files.
    pub changed_files: Vec<PrReviewTargetChangedFile>,
    /// Accepted symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<PrReviewTargetSymbol>,
    /// Accepted owners.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<PrReviewTargetOwner>,
    /// Accepted contexts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<PrReviewTargetContext>,
    /// Accepted tests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<PrReviewTargetTest>,
    /// Accepted dependency edges.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_edges: Vec<PrReviewTargetDependencyEdge>,
    /// Accepted evidence records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<PrReviewTargetEvidence>,
    /// Accepted risk signals.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<PrReviewTargetRiskSignal>,
    /// Reviewer-supplied context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_context: Option<PrReviewTargetReviewerContext>,
    /// Lifted HigherGraphen-style structure.
    pub lifted_structure: PrReviewTargetLiftedStructure,
}

/// Accepted changed file report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetChangedFile {
    pub id: Id,
    pub path: String,
    pub change_type: PrReviewTargetChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted symbol report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetSymbol {
    pub id: Id,
    pub file_id: Id,
    pub name: String,
    pub kind: PrReviewTargetSymbolKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted owner report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetOwner {
    pub id: Id,
    pub owner_type: PrReviewTargetOwnerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted context report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetContext {
    pub id: Id,
    pub name: String,
    pub context_type: PrReviewTargetContextType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted test report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetTest {
    pub id: Id,
    pub name: String,
    pub test_type: PrReviewTargetTestType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted dependency edge report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetDependencyEdge {
    pub id: Id,
    pub from_id: Id,
    pub to_id: Id,
    pub relation_type: PrReviewTargetDependencyRelationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<IncidenceOrientation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted evidence report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetEvidence {
    pub id: Id,
    pub evidence_type: PrReviewTargetEvidenceType,
    pub summary: String,
    pub source_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

/// Accepted risk signal report record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetRiskSignal {
    pub id: Id,
    pub signal_type: PrReviewTargetRiskSignalType,
    pub summary: String,
    pub source_ids: Vec<Id>,
    pub severity: Severity,
    pub confidence: Confidence,
}

/// Lifted structure embedded in the scenario.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLiftedStructure {
    pub space: PrReviewTargetLiftedSpace,
    pub contexts: Vec<PrReviewTargetLiftedContext>,
    pub cells: Vec<PrReviewTargetLiftedCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidences: Vec<PrReviewTargetLiftedIncidence>,
}

/// Lifted PR review space.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLiftedSpace {
    pub id: Id,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub cell_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
    pub context_ids: Vec<Id>,
}

/// Lifted context with provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLiftedContext {
    pub id: Id,
    pub space_id: Id,
    pub name: String,
    pub context_type: String,
    pub provenance: Provenance,
}

/// Lifted cell with provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLiftedCell {
    pub id: Id,
    pub space_id: Id,
    pub dimension: u32,
    pub cell_type: String,
    pub label: String,
    pub context_ids: Vec<Id>,
    pub provenance: Provenance,
}

/// Lifted incidence with provenance.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLiftedIncidence {
    pub id: Id,
    pub space_id: Id,
    pub from_cell_id: Id,
    pub to_cell_id: Id,
    pub relation_type: String,
    pub orientation: IncidenceOrientation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub provenance: Provenance,
}

/// Machine-checkable PR review target result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetResult {
    pub status: PrReviewTargetStatus,
    pub accepted_change_ids: Vec<Id>,
    pub review_targets: Vec<PrReviewTarget>,
    pub obstructions: Vec<PrReviewTargetObstruction>,
    pub completion_candidates: Vec<CompletionCandidate>,
    pub source_ids: Vec<Id>,
}

/// PR review target result status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetStatus {
    TargetsRecommended,
    NoTargets,
    UnsupportedInput,
}

/// Unreviewed review target recommendation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTarget {
    pub id: Id,
    pub target_type: PrReviewTargetType,
    pub target_ref: String,
    pub title: String,
    pub rationale: String,
    pub evidence_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<PrReviewTargetLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_target_ids: Vec<Id>,
    pub severity: Severity,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

/// PR review target location.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetLocation {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_id: Option<Id>,
}

/// Unreviewed review obstruction.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewTargetObstruction {
    pub id: Id,
    pub obstruction_type: PrReviewTargetObstructionType,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_resolution: Option<String>,
    pub severity: Severity,
    pub source_ids: Vec<Id>,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetSymbolKind {
    Function,
    Method,
    Type,
    Module,
    Test,
    Configuration,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetOwnerType {
    Person,
    Team,
    System,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetContextType {
    Repository,
    PullRequest,
    ReviewFocus,
    Ownership,
    TestScope,
    DependencyScope,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetTestType {
    Unit,
    Integration,
    E2e,
    Smoke,
    Manual,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetDependencyRelationType {
    Imports,
    Calls,
    Owns,
    Tests,
    Covers,
    DependsOn,
    GeneratedFrom,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetEvidenceType {
    DiffHunk,
    StaticAnalysis,
    TestResult,
    Coverage,
    OwnershipMetadata,
    DependencyScan,
    HumanNote,
    AiNote,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetRiskSignalType {
    LargeChange,
    TestGap,
    DependencyChange,
    OwnershipBoundary,
    SecuritySensitive,
    GeneratedCode,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetType {
    File,
    Symbol,
    Test,
    Dependency,
    Documentation,
    CrossCutting,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrReviewTargetObstructionType {
    TestGap,
    OwnershipBoundary,
    MissingContext,
    DependencyRisk,
    SecuritySensitiveChange,
    ProjectionLoss,
    Custom,
}
