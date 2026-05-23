use super::*;

pub(super) fn difference_witnesses(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<Vec<DifferenceWitness>> {
    let mut differences = Vec::new();
    push_modality_difference(&mut differences, left, right, input, overlaps)?;
    push_evidence_difference(&mut differences, left, right, input, overlaps)?;
    push_invariant_differences(&mut differences, left, right, input)?;
    push_context_difference(&mut differences, left, right, input, overlaps)?;
    Ok(differences)
}

fn push_modality_difference(
    differences: &mut Vec<DifferenceWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<()> {
    if !has_claim_overlap(left, right, overlaps) {
        return Ok(());
    }
    let (Some(left_modality), Some(right_modality)) = (&left.modality, &right.modality) else {
        return Ok(());
    };
    if left_modality == right_modality {
        return Ok(());
    }

    differences.push(difference(
        DifferenceDraft {
            suffix: "modality",
            difference_kind: DifferenceKind::ModalityMismatch,
            differing_structure: DifferingStructure::ModalityMismatch(role_map(
                left,
                right,
                |subject| subject.modality.clone().unwrap_or_default(),
            )),
            severity: DifferenceSeverity::Blocking,
            evidence: evidence_union(left, right),
            confidence: confidence(0.93)?,
        },
        left,
        right,
        input,
    )?);
    Ok(())
}

fn push_evidence_difference(
    differences: &mut Vec<DifferenceWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<()> {
    if !has_non_evidence_overlap(overlaps) || left.evidence.is_empty() || right.evidence.is_empty()
    {
        return Ok(());
    }
    let shared_evidence = intersection(&left.evidence, &right.evidence);
    if shared_evidence.len() == left.evidence.len() && shared_evidence.len() == right.evidence.len()
    {
        return Ok(());
    }

    let severity = if shared_evidence.is_empty() {
        DifferenceSeverity::Major
    } else {
        DifferenceSeverity::Minor
    };
    differences.push(difference(
        DifferenceDraft {
            suffix: "evidence",
            difference_kind: DifferenceKind::EvidenceMismatch,
            differing_structure: DifferingStructure::EvidenceMismatch(symmetric_difference(
                &left.evidence,
                &right.evidence,
            )),
            severity,
            evidence: evidence_union(left, right),
            confidence: confidence(0.82)?,
        },
        left,
        right,
        input,
    )?);
    Ok(())
}

fn push_invariant_differences(
    differences: &mut Vec<DifferenceWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    for mismatch in shared_invariant_state_mismatches(left, right) {
        differences.push(invariant_difference(mismatch, left, right, input)?);
    }
    Ok(())
}

fn invariant_difference(
    mismatch: (Id, InvariantSatisfaction, InvariantSatisfaction),
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<DifferenceWitness> {
    let (invariant, left_state, right_state) = mismatch;
    let severity = if left_state.is_failed() || right_state.is_failed() {
        DifferenceSeverity::Blocking
    } else {
        DifferenceSeverity::Major
    };
    difference(
        DifferenceDraft {
            suffix: "invariant",
            difference_kind: DifferenceKind::InvariantMismatch,
            differing_structure: invariant_difference_structure(
                invariant,
                left_state,
                right_state,
                left,
                right,
            ),
            severity,
            evidence: evidence_union(left, right),
            confidence: confidence(0.9)?,
        },
        left,
        right,
        input,
    )
}

fn invariant_difference_structure(
    invariant: Id,
    left_state: InvariantSatisfaction,
    right_state: InvariantSatisfaction,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
) -> DifferingStructure {
    DifferingStructure::InvariantMismatch(vec![
        InvariantCheckResult {
            invariant: invariant.clone(),
            result: left_state.as_result().to_owned(),
            detail: Some(format!(
                "{} reports {}",
                role_or_id(left),
                left_state.as_result()
            )),
        },
        InvariantCheckResult {
            invariant,
            result: right_state.as_result().to_owned(),
            detail: Some(format!(
                "{} reports {}",
                role_or_id(right),
                right_state.as_result()
            )),
        },
    ])
}

fn push_context_difference(
    differences: &mut Vec<DifferenceWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<()> {
    if !has_context_mismatch(left, right, overlaps) {
        return Ok(());
    }
    differences.push(difference(
        DifferenceDraft {
            suffix: "context",
            difference_kind: DifferenceKind::ContextMismatch,
            differing_structure: DifferingStructure::ContextMismatch(role_map(
                left,
                right,
                |subject| {
                    subject
                        .contexts
                        .iter()
                        .map(Id::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                },
            )),
            severity: DifferenceSeverity::Major,
            evidence: evidence_union(left, right),
            confidence: confidence(0.84)?,
        },
        left,
        right,
        input,
    )?);
    Ok(())
}

fn has_claim_overlap(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    overlaps: &[OverlapWitness],
) -> bool {
    !shared_typed_relations(left, right).is_empty() || has_semantic_claim_overlap(overlaps)
}

fn has_context_mismatch(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    overlaps: &[OverlapWitness],
) -> bool {
    has_non_context_overlap(overlaps)
        && !left.contexts.is_empty()
        && !right.contexts.is_empty()
        && intersection(&left.contexts, &right.contexts).is_empty()
}

struct DifferenceDraft {
    suffix: &'static str,
    difference_kind: DifferenceKind,
    differing_structure: DifferingStructure,
    severity: DifferenceSeverity,
    evidence: Vec<Id>,
    confidence: Confidence,
}

fn difference(
    draft: DifferenceDraft,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<DifferenceWitness> {
    Ok(DifferenceWitness {
        id: difference_id(draft.suffix, left.participant.id(), right.participant.id())?,
        difference_kind: draft.difference_kind,
        differing_structure: draft.differing_structure,
        participant_mappings: vec![
            ParticipantMapping {
                participant: left.participant.id().clone(),
                path: "$".to_owned(),
            },
            ParticipantMapping {
                participant: right.participant.id().clone(),
                path: "$".to_owned(),
            },
        ],
        severity: draft.severity,
        context: input.context.clone(),
        evidence: draft.evidence,
        confidence: draft.confidence,
        status: ReviewStatus::Candidate,
    })
}

fn has_non_evidence_overlap(overlaps: &[OverlapWitness]) -> bool {
    overlaps
        .iter()
        .any(|overlap| !matches!(overlap.witness_kind, OverlapWitnessKind::EvidenceSet))
}

fn has_non_context_overlap(overlaps: &[OverlapWitness]) -> bool {
    overlaps
        .iter()
        .any(|overlap| !matches!(overlap.witness_kind, OverlapWitnessKind::ContextRestriction))
}

fn has_semantic_claim_overlap(overlaps: &[OverlapWitness]) -> bool {
    overlaps.iter().any(|overlap| {
        matches!(
            overlap.witness_kind,
            OverlapWitnessKind::NormalizedClaim | OverlapWitnessKind::PredicateSet
        )
    })
}

fn role_map(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    value: impl Fn(&CorrespondenceSubject) -> String,
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (role_or_id(left), value(left)),
        (role_or_id(right), value(right)),
    ])
}

fn role_or_id(subject: &CorrespondenceSubject) -> String {
    subject
        .role
        .clone()
        .unwrap_or_else(|| subject.participant.id().to_string())
}

fn shared_invariant_state_mismatches(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
) -> Vec<(Id, InvariantSatisfaction, InvariantSatisfaction)> {
    let right_states = right
        .invariant_states
        .iter()
        .map(|state| (state.invariant.clone(), state.satisfaction))
        .collect::<BTreeMap<_, _>>();

    left.invariant_states
        .iter()
        .filter_map(|left_state| {
            right_states
                .get(&left_state.invariant)
                .copied()
                .filter(|right_state| right_state != &left_state.satisfaction)
                .map(|right_state| {
                    (
                        left_state.invariant.clone(),
                        left_state.satisfaction,
                        right_state,
                    )
                })
        })
        .collect()
}

fn evidence_union(left: &CorrespondenceSubject, right: &CorrespondenceSubject) -> Vec<Id> {
    unique_ids(
        left.evidence
            .iter()
            .chain(right.evidence.iter())
            .cloned()
            .collect(),
    )
}

fn symmetric_difference(left: &[Id], right: &[Id]) -> Vec<Id> {
    let left_ids = left.iter().collect::<BTreeSet<_>>();
    let right_ids = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .chain(right.iter())
        .filter(|id| left_ids.contains(*id) != right_ids.contains(*id))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
