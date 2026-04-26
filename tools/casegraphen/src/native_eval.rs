use crate::native_model::{
    CaseCell, CaseCellLifecycle, CaseCellType, CaseRelation, CaseRelationType, CaseSpace,
    RelationStrength,
};
use higher_graphen_core::{Id, ReviewStatus, Severity};
use std::collections::{BTreeMap, BTreeSet};

mod sections;
#[cfg(test)]
mod tests;
mod types;
mod util;
mod validation;
pub use types::*;
pub use validation::validate_native_case_space;

use sections::{
    acceptable_evidence, close_check_skeleton, completion_candidates, correspondence_summaries,
    evidence_findings, evolution_summary, projection_loss, review_gaps,
};
use util::*;

pub fn evaluate_native_case(case_space: &CaseSpace) -> NativeEvalResult<NativeCaseEvaluation> {
    validate_native_case_space(case_space)?;

    let context = NativeEvaluationContext::new(case_space);
    let cell_results = context.evaluate_cells();
    let readiness = readiness_result(case_space, &cell_results);
    let obstructions = merge_obstructions(&cell_results);
    let evidence_findings = evidence_findings(case_space, &obstructions);
    let completion_candidates = completion_candidates(case_space, &obstructions);
    let review_gaps = review_gaps(case_space, &evidence_findings, &completion_candidates);
    let projection_loss = projection_loss(case_space);
    let correspondence = correspondence_summaries(case_space);
    let evolution = evolution_summary(case_space);
    let close_check = close_check_skeleton(
        case_space,
        &obstructions,
        &completion_candidates,
        &review_gaps,
    );
    let frontier_cell_ids = frontier_cell_ids(case_space, &readiness);
    let status = reasoning_status(case_space, &readiness, &obstructions, &review_gaps);

    Ok(NativeCaseEvaluation {
        status,
        readiness,
        frontier_cell_ids,
        obstructions,
        completion_candidates,
        evidence_findings,
        review_gaps,
        projection_loss,
        correspondence,
        evolution,
        close_check,
    })
}

struct NativeEvaluationContext<'a> {
    case_space: &'a CaseSpace,
    cells: BTreeMap<&'a str, &'a CaseCell>,
    hard_relations: Vec<&'a CaseRelation>,
}

struct CellEvaluation {
    cell_id: Id,
    lifecycle: CaseCellLifecycle,
    hard_dependency_ids: Vec<Id>,
    wait_ids: Vec<Id>,
    evidence_requirement_ids: Vec<Id>,
    proof_requirement_ids: Vec<Id>,
    obstructions: Vec<NativeObstruction>,
    rule_results: Vec<NativeReadinessRuleResult>,
}

impl<'a> NativeEvaluationContext<'a> {
    fn new(case_space: &'a CaseSpace) -> Self {
        let cells = case_space
            .case_cells
            .iter()
            .map(|cell| (cell.id.as_str(), cell))
            .collect();
        let hard_relations = case_space
            .case_relations
            .iter()
            .filter(|relation| relation.relation_strength == RelationStrength::Hard)
            .collect();
        Self {
            case_space,
            cells,
            hard_relations,
        }
    }

    fn evaluate_cells(&self) -> Vec<CellEvaluation> {
        self.case_space
            .case_cells
            .iter()
            .filter(|cell| readiness_subject(cell))
            .map(|cell| self.evaluate_cell(cell))
            .collect()
    }

    fn evaluate_cell(&self, cell: &CaseCell) -> CellEvaluation {
        let hard_dependency_ids = self.requirement_ids(cell, CaseRelationType::DependsOn);
        let wait_ids = self.requirement_ids(cell, CaseRelationType::WaitsFor);
        let evidence_requirement_ids =
            self.requirement_ids(cell, CaseRelationType::RequiresEvidence);
        let proof_requirement_ids = self.requirement_ids(cell, CaseRelationType::RequiresProof);
        let mut by_check = BTreeMap::<ReadinessCheck, Vec<NativeObstruction>>::new();

        if let Some(obstruction) = self.lifecycle_obstruction(cell) {
            by_check
                .entry(ReadinessCheck::Lifecycle)
                .or_default()
                .push(obstruction);
        }
        self.add_dependency_obstructions(cell, &hard_dependency_ids, &mut by_check);
        self.add_wait_obstructions(cell, &wait_ids, &mut by_check);
        self.add_evidence_obstructions(cell, &evidence_requirement_ids, &mut by_check);
        self.add_proof_obstructions(cell, &proof_requirement_ids, &mut by_check);
        self.add_contradiction_obstructions(cell, &mut by_check);
        self.add_review_obstructions(cell, &mut by_check);

        let obstructions = sorted_obstructions(&by_check);
        let rule_results = self.rule_results_for(cell, &by_check);
        CellEvaluation {
            cell_id: cell.id.clone(),
            lifecycle: cell.lifecycle,
            hard_dependency_ids,
            wait_ids,
            evidence_requirement_ids,
            proof_requirement_ids,
            obstructions,
            rule_results,
        }
    }

    fn add_dependency_obstructions(
        &self,
        cell: &CaseCell,
        dependency_ids: &[Id],
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for dependency_id in dependency_ids {
            if self.complete_cell(dependency_id) {
                continue;
            }
            push_obstruction(
                by_check,
                ReadinessCheck::Dependencies,
                obstruction(
                    NativeObstructionType::UnresolvedDependency,
                    &cell.id,
                    dependency_id,
                    "constraint:native-dependency-closure",
                    format!(
                        "{cell_id} depends on unresolved cell {dependency_id}.",
                        cell_id = cell.id
                    ),
                    Severity::High,
                    "Resolve, accept, or retire the hard dependency before treating this cell as ready.",
                ),
            );
        }
    }

    fn add_wait_obstructions(
        &self,
        cell: &CaseCell,
        wait_ids: &[Id],
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for wait_id in wait_ids {
            if self.wait_satisfied(wait_id) {
                continue;
            }
            push_obstruction(
                by_check,
                ReadinessCheck::Waits,
                obstruction(
                    NativeObstructionType::ExternalWait,
                    &cell.id,
                    wait_id,
                    "constraint:native-wait-resolution",
                    format!("{} waits for unresolved cell {}.", cell.id, wait_id),
                    Severity::Medium,
                    "Record the waited-for event/review/evidence or explicitly waive the wait by accepted review.",
                ),
            );
        }
    }

    fn add_evidence_obstructions(
        &self,
        cell: &CaseCell,
        requirement_ids: &[Id],
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for requirement_id in requirement_ids {
            if self.evidence_requirement_satisfied(&cell.id, requirement_id) {
                continue;
            }
            push_obstruction(
                by_check,
                ReadinessCheck::Evidence,
                obstruction(
                    NativeObstructionType::MissingEvidence,
                    &cell.id,
                    requirement_id,
                    "constraint:native-evidence-availability",
                    format!("{} requires source-backed or accepted evidence {}, but none is available.", cell.id, requirement_id),
                    Severity::Medium,
                    "Attach source-backed evidence or promote inferred evidence through accepted review.",
                ),
            );
        }
    }

    fn add_proof_obstructions(
        &self,
        cell: &CaseCell,
        proof_ids: &[Id],
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for proof_id in proof_ids {
            if self.proof_requirement_satisfied(&cell.id, proof_id) {
                continue;
            }
            push_obstruction(
                by_check,
                ReadinessCheck::Proof,
                obstruction(
                    NativeObstructionType::MissingProof,
                    &cell.id,
                    proof_id,
                    "constraint:native-proof-availability",
                    format!(
                        "{} requires accepted proof {}, but no accepted proof is available.",
                        cell.id, proof_id
                    ),
                    Severity::Medium,
                    "Complete or accept the proof cell, or attach accepted proof evidence.",
                ),
            );
        }
    }

    fn add_contradiction_obstructions(
        &self,
        cell: &CaseCell,
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for relation in self.contradiction_relations(&cell.id) {
            push_obstruction(
                by_check,
                ReadinessCheck::Contradictions,
                obstruction(
                    NativeObstructionType::Contradiction,
                    &cell.id,
                    &relation.id,
                    "constraint:native-no-hard-contradiction",
                    format!(
                        "{} participates in hard contradictory relation {}.",
                        cell.id, relation.id
                    ),
                    Severity::High,
                    "Resolve or review the contradiction before treating this cell as ready.",
                ),
            );
        }
    }

    fn add_review_obstructions(
        &self,
        cell: &CaseCell,
        by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) {
        for relation in self.required_review_relations(&cell.id) {
            if self.review_satisfied(&relation.to_id) {
                continue;
            }
            push_obstruction(
                by_check,
                ReadinessCheck::Reviews,
                obstruction(
                    NativeObstructionType::ReviewRequired,
                    &cell.id,
                    &relation.to_id,
                    "constraint:native-review-accepted",
                    format!(
                        "{} requires accepted review {}, but it is not accepted.",
                        cell.id, relation.to_id
                    ),
                    Severity::Medium,
                    "Record an accepted review before treating this cell as ready.",
                ),
            );
        }
    }

    fn requirement_ids(&self, cell: &CaseCell, relation_type: CaseRelationType) -> Vec<Id> {
        dedupe_ids(
            self.hard_relations
                .iter()
                .filter(|relation| {
                    relation.from_id == cell.id && relation.relation_type == relation_type
                })
                .map(|relation| relation.to_id.clone())
                .collect(),
        )
    }

    fn lifecycle_obstruction(&self, cell: &CaseCell) -> Option<NativeObstruction> {
        let (severity, explanation, resolution) = match cell.lifecycle {
            CaseCellLifecycle::Rejected => (
                Severity::High,
                format!("{} is rejected and cannot be ready.", cell.id),
                "Create or accept a replacement cell.",
            ),
            CaseCellLifecycle::Retired | CaseCellLifecycle::Superseded => (
                Severity::Medium,
                format!(
                    "{} is retired or superseded and cannot be frontier work.",
                    cell.id
                ),
                "Use the active replacement cell if one exists.",
            ),
            _ => return None,
        };
        Some(obstruction(
            NativeObstructionType::ReviewRequired,
            &cell.id,
            &cell.id,
            "constraint:native-cell-lifecycle",
            explanation,
            severity,
            resolution,
        ))
    }

    fn complete_cell(&self, cell_id: &Id) -> bool {
        self.cells.get(cell_id.as_str()).is_some_and(|cell| {
            matches!(
                cell.lifecycle,
                CaseCellLifecycle::Resolved
                    | CaseCellLifecycle::Accepted
                    | CaseCellLifecycle::Retired
                    | CaseCellLifecycle::Superseded
            ) || cell.provenance.review_status == ReviewStatus::Accepted
        })
    }

    fn wait_satisfied(&self, wait_id: &Id) -> bool {
        self.complete_cell(wait_id)
            || self
                .acceptable_evidence_for(wait_id, wait_id)
                .next()
                .is_some()
    }

    fn evidence_requirement_satisfied(&self, cell_id: &Id, requirement_id: &Id) -> bool {
        self.acceptable_evidence_for(requirement_id, cell_id)
            .next()
            .is_some()
    }

    fn proof_requirement_satisfied(&self, cell_id: &Id, proof_id: &Id) -> bool {
        self.cells.get(proof_id.as_str()).is_some_and(|cell| {
            cell.cell_type == CaseCellType::Proof && self.complete_cell(proof_id)
        }) || self
            .acceptable_evidence_for(proof_id, cell_id)
            .next()
            .is_some()
    }

    fn acceptable_evidence_for(
        &'a self,
        requirement_id: &'a Id,
        cell_id: &'a Id,
    ) -> impl Iterator<Item = &'a CaseCell> + 'a {
        self.case_space.case_cells.iter().filter(move |cell| {
            cell.cell_type == CaseCellType::Evidence
                && acceptable_evidence(cell)
                && (cell.id == *requirement_id
                    || cell.structure_ids.contains(requirement_id)
                    || cell.structure_ids.contains(cell_id)
                    || self.case_space.case_relations.iter().any(|relation| {
                        matches!(
                            relation.relation_type,
                            CaseRelationType::SatisfiesEvidenceRequirement
                                | CaseRelationType::Verifies
                                | CaseRelationType::Accepts
                        ) && relation.from_id == cell.id
                            && (relation.to_id == *requirement_id || relation.to_id == *cell_id)
                    })
                    || self.case_space.case_relations.iter().any(|relation| {
                        relation.evidence_ids.contains(&cell.id)
                            && relation.from_id == *cell_id
                            && (relation.to_id == *requirement_id
                                || matches!(
                                    relation.relation_type,
                                    CaseRelationType::RequiresEvidence
                                        | CaseRelationType::RequiresProof
                                ))
                    }))
        })
    }

    fn contradiction_relations(&self, cell_id: &Id) -> Vec<&'a CaseRelation> {
        self.hard_relations
            .iter()
            .copied()
            .filter(|relation| {
                matches!(
                    relation.relation_type,
                    CaseRelationType::Contradicts
                        | CaseRelationType::Invalidates
                        | CaseRelationType::Blocks
                ) && (relation.from_id == *cell_id || relation.to_id == *cell_id)
                    && !self.unblocked_by_review(relation)
            })
            .collect()
    }

    fn unblocked_by_review(&self, blocked_relation: &CaseRelation) -> bool {
        self.hard_relations.iter().any(|relation| {
            relation.relation_type == CaseRelationType::Unblocks
                && relation.to_id == blocked_relation.id
                && self.review_satisfied(&relation.from_id)
        })
    }

    fn required_review_relations(&self, cell_id: &Id) -> Vec<&'a CaseRelation> {
        self.hard_relations
            .iter()
            .copied()
            .filter(|relation| {
                relation.from_id == *cell_id
                    && matches!(
                        relation.relation_type,
                        CaseRelationType::Accepts | CaseRelationType::Rejects
                    )
            })
            .collect()
    }

    fn review_satisfied(&self, review_id: &Id) -> bool {
        self.cells.get(review_id.as_str()).is_some_and(|cell| {
            cell.cell_type == CaseCellType::Review
                && (cell.lifecycle == CaseCellLifecycle::Accepted
                    || cell.provenance.review_status == ReviewStatus::Accepted)
        })
    }

    fn rule_results_for(
        &self,
        cell: &CaseCell,
        by_check: &BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    ) -> Vec<NativeReadinessRuleResult> {
        let mut checks = by_check.keys().copied().collect::<BTreeSet<_>>();
        if checks.is_empty() {
            checks.insert(ReadinessCheck::Lifecycle);
        }
        checks
            .into_iter()
            .map(|check| {
                let rule_id = default_rule_id(check);
                let obstruction_ids: Vec<Id> = by_check
                    .get(&check)
                    .map(|records| records.iter().map(|record| record.id.clone()).collect())
                    .unwrap_or_default();
                NativeReadinessRuleResult {
                    id: generated_id("readiness_result", &[cell.id.as_str(), rule_id.as_str()]),
                    rule_id,
                    target_cell_id: cell.id.clone(),
                    ready: obstruction_ids.is_empty(),
                    obstruction_ids,
                }
            })
            .collect()
    }
}

fn reasoning_status(
    case_space: &CaseSpace,
    readiness: &NativeReadiness,
    obstructions: &[NativeObstruction],
    review_gaps: &[NativeReviewGap],
) -> NativeReasoningStatus {
    if case_space.case_cells.is_empty() {
        NativeReasoningStatus::Incomplete
    } else if obstructions.iter().any(|obstruction| obstruction.blocking) {
        NativeReasoningStatus::Blocked
    } else if !review_gaps.is_empty() {
        NativeReasoningStatus::ReviewRequired
    } else if readiness.ready_cell_ids.is_empty() {
        NativeReasoningStatus::Incomplete
    } else {
        NativeReasoningStatus::Ready
    }
}

fn readiness_result(case_space: &CaseSpace, results: &[CellEvaluation]) -> NativeReadiness {
    let evaluated_cell_ids = case_space
        .case_cells
        .iter()
        .filter(|cell| readiness_subject(cell))
        .map(|cell| cell.id.clone())
        .collect();
    let ready_cell_ids = results
        .iter()
        .filter(|result| {
            result
                .obstructions
                .iter()
                .all(|obstruction| !obstruction.blocking)
        })
        .map(|result| result.cell_id.clone())
        .collect();
    let not_ready_cells = results
        .iter()
        .filter(|result| {
            result
                .obstructions
                .iter()
                .any(|obstruction| obstruction.blocking)
        })
        .map(|result| NativeNotReadyCell {
            cell_id: result.cell_id.clone(),
            lifecycle: result.lifecycle,
            hard_dependency_ids: result.hard_dependency_ids.clone(),
            wait_ids: result.wait_ids.clone(),
            evidence_requirement_ids: result.evidence_requirement_ids.clone(),
            proof_requirement_ids: result.proof_requirement_ids.clone(),
            obstruction_ids: result
                .obstructions
                .iter()
                .filter(|obstruction| obstruction.blocking)
                .map(|obstruction| obstruction.id.clone())
                .collect(),
        })
        .collect::<Vec<_>>();
    let blocked_cell_ids = not_ready_cells
        .iter()
        .map(|cell| cell.cell_id.clone())
        .collect::<Vec<_>>();
    let waiting_cell_ids = not_ready_cells
        .iter()
        .filter(|cell| {
            !cell.wait_ids.is_empty()
                && cell
                    .obstruction_ids
                    .iter()
                    .all(|id| id.as_str().contains("external-wait"))
        })
        .map(|cell| cell.cell_id.clone())
        .collect();
    let rule_results = results
        .iter()
        .flat_map(|result| result.rule_results.iter().cloned())
        .collect();

    NativeReadiness {
        evaluated_cell_ids,
        ready_cell_ids,
        not_ready_cells,
        waiting_cell_ids,
        blocked_cell_ids,
        rule_results,
    }
}

fn frontier_cell_ids(case_space: &CaseSpace, readiness: &NativeReadiness) -> Vec<Id> {
    let completed_targets = case_space
        .case_relations
        .iter()
        .filter(|relation| {
            relation.relation_strength == RelationStrength::Hard
                && matches!(
                    relation.relation_type,
                    CaseRelationType::Completes | CaseRelationType::Supersedes
                )
        })
        .map(|relation| relation.to_id.clone())
        .collect::<BTreeSet<_>>();
    readiness
        .ready_cell_ids
        .iter()
        .filter(|id| !completed_targets.contains(*id))
        .filter(|id| {
            case_space.case_cells.iter().any(|cell| {
                cell.id == **id
                    && !matches!(
                        cell.lifecycle,
                        CaseCellLifecycle::Resolved
                            | CaseCellLifecycle::Accepted
                            | CaseCellLifecycle::Retired
                            | CaseCellLifecycle::Rejected
                            | CaseCellLifecycle::Superseded
                    )
            })
        })
        .cloned()
        .collect()
}

fn merge_obstructions(results: &[CellEvaluation]) -> Vec<NativeObstruction> {
    let mut by_id = BTreeMap::new();
    for obstruction in results
        .iter()
        .flat_map(|result| result.obstructions.iter().cloned())
    {
        by_id.entry(obstruction.id.clone()).or_insert(obstruction);
    }
    by_id.into_values().collect()
}

fn readiness_subject(cell: &CaseCell) -> bool {
    !matches!(
        cell.cell_type,
        CaseCellType::Evidence
            | CaseCellType::Review
            | CaseCellType::Projection
            | CaseCellType::Revision
            | CaseCellType::Morphism
            | CaseCellType::ExternalRef
    ) && !matches!(
        cell.lifecycle,
        CaseCellLifecycle::Resolved
            | CaseCellLifecycle::Accepted
            | CaseCellLifecycle::Retired
            | CaseCellLifecycle::Rejected
            | CaseCellLifecycle::Superseded
    )
}

fn obstruction(
    obstruction_type: NativeObstructionType,
    cell_id: &Id,
    witness_id: &Id,
    constraint_id: &str,
    explanation: String,
    severity: Severity,
    required_resolution: &str,
) -> NativeObstruction {
    NativeObstruction {
        id: generated_id(
            "obstruction",
            &[
                obstruction_type_stem(obstruction_type),
                cell_id.as_str(),
                witness_id.as_str(),
            ],
        ),
        obstruction_type,
        affected_ids: vec![cell_id.clone()],
        source_constraint_id: id(constraint_id),
        witness_ids: vec![witness_id.clone()],
        explanation,
        severity,
        required_resolution: required_resolution.to_owned(),
        blocking: true,
        provenance: generated_provenance("Native readiness obstruction", 0.84),
    }
}

fn push_obstruction(
    by_check: &mut BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
    check: ReadinessCheck,
    obstruction: NativeObstruction,
) {
    by_check.entry(check).or_default().push(obstruction);
}

fn sorted_obstructions(
    by_check: &BTreeMap<ReadinessCheck, Vec<NativeObstruction>>,
) -> Vec<NativeObstruction> {
    let mut obstructions = by_check
        .values()
        .flat_map(|records| records.iter().cloned())
        .collect::<Vec<_>>();
    obstructions.sort_by(|left, right| left.id.cmp(&right.id));
    obstructions
}

fn default_rule_id(check: ReadinessCheck) -> Id {
    id(match check {
        ReadinessCheck::Lifecycle => "readiness:native-lifecycle-allows-work",
        ReadinessCheck::Dependencies => "readiness:native-dependencies-resolved",
        ReadinessCheck::Waits => "readiness:native-waits-satisfied",
        ReadinessCheck::Evidence => "readiness:native-evidence-available",
        ReadinessCheck::Proof => "readiness:native-proof-available",
        ReadinessCheck::Contradictions => "readiness:native-no-contradictions",
        ReadinessCheck::Reviews => "readiness:native-reviews-accepted",
    })
}
