//! Deterministic gluing checks for correspondence cells.

use higher_graphen_core::{
    CoreError, CorrespondenceCell, DifferenceKind, DifferenceSeverity, DifferenceWitness,
    DifferingStructure, GluingAttempt, GluingResult, Id, InvariantCheckResult, PreservationReport,
    Result, ReviewRequirement, ReviewStatus, SharedStructure,
};
use std::collections::BTreeSet;

/// Checks whether a correspondence can be glued without silent loss.
///
/// This Phase 4 checker is intentionally deterministic. It does not construct
/// a real pushout; it classifies gluing from explicit witnesses, differences,
/// invariant checks, evidence, and review state.
pub fn attempt_gluing(correspondence: &CorrespondenceCell) -> Result<GluingAttempt> {
    if correspondence.participants.len() < 2 {
        return Err(CoreError::MalformedField {
            field: "participants".to_owned(),
            reason: "gluing requires at least two participants".to_owned(),
        });
    }

    let invariant_checks = invariant_checks(correspondence);
    let preservation_report = preservation_report(correspondence, &invariant_checks);
    let evidence = evidence_union(correspondence);
    let base_id = safe_id_segment(&correspondence.id);
    let result = gluing_result(
        correspondence,
        &invariant_checks,
        &preservation_report,
        &evidence,
        &base_id,
    )?;

    let attempt = GluingAttempt {
        id: Id::new(format!("glue:check:{base_id}"))?,
        participants: correspondence
            .participants
            .iter()
            .map(|participant| participant.participant.clone())
            .collect(),
        overlap_witnesses: correspondence
            .overlap_witnesses
            .iter()
            .map(|witness| witness.id.clone())
            .collect(),
        difference_witnesses: correspondence
            .difference_witnesses
            .iter()
            .map(|witness| witness.id.clone())
            .collect(),
        context: correspondence.context.clone(),
        invariant_checks,
        preservation_report,
        result,
        evidence,
        confidence: correspondence.confidence,
        status: ReviewStatus::Candidate,
        override_review: None,
    };
    attempt.validate_with_differences(&correspondence.difference_witnesses)?;

    Ok(attempt)
}

fn gluing_result(
    correspondence: &CorrespondenceCell,
    invariant_checks: &[InvariantCheckResult],
    preservation_report: &PreservationReport,
    evidence: &[Id],
    base_id: &str,
) -> Result<GluingResult> {
    if let Some(reason) = failure_reason(correspondence, invariant_checks) {
        if let Some(obstruction) = existing_failure_obstruction(correspondence) {
            return Ok(GluingResult::Failure {
                obstruction: obstruction.clone(),
            });
        }
        return Ok(GluingResult::Failure {
            obstruction: Id::new(format!("obstruction:gluing:{base_id}:{reason}"))?,
        });
    }

    if let Some(reason) = candidate_review_reason(correspondence, preservation_report, evidence) {
        return Ok(GluingResult::Candidate {
            completion_candidate: Id::new(format!("completion:gluing-review:{base_id}"))?,
            required_review: ReviewRequirement::new(true).with_decision_reason(reason)?,
        });
    }

    Ok(GluingResult::Success {
        merged_complex: Id::new(format!("complex:glued:{base_id}"))?,
        preservation_report: preservation_report.clone(),
    })
}

fn failure_reason(
    correspondence: &CorrespondenceCell,
    invariant_checks: &[InvariantCheckResult],
) -> Option<&'static str> {
    if correspondence
        .difference_witnesses
        .iter()
        .any(is_blocking_difference)
    {
        return Some("blocking-difference");
    }

    if invariant_checks.iter().any(is_failed_invariant_check) {
        return Some("invariant-failed");
    }

    None
}

fn candidate_review_reason(
    correspondence: &CorrespondenceCell,
    preservation_report: &PreservationReport,
    evidence: &[Id],
) -> Option<&'static str> {
    if correspondence.review_status.is_rejected() {
        return Some("correspondence is rejected and cannot be silently glued");
    }

    if correspondence.overlap_witnesses.is_empty() {
        return Some("gluing requires at least one explicit overlap witness");
    }

    if evidence.is_empty() {
        return Some("gluing requires supporting evidence before acceptance");
    }

    if correspondence
        .difference_witnesses
        .iter()
        .any(is_major_difference)
    {
        return Some("major differences require explicit review before gluing");
    }

    if preservation_report.preserved_structures.is_empty()
        && preservation_report.preserved_invariants.is_empty()
    {
        return Some("gluing did not preserve any explicit structures or invariants");
    }

    None
}

fn is_blocking_difference(difference: &DifferenceWitness) -> bool {
    matches!(difference.severity, DifferenceSeverity::Blocking)
        || matches!(
            difference.difference_kind,
            DifferenceKind::Contradiction | DifferenceKind::InvariantMismatch
        ) && matches!(
            difference.differing_structure,
            DifferingStructure::InvariantMismatch(_)
        ) && invariant_mismatch_failed(difference)
}

fn is_major_difference(difference: &DifferenceWitness) -> bool {
    matches!(
        difference.severity,
        DifferenceSeverity::Major | DifferenceSeverity::Blocking
    )
}

fn invariant_mismatch_failed(difference: &DifferenceWitness) -> bool {
    match &difference.differing_structure {
        DifferingStructure::InvariantMismatch(checks) => {
            checks.iter().any(is_failed_invariant_check)
        }
        _ => false,
    }
}

fn invariant_checks(correspondence: &CorrespondenceCell) -> Vec<InvariantCheckResult> {
    let mut checks = correspondence
        .gluing
        .as_ref()
        .map(|gluing| gluing.invariant_checks.clone())
        .unwrap_or_default();

    checks.extend(
        correspondence
            .difference_witnesses
            .iter()
            .flat_map(|difference| match &difference.differing_structure {
                DifferingStructure::InvariantMismatch(checks) => checks.clone(),
                _ => Vec::new(),
            })
            .collect::<Vec<_>>(),
    );

    let existing = checks
        .iter()
        .map(|check| check.invariant.clone())
        .collect::<BTreeSet<_>>();
    for invariant in correspondence.overlap_witnesses.iter().flat_map(|witness| {
        match &witness.shared_structure {
            SharedStructure::ConstraintSet(invariants) => invariants.clone(),
            _ => Vec::new(),
        }
    }) {
        if !existing.contains(&invariant) {
            checks.push(InvariantCheckResult {
                invariant,
                result: "passed".to_owned(),
                detail: Some(
                    "shared constraint preserved by deterministic gluing check".to_owned(),
                ),
            });
        }
    }

    unique_checks(checks)
}

fn existing_failure_obstruction(correspondence: &CorrespondenceCell) -> Option<&Id> {
    match correspondence.gluing.as_ref().map(|gluing| &gluing.result) {
        Some(GluingResult::Failure { obstruction }) => Some(obstruction),
        _ => None,
    }
}

fn preservation_report(
    correspondence: &CorrespondenceCell,
    invariant_checks: &[InvariantCheckResult],
) -> PreservationReport {
    let preserved_invariants = invariant_checks
        .iter()
        .filter(|check| !is_failed_invariant_check(check))
        .map(|check| check.invariant.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let preserved_structures = correspondence
        .participants
        .iter()
        .map(|participant| participant.participant.id().clone())
        .chain(
            correspondence
                .overlap_witnesses
                .iter()
                .map(|witness| witness.id.clone()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let summary = if correspondence.overlap_witnesses.is_empty() {
        None
    } else {
        Some(format!(
            "checked {} participant(s), {} overlap witness(es), and {} difference witness(es)",
            correspondence.participants.len(),
            correspondence.overlap_witnesses.len(),
            correspondence.difference_witnesses.len()
        ))
    };

    PreservationReport {
        preserved_invariants,
        preserved_structures,
        summary,
    }
}

fn evidence_union(correspondence: &CorrespondenceCell) -> Vec<Id> {
    correspondence
        .evidence
        .iter()
        .chain(
            correspondence
                .overlap_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .chain(
            correspondence
                .difference_witnesses
                .iter()
                .flat_map(|witness| witness.evidence.iter()),
        )
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_checks(checks: Vec<InvariantCheckResult>) -> Vec<InvariantCheckResult> {
    let mut seen = BTreeSet::new();
    checks
        .into_iter()
        .filter(|check| {
            seen.insert((
                check.invariant.clone(),
                check.result.clone(),
                check.detail.clone(),
            ))
        })
        .collect()
}

fn is_failed_invariant_check(check: &InvariantCheckResult) -> bool {
    matches!(
        check.result.as_str(),
        "failed" | "violated" | "unsat" | "unsafe"
    )
}

fn safe_id_segment(id: &Id) -> String {
    id.as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_core::{
        Confidence, CorrespondenceKind, CorrespondenceParticipant, CorrespondencePolarity, Feature,
        OverlapWitness, OverlapWitnessKind, ParticipantMapping, ParticipantRef, Scope,
    };
    use std::collections::BTreeMap;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn confidence(value: f64) -> Confidence {
        Confidence::new(value).expect("valid confidence")
    }

    fn base_correspondence() -> CorrespondenceCell {
        CorrespondenceCell {
            id: id("corr:gluing-test"),
            participants: vec![
                CorrespondenceParticipant::new("left", ParticipantRef::Claim(id("claim:observed")))
                    .expect("participant"),
                CorrespondenceParticipant::new(
                    "right",
                    ParticipantRef::Claim(id("claim:required")),
                )
                .expect("participant"),
            ],
            correspondence_kind: CorrespondenceKind::SurfaceOverlap,
            polarity: CorrespondencePolarity::Agreeing,
            overlap_witnesses: vec![OverlapWitness {
                id: id("witness:shared-feature"),
                witness_kind: OverlapWitnessKind::FeatureSet,
                shared_structure: SharedStructure::FeatureSet(vec![Feature {
                    key: "label".to_owned(),
                    value: "OrderService".to_owned(),
                }]),
                participant_mappings: vec![ParticipantMapping {
                    participant: id("claim:observed"),
                    path: "$.label".to_owned(),
                }],
                scope: Scope::default(),
                context: id("ctx:test"),
                evidence: vec![id("evidence:test")],
                confidence: confidence(0.9),
                status: ReviewStatus::Candidate,
            }],
            difference_witnesses: Vec::new(),
            context: id("ctx:test"),
            evidence: vec![id("evidence:test")],
            provenance: id("provenance:test"),
            confidence: confidence(0.9),
            review_status: ReviewStatus::Candidate,
            gluing: None,
        }
    }

    #[test]
    fn blocking_difference_fails_gluing() {
        let mut correspondence = base_correspondence();
        correspondence.difference_witnesses.push(DifferenceWitness {
            id: id("diff:blocking"),
            difference_kind: DifferenceKind::ModalityMismatch,
            differing_structure: DifferingStructure::ModalityMismatch(BTreeMap::from([
                ("left".to_owned(), "observed".to_owned()),
                ("right".to_owned(), "forbidden".to_owned()),
            ])),
            participant_mappings: Vec::new(),
            severity: DifferenceSeverity::Blocking,
            context: id("ctx:test"),
            evidence: vec![id("evidence:test")],
            confidence: confidence(0.93),
            status: ReviewStatus::Candidate,
        });

        let attempt = attempt_gluing(&correspondence).expect("gluing check");

        assert!(matches!(attempt.result, GluingResult::Failure { .. }));
        assert!(attempt
            .validate_with_differences(&correspondence.difference_witnesses)
            .is_ok());
    }

    #[test]
    fn major_difference_yields_review_candidate() {
        let mut correspondence = base_correspondence();
        correspondence.difference_witnesses.push(DifferenceWitness {
            id: id("diff:context"),
            difference_kind: DifferenceKind::ContextMismatch,
            differing_structure: DifferingStructure::ContextMismatch(BTreeMap::from([
                ("left".to_owned(), "ctx:legacy".to_owned()),
                ("right".to_owned(), "ctx:production".to_owned()),
            ])),
            participant_mappings: Vec::new(),
            severity: DifferenceSeverity::Major,
            context: id("ctx:test"),
            evidence: vec![id("evidence:test")],
            confidence: confidence(0.84),
            status: ReviewStatus::Candidate,
        });

        let attempt = attempt_gluing(&correspondence).expect("gluing check");

        match attempt.result {
            GluingResult::Candidate {
                required_review, ..
            } => {
                assert!(required_review.required);
                assert!(required_review
                    .decision_reason
                    .expect("decision reason")
                    .contains("major differences"));
            }
            other => panic!("expected candidate, got {other:?}"),
        }
    }

    #[test]
    fn supported_overlap_succeeds_with_preservation_report() {
        let mut correspondence = base_correspondence();
        correspondence.review_status = ReviewStatus::Reviewed;

        let attempt = attempt_gluing(&correspondence).expect("gluing check");

        match attempt.result {
            GluingResult::Success {
                preservation_report,
                ..
            } => {
                assert!(!preservation_report.preserved_structures.is_empty());
                assert!(preservation_report.summary.is_some());
            }
            other => panic!("expected success, got {other:?}"),
        }
        assert!(attempt.preservation_report.summary.is_some());
    }

    #[test]
    fn evidence_absence_yields_review_candidate() {
        let mut correspondence = base_correspondence();
        correspondence.evidence.clear();
        correspondence.overlap_witnesses[0].evidence.clear();

        let attempt = attempt_gluing(&correspondence).expect("gluing check");

        assert!(matches!(attempt.result, GluingResult::Candidate { .. }));
    }
}
