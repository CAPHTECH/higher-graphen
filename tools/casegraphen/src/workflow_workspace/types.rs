use crate::{
    workflow_eval::CompletionCandidate,
    workflow_model::{ChangeSet, CompletionReviewRecord, TransitionRecord, WorkflowCaseGraph},
};
use higher_graphen_core::Id;
use higher_graphen_core::ReviewStatus;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const WORKFLOW_WORKSPACE_RECORD_SCHEMA: &str =
    "highergraphen.case.workflow.workspace_record.v1";
pub const WORKFLOW_HISTORY_ENTRY_SCHEMA: &str = "highergraphen.case.workflow.history_entry.v1";
pub const WORKFLOW_WORKSPACE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowWorkspaceRecord {
    pub schema: String,
    pub schema_version: u32,
    pub workflow_graph_id: Id,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub current_revision_id: Id,
    pub workflow_directory: String,
    pub history_path: String,
    pub current_graph_path: String,
    pub revision_count: u32,
    pub history_entry_count: u32,
    pub revisions: Vec<WorkflowRevisionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRevisionRecord {
    pub revision_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_revision_id: Option<Id>,
    pub event_type: WorkflowHistoryEventType,
    pub graph_path: String,
    pub changed_ids: ChangeSet,
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReplay {
    pub schema: String,
    pub schema_version: u32,
    pub workflow_graph_id: Id,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub current_revision_id: Id,
    pub graph: WorkflowCaseGraph,
    pub history: Vec<WorkflowHistoryEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowWorkspaceValidation {
    pub schema: String,
    pub schema_version: u32,
    pub workflow_graph_id: Id,
    pub current_revision_id: Id,
    pub history_entry_count: u32,
    pub valid: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowHistoryEntry {
    pub schema: String,
    pub schema_version: u32,
    pub id: Id,
    pub workflow_graph_id: Id,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub revision_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_revision_id: Option<Id>,
    pub event_type: WorkflowHistoryEventType,
    pub graph_path: String,
    pub changed_ids: ChangeSet,
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<Id>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowHistoryEventType {
    Imported,
    Snapshot,
    Transition,
    Patch,
    Review,
    Validation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPatchReviewAction {
    Apply,
    Reject,
}

impl WorkflowPatchReviewAction {
    pub fn review_status(self) -> ReviewStatus {
        match self {
            Self::Apply => ReviewStatus::Accepted,
            Self::Reject => ReviewStatus::Rejected,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCompletionReviewRequest {
    pub candidate_id: Id,
    pub reviewer_id: Id,
    pub reason: String,
    pub revision_id: Id,
    pub reviewed_at: Option<String>,
    pub evidence_ids: Vec<Id>,
    pub decision_ids: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCompletionReviewResult {
    pub action: crate::workflow_model::CompletionReviewAction,
    pub candidate_before_review: CompletionCandidate,
    pub candidate_after_review: CompletionCandidate,
    pub review_record: CompletionReviewRecord,
    pub transition_record: TransitionRecord,
    pub workspace_record: WorkflowWorkspaceRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCompletionPatchRequest {
    pub candidate_id: Id,
    pub reviewer_id: Id,
    pub reason: String,
    pub revision_id: Id,
    pub reviewed_at: Option<String>,
    pub transition_id: Option<Id>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCompletionPatchResult {
    pub candidate: CompletionCandidate,
    pub transition_record: TransitionRecord,
    pub workspace_record: WorkflowWorkspaceRecord,
    pub applied: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowPatchReviewRequest {
    pub transition_id: Id,
    pub reviewer_id: Id,
    pub reason: String,
    pub revision_id: Id,
    pub reviewed_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPatchCheckResult {
    pub transition_id: Id,
    pub transition_type: crate::workflow_model::TransitionType,
    pub review_status: ReviewStatus,
    pub valid: bool,
    pub applicable: bool,
    pub reason: String,
    pub changed_ids: ChangeSet,
    pub preserved_ids: Vec<Id>,
    pub violated_invariant_ids: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowPatchReviewResult {
    pub action: WorkflowPatchReviewAction,
    pub transition_before_review: TransitionRecord,
    pub transition_after_review: TransitionRecord,
    pub workspace_record: WorkflowWorkspaceRecord,
    pub materialized_record_count: u32,
}
