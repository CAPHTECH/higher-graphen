use crate::workflow_model::{
    CorrespondenceType, InformationLoss, ProjectionAudience, WorkflowProvenance, WorkflowSeverity,
};
use higher_graphen_core::{Confidence, Id, ReviewStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEvaluation {
    pub status: WorkflowReasoningStatus,
    pub readiness: ReadinessResult,
    pub obstructions: Vec<ObstructionRecord>,
    pub completion_candidates: Vec<CompletionCandidate>,
    pub evidence_findings: EvidenceFindings,
    pub projection: ProjectionResult,
    pub correspondence: Vec<CorrespondenceResult>,
    pub evolution: EvolutionResult,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowReasoningStatus {
    Ready,
    Blocked,
    Incomplete,
    ObstructionsDetected,
    ReviewRequired,
    InvalidWorkflow,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessResult {
    pub evaluated_work_item_ids: Vec<Id>,
    pub ready_item_ids: Vec<Id>,
    pub not_ready_items: Vec<NotReadyItem>,
    pub rule_results: Vec<ReadinessRuleResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NotReadyItem {
    pub work_item_id: Id,
    pub state: String,
    pub hard_dependency_ids: Vec<Id>,
    pub external_wait_ids: Vec<Id>,
    pub evidence_requirement_ids: Vec<Id>,
    pub proof_requirement_ids: Vec<Id>,
    pub obstruction_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessRuleResult {
    pub id: Id,
    pub rule_id: Id,
    pub target_work_item_id: Id,
    pub ready: bool,
    pub obstruction_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObstructionRecord {
    pub id: Id,
    pub obstruction_type: ObstructionType,
    pub affected_ids: Vec<Id>,
    pub source_constraint_id: Id,
    pub witness_ids: Vec<Id>,
    pub explanation: String,
    pub severity: WorkflowSeverity,
    pub required_resolution: String,
    pub blocking: bool,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObstructionType {
    UnresolvedDependency,
    ExternalWait,
    MissingEvidence,
    MissingProof,
    InvalidTransition,
    Contradiction,
    ImpossibleClosure,
    ProjectionLoss,
    CorrespondenceMismatch,
    ReviewRequired,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionCandidate {
    pub id: Id,
    pub candidate_type: CompletionCandidateType,
    pub target_ids: Vec<Id>,
    pub suggested_structure: Value,
    pub inferred_from: Vec<Id>,
    pub rationale: String,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionCandidateType {
    MissingTask,
    MissingEvidence,
    MissingTest,
    MissingDecision,
    MissingDependencyRelation,
    MissingCase,
    MissingProjection,
    MissingReviewAction,
    MissingProof,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFindings {
    pub accepted_evidence_ids: Vec<Id>,
    pub source_backed_evidence_ids: Vec<Id>,
    pub inference_record_ids: Vec<Id>,
    pub unreviewed_inference_ids: Vec<Id>,
    pub promoted_evidence_ids: Vec<Id>,
    pub boundary_violations: Vec<EvidenceBoundaryViolation>,
    pub findings: Vec<EvidenceFinding>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBoundaryViolation {
    pub id: Id,
    pub evidence_id: Id,
    pub violation_type: EvidenceBoundaryViolationType,
    pub explanation: String,
    pub severity: WorkflowSeverity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBoundaryViolationType {
    InferenceProjectedAsEvidence,
    MissingSource,
    MissingReviewPromotion,
    RejectedEvidenceUsed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFinding {
    pub id: Id,
    pub finding_type: EvidenceFindingType,
    pub evidence_ids: Vec<Id>,
    pub summary: String,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFindingType {
    AcceptedEvidencePresent,
    SourceBackedPendingReview,
    InferenceSeparated,
    PromotionRequired,
    EvidenceMissing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionResult {
    pub projection_profile_id: Id,
    pub audience: ProjectionAudience,
    pub represented_ids: Vec<Id>,
    pub omitted_ids: Vec<Id>,
    pub information_loss: Vec<InformationLoss>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorrespondenceResult {
    pub id: Id,
    pub correspondence_type: CorrespondenceType,
    pub left_ids: Vec<Id>,
    pub right_ids: Vec<Id>,
    pub mismatch_evidence_ids: Vec<Id>,
    pub transferable_pattern_ids: Vec<Id>,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvolutionResult {
    pub revision_id: Id,
    pub previous_revision_id: Id,
    pub transition_ids: Vec<Id>,
    pub appeared_obstruction_ids: Vec<Id>,
    pub resolved_obstruction_ids: Vec<Id>,
    pub accepted_completion_ids: Vec<Id>,
    pub rejected_completion_ids: Vec<Id>,
    pub persisted_shape_ids: Vec<Id>,
    pub invariant_breaks: Vec<InvariantBreak>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantBreak {
    pub transition_id: Id,
    pub invariant_id: Id,
    pub witness_ids: Vec<Id>,
}
