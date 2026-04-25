use higher_graphen_core::{Confidence, Id, ReviewStatus};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const WORKFLOW_GRAPH_SCHEMA: &str = "highergraphen.case.workflow.graph.v1";
pub const WORKFLOW_GRAPH_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCaseGraph {
    pub schema: String,
    pub schema_version: u32,
    pub workflow_graph_id: Id,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub work_items: Vec<WorkItem>,
    pub workflow_relations: Vec<WorkflowRelation>,
    pub readiness_rules: Vec<ReadinessRule>,
    pub evidence_records: Vec<EvidenceRecord>,
    pub transition_records: Vec<TransitionRecord>,
    pub projection_profiles: Vec<ProjectionProfile>,
    pub correspondence_records: Vec<CorrespondenceRecord>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkItem {
    pub id: Id,
    pub space_id: Id,
    pub item_type: WorkItemType,
    pub title: String,
    pub state: WorkItemState,
    pub case_ids: Vec<Id>,
    pub hard_dependency_ids: Vec<Id>,
    pub external_wait_ids: Vec<Id>,
    pub evidence_requirement_ids: Vec<Id>,
    pub proof_requirement_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: WorkflowProvenance,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemType {
    Task,
    Goal,
    Decision,
    Event,
    Evidence,
    Proof,
    ExternalWait,
    ReviewAction,
    Case,
    Milestone,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemState {
    Proposed,
    Todo,
    Doing,
    Waiting,
    Blocked,
    Done,
    Cancelled,
    Failed,
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRelation {
    pub id: Id,
    pub relation_type: WorkflowRelationType,
    pub from_id: Id,
    pub to_id: Id,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRelationType {
    DependsOn,
    WaitsFor,
    RequiresEvidence,
    RequiresProof,
    Verifies,
    Blocks,
    Contradicts,
    Completes,
    DerivesFrom,
    TransitionsTo,
    ProjectsTo,
    CorrespondsTo,
    Supersedes,
    RelatesTo,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessRule {
    pub id: Id,
    pub rule_type: ReadinessRuleType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub target_item_ids: Vec<Id>,
    pub required_relation_types: Vec<String>,
    pub required_evidence_types: Vec<String>,
    pub blocked_state_values: Vec<String>,
    pub obstruction_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_constraint_id: Option<Id>,
    pub severity: WorkflowSeverity,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessRuleType {
    DependencyClosure,
    ExternalWaitResolved,
    EvidenceAvailable,
    ProofAvailable,
    TransitionValid,
    ReviewStatus,
    ObstructionAbsent,
    Custom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub id: Id,
    pub evidence_type: EvidenceType,
    pub evidence_boundary: EvidenceBoundary,
    pub summary: String,
    pub supports_ids: Vec<Id>,
    pub contradicts_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Document,
    CommandOutput,
    TestResult,
    ReviewRecord,
    HumanObservation,
    AiInference,
    Proof,
    TransitionWitness,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBoundary {
    AcceptedEvidence,
    SourceBackedEvidence,
    AiInference,
    ReviewPromotion,
    RejectedEvidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionRecord {
    pub id: Id,
    pub transition_type: TransitionType,
    pub from_revision_id: Id,
    pub to_revision_id: Id,
    pub morphism_id: Id,
    pub source_workflow_graph_id: Id,
    pub target_workflow_graph_id: Id,
    pub changed_ids: ChangeSet,
    pub preserved_ids: Vec<Id>,
    pub violated_invariant_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    Patch,
    StateTransition,
    ReviewTransition,
    Projection,
    EvolutionRevision,
    Migration,
    CorrespondenceTransfer,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSet {
    pub added_ids: Vec<Id>,
    pub removed_ids: Vec<Id>,
    pub updated_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionProfile {
    pub id: Id,
    pub audience: ProjectionAudience,
    pub purpose: String,
    pub included_ids: Vec<Id>,
    pub omitted_ids: Vec<Id>,
    pub information_loss: Vec<InformationLoss>,
    pub allowed_operations: Vec<String>,
    pub source_ids: Vec<Id>,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    HumanReview,
    AiAgent,
    Audit,
    System,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InformationLoss {
    pub description: String,
    pub represented_ids: Vec<Id>,
    pub omitted_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorrespondenceRecord {
    pub id: Id,
    pub correspondence_type: CorrespondenceType,
    pub left_ids: Vec<Id>,
    pub right_ids: Vec<Id>,
    pub mismatch_evidence_ids: Vec<Id>,
    pub transferable_pattern_ids: Vec<Id>,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
    pub provenance: WorkflowProvenance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceType {
    Equivalent,
    SimilarWithLoss,
    ScenarioPatternMatch,
    Conflicting,
    NotComparable,
    TransferableMitigation,
    TransferableCompletion,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowSeverity {
    Info,
    Warning,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowProvenance {
    pub source: WorkflowSourceRef,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_method: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSourceRef {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    const WORKFLOW_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/workflow.graph.example.json");

    #[test]
    fn workflow_graph_example_deserializes() {
        let graph: WorkflowCaseGraph =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

        assert_eq!(graph.schema, WORKFLOW_GRAPH_SCHEMA);
        assert_eq!(graph.schema_version, WORKFLOW_GRAPH_SCHEMA_VERSION);
        assert_eq!(graph.work_items.len(), 3);
        assert_eq!(graph.workflow_relations.len(), 3);
        assert_eq!(graph.readiness_rules.len(), 2);
        assert_eq!(graph.evidence_records.len(), 2);
        assert_eq!(graph.transition_records.len(), 1);
        assert_eq!(graph.projection_profiles.len(), 2);
        assert_eq!(graph.correspondence_records.len(), 1);
    }

    #[test]
    fn workflow_boundaries_round_trip() {
        let graph: WorkflowCaseGraph =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
        let source_backed = graph
            .evidence_records
            .iter()
            .find(|record| record.id.as_str() == "evidence:workflow-target-doc")
            .expect("source-backed evidence");
        let inference = graph
            .evidence_records
            .iter()
            .find(|record| record.id.as_str() == "evidence:workflow-gap-inference")
            .expect("inference evidence");

        assert_eq!(
            source_backed.evidence_boundary,
            EvidenceBoundary::SourceBackedEvidence
        );
        assert_eq!(
            source_backed.provenance.review_status,
            ReviewStatus::Accepted
        );
        assert_eq!(source_backed.provenance.source.kind, "document");
        assert_eq!(inference.evidence_boundary, EvidenceBoundary::AiInference);
        assert_eq!(inference.provenance.review_status, ReviewStatus::Unreviewed);
        assert_eq!(inference.provenance.source.kind, "agent_inference");

        let round_trip: WorkflowCaseGraph =
            serde_json::from_str(&serde_json::to_string(&graph).expect("serialize workflow graph"))
                .expect("deserialize workflow graph");
        assert_eq!(round_trip, graph);
    }

    #[test]
    fn workflow_model_rejects_unknown_fields() {
        let mut value: Value =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example value");
        value["unexpected"] = Value::Bool(true);

        assert!(serde_json::from_value::<WorkflowCaseGraph>(value).is_err());
    }

    #[test]
    fn workflow_model_does_not_add_cli_workflow_command_surface() {
        let error = crate::cli::run([
            OsString::from("workflow"),
            OsString::from("validate"),
            OsString::from("--format"),
            OsString::from("json"),
        ])
        .expect_err("workflow CLI commands are intentionally out of scope");

        assert!(error.to_string().contains("unsupported command segment"));
    }
}
