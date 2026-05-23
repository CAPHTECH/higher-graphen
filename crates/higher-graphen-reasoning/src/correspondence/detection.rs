use super::*;

mod difference;

use difference::difference_witnesses;

/// Generates deterministic correspondence candidates from exact, reviewable signals.
pub fn derive_correspondence_candidates(
    input: CorrespondenceDetectionInput,
) -> Result<CorrespondenceDetectionResult> {
    let CorrespondenceDetectionInput {
        context,
        provenance,
        scope,
        subjects,
        semantic_signals,
    } = input;
    let input_context = CorrespondenceDetectionInput {
        context: context.clone(),
        provenance,
        scope,
        subjects: Vec::new(),
        semantic_signals: Vec::new(),
    };
    let subjects = subjects
        .into_iter()
        .filter(|subject| scope.includes(&subject.participant))
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    let mut candidate_ids = BTreeSet::new();

    for left_index in 0..subjects.len() {
        for right_index in (left_index + 1)..subjects.len() {
            if let Some(candidate) = candidate_for_pair(
                &subjects[left_index],
                &subjects[right_index],
                &input_context,
            )? {
                if !candidate_ids.insert(candidate.id.clone()) {
                    return Err(CoreError::MalformedField {
                        field: "candidates.id".to_owned(),
                        reason: format!("duplicate correspondence candidate id {}", candidate.id),
                    });
                }
                candidates.push(candidate);
            }
        }
    }

    for signal in semantic_signals {
        if let Some(candidate) = semantic_candidate(&signal, &subjects, &input_context)? {
            if !candidate_ids.insert(candidate.id.clone()) {
                return Err(CoreError::MalformedField {
                    field: "candidates.id".to_owned(),
                    reason: format!("duplicate correspondence candidate id {}", candidate.id),
                });
            }
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(CorrespondenceDetectionResult {
        context,
        scope,
        candidates,
    })
}

impl CorrespondenceScope {
    fn includes(self, participant: &ParticipantRef) -> bool {
        match self {
            Self::All => true,
            Self::Cells => matches!(participant, ParticipantRef::Cell(_)),
            Self::Claims => matches!(participant, ParticipantRef::Claim(_)),
            Self::Evidence => matches!(participant, ParticipantRef::Evidence(_)),
            Self::Invariants => matches!(participant, ParticipantRef::Invariant(_)),
        }
    }
}

fn candidate_for_pair(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<Option<CorrespondenceCell>> {
    let mut witnesses = Vec::new();
    push_same_id_witness(&mut witnesses, left, right, input)?;
    push_normalized_label_witness(&mut witnesses, left, right, input)?;
    push_evidence_witness(&mut witnesses, left, right, input)?;
    push_invariant_witness(&mut witnesses, left, right, input)?;
    push_typed_relation_witnesses(&mut witnesses, left, right, input)?;

    if witnesses.is_empty() {
        return Ok(None);
    }

    build_pair_candidate(left, right, input, witnesses).map(Some)
}

fn push_same_id_witness(
    witnesses: &mut Vec<OverlapWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    if left.participant.id() == right.participant.id() {
        witnesses.push(witness(
            "same-id",
            OverlapWitnessKind::FeatureSet,
            SharedStructure::FeatureSet(vec![Feature {
                key: "id".to_owned(),
                value: left.participant.id().to_string(),
            }]),
            left,
            right,
            input,
            Confidence::ONE,
            Vec::new(),
        )?);
    }
    Ok(())
}

fn push_normalized_label_witness(
    witnesses: &mut Vec<OverlapWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    if let (Some(left_label), Some(right_label)) = (&left.normalized_label, &right.normalized_label)
    {
        if left_label == right_label {
            witnesses.push(witness(
                "normalized-label",
                OverlapWitnessKind::FeatureSet,
                SharedStructure::FeatureSet(vec![Feature {
                    key: "normalized_label".to_owned(),
                    value: left_label.clone(),
                }]),
                left,
                right,
                input,
                confidence(0.8)?,
                Vec::new(),
            )?);
        }
    }
    Ok(())
}

fn push_evidence_witness(
    witnesses: &mut Vec<OverlapWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    let shared_evidence = intersection(&left.evidence, &right.evidence);
    if !shared_evidence.is_empty() {
        witnesses.push(witness(
            "evidence",
            OverlapWitnessKind::EvidenceSet,
            SharedStructure::EvidenceSet(shared_evidence.clone()),
            left,
            right,
            input,
            confidence(0.9)?,
            shared_evidence,
        )?);
    }
    Ok(())
}

fn push_invariant_witness(
    witnesses: &mut Vec<OverlapWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    let shared_invariants = intersection(&left.invariants, &right.invariants);
    if !shared_invariants.is_empty() {
        witnesses.push(witness(
            "invariant",
            OverlapWitnessKind::ConstraintSet,
            SharedStructure::ConstraintSet(shared_invariants.clone()),
            left,
            right,
            input,
            confidence(0.9)?,
            Vec::new(),
        )?);
    }
    Ok(())
}

fn push_typed_relation_witnesses(
    witnesses: &mut Vec<OverlapWitness>,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
) -> Result<()> {
    let shared_relations = shared_typed_relations(left, right);
    if !shared_relations.is_empty() {
        let predicates = shared_relations
            .iter()
            .map(TypedRelation::to_predicate)
            .collect::<Vec<_>>();
        let first_relation = &shared_relations[0];
        witnesses.push(witness(
            "typed-relation",
            OverlapWitnessKind::PredicateSet,
            SharedStructure::PredicateSet(predicates),
            left,
            right,
            input,
            confidence(0.95)?,
            Vec::new(),
        )?);
        witnesses.push(witness(
            "normalized-claim",
            OverlapWitnessKind::NormalizedClaim,
            SharedStructure::NormalizedClaim(NormalizedClaim {
                subject: first_relation.subject.clone(),
                relation: first_relation.relation.clone(),
                object: first_relation.object.clone(),
                modality: None,
                temporal_scope: None,
            }),
            left,
            right,
            input,
            confidence(0.95)?,
            Vec::new(),
        )?);
    }
    Ok(())
}

fn build_pair_candidate(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    witnesses: Vec<OverlapWitness>,
) -> Result<CorrespondenceCell> {
    let differences = difference_witnesses(left, right, input, &witnesses)?;
    let correspondence_kind = candidate_kind(&witnesses);
    let evidence = unique_ids(
        witnesses
            .iter()
            .flat_map(|witness| witness.evidence.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let confidence = witnesses
        .iter()
        .map(|witness| witness.confidence.value())
        .fold(0.0, f64::max);

    let candidate = CorrespondenceCell {
        id: candidate_id(left.participant.id(), right.participant.id())?,
        participants: vec![participant("left", left)?, participant("right", right)?],
        correspondence_kind,
        polarity: CorrespondencePolarity::Unknown,
        overlap_witnesses: witnesses,
        difference_witnesses: differences,
        context: input.context.clone(),
        evidence,
        provenance: input.provenance.clone(),
        confidence: Confidence::new(confidence)?,
        review_status: ReviewStatus::Candidate,
        gluing: None,
    };
    candidate.validate()?;

    Ok(candidate)
}

fn semantic_candidate(
    signal: &SemanticCorrespondenceSignal,
    subjects: &[CorrespondenceSubject],
    input: &CorrespondenceDetectionInput,
) -> Result<Option<CorrespondenceCell>> {
    validate_semantic_signal(signal)?;
    if !semantic_signal_in_scope(signal, input.scope) {
        return Ok(None);
    }

    let subject_by_id = subject_index(subjects);
    let participants = semantic_participants(signal, &subject_by_id)?;
    let calibrated_confidence = calibrate_semantic_confidence(signal)?;
    let overlap_witnesses = semantic_overlap_witnesses(signal, input, calibrated_confidence)?;
    let differences =
        semantic_difference_witnesses(signal, &subject_by_id, input, &overlap_witnesses)?;
    let evidence = semantic_evidence(signal, &overlap_witnesses);

    build_semantic_candidate(
        signal,
        input,
        participants,
        overlap_witnesses,
        differences,
        evidence,
        calibrated_confidence,
    )
    .map(Some)
}

fn validate_semantic_signal(signal: &SemanticCorrespondenceSignal) -> Result<()> {
    if signal.participants.len() < 2 {
        return Err(CoreError::MalformedField {
            field: "semantic_signals.participants".to_owned(),
            reason: "semantic signal requires at least two participants".to_owned(),
        });
    }
    Ok(())
}

fn semantic_signal_in_scope(
    signal: &SemanticCorrespondenceSignal,
    scope: CorrespondenceScope,
) -> bool {
    signal
        .participants
        .iter()
        .all(|participant| scope.includes(participant))
}

fn subject_index(subjects: &[CorrespondenceSubject]) -> BTreeMap<Id, &CorrespondenceSubject> {
    subjects
        .iter()
        .map(|subject| (subject.participant.id().clone(), subject))
        .collect()
}

fn semantic_participants(
    signal: &SemanticCorrespondenceSignal,
    subject_by_id: &BTreeMap<Id, &CorrespondenceSubject>,
) -> Result<Vec<CorrespondenceParticipant>> {
    signal
        .participants
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            let role = subject_by_id
                .get(participant.id())
                .and_then(|subject| subject.role.as_deref())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("participant_{}", index + 1));
            CorrespondenceParticipant::new(role, participant.clone())
        })
        .collect()
}

fn semantic_overlap_witnesses(
    signal: &SemanticCorrespondenceSignal,
    input: &CorrespondenceDetectionInput,
    calibrated_confidence: Confidence,
) -> Result<Vec<OverlapWitness>> {
    let mut overlap_witnesses = Vec::new();
    if let Some(normalized_claim) = &signal.normalized_claim {
        overlap_witnesses.push(semantic_witness(
            "normalized-claim",
            signal,
            OverlapWitnessKind::NormalizedClaim,
            SharedStructure::NormalizedClaim(normalized_claim.clone()),
            input,
            calibrated_confidence,
        )?);
    }

    let features = semantic_features(signal);
    if !features.is_empty() {
        overlap_witnesses.push(semantic_witness(
            "features",
            signal,
            OverlapWitnessKind::FeatureSet,
            SharedStructure::FeatureSet(features),
            input,
            calibrated_confidence,
        )?);
    }

    if overlap_witnesses.is_empty() {
        return Err(CoreError::MalformedField {
            field: "semantic_signals.shared_structure".to_owned(),
            reason:
                "semantic signal requires normalizedClaim, features, embeddingScore, or rationale"
                    .to_owned(),
        });
    }

    Ok(overlap_witnesses)
}

fn semantic_features(signal: &SemanticCorrespondenceSignal) -> Vec<Feature> {
    let mut features = signal.features.clone();
    features.push(Feature {
        key: "semantic_signal_source".to_owned(),
        value: format!("{:?}", signal.source),
    });
    if let Some(embedding_score) = signal.embedding_score {
        features.push(Feature {
            key: "embedding_score".to_owned(),
            value: embedding_score.to_string(),
        });
    }
    if let Some(rationale) = &signal.rationale {
        features.push(Feature {
            key: "rationale".to_owned(),
            value: rationale.clone(),
        });
    }
    features
}

fn semantic_evidence(
    signal: &SemanticCorrespondenceSignal,
    overlap_witnesses: &[OverlapWitness],
) -> Vec<Id> {
    unique_ids(
        signal
            .evidence
            .iter()
            .cloned()
            .chain(
                overlap_witnesses
                    .iter()
                    .flat_map(|witness| witness.evidence.iter().cloned()),
            )
            .collect(),
    )
}

#[allow(clippy::too_many_arguments)]
fn build_semantic_candidate(
    signal: &SemanticCorrespondenceSignal,
    input: &CorrespondenceDetectionInput,
    participants: Vec<CorrespondenceParticipant>,
    overlap_witnesses: Vec<OverlapWitness>,
    differences: Vec<DifferenceWitness>,
    evidence: Vec<Id>,
    calibrated_confidence: Confidence,
) -> Result<CorrespondenceCell> {
    let candidate = CorrespondenceCell {
        id: semantic_candidate_id(&signal.id)?,
        participants,
        correspondence_kind: CorrespondenceKind::SemanticOverlap,
        polarity: signal.polarity,
        overlap_witnesses,
        difference_witnesses: differences,
        context: input.context.clone(),
        evidence,
        provenance: input.provenance.clone(),
        confidence: calibrated_confidence,
        review_status: ReviewStatus::Candidate,
        gluing: None,
    };
    candidate.validate()?;

    Ok(candidate)
}

fn semantic_witness(
    suffix: &str,
    signal: &SemanticCorrespondenceSignal,
    witness_kind: OverlapWitnessKind,
    shared_structure: SharedStructure,
    input: &CorrespondenceDetectionInput,
    confidence: Confidence,
) -> Result<OverlapWitness> {
    Ok(OverlapWitness {
        id: semantic_witness_id(suffix, &signal.id)?,
        witness_kind,
        shared_structure,
        participant_mappings: signal
            .participants
            .iter()
            .map(|participant| ParticipantMapping {
                participant: participant.id().clone(),
                path: "$.semanticSignal".to_owned(),
            })
            .collect(),
        scope: Scope {
            structure_ids: signal
                .participants
                .iter()
                .map(|participant| participant.id().clone())
                .collect(),
            boundary: None,
        },
        context: input.context.clone(),
        evidence: signal.evidence.clone(),
        confidence,
        status: ReviewStatus::Candidate,
    })
}

fn semantic_difference_witnesses(
    signal: &SemanticCorrespondenceSignal,
    subject_by_id: &BTreeMap<Id, &CorrespondenceSubject>,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<Vec<DifferenceWitness>> {
    if signal.participants.len() != 2 {
        return Ok(Vec::new());
    }

    let Some(left) = subject_by_id.get(signal.participants[0].id()).copied() else {
        return Ok(Vec::new());
    };
    let Some(right) = subject_by_id.get(signal.participants[1].id()).copied() else {
        return Ok(Vec::new());
    };

    difference_witnesses(left, right, input, overlaps)
}

fn calibrate_semantic_confidence(signal: &SemanticCorrespondenceSignal) -> Result<Confidence> {
    let mut score = signal.confidence.value();
    if let Some(embedding_score) = signal.embedding_score {
        score = (score + embedding_score.value()) / 2.0;
    }
    if !signal.evidence.is_empty() {
        score = (score + 0.05).min(1.0);
    }

    Confidence::new(score.min(signal.source.confidence_cap()))
}

fn candidate_kind(witnesses: &[OverlapWitness]) -> CorrespondenceKind {
    if witnesses
        .iter()
        .any(|witness| matches!(witness.witness_kind, OverlapWitnessKind::PredicateSet))
    {
        CorrespondenceKind::StructuralOverlap
    } else if witnesses
        .iter()
        .any(|witness| matches!(witness.witness_kind, OverlapWitnessKind::ConstraintSet))
    {
        CorrespondenceKind::ConstraintOverlap
    } else if witnesses
        .iter()
        .any(|witness| matches!(witness.witness_kind, OverlapWitnessKind::EvidenceSet))
    {
        CorrespondenceKind::EvidenceOverlap
    } else {
        CorrespondenceKind::SurfaceOverlap
    }
}

fn participant(role: &str, subject: &CorrespondenceSubject) -> Result<CorrespondenceParticipant> {
    CorrespondenceParticipant::new(
        subject.role.as_deref().unwrap_or(role),
        subject.participant.clone(),
    )
}

#[allow(clippy::too_many_arguments)]
fn witness(
    suffix: &str,
    witness_kind: OverlapWitnessKind,
    shared_structure: SharedStructure,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    confidence: Confidence,
    evidence: Vec<Id>,
) -> Result<OverlapWitness> {
    Ok(OverlapWitness {
        id: witness_id(suffix, left.participant.id(), right.participant.id())?,
        witness_kind,
        shared_structure,
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
        scope: Scope {
            structure_ids: vec![
                left.participant.id().clone(),
                right.participant.id().clone(),
            ],
            boundary: None,
        },
        context: input.context.clone(),
        evidence,
        confidence,
        status: ReviewStatus::Candidate,
    })
}

fn shared_typed_relations(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
) -> Vec<TypedRelation> {
    let right_relations = right
        .typed_relations
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    left.typed_relations
        .iter()
        .filter(|relation| right_relations.contains(*relation))
        .cloned()
        .collect()
}

fn intersection(left: &[Id], right: &[Id]) -> Vec<Id> {
    let right_ids = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|id| right_ids.contains(id))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn candidate_id(left: &Id, right: &Id) -> Result<Id> {
    Id::new(format!(
        "corr:candidate:{}:{}",
        safe_id_segment(left),
        safe_id_segment(right)
    ))
}

fn witness_id(suffix: &str, left: &Id, right: &Id) -> Result<Id> {
    Id::new(format!(
        "witness:candidate:{}:{}:{}",
        suffix,
        safe_id_segment(left),
        safe_id_segment(right)
    ))
}

fn difference_id(suffix: &str, left: &Id, right: &Id) -> Result<Id> {
    Id::new(format!(
        "diff:candidate:{}:{}:{}",
        suffix,
        safe_id_segment(left),
        safe_id_segment(right)
    ))
}

fn semantic_candidate_id(signal: &Id) -> Result<Id> {
    Id::new(format!("corr:semantic:{}", safe_id_segment(signal)))
}

fn semantic_witness_id(suffix: &str, signal: &Id) -> Result<Id> {
    Id::new(format!(
        "witness:semantic:{}:{}",
        suffix,
        safe_id_segment(signal)
    ))
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

fn confidence(value: f64) -> Result<Confidence> {
    Confidence::new(value)
}
