use crate::native_model::{CaseCellLifecycle, ProjectionAudience};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseEvaluation {
    pub status: NativeReasoningStatus,
    pub readiness: NativeReadiness,
    pub frontier_cell_ids: Vec<Id>,
    pub obstructions: Vec<NativeObstruction>,
    pub completion_candidates: Vec<NativeCompletionCandidate>,
    pub evidence_findings: NativeEvidenceFindings,
    pub review_gaps: Vec<NativeReviewGap>,
    pub projection_loss: Vec<NativeProjectionLoss>,
    pub correspondence: Vec<NativeCorrespondenceSummary>,
    pub evolution: NativeEvolutionSummary,
    pub close_check: NativeCloseCheckSkeleton,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeReasoningStatus {
    Ready,
    Blocked,
    Incomplete,
    ReviewRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReadiness {
    pub evaluated_cell_ids: Vec<Id>,
    pub ready_cell_ids: Vec<Id>,
    pub not_ready_cells: Vec<NativeNotReadyCell>,
    pub waiting_cell_ids: Vec<Id>,
    pub blocked_cell_ids: Vec<Id>,
    pub rule_results: Vec<NativeReadinessRuleResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeNotReadyCell {
    pub cell_id: Id,
    pub lifecycle: CaseCellLifecycle,
    pub hard_dependency_ids: Vec<Id>,
    pub wait_ids: Vec<Id>,
    pub evidence_requirement_ids: Vec<Id>,
    pub proof_requirement_ids: Vec<Id>,
    pub obstruction_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReadinessRuleResult {
    pub id: Id,
    pub rule_id: Id,
    pub target_cell_id: Id,
    pub ready: bool,
    pub obstruction_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeObstruction {
    pub id: Id,
    pub obstruction_type: NativeObstructionType,
    pub affected_ids: Vec<Id>,
    pub source_constraint_id: Id,
    pub witness_ids: Vec<Id>,
    pub explanation: String,
    pub severity: Severity,
    pub required_resolution: String,
    pub blocking: bool,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeObstructionType {
    UnresolvedDependency,
    ExternalWait,
    MissingEvidence,
    MissingProof,
    Contradiction,
    ReviewRequired,
    ProjectionLoss,
    InvalidClose,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCompletionCandidate {
    pub id: Id,
    pub candidate_type: NativeCompletionCandidateType,
    pub target_ids: Vec<Id>,
    pub suggested_structure: Value,
    pub inferred_from: Vec<Id>,
    pub rationale: String,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeCompletionCandidateType {
    NativeCompletionCell,
    MissingEvidence,
    MissingProof,
    MissingReview,
    MissingDependencyResolution,
    ContradictionResolution,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceFindings {
    pub accepted_evidence_ids: Vec<Id>,
    pub source_backed_evidence_ids: Vec<Id>,
    pub inference_record_ids: Vec<Id>,
    pub unreviewed_inference_ids: Vec<Id>,
    pub promoted_evidence_ids: Vec<Id>,
    pub boundary_violations: Vec<NativeEvidenceBoundaryViolation>,
    pub findings: Vec<NativeEvidenceFinding>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceBoundaryViolation {
    pub id: Id,
    pub evidence_id: Id,
    pub violation_type: NativeEvidenceBoundaryViolationType,
    pub explanation: String,
    pub severity: Severity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeEvidenceBoundaryViolationType {
    InferenceProjectedAsEvidence,
    MissingSource,
    MissingReviewPromotion,
    RejectedEvidenceUsed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvidenceFinding {
    pub id: Id,
    pub finding_type: NativeEvidenceFindingType,
    pub evidence_ids: Vec<Id>,
    pub summary: String,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeEvidenceFindingType {
    AcceptedEvidencePresent,
    SourceBackedPendingReview,
    InferenceSeparated,
    PromotionRequired,
    EvidenceMissing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReviewGap {
    pub id: Id,
    pub target_id: Id,
    pub gap_type: NativeReviewGapType,
    pub explanation: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeReviewGapType {
    UnreviewedCompletion,
    UnreviewedInference,
    UnreviewedMorphism,
    UnreviewedProjectionLoss,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectionLoss {
    pub projection_id: Id,
    pub audience: ProjectionAudience,
    pub omitted_cell_ids: Vec<Id>,
    pub omitted_relation_ids: Vec<Id>,
    pub information_loss_descriptions: Vec<String>,
    pub warning_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCorrespondenceSummary {
    pub id: Id,
    pub left_ids: Vec<Id>,
    pub right_ids: Vec<Id>,
    pub relation_ids: Vec<Id>,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvolutionSummary {
    pub revision_id: Id,
    pub previous_revision_id: Option<Id>,
    pub morphism_ids: Vec<Id>,
    pub added_ids: Vec<Id>,
    pub updated_ids: Vec<Id>,
    pub retired_ids: Vec<Id>,
    pub preserved_ids: Vec<Id>,
    pub invariant_breaks: Vec<NativeInvariantBreak>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeInvariantBreak {
    pub morphism_id: Id,
    pub invariant_id: Id,
    pub witness_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCloseCheckSkeleton {
    pub check_id: Id,
    pub case_space_id: Id,
    pub revision_id: Id,
    pub close_policy_id: Option<Id>,
    pub closable: bool,
    pub invariant_results: Vec<NativeCloseInvariantResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCloseInvariantResult {
    pub invariant_id: Id,
    pub passed: bool,
    pub severity: Severity,
    pub witness_ids: Vec<Id>,
    pub message: Option<String>,
}

pub type NativeEvalResult<T> = Result<T, NativeEvalError>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvalError {
    pub violations: Vec<NativeEvalViolation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeEvalViolation {
    pub code: NativeEvalViolationCode,
    pub record_id: Option<Id>,
    pub field: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeEvalViolationCode {
    SchemaMismatch,
    UnsupportedSchemaVersion,
    DuplicateId,
    EmptyRequiredField,
    SpaceMismatch,
    DanglingReference,
    InvalidMorphism,
    InvalidMorphismLog,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum ReadinessCheck {
    Lifecycle,
    Dependencies,
    Waits,
    Evidence,
    Proof,
    Contradictions,
    Reviews,
}
