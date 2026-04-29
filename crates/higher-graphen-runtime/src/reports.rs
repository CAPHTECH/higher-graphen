//! Stable report shapes returned by runtime workflows.

use higher_graphen_completion::{CompletionCandidate, CompletionReviewRecord, MissingType};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind};
use higher_graphen_invariant::CheckResult;
use higher_graphen_obstruction::Obstruction;
use higher_graphen_projection::InformationLoss;
use higher_graphen_space::{Cell, Incidence, IncidenceOrientation, Space};
use serde::{Deserialize, Serialize};

/// Reusable runtime report envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportEnvelope<S, R, P> {
    /// Stable schema identifier.
    pub schema: String,
    /// Stable report type identifier.
    pub report_type: String,
    /// Report schema version.
    pub report_version: u32,
    /// Runtime and consumer metadata.
    pub metadata: ReportMetadata,
    /// Deterministic or input scenario represented by the report.
    pub scenario: S,
    /// Machine-checkable workflow result.
    pub result: R,
    /// Audience-specific projection of the result.
    pub projection: P,
}

/// Metadata shared by runtime workflow reports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReportMetadata {
    /// CLI command represented by this runtime report.
    pub command: String,
    /// Runtime Cargo package name.
    pub runtime_package: String,
    /// Runtime Rust crate name.
    pub runtime_crate: String,
    /// CLI Cargo package name expected to consume this report.
    pub cli_package: String,
}

impl ReportMetadata {
    /// Creates metadata for the architecture direct DB access smoke workflow.
    #[must_use]
    pub fn architecture_direct_db_access_smoke() -> Self {
        Self {
            command: "highergraphen architecture smoke direct-db-access".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }

    /// Creates metadata for the architecture input lift workflow.
    #[must_use]
    pub fn architecture_input_lift() -> Self {
        Self {
            command: "highergraphen architecture input lift".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }

    /// Creates metadata for an explicit completion review workflow.
    #[must_use]
    pub fn completion_review(command_action: &str) -> Self {
        Self {
            command: format!("highergraphen completion review {command_action}"),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }

    /// Creates metadata for the PR review target recommender workflow.
    #[must_use]
    pub fn pr_review_target() -> Self {
        Self {
            command: "highergraphen pr-review targets recommend".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }

    /// Creates metadata for the bounded missing unit test detector workflow.
    #[must_use]
    pub fn test_gap_detection() -> Self {
        Self {
            command: "highergraphen test-gap detect".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }

    /// Creates metadata for semantic proof certificate verification.
    #[must_use]
    pub fn semantic_proof_verify() -> Self {
        Self {
            command: "highergraphen semantic-proof verify".to_owned(),
            runtime_package: "higher-graphen-runtime".to_owned(),
            runtime_crate: "higher_graphen_runtime".to_owned(),
            cli_package: "highergraphen-cli".to_owned(),
        }
    }
}

/// Architecture smoke workflow report envelope.
pub type ArchitectureDirectDbAccessSmokeReport = ReportEnvelope<
    ArchitectureDirectDbAccessSmokeScenario,
    ArchitectureDirectDbAccessSmokeResult,
    ArchitectureDirectDbAccessSmokeProjection,
>;

/// Architecture input lift workflow report envelope.
pub type ArchitectureInputLiftReport = ReportEnvelope<
    ArchitectureInputLiftScenario,
    ArchitectureInputLiftResult,
    ArchitectureInputLiftProjection,
>;

/// Explicit completion review workflow report envelope.
pub type CompletionReviewReport =
    ReportEnvelope<CompletionReviewScenario, CompletionReviewResult, CompletionReviewProjection>;

/// Bounded v1 architecture JSON document accepted by the input lift workflow.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputLiftDocument {
    /// Stable input schema identifier.
    pub schema: String,
    /// Source metadata shared by accepted facts in the document.
    pub source: ArchitectureInputSource,
    /// Space to create for the architecture graph.
    pub space: ArchitectureInputSpace,
    /// Explicit contexts referenced by components and inference rules.
    #[serde(default)]
    pub contexts: Vec<ArchitectureInputContext>,
    /// Accepted component facts to lift as zero-dimensional cells.
    pub components: Vec<ArchitectureInputComponent>,
    /// Accepted relation facts to lift as incidences.
    pub relations: Vec<ArchitectureInputRelation>,
    /// Unreviewed inferred structures to preserve as completion candidates.
    #[serde(default)]
    pub inferred_structures: Vec<ArchitectureInputInference>,
}

/// Source metadata for a bounded architecture input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputSource {
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
    /// Confidence applied to accepted facts that do not override it.
    pub confidence: Confidence,
}

/// Space declaration in a bounded architecture input document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputSpace {
    /// Stable space identifier.
    pub id: Id,
    /// Human-readable space name.
    pub name: String,
    /// Optional space description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Explicit architecture context declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputContext {
    /// Stable context identifier.
    pub id: Id,
    /// Human-readable context name.
    pub name: String,
}

/// Accepted component fact in a bounded architecture input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputComponent {
    /// Stable cell identifier to assign to the component.
    pub id: Id,
    /// Component type to map into the cell type.
    #[serde(rename = "type")]
    pub component_type: String,
    /// Human-readable component label.
    pub label: String,
    /// Context containing the component.
    pub context_id: Id,
    /// Optional source-local identifier for provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
    /// Optional fact-specific confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Accepted relation fact in a bounded architecture input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputRelation {
    /// Stable incidence identifier to assign to the relation.
    pub id: Id,
    /// Relation type to map into the incidence relation type.
    #[serde(rename = "type")]
    pub relation_type: String,
    /// Source component cell identifier.
    #[serde(rename = "from")]
    pub from_cell_id: Id,
    /// Target component cell identifier.
    #[serde(rename = "to")]
    pub to_cell_id: Id,
    /// Relation orientation.
    pub orientation: IncidenceOrientation,
    /// Optional finite relation weight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Optional source-local identifier for provenance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
    /// Optional fact-specific confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Unreviewed inferred architecture structure in a bounded input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputInference {
    /// Completion candidate identifier.
    pub id: Id,
    /// Kind of missing structure the inference proposes.
    pub missing_type: MissingType,
    /// Optional identifier for the proposed structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_id: Option<Id>,
    /// Proposed structure type.
    pub structure_type: String,
    /// Human-readable summary of the proposed structure.
    pub summary: String,
    /// Existing structures related to the proposed structure.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_ids: Vec<Id>,
    /// Accepted facts used to infer the candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_from: Vec<Id>,
    /// Explanation for why the structure is proposed.
    pub rationale: String,
    /// Contexts required before this inference applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Confidence in the inference.
    pub confidence: Confidence,
    /// Optional source-local identifier for the inference record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
}

/// Source report or snapshot used by the explicit completion review workflow.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewSourceReport {
    /// Stable schema identifier for the source report.
    pub schema: String,
    /// Stable type identifier for the source report.
    pub report_type: String,
    /// Version of the source report contract.
    pub report_version: u32,
    /// Command that produced the source report, if it came from the CLI.
    pub command: String,
}

/// Candidate snapshot and source metadata available for review.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewSnapshot {
    /// Source report or snapshot metadata.
    pub source_report: CompletionReviewSourceReport,
    /// Reviewable candidates captured from the source.
    pub completion_candidates: Vec<CompletionCandidate>,
}

/// Report view of the deterministic architecture direct DB access scenario.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDirectDbAccessSmokeScenario {
    /// Architecture smoke space identifier.
    pub space_id: Id,
    /// Workflow context identifier.
    pub workflow_context_id: Id,
    /// Context identifiers represented by the scenario.
    pub context_ids: Vec<Id>,
    /// Accepted scenario cells.
    pub cells: Vec<Cell>,
    /// Accepted scenario incidences.
    pub incidences: Vec<Incidence>,
    /// Invariant identifier evaluated by the workflow.
    pub invariant_id: Id,
    /// Human-readable invariant name.
    pub invariant_name: String,
}

/// Report view of a lifted architecture input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputLiftScenario {
    /// Input schema accepted by the lift workflow.
    pub input_schema: String,
    /// Source metadata shared by accepted facts.
    pub source: ArchitectureInputSource,
    /// Lifted space including registered cells, incidences, and contexts.
    pub space: Space,
    /// Explicit context declarations from the input document.
    pub contexts: Vec<ArchitectureInputContext>,
    /// Accepted component facts lifted into cells.
    pub cells: Vec<Cell>,
    /// Accepted relation facts lifted into incidences.
    pub incidences: Vec<Incidence>,
}

/// Report view of the explicit completion review scenario.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewScenario {
    /// Source report or snapshot metadata.
    pub source_report: CompletionReviewSourceReport,
    /// Candidate snapshot selected for explicit review.
    pub candidate: CompletionCandidate,
}

/// Machine-checkable architecture smoke workflow outcome.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDirectDbAccessSmokeResult {
    /// Deterministic workflow status.
    pub status: ArchitectureSmokeStatus,
    /// Violated invariant identifier.
    pub violated_invariant_id: Id,
    /// Lower-crate check result proving the violation.
    pub check_result: CheckResult,
    /// Obstructions produced by the violation.
    pub obstructions: Vec<Obstruction>,
    /// Reviewable completion candidates inferred from the obstruction.
    pub completion_candidates: Vec<CompletionCandidate>,
}

/// Machine-checkable architecture input lift workflow outcome.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureInputLiftResult {
    /// Input lift status.
    pub status: ArchitectureInputLiftStatus,
    /// Accepted fact identifiers lifted into the space.
    pub accepted_fact_ids: Vec<Id>,
    /// Unreviewed inferred structure candidate identifiers.
    pub inferred_structure_ids: Vec<Id>,
    /// Reviewable completion candidates preserved from the input.
    pub completion_candidates: Vec<CompletionCandidate>,
}

/// Machine-checkable explicit completion review workflow outcome.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewResult {
    /// Explicit review workflow status.
    pub status: CompletionReviewStatus,
    /// Audit record preserving the request, source candidate, and outcome.
    pub review_record: CompletionReviewRecord,
}

/// Runtime-owned status values for the architecture smoke workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureSmokeStatus {
    /// The checked invariant was satisfied.
    Satisfied,
    /// The workflow found the deterministic architecture violation.
    ViolationDetected,
}

/// Runtime-owned status values for the architecture input lift workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureInputLiftStatus {
    /// The bounded input was lifted into HigherGraphen structures.
    Lifted,
}

/// Runtime-owned status values for the explicit completion review workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionReviewStatus {
    /// The selected candidate was explicitly accepted.
    Accepted,
    /// The selected candidate was explicitly rejected.
    Rejected,
}

/// Stable architecture review projection with human, AI-agent, and audit views.
pub type ArchitectureDirectDbAccessSmokeProjection = ProjectionViewSet;

/// Stable architecture input lift projection with human, AI-agent, and audit views.
pub type ArchitectureInputLiftProjection = ProjectionViewSet;

/// Stable completion review projection with human, AI-agent, and audit views.
pub type CompletionReviewProjection = ProjectionViewSet;

/// Audience-specific projection bundle used by runtime workflow reports.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionViewSet {
    /// Compatibility audience for consumers that read the legacy top-level view.
    pub audience: ProjectionAudience,
    /// Compatibility purpose for consumers that read the legacy top-level view.
    pub purpose: ProjectionPurpose,
    /// Compatibility human-readable summary.
    pub summary: String,
    /// Compatibility recommended follow-up actions.
    pub recommended_actions: Vec<String>,
    /// Declared projection information loss for the compatibility view.
    pub information_loss: Vec<InformationLoss>,
    /// Source identifiers represented in the compatibility view.
    pub source_ids: Vec<Id>,
    /// Concise human architecture or completion review.
    pub human_review: HumanReviewProjectionView,
    /// Machine-oriented source-stable projection for AI agents.
    pub ai_view: AiProjectionView,
    /// Audit trace of represented source IDs and view coverage.
    pub audit_trace: AuditProjectionView,
}

/// Human-oriented projection view.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HumanReviewProjectionView {
    /// Target projection audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Human-readable summary.
    pub summary: String,
    /// Recommended follow-up actions.
    pub recommended_actions: Vec<String>,
    /// Source identifiers represented in this view.
    pub source_ids: Vec<Id>,
    /// Declared information loss for this view.
    pub information_loss: Vec<InformationLoss>,
}

/// AI-agent-oriented projection view.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiProjectionView {
    /// Target projection audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Structured records preserved for machine consumers.
    pub records: Vec<AiProjectionRecord>,
    /// Source identifiers represented in this view.
    pub source_ids: Vec<Id>,
    /// Declared information loss for this view.
    pub information_loss: Vec<InformationLoss>,
}

/// One source-stable record in an AI projection view.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiProjectionRecord {
    /// Stable source or generated record identifier.
    pub id: Id,
    /// Record category.
    pub record_type: AiProjectionRecordType,
    /// Short machine-consumable summary.
    pub summary: String,
    /// Source identifiers represented by this record.
    pub source_ids: Vec<Id>,
    /// Confidence when the source structure carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    /// Review status when the source structure carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_status: Option<ReviewStatus>,
    /// Severity when the source structure carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    /// Source provenance when the source structure carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

/// AI projection record categories.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProjectionRecordType {
    /// Accepted cell structure.
    Cell,
    /// Accepted incidence or relation structure.
    Incidence,
    /// Invariant check result.
    CheckResult,
    /// Obstruction or violation.
    Obstruction,
    /// Reviewable completion candidate.
    CompletionCandidate,
    /// Explicit completion review outcome.
    CompletionReview,
    /// PR review context.
    Context,
    /// Changed file from a bounded PR snapshot.
    ChangedFile,
    /// Symbol from a bounded PR snapshot.
    Symbol,
    /// Owner from a bounded PR snapshot.
    Owner,
    /// Test from a bounded PR snapshot.
    Test,
    /// Dependency edge from a bounded PR snapshot.
    DependencyEdge,
    /// Evidence from a bounded PR snapshot.
    Evidence,
    /// Risk signal from a bounded PR snapshot.
    RiskSignal,
    /// Unreviewed review target recommendation.
    ReviewTarget,
}

/// Audit-oriented projection view.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuditProjectionView {
    /// Target projection audience.
    pub audience: ProjectionAudience,
    /// Projection purpose.
    pub purpose: ProjectionPurpose,
    /// Source identifiers represented in this view.
    pub source_ids: Vec<Id>,
    /// Declared information loss for this view.
    pub information_loss: Vec<InformationLoss>,
    /// Per-source coverage trace.
    pub traces: Vec<ProjectionTrace>,
}

/// Trace from a represented source ID into one or more projection views.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionTrace {
    /// Source identifier being represented.
    pub source_id: Id,
    /// Source role in the report.
    pub role: String,
    /// View names that represent this source.
    pub represented_in: Vec<String>,
}

/// Runtime projection audience values required by workflow reports.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    /// A human reviewer.
    Human,
    /// An AI agent or model using structured report data.
    AiAgent,
    /// An audit or traceability consumer.
    Audit,
}

/// Runtime projection purpose values required by workflow reports.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionPurpose {
    /// Architecture review workflow.
    ArchitectureReview,
    /// Completion candidate review workflow.
    CompletionReview,
    /// Audit traceability workflow.
    AuditTrace,
    /// PR review target recommendation workflow.
    PrReviewTargeting,
    /// Missing unit test detector workflow.
    TestGapDetection,
}
