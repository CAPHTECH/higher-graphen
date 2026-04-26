use super::{
    dedupe_ids, sanitize, CompletionCandidate, CorrespondenceResult, EvidenceFindings,
    EvolutionResult, InvariantBreak, ObstructionRecord, ProjectionResult, ReadinessResult,
    WorkflowReasoningStatus,
};
use crate::workflow_model::{
    InformationLoss, ProjectionAudience, ProjectionProfile, TransitionRecord, WorkflowCaseGraph,
};
use higher_graphen_core::{Id, ReviewStatus};
use std::collections::BTreeSet;

pub(super) fn projection_result(graph: &WorkflowCaseGraph) -> ProjectionResult {
    let profile = graph
        .projection_profiles
        .iter()
        .find(|profile| profile.audience == ProjectionAudience::AiAgent)
        .or_else(|| graph.projection_profiles.first());
    match profile {
        Some(profile) => ProjectionResult {
            projection_profile_id: profile.id.clone(),
            audience: profile.audience,
            represented_ids: dedupe_ids(profile.included_ids.clone()),
            omitted_ids: dedupe_ids(profile.omitted_ids.clone()),
            information_loss: profile.information_loss.clone(),
        },
        None => default_projection_result(graph),
    }
}

pub(super) fn correspondence_results(graph: &WorkflowCaseGraph) -> Vec<CorrespondenceResult> {
    graph
        .correspondence_records
        .iter()
        .map(|record| CorrespondenceResult {
            id: record.id.clone(),
            correspondence_type: record.correspondence_type,
            left_ids: record.left_ids.clone(),
            right_ids: record.right_ids.clone(),
            mismatch_evidence_ids: record.mismatch_evidence_ids.clone(),
            transferable_pattern_ids: record.transferable_pattern_ids.clone(),
            confidence: record.confidence,
            review_status: record.review_status,
        })
        .collect()
}

pub(super) fn evolution_result(
    graph: &WorkflowCaseGraph,
    obstructions: &[ObstructionRecord],
    completion_candidates: &[CompletionCandidate],
) -> EvolutionResult {
    let latest = graph.transition_records.last();
    EvolutionResult {
        revision_id: latest
            .map(|transition| transition.to_revision_id.clone())
            .unwrap_or_else(|| generated_revision_id(&graph.workflow_graph_id, "current")),
        previous_revision_id: latest
            .map(|transition| transition.from_revision_id.clone())
            .unwrap_or_else(|| generated_revision_id(&graph.workflow_graph_id, "previous")),
        transition_ids: transition_ids(graph),
        appeared_obstruction_ids: appeared_obstruction_ids(obstructions),
        resolved_obstruction_ids: transition_ids_with_prefix(
            &graph.transition_records,
            "obstruction:",
        ),
        accepted_completion_ids: reviewed_completion_ids(
            completion_candidates,
            ReviewStatus::Accepted,
        ),
        rejected_completion_ids: reviewed_completion_ids(
            completion_candidates,
            ReviewStatus::Rejected,
        ),
        persisted_shape_ids: persisted_shape_ids(&graph.transition_records),
        invariant_breaks: graph
            .transition_records
            .iter()
            .flat_map(invariant_breaks_for_transition)
            .collect(),
    }
}

pub(super) fn evaluation_status(
    graph: &WorkflowCaseGraph,
    readiness: &ReadinessResult,
    obstructions: &[ObstructionRecord],
    evidence_findings: &EvidenceFindings,
) -> WorkflowReasoningStatus {
    if graph.work_items.is_empty() {
        WorkflowReasoningStatus::Incomplete
    } else if obstructions.iter().any(|obstruction| obstruction.blocking) {
        WorkflowReasoningStatus::ObstructionsDetected
    } else if !evidence_findings.unreviewed_inference_ids.is_empty() {
        WorkflowReasoningStatus::ReviewRequired
    } else if readiness.ready_item_ids.is_empty() {
        WorkflowReasoningStatus::Blocked
    } else {
        WorkflowReasoningStatus::Ready
    }
}

pub fn projection_profile_for(
    graph: &WorkflowCaseGraph,
    audience: ProjectionAudience,
) -> Option<&ProjectionProfile> {
    graph
        .projection_profiles
        .iter()
        .find(|profile| profile.audience == audience)
}

fn default_projection_result(graph: &WorkflowCaseGraph) -> ProjectionResult {
    let work_item_ids = graph
        .work_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    ProjectionResult {
        projection_profile_id: Id::new("projection:workflow-default").expect("static id"),
        audience: ProjectionAudience::AiAgent,
        represented_ids: work_item_ids.clone(),
        omitted_ids: Vec::new(),
        information_loss: vec![InformationLoss {
            description: "No projection profile was supplied; default projection lists work item identifiers only.".to_owned(),
            represented_ids: work_item_ids,
            omitted_ids: Vec::new(),
        }],
    }
}

fn transition_ids(graph: &WorkflowCaseGraph) -> Vec<Id> {
    graph
        .transition_records
        .iter()
        .map(|transition| transition.id.clone())
        .collect()
}

fn appeared_obstruction_ids(obstructions: &[ObstructionRecord]) -> Vec<Id> {
    obstructions
        .iter()
        .filter(|obstruction| obstruction.blocking)
        .map(|obstruction| obstruction.id.clone())
        .collect()
}

fn reviewed_completion_ids(
    completion_candidates: &[CompletionCandidate],
    review_status: ReviewStatus,
) -> Vec<Id> {
    completion_candidates
        .iter()
        .filter(|candidate| candidate.review_status == review_status)
        .map(|candidate| candidate.id.clone())
        .collect()
}

fn persisted_shape_ids(transitions: &[TransitionRecord]) -> Vec<Id> {
    transitions
        .iter()
        .flat_map(|transition| transition.preserved_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn transition_ids_with_prefix(transitions: &[TransitionRecord], prefix: &str) -> Vec<Id> {
    transitions
        .iter()
        .flat_map(|transition| transition.changed_ids.removed_ids.iter())
        .filter(|id| id.as_str().starts_with(prefix))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn invariant_breaks_for_transition(transition: &TransitionRecord) -> Vec<InvariantBreak> {
    transition
        .violated_invariant_ids
        .iter()
        .map(|invariant_id| InvariantBreak {
            transition_id: transition.id.clone(),
            invariant_id: invariant_id.clone(),
            witness_ids: transition
                .changed_ids
                .added_ids
                .iter()
                .chain(&transition.changed_ids.updated_ids)
                .chain(&transition.source_ids)
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect()
}

fn generated_revision_id(workflow_graph_id: &Id, suffix: &str) -> Id {
    Id::new(format!(
        "revision:{}:{suffix}",
        sanitize(workflow_graph_id.as_str())
    ))
    .expect("generated revision id")
}
