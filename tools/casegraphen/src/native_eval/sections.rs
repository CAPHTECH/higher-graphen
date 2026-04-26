use super::*;
use crate::native_model::EvidenceBoundary;
use higher_graphen_core::SourceKind;
use serde_json::{json, Value};

pub(super) fn completion_candidates(
    case_space: &CaseSpace,
    obstructions: &[NativeObstruction],
) -> Vec<NativeCompletionCandidate> {
    let mut candidates = BTreeMap::new();
    for cell in case_space.case_cells.iter().filter(|cell| {
        cell.cell_type == CaseCellType::Completion
            && cell.provenance.review_status == ReviewStatus::Unreviewed
            && !matches!(
                cell.lifecycle,
                CaseCellLifecycle::Accepted | CaseCellLifecycle::Rejected
            )
    }) {
        candidates.insert(
            cell.id.clone(),
            NativeCompletionCandidate {
                id: cell.id.clone(),
                candidate_type: NativeCompletionCandidateType::NativeCompletionCell,
                target_ids: cell.structure_ids.clone(),
                suggested_structure: Value::Object(cell.metadata.clone()),
                inferred_from: cell.source_ids.clone(),
                rationale: cell.summary.clone().unwrap_or_else(|| cell.title.clone()),
                confidence: cell.provenance.confidence,
                review_status: cell.provenance.review_status,
                provenance: cell.provenance.clone(),
            },
        );
    }
    for obstruction in obstructions {
        let Some((candidate_type, suggested_structure, rationale)) =
            candidate_shape(obstruction.obstruction_type)
        else {
            continue;
        };
        let id = generated_id(
            "completion_candidate",
            &[candidate_type_stem(candidate_type), obstruction.id.as_str()],
        );
        candidates
            .entry(id.clone())
            .or_insert(NativeCompletionCandidate {
                id,
                candidate_type,
                target_ids: dedupe_ids(
                    obstruction
                        .affected_ids
                        .iter()
                        .chain(&obstruction.witness_ids)
                        .cloned()
                        .collect(),
                ),
                suggested_structure,
                inferred_from: vec![obstruction.id.clone()],
                rationale: rationale.to_owned(),
                confidence: confidence(0.82),
                review_status: ReviewStatus::Unreviewed,
                provenance: generated_provenance("Native completion candidate", 0.82),
            });
    }
    candidates.into_values().collect()
}

pub(super) fn evidence_findings(
    case_space: &CaseSpace,
    obstructions: &[NativeObstruction],
) -> NativeEvidenceFindings {
    let mut accum = EvidenceAccum::default();

    for evidence in case_space
        .case_cells
        .iter()
        .filter(|cell| cell.cell_type == CaseCellType::Evidence)
    {
        record_evidence(&mut accum, evidence);
    }
    for obstruction in obstructions
        .iter()
        .filter(|record| record.obstruction_type == NativeObstructionType::MissingEvidence)
    {
        accum.findings.push(NativeEvidenceFinding {
            id: generated_id("finding", &[obstruction.id.as_str(), "evidence-missing"]),
            finding_type: NativeEvidenceFindingType::EvidenceMissing,
            evidence_ids: obstruction.witness_ids.clone(),
            summary: obstruction.explanation.clone(),
            review_status: ReviewStatus::Unreviewed,
        });
    }

    NativeEvidenceFindings {
        accepted_evidence_ids: dedupe_ids(accum.accepted_evidence_ids),
        source_backed_evidence_ids: dedupe_ids(accum.source_backed_evidence_ids),
        inference_record_ids: dedupe_ids(accum.inference_record_ids),
        unreviewed_inference_ids: dedupe_ids(accum.unreviewed_inference_ids),
        promoted_evidence_ids: dedupe_ids(accum.promoted_evidence_ids),
        boundary_violations: accum.boundary_violations,
        findings: accum.findings,
    }
}

#[derive(Default)]
struct EvidenceAccum {
    accepted_evidence_ids: Vec<Id>,
    source_backed_evidence_ids: Vec<Id>,
    inference_record_ids: Vec<Id>,
    unreviewed_inference_ids: Vec<Id>,
    promoted_evidence_ids: Vec<Id>,
    boundary_violations: Vec<NativeEvidenceBoundaryViolation>,
    findings: Vec<NativeEvidenceFinding>,
}

fn record_evidence(accum: &mut EvidenceAccum, evidence: &CaseCell) {
    let boundary = evidence_boundary(evidence);
    record_accepted_evidence(accum, evidence, boundary);
    record_source_backed_evidence(accum, evidence, boundary);
    record_inference_evidence(accum, evidence, boundary);
    record_review_promotion(accum, evidence, boundary);
    record_rejected_evidence(accum, evidence, boundary);
}

fn record_accepted_evidence(
    accum: &mut EvidenceAccum,
    evidence: &CaseCell,
    boundary: EvidenceBoundary,
) {
    if evidence.provenance.review_status != ReviewStatus::Accepted
        && boundary != EvidenceBoundary::ReviewPromoted
    {
        return;
    }
    accum.accepted_evidence_ids.push(evidence.id.clone());
    accum.findings.push(NativeEvidenceFinding {
        id: generated_id("finding", &[evidence.id.as_str(), "accepted"]),
        finding_type: NativeEvidenceFindingType::AcceptedEvidencePresent,
        evidence_ids: vec![evidence.id.clone()],
        summary: format!("{} is accepted or review-promoted evidence.", evidence.id),
        review_status: evidence.provenance.review_status,
    });
}

fn record_source_backed_evidence(
    accum: &mut EvidenceAccum,
    evidence: &CaseCell,
    boundary: EvidenceBoundary,
) {
    if !matches!(
        boundary,
        EvidenceBoundary::SourceBacked | EvidenceBoundary::ReviewPromoted
    ) {
        return;
    }
    accum.source_backed_evidence_ids.push(evidence.id.clone());
    if evidence.source_ids.is_empty() {
        accum
            .boundary_violations
            .push(NativeEvidenceBoundaryViolation {
                id: generated_id("violation", &[evidence.id.as_str(), "missing-source"]),
                evidence_id: evidence.id.clone(),
                violation_type: NativeEvidenceBoundaryViolationType::MissingSource,
                explanation: "Source-backed evidence must retain at least one source id."
                    .to_owned(),
                severity: Severity::High,
            });
    } else if evidence.provenance.review_status != ReviewStatus::Accepted {
        accum.findings.push(NativeEvidenceFinding {
            id: generated_id(
                "finding",
                &[evidence.id.as_str(), "source-backed-pending-review"],
            ),
            finding_type: NativeEvidenceFindingType::SourceBackedPendingReview,
            evidence_ids: vec![evidence.id.clone()],
            summary: format!("{} is source-backed but not accepted.", evidence.id),
            review_status: evidence.provenance.review_status,
        });
    }
}

fn record_inference_evidence(
    accum: &mut EvidenceAccum,
    evidence: &CaseCell,
    boundary: EvidenceBoundary,
) {
    if boundary != EvidenceBoundary::Inferred {
        return;
    }
    accum.inference_record_ids.push(evidence.id.clone());
    if evidence.provenance.review_status == ReviewStatus::Unreviewed {
        accum.unreviewed_inference_ids.push(evidence.id.clone());
    }
    accum.findings.push(NativeEvidenceFinding {
        id: generated_id("finding", &[evidence.id.as_str(), "inference-separated"]),
        finding_type: NativeEvidenceFindingType::InferenceSeparated,
        evidence_ids: vec![evidence.id.clone()],
        summary: format!("{} is inference and is not accepted evidence.", evidence.id),
        review_status: evidence.provenance.review_status,
    });
}

fn record_review_promotion(
    accum: &mut EvidenceAccum,
    evidence: &CaseCell,
    boundary: EvidenceBoundary,
) {
    if boundary != EvidenceBoundary::ReviewPromoted {
        return;
    }
    accum.promoted_evidence_ids.push(evidence.id.clone());
    if evidence.provenance.review_status != ReviewStatus::Accepted {
        accum.findings.push(NativeEvidenceFinding {
            id: generated_id("finding", &[evidence.id.as_str(), "promotion-required"]),
            finding_type: NativeEvidenceFindingType::PromotionRequired,
            evidence_ids: vec![evidence.id.clone()],
            summary: format!("{} requires accepted review before promotion.", evidence.id),
            review_status: evidence.provenance.review_status,
        });
    }
}

fn record_rejected_evidence(
    accum: &mut EvidenceAccum,
    evidence: &CaseCell,
    boundary: EvidenceBoundary,
) {
    if evidence.provenance.review_status != ReviewStatus::Rejected
        && boundary != EvidenceBoundary::Rejected
    {
        return;
    }
    accum
        .boundary_violations
        .push(NativeEvidenceBoundaryViolation {
            id: generated_id("violation", &[evidence.id.as_str(), "rejected-used"]),
            evidence_id: evidence.id.clone(),
            violation_type: NativeEvidenceBoundaryViolationType::RejectedEvidenceUsed,
            explanation: "Rejected evidence is present and must not satisfy readiness.".to_owned(),
            severity: Severity::High,
        });
}

pub(super) fn review_gaps(
    case_space: &CaseSpace,
    evidence_findings: &NativeEvidenceFindings,
    completion_candidates: &[NativeCompletionCandidate],
) -> Vec<NativeReviewGap> {
    let mut gaps = BTreeMap::new();
    for candidate in completion_candidates
        .iter()
        .filter(|candidate| candidate.review_status == ReviewStatus::Unreviewed)
    {
        gaps.insert(
            generated_id("review_gap", &[candidate.id.as_str(), "completion"]),
            NativeReviewGap {
                id: generated_id("review_gap", &[candidate.id.as_str(), "completion"]),
                target_id: candidate.id.clone(),
                gap_type: NativeReviewGapType::UnreviewedCompletion,
                explanation: "Completion candidates remain reviewable findings until explicitly accepted or rejected.".to_owned(),
            },
        );
    }
    for evidence_id in &evidence_findings.unreviewed_inference_ids {
        gaps.insert(
            generated_id("review_gap", &[evidence_id.as_str(), "inference"]),
            NativeReviewGap {
                id: generated_id("review_gap", &[evidence_id.as_str(), "inference"]),
                target_id: evidence_id.clone(),
                gap_type: NativeReviewGapType::UnreviewedInference,
                explanation:
                    "AI inference is separated from accepted evidence until review promotion."
                        .to_owned(),
            },
        );
    }
    for entry in &case_space.morphism_log {
        if entry.morphism.review_status == ReviewStatus::Unreviewed {
            gaps.insert(
                generated_id("review_gap", &[entry.morphism_id.as_str(), "morphism"]),
                NativeReviewGap {
                    id: generated_id("review_gap", &[entry.morphism_id.as_str(), "morphism"]),
                    target_id: entry.morphism_id.clone(),
                    gap_type: NativeReviewGapType::UnreviewedMorphism,
                    explanation:
                        "Generated morphisms do not count as accepted evolution until reviewed."
                            .to_owned(),
                },
            );
        }
    }
    for projection in &case_space.projections {
        if !projection.information_loss.is_empty() {
            gaps.insert(
                generated_id(
                    "review_gap",
                    &[projection.projection_id.as_str(), "projection-loss"],
                ),
                NativeReviewGap {
                    id: generated_id(
                        "review_gap",
                        &[projection.projection_id.as_str(), "projection-loss"],
                    ),
                    target_id: projection.projection_id.clone(),
                    gap_type: NativeReviewGapType::UnreviewedProjectionLoss,
                    explanation: "Projection loss must stay visible to reviewers and close checks."
                        .to_owned(),
                },
            );
        }
    }
    gaps.into_values().collect()
}

pub(super) fn projection_loss(case_space: &CaseSpace) -> Vec<NativeProjectionLoss> {
    case_space
        .projections
        .iter()
        .filter(|projection| {
            !projection.omitted_cell_ids.is_empty()
                || !projection.omitted_relation_ids.is_empty()
                || !projection.information_loss.is_empty()
                || !projection.warnings.is_empty()
        })
        .map(|projection| NativeProjectionLoss {
            projection_id: projection.projection_id.clone(),
            audience: projection.audience,
            omitted_cell_ids: projection.omitted_cell_ids.clone(),
            omitted_relation_ids: projection.omitted_relation_ids.clone(),
            information_loss_descriptions: projection
                .information_loss
                .iter()
                .map(|loss| loss.description.clone())
                .collect(),
            warning_ids: projection
                .warnings
                .iter()
                .map(|warning| {
                    serde_json::to_value(warning)
                        .unwrap_or(Value::Null)
                        .as_str()
                        .unwrap_or("unknown")
                        .to_owned()
                })
                .collect(),
        })
        .collect()
}

pub(super) fn correspondence_summaries(case_space: &CaseSpace) -> Vec<NativeCorrespondenceSummary> {
    case_space
        .case_relations
        .iter()
        .filter(|relation| relation.relation_type == CaseRelationType::CorrespondsTo)
        .map(|relation| NativeCorrespondenceSummary {
            id: generated_id("correspondence", &[relation.id.as_str()]),
            left_ids: vec![relation.from_id.clone()],
            right_ids: vec![relation.to_id.clone()],
            relation_ids: vec![relation.id.clone()],
            confidence: relation.provenance.confidence,
            review_status: relation.provenance.review_status,
        })
        .collect()
}

pub(super) fn evolution_summary(case_space: &CaseSpace) -> NativeEvolutionSummary {
    let latest = case_space.morphism_log.last();
    NativeEvolutionSummary {
        revision_id: case_space.revision.revision_id.clone(),
        previous_revision_id: latest.and_then(|entry| entry.source_revision_id.clone()),
        morphism_ids: case_space
            .morphism_log
            .iter()
            .map(|entry| entry.morphism_id.clone())
            .collect(),
        added_ids: dedupe_ids(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| entry.morphism.added_ids.iter().cloned())
                .collect(),
        ),
        updated_ids: dedupe_ids(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| entry.morphism.updated_ids.iter().cloned())
                .collect(),
        ),
        retired_ids: dedupe_ids(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| entry.morphism.retired_ids.iter().cloned())
                .collect(),
        ),
        preserved_ids: dedupe_ids(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| entry.morphism.preserved_ids.iter().cloned())
                .collect(),
        ),
        invariant_breaks: case_space
            .morphism_log
            .iter()
            .flat_map(|entry| {
                entry
                    .morphism
                    .violated_invariant_ids
                    .iter()
                    .map(|invariant_id| NativeInvariantBreak {
                        morphism_id: entry.morphism_id.clone(),
                        invariant_id: invariant_id.clone(),
                        witness_ids: dedupe_ids(
                            entry
                                .morphism
                                .added_ids
                                .iter()
                                .chain(&entry.morphism.updated_ids)
                                .chain(&entry.morphism.retired_ids)
                                .chain(&entry.source_ids)
                                .cloned()
                                .collect(),
                        ),
                    })
            })
            .collect(),
    }
}

pub(super) fn close_check_skeleton(
    case_space: &CaseSpace,
    obstructions: &[NativeObstruction],
    completion_candidates: &[NativeCompletionCandidate],
    review_gaps: &[NativeReviewGap],
) -> NativeCloseCheckSkeleton {
    let blocking_obstruction_ids = obstructions
        .iter()
        .filter(|obstruction| obstruction.blocking)
        .map(|obstruction| obstruction.id.clone())
        .collect::<Vec<_>>();
    let unreviewed_completion_ids = completion_candidates
        .iter()
        .filter(|candidate| candidate.review_status == ReviewStatus::Unreviewed)
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let unreviewed_morphism_ids = case_space
        .morphism_log
        .iter()
        .filter(|entry| entry.morphism.review_status == ReviewStatus::Unreviewed)
        .map(|entry| entry.morphism_id.clone())
        .collect::<Vec<_>>();
    let projection_loss_ids = case_space
        .projections
        .iter()
        .filter(|projection| {
            !projection.information_loss.is_empty() || !projection.warnings.is_empty()
        })
        .map(|projection| projection.projection_id.clone())
        .collect::<Vec<_>>();
    let review_gap_ids = review_gaps
        .iter()
        .map(|gap| gap.id.clone())
        .collect::<Vec<_>>();
    let invariant_results = vec![
        close_invariant(
            "close:native-no-hard-obstructions",
            blocking_obstruction_ids,
            "No hard obstructions remain.",
        ),
        close_invariant(
            "close:native-completions-reviewed",
            unreviewed_completion_ids,
            "Completion candidates must be reviewed before close.",
        ),
        close_invariant(
            "close:native-morphisms-reviewed",
            unreviewed_morphism_ids,
            "Morphism log entries must be reviewed before close.",
        ),
        close_invariant(
            "close:native-projection-loss-disclosed",
            projection_loss_ids,
            "Projection loss must be disclosed before close.",
        ),
        close_invariant(
            "close:native-review-gaps-closed",
            review_gap_ids,
            "Review gaps remain open.",
        ),
    ];
    let closable = invariant_results.iter().all(|result| result.passed);
    NativeCloseCheckSkeleton {
        check_id: generated_id(
            "close_check",
            &[
                case_space.case_space_id.as_str(),
                case_space.revision.revision_id.as_str(),
            ],
        ),
        case_space_id: case_space.case_space_id.clone(),
        revision_id: case_space.revision.revision_id.clone(),
        close_policy_id: case_space.close_policy_id.clone(),
        closable,
        invariant_results,
    }
}

pub(super) fn acceptable_evidence(cell: &CaseCell) -> bool {
    cell.cell_type == CaseCellType::Evidence
        && cell.provenance.review_status != ReviewStatus::Rejected
        && !cell.source_ids.is_empty()
        && match evidence_boundary(cell) {
            EvidenceBoundary::SourceBacked => true,
            EvidenceBoundary::ReviewPromoted => {
                cell.provenance.review_status == ReviewStatus::Accepted
            }
            EvidenceBoundary::Inferred
            | EvidenceBoundary::Rejected
            | EvidenceBoundary::Contradicting => false,
        }
}

fn evidence_boundary(cell: &CaseCell) -> EvidenceBoundary {
    let Some(value) = cell
        .metadata
        .get("evidence_boundary")
        .and_then(Value::as_str)
    else {
        return if cell.provenance.source.kind == SourceKind::Ai {
            EvidenceBoundary::Inferred
        } else {
            EvidenceBoundary::SourceBacked
        };
    };
    match value {
        "source_backed" | "source_backed_evidence" => EvidenceBoundary::SourceBacked,
        "inferred" | "ai_inference" => EvidenceBoundary::Inferred,
        "review_promoted" | "review_promotion" => EvidenceBoundary::ReviewPromoted,
        "rejected" => EvidenceBoundary::Rejected,
        "contradicting" => EvidenceBoundary::Contradicting,
        _ => EvidenceBoundary::Inferred,
    }
}

fn candidate_shape(
    obstruction_type: NativeObstructionType,
) -> Option<(NativeCompletionCandidateType, Value, &'static str)> {
    match obstruction_type {
        NativeObstructionType::MissingEvidence => Some((
            NativeCompletionCandidateType::MissingEvidence,
            json!({"cell_type": "evidence", "evidence_boundary": "source_backed"}),
            "A required evidence cell is absent or only represented by inference.",
        )),
        NativeObstructionType::MissingProof => Some((
            NativeCompletionCandidateType::MissingProof,
            json!({"cell_type": "proof", "lifecycle": "active"}),
            "A required proof cell or accepted proof evidence is missing.",
        )),
        NativeObstructionType::ReviewRequired | NativeObstructionType::ExternalWait => Some((
            NativeCompletionCandidateType::MissingReview,
            json!({"cell_type": "review", "lifecycle": "accepted"}),
            "A wait, generated candidate, or review requirement needs explicit review.",
        )),
        NativeObstructionType::UnresolvedDependency => Some((
            NativeCompletionCandidateType::MissingDependencyResolution,
            json!({"relation_type": "depends_on", "resolution": "accept_or_resolve_target"}),
            "A hard dependency must be resolved before downstream readiness.",
        )),
        NativeObstructionType::Contradiction => Some((
            NativeCompletionCandidateType::ContradictionResolution,
            json!({"cell_type": "decision", "purpose": "contradiction_resolution"}),
            "A hard contradiction needs a decision or review before readiness.",
        )),
        _ => None,
    }
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
