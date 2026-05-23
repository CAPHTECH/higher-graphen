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

mod detection;

pub use detection::derive_correspondence_candidates;

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

fn unique_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
mod tests;
