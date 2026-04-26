use super::{NativeReviewError, NativeReviewTargetKind, REVIEW_SCHEMA_VERSION};
use crate::{
    native_eval::{NativeCloseInvariantResult, NativeCompletionCandidate, NativeObstruction},
    native_model::{
        CaseCell, CaseCellType, CaseMorphismType, CaseRelationType, CaseSpace, EvidenceBoundary,
        ReviewAction,
    },
};
use higher_graphen_core::{Id, ReviewStatus, Severity, SourceKind};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn review_metadata(
    request: &super::NativeReviewRequest,
    outcome_review_status: ReviewStatus,
    morphism_id: &Id,
) -> Map<String, Value> {
    let mut metadata = Map::new();
    metadata.insert(
        "native_review_schema_version".to_owned(),
        json!(REVIEW_SCHEMA_VERSION),
    );
    metadata.insert(
        "review_id".to_owned(),
        json!(generated_id("review", &[morphism_id.as_str()])),
    );
    metadata.insert("target_kind".to_owned(), json!(request.target_kind));
    metadata.insert("target_id".to_owned(), json!(request.target_id));
    metadata.insert("action".to_owned(), json!(request.action));
    metadata.insert(
        "outcome_review_status".to_owned(),
        json!(outcome_review_status),
    );
    metadata.insert("reviewer_id".to_owned(), json!(request.reviewer_id));
    metadata.insert("reviewed_at".to_owned(), json!(request.reviewed_at));
    metadata.insert("reason".to_owned(), json!(request.reason.trim()));
    metadata
}

pub(super) fn explicit_reviews(case_space: &CaseSpace) -> BTreeMap<Id, Vec<ExplicitReview>> {
    let mut reviews = BTreeMap::<Id, Vec<ExplicitReview>>::new();
    for morphism in case_space.morphism_log.iter().map(|entry| &entry.morphism) {
        if !morphism.review_status.has_review_action() {
            continue;
        }
        let Some(target_id) = metadata_id(&morphism.metadata, "target_id") else {
            continue;
        };
        let Some(action) = metadata_review_action(&morphism.metadata, "action") else {
            continue;
        };
        let Some(_target_kind) = metadata_target_kind(&morphism.metadata, "target_kind") else {
            continue;
        };
        reviews
            .entry(target_id.clone())
            .or_default()
            .push(ExplicitReview {
                target_id,
                action,
                outcome: metadata_review_status(&morphism.metadata, "outcome_review_status")
                    .unwrap_or_else(|| outcome_status(action)),
            });
    }
    reviews
}

#[derive(Clone, Debug)]
pub(super) struct ExplicitReview {
    target_id: Id,
    action: ReviewAction,
    outcome: ReviewStatus,
}

pub(super) fn unresolved_hard_obstruction(
    obstruction: &NativeObstruction,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> bool {
    if !obstruction.blocking || !matches!(obstruction.severity, Severity::High | Severity::Critical)
    {
        return false;
    }
    !target_has_action(reviews, &obstruction.id, ReviewAction::Waive)
        && !target_has_action(reviews, &obstruction.id, ReviewAction::Defer)
}

pub(super) fn completion_reviewed_or_deferred(
    candidate: &NativeCompletionCandidate,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> bool {
    candidate.review_status.has_review_action()
        || target_has_terminal_review(reviews, &candidate.id)
}

pub(super) fn evidence_requirement_blockers(
    case_space: &CaseSpace,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> Vec<Id> {
    let cells = case_space
        .case_cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>();
    let mut blockers = Vec::new();
    for relation in case_space
        .case_relations
        .iter()
        .filter(|relation| relation.relation_type == CaseRelationType::RequiresEvidence)
    {
        if target_has_action(reviews, &relation.id, ReviewAction::Waive)
            || target_has_action(reviews, &relation.to_id, ReviewAction::Waive)
        {
            continue;
        }
        let acceptable = cells.get(&relation.to_id).is_some_and(|cell| {
            cell.cell_type == CaseCellType::Evidence && evidence_acceptable_for_close(cell, reviews)
        });
        if !acceptable {
            blockers.push(relation.to_id.clone());
        }
    }
    dedupe_ids(blockers)
}

fn evidence_acceptable_for_close(
    cell: &CaseCell,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> bool {
    if cell.provenance.review_status == ReviewStatus::Rejected
        || target_has_action(reviews, &cell.id, ReviewAction::Reject)
    {
        return false;
    }
    let boundary = cell
        .metadata
        .get("evidence_boundary")
        .and_then(Value::as_str)
        .map(evidence_boundary_value)
        .unwrap_or_else(|| {
            if cell.provenance.source.kind == SourceKind::Ai {
                EvidenceBoundary::Inferred
            } else {
                EvidenceBoundary::SourceBacked
            }
        });
    let review_promoted = target_has_action(reviews, &cell.id, ReviewAction::Accept);
    let has_source = !cell.source_ids.is_empty();
    let accepted = cell.provenance.review_status == ReviewStatus::Accepted;
    match boundary {
        EvidenceBoundary::SourceBacked => has_source,
        EvidenceBoundary::ReviewPromoted => has_source && (accepted || review_promoted),
        EvidenceBoundary::Inferred => has_source && review_promoted,
        EvidenceBoundary::Rejected | EvidenceBoundary::Contradicting => false,
    }
}

fn evidence_boundary_value(value: &str) -> EvidenceBoundary {
    match value {
        "source_backed" | "source_backed_evidence" => EvidenceBoundary::SourceBacked,
        "review_promoted" | "review_promotion" => EvidenceBoundary::ReviewPromoted,
        "rejected" => EvidenceBoundary::Rejected,
        "contradicting" => EvidenceBoundary::Contradicting,
        _ => EvidenceBoundary::Inferred,
    }
}

pub(super) fn target_has_terminal_review(
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
    target_id: &Id,
) -> bool {
    latest_review_for(reviews, target_id).is_some_and(|review| {
        matches!(
            review.action,
            ReviewAction::Accept | ReviewAction::Reject | ReviewAction::Defer
        ) && review.outcome.has_review_action()
    })
}

pub(super) fn target_has_action(
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
    target_id: &Id,
    action: ReviewAction,
) -> bool {
    latest_review_for(reviews, target_id)
        .is_some_and(|review| review.action == action && review.outcome.has_review_action())
}

fn latest_review_for<'a>(
    reviews: &'a BTreeMap<Id, Vec<ExplicitReview>>,
    target_id: &Id,
) -> Option<&'a ExplicitReview> {
    reviews
        .get(target_id)?
        .iter()
        .rev()
        .find(|review| review.target_id == *target_id)
}

fn metadata_id(metadata: &Map<String, Value>, key: &str) -> Option<Id> {
    metadata
        .get(key)?
        .as_str()
        .and_then(|value| Id::new(value).ok())
}

fn metadata_review_action(metadata: &Map<String, Value>, key: &str) -> Option<ReviewAction> {
    serde_json::from_value(metadata.get(key)?.clone()).ok()
}

fn metadata_target_kind(
    metadata: &Map<String, Value>,
    key: &str,
) -> Option<NativeReviewTargetKind> {
    serde_json::from_value(metadata.get(key)?.clone()).ok()
}

fn metadata_review_status(metadata: &Map<String, Value>, key: &str) -> Option<ReviewStatus> {
    serde_json::from_value(metadata.get(key)?.clone()).ok()
}

pub(super) fn close_invariant(
    invariant_id: &str,
    witness_ids: Vec<Id>,
    message: &str,
) -> NativeCloseInvariantResult {
    NativeCloseInvariantResult {
        invariant_id: id(invariant_id),
        passed: witness_ids.is_empty(),
        severity: Severity::High,
        witness_ids,
        message: Some(message.to_owned()),
    }
}

pub(super) fn outcome_status(action: ReviewAction) -> ReviewStatus {
    match action {
        ReviewAction::Accept | ReviewAction::Waive => ReviewStatus::Accepted,
        ReviewAction::Reject => ReviewStatus::Rejected,
        ReviewAction::Reopen => ReviewStatus::Unreviewed,
        ReviewAction::Defer | ReviewAction::Supersede => ReviewStatus::Reviewed,
    }
}

pub(super) fn has_known_id(case_space: &CaseSpace, target_id: &Id) -> bool {
    case_space
        .case_cells
        .iter()
        .any(|cell| cell.id == *target_id)
        || case_space
            .case_relations
            .iter()
            .any(|relation| relation.id == *target_id)
        || case_space
            .projections
            .iter()
            .any(|projection| projection.projection_id == *target_id)
        || case_space
            .morphism_log
            .iter()
            .any(|entry| entry.entry_id == *target_id || entry.morphism_id == *target_id)
        || case_space.revision.revision_id == *target_id
}

pub(super) fn target_kind_stem(target_kind: NativeReviewTargetKind) -> &'static str {
    match target_kind {
        NativeReviewTargetKind::Completion => "completion",
        NativeReviewTargetKind::Evidence => "evidence",
        NativeReviewTargetKind::Morphism => "morphism",
        NativeReviewTargetKind::ResidualRisk => "residual-risk",
        NativeReviewTargetKind::Waiver => "waiver",
    }
}

pub(super) fn action_stem(action: ReviewAction) -> &'static str {
    match action {
        ReviewAction::Accept => "accept",
        ReviewAction::Reject => "reject",
        ReviewAction::Reopen => "reopen",
        ReviewAction::Waive => "waive",
        ReviewAction::Defer => "defer",
        ReviewAction::Supersede => "supersede",
    }
}

pub(super) fn generated_id(prefix: &str, parts: &[&str]) -> Id {
    let suffix = parts
        .iter()
        .map(|part| sanitize(part))
        .collect::<Vec<_>>()
        .join(":");
    id(&format!("{prefix}:{suffix}"))
}

pub(super) fn dedupe_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn id(value: &str) -> Id {
    Id::new(value).expect("static or generated id")
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

pub(super) fn error(message: impl Into<String>) -> NativeReviewError {
    NativeReviewError {
        message: message.into(),
    }
}

pub(super) fn morphism_type_for_review(
    target_kind: NativeReviewTargetKind,
    action: ReviewAction,
) -> CaseMorphismType {
    match (target_kind, action) {
        (NativeReviewTargetKind::Completion, ReviewAction::Accept) => {
            CaseMorphismType::CompletionAccept
        }
        (NativeReviewTargetKind::Completion, ReviewAction::Reject) => {
            CaseMorphismType::CompletionReject
        }
        _ => CaseMorphismType::Review,
    }
}
