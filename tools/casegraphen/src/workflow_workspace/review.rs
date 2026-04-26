use super::{
    contract_error, path_segment, workflow_record_ids, WorkflowCompletionPatchRequest,
    WorkflowCompletionPatchResult, WorkflowCompletionReviewRequest, WorkflowCompletionReviewResult,
    WorkflowHistoryEntry, WorkflowHistoryEventType, WorkflowPatchCheckResult,
    WorkflowPatchReviewAction, WorkflowPatchReviewRequest, WorkflowPatchReviewResult,
    WorkflowWorkspaceStore,
};
use crate::{
    store::{StoreError, StoreResult},
    workflow_eval::{evaluate_completion_candidates, CompletionCandidate},
    workflow_model::{
        ChangeSet, CompletionReviewAction, CompletionReviewRecord, TransitionRecord,
        TransitionType, WorkflowCaseGraph, WorkflowProvenance, WorkflowSourceRef,
    },
};
use higher_graphen_core::{Confidence, Id, ReviewStatus};
use serde_json::{json, Map, Value};
use std::{collections::BTreeSet, path::Path, path::PathBuf};

impl WorkflowWorkspaceStore {
    pub fn review_completion_candidate(
        &self,
        workflow_graph_id: &Id,
        action: CompletionReviewAction,
        request: WorkflowCompletionReviewRequest,
    ) -> StoreResult<WorkflowCompletionReviewResult> {
        let replay = self.replay_current_graph(workflow_graph_id)?;
        let mut graph = replay.graph;
        let previous_revision_id = replay.current_revision_id;
        let candidate_before =
            completion_candidate_by_id(&self.root, &graph, &request.candidate_id)?;
        let review_record = completion_review_record(&candidate_before, action, &request, &graph)?;
        graph.completion_reviews.push(review_record.clone());
        let candidate_after =
            completion_candidate_by_id(&self.root, &graph, &request.candidate_id)?;
        let transition_record = completion_review_transition(
            &graph,
            &candidate_before,
            &review_record,
            &previous_revision_id,
            &request.revision_id,
        );
        graph.transition_records.push(transition_record.clone());

        let workspace_record = self.append_history_entry(
            &graph,
            WorkflowHistoryEntry::snapshot(
                &graph,
                request.revision_id,
                Some(previous_revision_id),
                WorkflowHistoryEventType::Review,
                ChangeSet {
                    added_ids: vec![review_record.id.clone(), transition_record.id.clone()],
                    removed_ids: Vec::new(),
                    updated_ids: vec![candidate_before.id.clone()],
                },
            ),
        )?;

        Ok(WorkflowCompletionReviewResult {
            action,
            candidate_before_review: candidate_before,
            candidate_after_review: candidate_after,
            review_record,
            transition_record,
            workspace_record,
        })
    }

    pub fn convert_completion_to_patch(
        &self,
        workflow_graph_id: &Id,
        request: WorkflowCompletionPatchRequest,
    ) -> StoreResult<WorkflowCompletionPatchResult> {
        let replay = self.replay_current_graph(workflow_graph_id)?;
        let mut graph = replay.graph;
        let previous_revision_id = replay.current_revision_id;
        let candidate = completion_candidate_by_id(&self.root, &graph, &request.candidate_id)?;
        if candidate.review_status != ReviewStatus::Accepted {
            return Err(contract_error(
                &self.root,
                format!(
                    "completion candidate {} must be accepted before patch conversion",
                    candidate.id
                ),
            ));
        }

        let transition_record =
            completion_patch_transition(&graph, &candidate, &previous_revision_id, &request)?;
        if graph
            .transition_records
            .iter()
            .any(|record| record.id == transition_record.id)
        {
            return Err(contract_error(
                &self.root,
                format!(
                    "workflow transition {} already exists",
                    transition_record.id
                ),
            ));
        }

        graph.transition_records.push(transition_record.clone());
        let workspace_record = self.append_history_entry(
            &graph,
            WorkflowHistoryEntry::snapshot(
                &graph,
                request.revision_id,
                Some(previous_revision_id),
                WorkflowHistoryEventType::Patch,
                ChangeSet {
                    added_ids: vec![transition_record.id.clone()],
                    removed_ids: Vec::new(),
                    updated_ids: vec![candidate.id.clone()],
                },
            ),
        )?;

        Ok(WorkflowCompletionPatchResult {
            candidate,
            transition_record,
            workspace_record,
            applied: false,
        })
    }

    pub fn check_patch_transition(
        &self,
        workflow_graph_id: &Id,
        transition_id: &Id,
    ) -> StoreResult<WorkflowPatchCheckResult> {
        let replay = self.replay_current_graph(workflow_graph_id)?;
        let transition = patch_transition_by_id(&self.root, &replay.graph, transition_id)?;
        Ok(patch_check_result(transition))
    }

    pub fn review_patch_transition(
        &self,
        workflow_graph_id: &Id,
        action: WorkflowPatchReviewAction,
        request: WorkflowPatchReviewRequest,
    ) -> StoreResult<WorkflowPatchReviewResult> {
        let replay = self.replay_current_graph(workflow_graph_id)?;
        let mut graph = replay.graph;
        let previous_revision_id = replay.current_revision_id;
        let transition_index = graph
            .transition_records
            .iter()
            .position(|transition| transition.id == request.transition_id)
            .ok_or_else(|| {
                contract_error(
                    &self.root,
                    format!(
                        "unknown workflow patch transition {}",
                        request.transition_id
                    ),
                )
            })?;
        let before = graph.transition_records[transition_index].clone();
        require_patch_reviewable(&self.root, &before, action)?;

        graph.transition_records[transition_index].from_revision_id = previous_revision_id.clone();
        graph.transition_records[transition_index].to_revision_id = request.revision_id.clone();
        graph.transition_records[transition_index]
            .provenance
            .review_status = action.review_status();
        graph.transition_records[transition_index]
            .provenance
            .actor_id = Some(request.reviewer_id.clone());
        graph.transition_records[transition_index]
            .provenance
            .recorded_at = request.reviewed_at.clone();
        graph.transition_records[transition_index]
            .metadata
            .insert("patch_review_action".to_owned(), json!(action));
        graph.transition_records[transition_index]
            .metadata
            .insert("patch_review_reason".to_owned(), json!(request.reason));
        graph.transition_records[transition_index]
            .metadata
            .insert("patch_reviewer_id".to_owned(), json!(request.reviewer_id));
        let after = graph.transition_records[transition_index].clone();

        let workspace_record = self.append_history_entry(
            &graph,
            WorkflowHistoryEntry::snapshot(
                &graph,
                request.revision_id,
                Some(previous_revision_id),
                match action {
                    WorkflowPatchReviewAction::Apply => WorkflowHistoryEventType::Patch,
                    WorkflowPatchReviewAction::Reject => WorkflowHistoryEventType::Review,
                },
                ChangeSet {
                    added_ids: Vec::new(),
                    removed_ids: Vec::new(),
                    updated_ids: vec![after.id.clone()],
                },
            ),
        )?;

        Ok(WorkflowPatchReviewResult {
            action,
            transition_before_review: before,
            transition_after_review: after,
            workspace_record,
            materialized_record_count: 0,
        })
    }
}

fn completion_candidate_by_id(
    path: &Path,
    graph: &WorkflowCaseGraph,
    candidate_id: &Id,
) -> StoreResult<CompletionCandidate> {
    let candidates =
        evaluate_completion_candidates(graph).map_err(|source| StoreError::Validation {
            path: path.to_owned(),
            source,
        })?;
    candidates
        .into_iter()
        .find(|candidate| candidate.id == *candidate_id)
        .ok_or_else(|| contract_error(path, format!("unknown completion candidate {candidate_id}")))
}

fn completion_review_record(
    candidate: &CompletionCandidate,
    action: CompletionReviewAction,
    request: &WorkflowCompletionReviewRequest,
    graph: &WorkflowCaseGraph,
) -> StoreResult<CompletionReviewRecord> {
    require_non_empty(path_for_contract(graph), "reason", &request.reason)?;
    require_linked_evidence(graph, &request.evidence_ids)?;
    require_linked_decisions(graph, &request.decision_ids)?;

    let outcome_review_status = action.outcome_review_status();
    let id = Id::new(format!(
        "completion_review:{}:{}:{}",
        path_segment(&candidate.id),
        path_segment(&request.revision_id),
        completion_review_action_segment(action)
    ))
    .expect("review id is derived from validated ids");
    let mut source_ids = candidate.inferred_from.clone();
    source_ids.extend(candidate.target_ids.clone());
    source_ids.extend(request.evidence_ids.clone());
    source_ids.extend(request.decision_ids.clone());

    Ok(CompletionReviewRecord {
        id,
        candidate_id: candidate.id.clone(),
        action,
        outcome_review_status,
        reviewer_id: request.reviewer_id.clone(),
        reason: request.reason.trim().to_owned(),
        evidence_ids: request.evidence_ids.clone(),
        decision_ids: request.decision_ids.clone(),
        source_ids: dedupe_ids(source_ids),
        candidate_snapshot: serde_json::to_value(candidate).map_err(|source| StoreError::Json {
            path: PathBuf::from(graph.workflow_graph_id.as_str()),
            source,
        })?,
        provenance: reviewer_provenance(
            "workflow_completion_review",
            outcome_review_status,
            &request.reviewer_id,
            request.reviewed_at.clone(),
        ),
    })
}

fn completion_review_transition(
    graph: &WorkflowCaseGraph,
    candidate: &CompletionCandidate,
    review: &CompletionReviewRecord,
    from_revision_id: &Id,
    to_revision_id: &Id,
) -> TransitionRecord {
    let mut metadata = Map::new();
    metadata.insert("completion_review_id".to_owned(), json!(review.id));
    metadata.insert("completion_candidate_id".to_owned(), json!(candidate.id));
    metadata.insert("completion_review_action".to_owned(), json!(review.action));

    TransitionRecord {
        id: Id::new(format!(
            "transition:review:{}:{}",
            path_segment(&candidate.id),
            path_segment(to_revision_id)
        ))
        .expect("transition id is derived from validated ids"),
        transition_type: TransitionType::ReviewTransition,
        from_revision_id: from_revision_id.clone(),
        to_revision_id: to_revision_id.clone(),
        morphism_id: Id::new(format!(
            "morphism:completion-review:{}",
            path_segment(&review.id)
        ))
        .expect("morphism id is derived from validated ids"),
        source_workflow_graph_id: graph.workflow_graph_id.clone(),
        target_workflow_graph_id: graph.workflow_graph_id.clone(),
        changed_ids: ChangeSet {
            added_ids: vec![review.id.clone()],
            removed_ids: Vec::new(),
            updated_ids: vec![candidate.id.clone()],
        },
        preserved_ids: workflow_record_ids(graph),
        violated_invariant_ids: Vec::new(),
        source_ids: review.source_ids.clone(),
        provenance: review.provenance.clone(),
        metadata,
    }
}

fn completion_patch_transition(
    graph: &WorkflowCaseGraph,
    candidate: &CompletionCandidate,
    from_revision_id: &Id,
    request: &WorkflowCompletionPatchRequest,
) -> StoreResult<TransitionRecord> {
    require_non_empty(path_for_contract(graph), "reason", &request.reason)?;
    let transition_id = request.transition_id.clone().unwrap_or_else(|| {
        Id::new(format!("transition:patch:{}", path_segment(&candidate.id)))
            .expect("transition id is derived from validated candidate id")
    });
    let mut metadata = Map::new();
    metadata.insert("completion_candidate_id".to_owned(), json!(candidate.id));
    metadata.insert(
        "completion_candidate_snapshot".to_owned(),
        serde_json::to_value(candidate).map_err(|source| StoreError::Json {
            path: PathBuf::from(graph.workflow_graph_id.as_str()),
            source,
        })?,
    );
    metadata.insert("patch_state".to_owned(), json!("proposed"));
    metadata.insert("conversion_reason".to_owned(), json!(request.reason.trim()));
    metadata.insert(
        "conversion_reviewer_id".to_owned(),
        json!(request.reviewer_id),
    );

    let mut source_ids = candidate.inferred_from.clone();
    source_ids.extend(candidate.target_ids.clone());

    Ok(TransitionRecord {
        id: transition_id.clone(),
        transition_type: TransitionType::Patch,
        from_revision_id: from_revision_id.clone(),
        to_revision_id: request.revision_id.clone(),
        morphism_id: Id::new(format!(
            "morphism:completion-patch:{}",
            path_segment(&transition_id)
        ))
        .expect("morphism id is derived from validated transition id"),
        source_workflow_graph_id: graph.workflow_graph_id.clone(),
        target_workflow_graph_id: graph.workflow_graph_id.clone(),
        changed_ids: ChangeSet {
            added_ids: proposed_patch_added_ids(candidate),
            removed_ids: Vec::new(),
            updated_ids: candidate.target_ids.clone(),
        },
        preserved_ids: workflow_record_ids(graph),
        violated_invariant_ids: Vec::new(),
        source_ids: dedupe_ids(source_ids),
        provenance: reviewer_provenance(
            "workflow_completion_patch",
            ReviewStatus::Unreviewed,
            &request.reviewer_id,
            request.reviewed_at.clone(),
        ),
        metadata,
    })
}

fn patch_transition_by_id<'a>(
    path: &Path,
    graph: &'a WorkflowCaseGraph,
    transition_id: &Id,
) -> StoreResult<&'a TransitionRecord> {
    let transition = graph
        .transition_records
        .iter()
        .find(|transition| transition.id == *transition_id)
        .ok_or_else(|| {
            contract_error(
                path,
                format!("unknown workflow patch transition {transition_id}"),
            )
        })?;
    if transition.transition_type != TransitionType::Patch {
        return Err(contract_error(
            path,
            format!("{transition_id} is not a patch transition"),
        ));
    }
    Ok(transition)
}

fn patch_check_result(transition: &TransitionRecord) -> WorkflowPatchCheckResult {
    let valid = transition.transition_type == TransitionType::Patch
        && transition.violated_invariant_ids.is_empty();
    let applicable = valid
        && transition.provenance.review_status != ReviewStatus::Accepted
        && transition.provenance.review_status != ReviewStatus::Rejected;
    let reason = if !valid {
        "Patch transition has violated invariants or wrong transition type."
    } else if applicable {
        "Patch transition is reviewable and can be applied or rejected explicitly."
    } else if transition.provenance.review_status == ReviewStatus::Accepted {
        "Patch transition is already accepted."
    } else {
        "Patch transition is rejected."
    };

    WorkflowPatchCheckResult {
        transition_id: transition.id.clone(),
        transition_type: transition.transition_type,
        review_status: transition.provenance.review_status,
        valid,
        applicable,
        reason: reason.to_owned(),
        changed_ids: transition.changed_ids.clone(),
        preserved_ids: transition.preserved_ids.clone(),
        violated_invariant_ids: transition.violated_invariant_ids.clone(),
    }
}

fn require_patch_reviewable(
    path: &Path,
    transition: &TransitionRecord,
    action: WorkflowPatchReviewAction,
) -> StoreResult<()> {
    if transition.transition_type != TransitionType::Patch {
        return Err(contract_error(
            path,
            format!("{} is not a patch transition", transition.id),
        ));
    }
    if action == WorkflowPatchReviewAction::Apply && !transition.violated_invariant_ids.is_empty() {
        return Err(contract_error(
            path,
            format!(
                "{} cannot be applied because it violates invariant(s): {:?}",
                transition.id, transition.violated_invariant_ids
            ),
        ));
    }
    if transition.provenance.review_status == ReviewStatus::Accepted {
        return Err(contract_error(
            path,
            format!("{} has already been accepted", transition.id),
        ));
    }
    if transition.provenance.review_status == ReviewStatus::Rejected {
        return Err(contract_error(
            path,
            format!("{} has already been rejected", transition.id),
        ));
    }
    Ok(())
}

fn proposed_patch_added_ids(candidate: &CompletionCandidate) -> Vec<Id> {
    for key in ["id", "structure_id", "target_record_id"] {
        if let Some(value) = candidate
            .suggested_structure
            .get(key)
            .and_then(Value::as_str)
            .and_then(|value| Id::new(value.to_owned()).ok())
        {
            return vec![value];
        }
    }
    vec![
        Id::new(format!("patch_target:{}", path_segment(&candidate.id)))
            .expect("patch target id is derived from validated candidate id"),
    ]
}

fn require_linked_evidence(graph: &WorkflowCaseGraph, evidence_ids: &[Id]) -> StoreResult<()> {
    for evidence_id in evidence_ids {
        if graph
            .evidence_records
            .iter()
            .all(|record| record.id != *evidence_id)
        {
            return Err(contract_error(
                path_for_contract(graph),
                format!("unknown linked evidence record {evidence_id}"),
            ));
        }
    }
    Ok(())
}

fn require_linked_decisions(graph: &WorkflowCaseGraph, decision_ids: &[Id]) -> StoreResult<()> {
    for decision_id in decision_ids {
        if internal_decision_like_id(decision_id)
            && graph.work_items.iter().all(|item| item.id != *decision_id)
        {
            return Err(contract_error(
                path_for_contract(graph),
                format!("unknown linked decision record {decision_id}"),
            ));
        }
    }
    Ok(())
}

fn internal_decision_like_id(id: &Id) -> bool {
    id.as_str().starts_with("decision:")
}

fn reviewer_provenance(
    source_kind: &str,
    review_status: ReviewStatus,
    actor_id: &Id,
    recorded_at: Option<String>,
) -> WorkflowProvenance {
    WorkflowProvenance {
        source: WorkflowSourceRef {
            kind: source_kind.to_owned(),
            uri: None,
            title: None,
            captured_at: None,
            source_local_id: None,
        },
        confidence: Confidence::new(1.0).expect("static confidence"),
        review_status,
        recorded_at,
        actor_id: Some(actor_id.clone()),
        extraction_method: Some("casegraphen.workflow_workspace.review.v1".to_owned()),
    }
}

fn require_non_empty(path: &Path, field: &'static str, value: &str) -> StoreResult<()> {
    if value.trim().is_empty() {
        return Err(contract_error(
            path,
            format!("{field} must not be empty after trimming"),
        ));
    }
    Ok(())
}

fn dedupe_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn completion_review_action_segment(action: CompletionReviewAction) -> &'static str {
    match action {
        CompletionReviewAction::Accept => "accept",
        CompletionReviewAction::Reject => "reject",
        CompletionReviewAction::Reopen => "reopen",
    }
}

fn path_for_contract(graph: &WorkflowCaseGraph) -> &Path {
    Path::new(graph.workflow_graph_id.as_str())
}
