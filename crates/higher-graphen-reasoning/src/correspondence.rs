//! Deterministic correspondence candidate generation.

use higher_graphen_core::{
    Confidence, CoreError, CorrespondenceCell, CorrespondenceKind, CorrespondenceParticipant,
    CorrespondencePolarity, DifferenceKind, DifferenceSeverity, DifferenceWitness,
    DifferingStructure, Feature, Id, InvariantCheckResult, NormalizedClaim, OverlapWitness,
    OverlapWitnessKind, ParticipantMapping, ParticipantRef, Predicate, Result, ReviewStatus, Scope,
    SharedStructure,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Input scope for deterministic correspondence generation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceScope {
    /// Consider every supplied item.
    All,
    /// Consider cells only.
    Cells,
    /// Consider claims only.
    Claims,
    /// Consider evidence items only.
    Evidence,
    /// Consider invariants only.
    Invariants,
}

/// Deterministic relation triple used for exact structural overlap detection.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypedRelation {
    /// Subject identifier or normalized label.
    pub subject: String,
    /// Relation or predicate name.
    pub relation: String,
    /// Object identifier or normalized label.
    pub object: String,
}

/// Deterministic invariant satisfaction state observed for one subject.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantSatisfaction {
    /// The invariant is satisfied.
    Satisfied,
    /// The invariant is violated or failed.
    Failed,
    /// The invariant was checked but the outcome is unknown.
    Unknown,
    /// The invariant was not applicable in this local subject.
    NotApplicable,
}

impl InvariantSatisfaction {
    fn as_result(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
            Self::NotApplicable => "not_applicable",
        }
    }

    fn is_failed(self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// Invariant state attached to a correspondence subject.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvariantState {
    /// Invariant identifier.
    pub invariant: Id,
    /// Satisfaction state for this subject.
    pub satisfaction: InvariantSatisfaction,
}

impl InvariantState {
    /// Creates an invariant state.
    #[must_use]
    pub fn new(invariant: Id, satisfaction: InvariantSatisfaction) -> Self {
        Self {
            invariant,
            satisfaction,
        }
    }
}

impl TypedRelation {
    /// Creates a typed relation with non-empty normalized fields.
    pub fn new(
        subject: impl Into<String>,
        relation: impl Into<String>,
        object: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            subject: required_text("typed_relation.subject", subject)?,
            relation: required_text("typed_relation.relation", relation)?,
            object: required_text("typed_relation.object", object)?,
        })
    }

    fn to_predicate(&self) -> Predicate {
        Predicate {
            subject: self.subject.clone(),
            relation: self.relation.clone(),
            object: self.object.clone(),
        }
    }
}

/// One structure snapshot used by deterministic correspondence generation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceSubject {
    /// Structure participating in possible correspondences.
    pub participant: ParticipantRef,
    /// Optional role used when emitting candidates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Optional normalized label for surface overlap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_label: Option<String>,
    /// Optional modality such as observed, inferred, required, or forbidden.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    /// Context identifiers in which this subject is valid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Id>,
    /// Evidence identifiers supporting this subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Invariant identifiers related to this subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<Id>,
    /// Invariant states observed for this subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_states: Vec<InvariantState>,
    /// Exact typed relations extracted from this subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub typed_relations: Vec<TypedRelation>,
}

impl CorrespondenceSubject {
    /// Creates a subject snapshot for candidate generation.
    #[must_use]
    pub fn new(participant: ParticipantRef) -> Self {
        Self {
            participant,
            role: None,
            normalized_label: None,
            modality: None,
            contexts: Vec::new(),
            evidence: Vec::new(),
            invariants: Vec::new(),
            invariant_states: Vec::new(),
            typed_relations: Vec::new(),
        }
    }

    /// Returns this subject with a role.
    pub fn with_role(mut self, role: impl Into<String>) -> Result<Self> {
        self.role = Some(required_text("role", role)?);
        Ok(self)
    }

    /// Returns this subject with a normalized label.
    pub fn with_normalized_label(mut self, normalized_label: impl Into<String>) -> Result<Self> {
        self.normalized_label = Some(required_text("normalized_label", normalized_label)?);
        Ok(self)
    }

    /// Returns this subject with a modality.
    pub fn with_modality(mut self, modality: impl Into<String>) -> Result<Self> {
        self.modality = Some(required_text("modality", modality)?);
        Ok(self)
    }

    /// Returns this subject with context references.
    #[must_use]
    pub fn with_contexts(mut self, contexts: Vec<Id>) -> Self {
        self.contexts = unique_ids(contexts);
        self
    }

    /// Returns this subject with evidence references.
    #[must_use]
    pub fn with_evidence(mut self, evidence: Vec<Id>) -> Self {
        self.evidence = unique_ids(evidence);
        self
    }

    /// Returns this subject with invariant references.
    #[must_use]
    pub fn with_invariants(mut self, invariants: Vec<Id>) -> Self {
        self.invariants = unique_ids(invariants);
        self
    }

    /// Returns this subject with invariant states.
    #[must_use]
    pub fn with_invariant_states(mut self, invariant_states: Vec<InvariantState>) -> Self {
        self.invariant_states = invariant_states;
        self
    }

    /// Returns this subject with typed relation triples.
    #[must_use]
    pub fn with_typed_relations(mut self, typed_relations: Vec<TypedRelation>) -> Self {
        self.typed_relations = typed_relations;
        self
    }
}

/// Input for deterministic correspondence candidate generation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceDetectionInput {
    /// Context in which generated candidates are valid.
    pub context: Id,
    /// Provenance reference to attach to generated candidate cells.
    pub provenance: Id,
    /// Candidate generation scope.
    pub scope: CorrespondenceScope,
    /// Subject snapshots to compare pairwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<CorrespondenceSubject>,
    /// Reviewable semantic signals supplied by bounded LLM, embedding, tool, or human adapters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_signals: Vec<SemanticCorrespondenceSignal>,
}

impl CorrespondenceDetectionInput {
    /// Creates detection input for a context and subject set.
    #[must_use]
    pub fn new(context: Id, provenance: Id, subjects: Vec<CorrespondenceSubject>) -> Self {
        Self {
            context,
            provenance,
            scope: CorrespondenceScope::All,
            subjects,
            semantic_signals: Vec::new(),
        }
    }

    /// Returns this input with an explicit scope.
    #[must_use]
    pub fn with_scope(mut self, scope: CorrespondenceScope) -> Self {
        self.scope = scope;
        self
    }
}

/// Source category for a semantic candidate signal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSignalSource {
    /// Signal produced by an LLM adapter.
    Llm,
    /// Signal produced by an embedding or vector search adapter.
    Embedding,
    /// Signal supplied by a human reviewer or analyst.
    Human,
    /// Signal produced by a deterministic tool.
    Tool,
    /// Signal imported from another bounded system.
    Import,
}

impl SemanticSignalSource {
    fn confidence_cap(self) -> f64 {
        match self {
            Self::Llm => 0.86,
            Self::Embedding => 0.72,
            Self::Human => 0.9,
            Self::Tool => 0.82,
            Self::Import => 0.8,
        }
    }
}

/// Reviewable semantic correspondence signal from a bounded adapter.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticCorrespondenceSignal {
    /// Stable signal identifier.
    pub id: Id,
    /// Participants that the semantic signal connects.
    pub participants: Vec<ParticipantRef>,
    /// Signal source category.
    pub source: SemanticSignalSource,
    /// Optional polarity inferred by the adapter.
    #[serde(default = "default_semantic_polarity")]
    pub polarity: CorrespondencePolarity,
    /// Optional normalized claim extracted by the adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_claim: Option<NormalizedClaim>,
    /// Optional feature-level explanation from the adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Feature>,
    /// Evidence references supporting the signal.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Adapter confidence before calibration.
    pub confidence: Confidence,
    /// Optional embedding similarity score before calibration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_score: Option<Confidence>,
    /// Optional bounded rationale retained as a review feature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

impl SemanticCorrespondenceSignal {
    /// Creates a semantic correspondence signal.
    pub fn new(
        id: Id,
        participants: Vec<ParticipantRef>,
        source: SemanticSignalSource,
        confidence: Confidence,
    ) -> Result<Self> {
        if participants.len() < 2 {
            return Err(CoreError::MalformedField {
                field: "semantic_signals.participants".to_owned(),
                reason: "semantic signal requires at least two participants".to_owned(),
            });
        }

        Ok(Self {
            id,
            participants,
            source,
            polarity: CorrespondencePolarity::Ambiguous,
            normalized_claim: None,
            features: Vec::new(),
            evidence: Vec::new(),
            confidence,
            embedding_score: None,
            rationale: None,
        })
    }
}

fn default_semantic_polarity() -> CorrespondencePolarity {
    CorrespondencePolarity::Ambiguous
}

/// Human review decision for semantic correspondence candidates.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticReviewDecision {
    /// Accept the semantic correspondence as reviewed structure.
    Accept,
    /// Reject the semantic correspondence.
    Reject,
}

/// Explicit review request for a semantic correspondence candidate.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticCorrespondenceReviewRequest {
    /// Candidate correspondence under review.
    pub candidate_id: Id,
    /// Reviewer identity.
    pub reviewer: Id,
    /// Review decision.
    pub decision: SemanticReviewDecision,
    /// Non-empty review rationale.
    pub reason: String,
}

impl SemanticCorrespondenceReviewRequest {
    /// Creates a semantic correspondence review request.
    pub fn new(
        candidate_id: Id,
        reviewer: Id,
        decision: SemanticReviewDecision,
        reason: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            candidate_id,
            reviewer,
            decision,
            reason: required_text("reason", reason)?,
        })
    }
}

/// Applies a human review decision to a semantic correspondence candidate.
pub fn review_semantic_correspondence(
    candidate: &CorrespondenceCell,
    request: SemanticCorrespondenceReviewRequest,
) -> Result<CorrespondenceCell> {
    if candidate.id != request.candidate_id {
        return Err(CoreError::MalformedField {
            field: "candidate_id".to_owned(),
            reason: "review request candidate_id does not match correspondence id".to_owned(),
        });
    }
    if !matches!(
        candidate.correspondence_kind,
        CorrespondenceKind::SemanticOverlap
    ) {
        return Err(CoreError::MalformedField {
            field: "correspondence_kind".to_owned(),
            reason: "semantic review only applies to SemanticOverlap candidates".to_owned(),
        });
    }
    if !matches!(
        candidate.review_status,
        ReviewStatus::Candidate | ReviewStatus::Reviewed
    ) {
        return Err(CoreError::MalformedField {
            field: "review_status".to_owned(),
            reason: "semantic review can only update candidate or reviewed correspondence"
                .to_owned(),
        });
    }

    let mut reviewed = candidate.clone();
    reviewed.review_status = match request.decision {
        SemanticReviewDecision::Accept => ReviewStatus::Accepted,
        SemanticReviewDecision::Reject => ReviewStatus::Rejected,
    };
    reviewed.validate()?;

    Ok(reviewed)
}

/// Deterministic candidate generation output.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceDetectionResult {
    /// Context used during generation.
    pub context: Id,
    /// Scope used during generation.
    pub scope: CorrespondenceScope,
    /// Candidate correspondence cells. These are always `ReviewStatus::Candidate`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CorrespondenceCell>,
}

impl CorrespondenceDetectionResult {
    /// Consumes the result and returns candidate cells.
    #[must_use]
    pub fn into_candidates(self) -> Vec<CorrespondenceCell> {
        self.candidates
    }
}

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

    if witnesses.is_empty() {
        return Ok(None);
    }

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

    Ok(Some(candidate))
}

fn semantic_candidate(
    signal: &SemanticCorrespondenceSignal,
    subjects: &[CorrespondenceSubject],
    input: &CorrespondenceDetectionInput,
) -> Result<Option<CorrespondenceCell>> {
    if signal.participants.len() < 2 {
        return Err(CoreError::MalformedField {
            field: "semantic_signals.participants".to_owned(),
            reason: "semantic signal requires at least two participants".to_owned(),
        });
    }
    if signal
        .participants
        .iter()
        .any(|participant| !input.scope.includes(participant))
    {
        return Ok(None);
    }

    let subject_by_id = subjects
        .iter()
        .map(|subject| (subject.participant.id().clone(), subject))
        .collect::<BTreeMap<_, _>>();
    let participants = signal
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
        .collect::<Result<Vec<_>>>()?;

    let calibrated_confidence = calibrate_semantic_confidence(signal)?;
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

    let differences =
        semantic_difference_witnesses(signal, &subject_by_id, input, &overlap_witnesses)?;
    let evidence = unique_ids(
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
    );
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

    Ok(Some(candidate))
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

fn difference_witnesses(
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    overlaps: &[OverlapWitness],
) -> Result<Vec<DifferenceWitness>> {
    let mut differences = Vec::new();
    let shared_relations = shared_typed_relations(left, right);

    if !shared_relations.is_empty() || has_semantic_claim_overlap(overlaps) {
        if let (Some(left_modality), Some(right_modality)) = (&left.modality, &right.modality) {
            if left_modality != right_modality {
                differences.push(difference(
                    "modality",
                    DifferenceKind::ModalityMismatch,
                    DifferingStructure::ModalityMismatch(role_map(left, right, |subject| {
                        subject.modality.clone().unwrap_or_default()
                    })),
                    DifferenceSeverity::Blocking,
                    left,
                    right,
                    input,
                    evidence_union(left, right),
                    confidence(0.93)?,
                )?);
            }
        }
    }

    if has_non_evidence_overlap(overlaps) && !left.evidence.is_empty() && !right.evidence.is_empty()
    {
        let shared_evidence = intersection(&left.evidence, &right.evidence);
        if shared_evidence.len() != left.evidence.len()
            || shared_evidence.len() != right.evidence.len()
        {
            differences.push(difference(
                "evidence",
                DifferenceKind::EvidenceMismatch,
                DifferingStructure::EvidenceMismatch(symmetric_difference(
                    &left.evidence,
                    &right.evidence,
                )),
                if shared_evidence.is_empty() {
                    DifferenceSeverity::Major
                } else {
                    DifferenceSeverity::Minor
                },
                left,
                right,
                input,
                evidence_union(left, right),
                confidence(0.82)?,
            )?);
        }
    }

    for (invariant, left_state, right_state) in shared_invariant_state_mismatches(left, right) {
        let severity = if left_state.is_failed() || right_state.is_failed() {
            DifferenceSeverity::Blocking
        } else {
            DifferenceSeverity::Major
        };
        differences.push(difference(
            "invariant",
            DifferenceKind::InvariantMismatch,
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
            ]),
            severity,
            left,
            right,
            input,
            evidence_union(left, right),
            confidence(0.9)?,
        )?);
    }

    if has_non_context_overlap(overlaps) && !left.contexts.is_empty() && !right.contexts.is_empty()
    {
        let shared_contexts = intersection(&left.contexts, &right.contexts);
        if shared_contexts.is_empty() {
            differences.push(difference(
                "context",
                DifferenceKind::ContextMismatch,
                DifferingStructure::ContextMismatch(role_map(left, right, |subject| {
                    subject
                        .contexts
                        .iter()
                        .map(Id::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                })),
                DifferenceSeverity::Major,
                left,
                right,
                input,
                evidence_union(left, right),
                confidence(0.84)?,
            )?);
        }
    }

    Ok(differences)
}

fn difference(
    suffix: &str,
    difference_kind: DifferenceKind,
    differing_structure: DifferingStructure,
    severity: DifferenceSeverity,
    left: &CorrespondenceSubject,
    right: &CorrespondenceSubject,
    input: &CorrespondenceDetectionInput,
    evidence: Vec<Id>,
    confidence: Confidence,
) -> Result<DifferenceWitness> {
    Ok(DifferenceWitness {
        id: difference_id(suffix, left.participant.id(), right.participant.id())?,
        difference_kind,
        differing_structure,
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
        severity,
        context: input.context.clone(),
        evidence,
        confidence,
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

fn required_text(field: &str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    if normalized.is_empty() {
        Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        })
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    #[test]
    fn deterministic_detection_finds_label_evidence_invariant_and_relation_overlap() {
        let relation =
            TypedRelation::new("OrderService", "accesses", "BillingDB").expect("valid relation");
        let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:observed")))
            .with_role("observed_claim")
            .expect("role")
            .with_normalized_label("order service billing access")
            .expect("label")
            .with_evidence(vec![id("evidence:scan")])
            .with_invariants(vec![id("invariant:no-cross-context-db-access")])
            .with_typed_relations(vec![relation.clone()]);
        let right = CorrespondenceSubject::new(ParticipantRef::Invariant(id(
            "invariant:no-cross-context-db-access",
        )))
        .with_role("constraint")
        .expect("role")
        .with_normalized_label("order service billing access")
        .expect("label")
        .with_evidence(vec![id("evidence:scan")])
        .with_invariants(vec![id("invariant:no-cross-context-db-access")])
        .with_typed_relations(vec![relation]);

        let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
            id("ctx:architecture-review"),
            id("provenance:deterministic-overlap"),
            vec![left, right],
        ))
        .expect("detection succeeds");

        assert_eq!(result.candidates.len(), 1);
        let candidate = &result.candidates[0];
        assert_eq!(
            candidate.correspondence_kind,
            CorrespondenceKind::StructuralOverlap
        );
        assert_eq!(candidate.review_status, ReviewStatus::Candidate);
        assert_eq!(candidate.participants[0].role, "observed_claim");
        assert!(candidate
            .overlap_witnesses
            .iter()
            .any(|witness| witness.witness_kind == OverlapWitnessKind::EvidenceSet));
        assert!(candidate
            .overlap_witnesses
            .iter()
            .any(|witness| witness.witness_kind == OverlapWitnessKind::ConstraintSet));
        assert!(candidate
            .overlap_witnesses
            .iter()
            .any(|witness| witness.witness_kind == OverlapWitnessKind::PredicateSet));
        assert!(candidate.validate_report().is_valid());
    }

    #[test]
    fn deterministic_detection_respects_scope() {
        let claim = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:one")))
            .with_normalized_label("same")
            .expect("label");
        let evidence = CorrespondenceSubject::new(ParticipantRef::Evidence(id("evidence:one")))
            .with_normalized_label("same")
            .expect("label");

        let result = derive_correspondence_candidates(
            CorrespondenceDetectionInput::new(
                id("ctx:test"),
                id("provenance:test"),
                vec![claim, evidence],
            )
            .with_scope(CorrespondenceScope::Claims),
        )
        .expect("detection succeeds");

        assert!(result.candidates.is_empty());
    }

    #[test]
    fn deterministic_detection_extracts_phase3_difference_witnesses() {
        let relation =
            TypedRelation::new("OrderService", "accesses", "BillingDB").expect("valid relation");
        let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:observed")))
            .with_role("observed_claim")
            .expect("role")
            .with_normalized_label("order service billing access")
            .expect("label")
            .with_modality("observed")
            .expect("modality")
            .with_contexts(vec![id("ctx:legacy-migration")])
            .with_evidence(vec![id("evidence:scan"), id("evidence:architecture-doc")])
            .with_invariants(vec![id("invariant:no-cross-context-db-access")])
            .with_invariant_states(vec![InvariantState::new(
                id("invariant:no-cross-context-db-access"),
                InvariantSatisfaction::Failed,
            )])
            .with_typed_relations(vec![relation.clone()]);
        let right = CorrespondenceSubject::new(ParticipantRef::Invariant(id(
            "invariant:no-cross-context-db-access",
        )))
        .with_role("constraint")
        .expect("role")
        .with_normalized_label("order service billing access")
        .expect("label")
        .with_modality("forbidden")
        .expect("modality")
        .with_contexts(vec![id("ctx:production-architecture")])
        .with_evidence(vec![id("evidence:architecture-doc")])
        .with_invariants(vec![id("invariant:no-cross-context-db-access")])
        .with_invariant_states(vec![InvariantState::new(
            id("invariant:no-cross-context-db-access"),
            InvariantSatisfaction::Satisfied,
        )])
        .with_typed_relations(vec![relation]);

        let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
            id("ctx:architecture-review"),
            id("provenance:deterministic-overlap"),
            vec![left, right],
        ))
        .expect("detection succeeds");

        let candidate = &result.candidates[0];
        assert!(candidate
            .difference_witnesses
            .iter()
            .any(
                |difference| difference.difference_kind == DifferenceKind::ModalityMismatch
                    && difference.severity == DifferenceSeverity::Blocking
            ));
        assert!(candidate
            .difference_witnesses
            .iter()
            .any(
                |difference| difference.difference_kind == DifferenceKind::EvidenceMismatch
                    && difference.severity == DifferenceSeverity::Minor
            ));
        assert!(candidate
            .difference_witnesses
            .iter()
            .any(
                |difference| difference.difference_kind == DifferenceKind::InvariantMismatch
                    && difference.severity == DifferenceSeverity::Blocking
            ));
        assert!(candidate
            .difference_witnesses
            .iter()
            .any(
                |difference| difference.difference_kind == DifferenceKind::ContextMismatch
                    && difference.severity == DifferenceSeverity::Major
            ));
        assert!(candidate.validate_report().is_valid());
    }

    #[test]
    fn compact_participant_ids_deserialize_by_prefix() {
        let participant: ParticipantRef =
            serde_json::from_str("\"claim:architecture\"").expect("participant ref");

        assert!(matches!(participant, ParticipantRef::Claim(_)));
    }

    #[test]
    fn detection_result_roundtrips() {
        let left = CorrespondenceSubject::new(ParticipantRef::Cell(id("cell:a")))
            .with_normalized_label("same")
            .expect("label");
        let right = CorrespondenceSubject::new(ParticipantRef::Cell(id("cell:b")))
            .with_normalized_label("same")
            .expect("label");
        let result = derive_correspondence_candidates(CorrespondenceDetectionInput::new(
            id("ctx:test"),
            id("provenance:test"),
            vec![left, right],
        ))
        .expect("detection succeeds");

        let value = serde_json::to_value(&result).expect("serialize");
        let roundtrip: CorrespondenceDetectionResult =
            serde_json::from_value(value).expect("deserialize");

        assert_eq!(roundtrip, result);
    }

    #[test]
    fn semantic_signals_generate_candidate_overlap_with_calibrated_confidence() {
        let left = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:doc")))
            .with_role("doc_claim")
            .expect("role")
            .with_modality("observed")
            .expect("modality")
            .with_evidence(vec![id("evidence:doc")]);
        let right = CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:scan")))
            .with_role("scan_claim")
            .expect("role")
            .with_modality("forbidden")
            .expect("modality")
            .with_evidence(vec![id("evidence:scan")]);
        let mut signal = SemanticCorrespondenceSignal::new(
            id("semantic-signal:order-billing-access"),
            vec![
                ParticipantRef::Claim(id("claim:doc")),
                ParticipantRef::Claim(id("claim:scan")),
            ],
            SemanticSignalSource::Llm,
            Confidence::new(0.97).expect("confidence"),
        )
        .expect("semantic signal");
        signal.normalized_claim = Some(NormalizedClaim {
            subject: "OrderService".to_owned(),
            relation: "accesses".to_owned(),
            object: "BillingDB".to_owned(),
            modality: None,
            temporal_scope: None,
        });
        signal.embedding_score = Some(Confidence::new(0.91).expect("confidence"));
        signal.evidence = vec![id("evidence:semantic-adapter")];
        signal.rationale = Some("same normalized access claim".to_owned());

        let mut input = CorrespondenceDetectionInput::new(
            id("ctx:architecture-review"),
            id("provenance:semantic-adapter"),
            vec![left, right],
        );
        input.semantic_signals.push(signal);

        let result = derive_correspondence_candidates(input).expect("detection succeeds");
        let semantic = result
            .candidates
            .iter()
            .find(|candidate| candidate.correspondence_kind == CorrespondenceKind::SemanticOverlap)
            .expect("semantic candidate");

        assert_eq!(semantic.review_status, ReviewStatus::Candidate);
        assert_eq!(semantic.confidence, Confidence::new(0.86).expect("cap"));
        assert!(semantic
            .overlap_witnesses
            .iter()
            .any(|witness| witness.witness_kind == OverlapWitnessKind::NormalizedClaim));
        assert!(semantic
            .difference_witnesses
            .iter()
            .any(|difference| difference.difference_kind == DifferenceKind::ModalityMismatch));
    }

    #[test]
    fn semantic_review_accepts_only_explicit_reviewed_candidate() {
        let mut signal = SemanticCorrespondenceSignal::new(
            id("semantic-signal:accepted"),
            vec![
                ParticipantRef::Claim(id("claim:left")),
                ParticipantRef::Claim(id("claim:right")),
            ],
            SemanticSignalSource::Human,
            Confidence::new(0.88).expect("confidence"),
        )
        .expect("semantic signal");
        signal.normalized_claim = Some(NormalizedClaim {
            subject: "OrderService".to_owned(),
            relation: "accesses".to_owned(),
            object: "BillingDB".to_owned(),
            modality: None,
            temporal_scope: None,
        });
        signal.evidence = vec![id("evidence:review")];
        let mut input = CorrespondenceDetectionInput::new(
            id("ctx:review"),
            id("provenance:semantic"),
            vec![
                CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:left"))),
                CorrespondenceSubject::new(ParticipantRef::Claim(id("claim:right"))),
            ],
        );
        input.semantic_signals.push(signal);
        let candidate = derive_correspondence_candidates(input)
            .expect("detection succeeds")
            .candidates
            .into_iter()
            .next()
            .expect("candidate");

        let reviewed = review_semantic_correspondence(
            &candidate,
            SemanticCorrespondenceReviewRequest::new(
                candidate.id.clone(),
                id("reviewer:human"),
                SemanticReviewDecision::Accept,
                "normalized claim and evidence reviewed",
            )
            .expect("review request"),
        )
        .expect("review succeeds");

        assert_eq!(reviewed.review_status, ReviewStatus::Accepted);
    }
}
