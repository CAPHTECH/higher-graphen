use crate::{
    native_eval::{
        evaluate_native_case, NativeCaseEvaluation, NativeCloseInvariantResult,
        NativeCompletionCandidate, NativeEvalError, NativeObstruction, NativeReviewGapType,
    },
    native_model::{
        CaseCell, CaseCellType, CaseMorphism, CaseMorphismType, CaseRelationType, CaseSpace,
        EvidenceBoundary, ReviewAction,
    },
};
use higher_graphen_core::{Id, ReviewStatus, Severity, SourceKind};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

const REVIEW_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeReviewTargetKind {
    Completion,
    Evidence,
    Morphism,
    ResidualRisk,
    Waiver,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReviewRequest {
    pub target_kind: NativeReviewTargetKind,
    pub target_id: Id,
    pub action: ReviewAction,
    pub reviewer_id: Id,
    pub reviewed_at: String,
    pub reason: String,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub target_revision_id: Id,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCloseCheckRequest {
    pub close_policy_id: Option<Id>,
    pub base_revision_id: Id,
    pub declared_projection_loss_ids: Vec<Id>,
    pub validation_evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCloseCheck {
    pub check_id: Id,
    pub case_space_id: Id,
    pub revision_id: Id,
    pub close_policy_id: Option<Id>,
    pub closeable: bool,
    pub invariant_results: Vec<NativeCloseInvariantResult>,
    pub blocker_ids: Vec<Id>,
}

pub type NativeReviewResult<T> = Result<T, NativeReviewError>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReviewError {
    pub message: String,
}

impl std::fmt::Display for NativeReviewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NativeReviewError {}

impl From<NativeEvalError> for NativeReviewError {
    fn from(error: NativeEvalError) -> Self {
        Self {
            message: format!("native case evaluation failed: {error:?}"),
        }
    }
}

pub fn accept_review_morphism(
    case_space: &CaseSpace,
    request: NativeReviewRequest,
) -> NativeReviewResult<CaseMorphism> {
    build_review_morphism(case_space, ReviewAction::Accept, request)
}

pub fn reject_review_morphism(
    case_space: &CaseSpace,
    request: NativeReviewRequest,
) -> NativeReviewResult<CaseMorphism> {
    build_review_morphism(case_space, ReviewAction::Reject, request)
}

pub fn reopen_review_morphism(
    case_space: &CaseSpace,
    request: NativeReviewRequest,
) -> NativeReviewResult<CaseMorphism> {
    build_review_morphism(case_space, ReviewAction::Reopen, request)
}

pub fn defer_review_morphism(
    case_space: &CaseSpace,
    request: NativeReviewRequest,
) -> NativeReviewResult<CaseMorphism> {
    build_review_morphism(case_space, ReviewAction::Defer, request)
}

pub fn build_review_morphism(
    case_space: &CaseSpace,
    action: ReviewAction,
    mut request: NativeReviewRequest,
) -> NativeReviewResult<CaseMorphism> {
    request.action = action;
    require_review_request(case_space, &request)?;
    let outcome_review_status = outcome_status(action);
    let morphism_type = match (request.target_kind, action) {
        (NativeReviewTargetKind::Completion, ReviewAction::Accept) => {
            CaseMorphismType::CompletionAccept
        }
        (NativeReviewTargetKind::Completion, ReviewAction::Reject) => {
            CaseMorphismType::CompletionReject
        }
        _ => CaseMorphismType::Review,
    };
    let mut source_ids = dedupe_ids(
        request
            .source_ids
            .iter()
            .chain(&request.evidence_ids)
            .cloned()
            .collect(),
    );
    if source_ids.is_empty() {
        source_ids = vec![request.target_id.clone()];
    }
    let morphism_id = generated_id(
        "morphism:review",
        &[
            target_kind_stem(request.target_kind),
            request.target_id.as_str(),
            action_stem(action),
            request.target_revision_id.as_str(),
        ],
    );
    Ok(CaseMorphism {
        morphism_id: morphism_id.clone(),
        morphism_type,
        source_revision_id: Some(case_space.revision.revision_id.clone()),
        target_revision_id: request.target_revision_id.clone(),
        added_ids: Vec::new(),
        updated_ids: Vec::new(),
        retired_ids: Vec::new(),
        preserved_ids: vec![request.target_id.clone()],
        violated_invariant_ids: Vec::new(),
        review_status: ReviewStatus::Accepted,
        evidence_ids: request.evidence_ids.clone(),
        source_ids: source_ids.clone(),
        metadata: review_metadata(&request, outcome_review_status, &morphism_id),
    })
}

pub fn check_native_close(
    case_space: &CaseSpace,
    request: NativeCloseCheckRequest,
) -> NativeReviewResult<NativeCloseCheck> {
    let evaluation = evaluate_native_case(case_space)?;
    let reviews = explicit_reviews(case_space);
    let invariant_results = close_invariants(case_space, &request, &evaluation, &reviews);

    let blocker_ids = dedupe_ids(
        invariant_results
            .iter()
            .filter(|result| !result.passed)
            .flat_map(|result| result.witness_ids.iter().cloned())
            .collect(),
    );
    Ok(NativeCloseCheck {
        check_id: generated_id(
            "close_check",
            &[
                case_space.case_space_id.as_str(),
                request.base_revision_id.as_str(),
                "native-review",
            ],
        ),
        case_space_id: case_space.case_space_id.clone(),
        revision_id: case_space.revision.revision_id.clone(),
        close_policy_id: request
            .close_policy_id
            .or_else(|| case_space.close_policy_id.clone()),
        closeable: invariant_results.iter().all(|result| result.passed),
        invariant_results,
        blocker_ids,
    })
}

fn close_invariants(
    case_space: &CaseSpace,
    request: &NativeCloseCheckRequest,
    evaluation: &NativeCaseEvaluation,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> Vec<NativeCloseInvariantResult> {
    vec![
        base_revision_invariant(case_space, request),
        hard_obstructions_invariant(evaluation, reviews),
        completions_reviewed_invariant(evaluation, reviews),
        morphisms_reviewed_invariant(evaluation, reviews),
        evidence_accepted_invariant(case_space, reviews),
        projection_loss_declared_invariant(request, evaluation, reviews),
        validation_evidence_invariant(case_space, request),
    ]
}

fn base_revision_invariant(
    case_space: &CaseSpace,
    request: &NativeCloseCheckRequest,
) -> NativeCloseInvariantResult {
    let witness_ids = if request.base_revision_id == case_space.revision.revision_id {
        Vec::new()
    } else {
        vec![
            request.base_revision_id.clone(),
            case_space.revision.revision_id.clone(),
        ]
    };
    close_invariant(
        "close:native-base-revision-matches",
        witness_ids,
        "The close-check base revision must match the materialized case-space revision.",
    )
}

fn hard_obstructions_invariant(
    evaluation: &NativeCaseEvaluation,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-no-hard-obstructions",
        evaluation
            .obstructions
            .iter()
            .filter(|obstruction| unresolved_hard_obstruction(obstruction, reviews))
            .map(|obstruction| obstruction.id.clone())
            .collect(),
        "No unresolved high or critical hard obstruction may remain.",
    )
}

fn completions_reviewed_invariant(
    evaluation: &NativeCaseEvaluation,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-completions-reviewed",
        evaluation
            .completion_candidates
            .iter()
            .filter(|candidate| !completion_reviewed_or_deferred(candidate, reviews))
            .map(|candidate| candidate.id.clone())
            .collect(),
        "Completion candidates must be accepted, rejected, or explicitly deferred.",
    )
}

fn morphisms_reviewed_invariant(
    evaluation: &NativeCaseEvaluation,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-morphisms-reviewed",
        evaluation
            .review_gaps
            .iter()
            .filter(|gap| gap.gap_type == NativeReviewGapType::UnreviewedMorphism)
            .filter(|gap| !target_has_terminal_review(reviews, &gap.target_id))
            .map(|gap| gap.target_id.clone())
            .collect(),
        "Generated morphisms must remain reviewable until accepted, rejected, or deferred.",
    )
}

fn evidence_accepted_invariant(
    case_space: &CaseSpace,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-evidence-accepted-or-waived",
        evidence_requirement_blockers(case_space, reviews),
        "Required evidence must be source-backed, review-promoted, accepted, or explicitly waived.",
    )
}

fn projection_loss_declared_invariant(
    request: &NativeCloseCheckRequest,
    evaluation: &NativeCaseEvaluation,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-projection-loss-declared",
        evaluation
            .projection_loss
            .iter()
            .filter(|loss| {
                !request
                    .declared_projection_loss_ids
                    .contains(&loss.projection_id)
            })
            .filter(|loss| !target_has_action(reviews, &loss.projection_id, ReviewAction::Accept))
            .map(|loss| loss.projection_id.clone())
            .collect(),
        "Projection loss must be declared by the close-check caller or accepted by review.",
    )
}

fn validation_evidence_invariant(
    case_space: &CaseSpace,
    request: &NativeCloseCheckRequest,
) -> NativeCloseInvariantResult {
    close_invariant(
        "close:native-validation-evidence-named",
        if request.validation_evidence_ids.is_empty() {
            vec![case_space.revision.revision_id.clone()]
        } else {
            Vec::new()
        },
        "Close checks must name validation evidence for the exact revision.",
    )
}

fn require_review_request(
    case_space: &CaseSpace,
    request: &NativeReviewRequest,
) -> NativeReviewResult<()> {
    if request.reason.trim().is_empty() {
        return Err(error("review reason must not be empty"));
    }
    if request.target_revision_id == case_space.revision.revision_id {
        return Err(error("review target_revision_id must advance the revision"));
    }
    for evidence_id in &request.evidence_ids {
        if !has_known_id(case_space, evidence_id) {
            return Err(error(format!("unknown review evidence id {evidence_id}")));
        }
    }
    match request.target_kind {
        NativeReviewTargetKind::Completion => {
            require_completion_target(case_space, &request.target_id)
        }
        NativeReviewTargetKind::Evidence => require_cell_target(
            case_space,
            &request.target_id,
            CaseCellType::Evidence,
            "evidence",
        ),
        NativeReviewTargetKind::Morphism => {
            if case_space
                .morphism_log
                .iter()
                .any(|entry| entry.morphism_id == request.target_id)
            {
                Ok(())
            } else {
                Err(error(format!(
                    "unknown morphism target {}",
                    request.target_id
                )))
            }
        }
        NativeReviewTargetKind::ResidualRisk => {
            require_obstruction_target(case_space, &request.target_id)
        }
        NativeReviewTargetKind::Waiver => {
            if has_known_id(case_space, &request.target_id)
                || evaluate_native_case(case_space)?
                    .obstructions
                    .iter()
                    .any(|obstruction| obstruction.id == request.target_id)
            {
                Ok(())
            } else {
                Err(error(format!(
                    "unknown waiver target {}",
                    request.target_id
                )))
            }
        }
    }
}

fn require_completion_target(case_space: &CaseSpace, target_id: &Id) -> NativeReviewResult<()> {
    if case_space
        .case_cells
        .iter()
        .any(|cell| cell.id == *target_id && cell.cell_type == CaseCellType::Completion)
        || evaluate_native_case(case_space)?
            .completion_candidates
            .iter()
            .any(|candidate| candidate.id == *target_id)
    {
        Ok(())
    } else {
        Err(error(format!("unknown completion target {target_id}")))
    }
}

fn require_obstruction_target(case_space: &CaseSpace, target_id: &Id) -> NativeReviewResult<()> {
    if evaluate_native_case(case_space)?
        .obstructions
        .iter()
        .any(|obstruction| obstruction.id == *target_id)
    {
        Ok(())
    } else {
        Err(error(format!("unknown residual-risk target {target_id}")))
    }
}

fn require_cell_target(
    case_space: &CaseSpace,
    target_id: &Id,
    cell_type: CaseCellType,
    label: &str,
) -> NativeReviewResult<()> {
    if case_space
        .case_cells
        .iter()
        .any(|cell| cell.id == *target_id && cell.cell_type == cell_type)
    {
        Ok(())
    } else {
        Err(error(format!("unknown {label} target {target_id}")))
    }
}

fn review_metadata(
    request: &NativeReviewRequest,
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

fn explicit_reviews(case_space: &CaseSpace) -> BTreeMap<Id, Vec<ExplicitReview>> {
    let mut reviews = BTreeMap::<Id, Vec<ExplicitReview>>::new();
    for morphism in case_space.morphism_log.iter().map(|entry| &entry.morphism) {
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
struct ExplicitReview {
    target_id: Id,
    action: ReviewAction,
    outcome: ReviewStatus,
}

fn unresolved_hard_obstruction(
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

fn completion_reviewed_or_deferred(
    candidate: &NativeCompletionCandidate,
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
) -> bool {
    candidate.review_status.has_review_action()
        || target_has_terminal_review(reviews, &candidate.id)
}

fn evidence_requirement_blockers(
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
        let acceptable = cells
            .get(&relation.to_id)
            .is_some_and(|cell| evidence_acceptable_for_close(cell, reviews));
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

fn target_has_terminal_review(reviews: &BTreeMap<Id, Vec<ExplicitReview>>, target_id: &Id) -> bool {
    reviews.get(target_id).is_some_and(|reviews| {
        reviews.iter().any(|review| {
            matches!(
                review.action,
                ReviewAction::Accept | ReviewAction::Reject | ReviewAction::Defer
            ) && review.outcome.has_review_action()
        })
    })
}

fn target_has_action(
    reviews: &BTreeMap<Id, Vec<ExplicitReview>>,
    target_id: &Id,
    action: ReviewAction,
) -> bool {
    reviews.get(target_id).is_some_and(|reviews| {
        reviews
            .iter()
            .any(|review| review.target_id == *target_id && review.action == action)
    })
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

fn close_invariant(
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

fn outcome_status(action: ReviewAction) -> ReviewStatus {
    match action {
        ReviewAction::Accept | ReviewAction::Waive => ReviewStatus::Accepted,
        ReviewAction::Reject => ReviewStatus::Rejected,
        ReviewAction::Reopen => ReviewStatus::Unreviewed,
        ReviewAction::Defer | ReviewAction::Supersede => ReviewStatus::Reviewed,
    }
}

fn has_known_id(case_space: &CaseSpace, target_id: &Id) -> bool {
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

fn target_kind_stem(target_kind: NativeReviewTargetKind) -> &'static str {
    match target_kind {
        NativeReviewTargetKind::Completion => "completion",
        NativeReviewTargetKind::Evidence => "evidence",
        NativeReviewTargetKind::Morphism => "morphism",
        NativeReviewTargetKind::ResidualRisk => "residual-risk",
        NativeReviewTargetKind::Waiver => "waiver",
    }
}

fn action_stem(action: ReviewAction) -> &'static str {
    match action {
        ReviewAction::Accept => "accept",
        ReviewAction::Reject => "reject",
        ReviewAction::Reopen => "reopen",
        ReviewAction::Waive => "waive",
        ReviewAction::Defer => "defer",
        ReviewAction::Supersede => "supersede",
    }
}

fn generated_id(prefix: &str, parts: &[&str]) -> Id {
    let suffix = parts
        .iter()
        .map(|part| sanitize(part))
        .collect::<Vec<_>>()
        .join(":");
    id(&format!("{prefix}:{suffix}"))
}

fn dedupe_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn id(value: &str) -> Id {
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

fn error(message: impl Into<String>) -> NativeReviewError {
    NativeReviewError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests;
