use super::{
    confidence, dedupe_ids, generated_provenance, sanitize, CompletionCandidate,
    CompletionCandidateType, ObstructionRecord, ObstructionType,
};
use crate::workflow_model::{CompletionReviewAction, CompletionReviewRecord, WorkflowCaseGraph};
use higher_graphen_core::{Id, ReviewStatus};
use serde_json::{json, Value};
use std::collections::BTreeMap;

struct CandidateShape {
    candidate_type: CompletionCandidateType,
    suggested_structure: Value,
    rationale: &'static str,
}

pub(super) fn completion_candidates(
    graph: &WorkflowCaseGraph,
    obstructions: &[ObstructionRecord],
) -> Vec<CompletionCandidate> {
    let reviews = latest_reviews_by_candidate(&graph.completion_reviews);
    let mut candidates: BTreeMap<Id, CompletionCandidate> = BTreeMap::new();
    for obstruction in obstructions {
        let Some(shape) = candidate_shape(obstruction.obstruction_type) else {
            continue;
        };
        let id = Id::new(format!(
            "candidate:{}:{}",
            candidate_type_stem(shape.candidate_type),
            sanitize(obstruction.id.as_str())
        ))
        .expect("generated candidate id");
        if candidates.contains_key(&id) {
            continue;
        }

        candidates.insert(
            id.clone(),
            CompletionCandidate {
                id,
                candidate_type: shape.candidate_type,
                target_ids: dedupe_ids(
                    obstruction
                        .affected_ids
                        .iter()
                        .chain(&obstruction.witness_ids)
                        .cloned()
                        .collect(),
                ),
                suggested_structure: shape.suggested_structure,
                inferred_from: vec![obstruction.id.clone()],
                rationale: shape.rationale.to_owned(),
                confidence: confidence(0.82),
                review_status: ReviewStatus::Unreviewed,
                review_record_ids: Vec::new(),
                provenance: generated_provenance("Workflow completion candidate", 0.82),
            },
        );
    }
    candidates
        .into_values()
        .map(|candidate| apply_latest_review(candidate, &reviews))
        .collect()
}

fn latest_reviews_by_candidate(
    reviews: &[CompletionReviewRecord],
) -> BTreeMap<&str, Vec<&CompletionReviewRecord>> {
    let mut by_candidate = BTreeMap::<&str, Vec<&CompletionReviewRecord>>::new();
    for review in reviews {
        by_candidate
            .entry(review.candidate_id.as_str())
            .or_default()
            .push(review);
    }
    by_candidate
}

fn apply_latest_review(
    mut candidate: CompletionCandidate,
    reviews: &BTreeMap<&str, Vec<&CompletionReviewRecord>>,
) -> CompletionCandidate {
    let Some(candidate_reviews) = reviews.get(candidate.id.as_str()) else {
        return candidate;
    };
    candidate.review_record_ids = candidate_reviews
        .iter()
        .map(|review| review.id.clone())
        .collect();
    if let Some(review) = candidate_reviews.last() {
        candidate.review_status = match review.action {
            CompletionReviewAction::Accept => ReviewStatus::Accepted,
            CompletionReviewAction::Reject => ReviewStatus::Rejected,
            CompletionReviewAction::Reopen => ReviewStatus::Unreviewed,
        };
    }
    candidate
}

fn candidate_shape(obstruction_type: ObstructionType) -> Option<CandidateShape> {
    let shape = match obstruction_type {
        ObstructionType::MissingEvidence => CandidateShape {
            candidate_type: CompletionCandidateType::MissingEvidence,
            suggested_structure: json!({
                "structure_type": "evidence_record",
                "evidence_boundary": "source_backed_evidence",
                "review_status": "unreviewed"
            }),
            rationale: "A required evidence record is missing or only represented by inference.",
        },
        ObstructionType::MissingProof => CandidateShape {
            candidate_type: CompletionCandidateType::MissingProof,
            suggested_structure: json!({
                "structure_type": "work_item",
                "item_type": "proof",
                "state": "todo"
            }),
            rationale: "A required proof is absent or has not reached an accepted terminal state.",
        },
        ObstructionType::UnresolvedDependency => CandidateShape {
            candidate_type: CompletionCandidateType::MissingTask,
            suggested_structure: json!({
                "structure_type": "work_item",
                "item_type": "task",
                "state": "todo"
            }),
            rationale: "A dependency needs a completed task-like structure before downstream work is ready.",
        },
        ObstructionType::ExternalWait => CandidateShape {
            candidate_type: CompletionCandidateType::MissingReviewAction,
            suggested_structure: json!({
                "structure_type": "review_action",
                "purpose": "external_wait_resolution"
            }),
            rationale: "An external wait needs a reviewed resolution witness.",
        },
        ObstructionType::ReviewRequired => CandidateShape {
            candidate_type: CompletionCandidateType::MissingReviewAction,
            suggested_structure: json!({
                "structure_type": "review_action",
                "purpose": "blocked_state_resolution"
            }),
            rationale: "A blocked, failed, cancelled, or rejected state needs explicit review.",
        },
        ObstructionType::Contradiction => CandidateShape {
            candidate_type: CompletionCandidateType::MissingDecision,
            suggested_structure: json!({
                "structure_type": "work_item",
                "item_type": "decision",
                "state": "todo"
            }),
            rationale: "A contradictory relation needs a decision or review record.",
        },
        _ => return None,
    };
    Some(shape)
}

fn candidate_type_stem(candidate_type: CompletionCandidateType) -> &'static str {
    match candidate_type {
        CompletionCandidateType::MissingTask => "missing-task",
        CompletionCandidateType::MissingEvidence => "missing-evidence",
        CompletionCandidateType::MissingTest => "missing-test",
        CompletionCandidateType::MissingDecision => "missing-decision",
        CompletionCandidateType::MissingDependencyRelation => "missing-dependency-relation",
        CompletionCandidateType::MissingCase => "missing-case",
        CompletionCandidateType::MissingProjection => "missing-projection",
        CompletionCandidateType::MissingReviewAction => "missing-review-action",
        CompletionCandidateType::MissingProof => "missing-proof",
    }
}
