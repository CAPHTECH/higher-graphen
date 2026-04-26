//! Completion candidates, rules, engine, and review workflow for HigherGraphen.

use higher_graphen_core::{Confidence, CoreError, Id, Result, ReviewStatus};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

/// Kind of missing structure a completion candidate proposes to fill.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingType {
    /// A missing cell in a space or complex.
    Cell,
    /// A missing incidence relation between cells.
    Incidence,
    /// A missing morphism between structures.
    Morphism,
    /// A missing constraint that should be checked.
    Constraint,
    /// A missing invariant that should hold across structures.
    Invariant,
    /// A missing section in a contextual structure.
    Section,
    /// A missing projection for a target audience or purpose.
    Projection,
    /// A missing context boundary or grouping.
    Context,
}

/// Minimal portable payload for a proposed missing structure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedStructure {
    /// Optional identifier for the structure that would be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure_id: Option<Id>,
    /// Stable type name meaningful to the crate or product using completion.
    pub structure_type: String,
    /// Human-readable summary of the proposed structure.
    pub summary: String,
    /// Existing structures that the suggested structure directly relates to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_ids: Vec<Id>,
}

impl SuggestedStructure {
    /// Creates a suggested structure with validated type and summary text.
    pub fn new(structure_type: impl Into<String>, summary: impl Into<String>) -> Result<Self> {
        Ok(Self {
            structure_id: None,
            structure_type: required_text("structure_type", structure_type)?,
            summary: required_text("summary", summary)?,
            related_ids: Vec::new(),
        })
    }

    /// Returns this suggestion with an explicit created-structure identifier.
    #[must_use]
    pub fn with_structure_id(mut self, structure_id: Id) -> Self {
        self.structure_id = Some(structure_id);
        self
    }

    /// Returns this suggestion with related source or target structure IDs.
    #[must_use]
    pub fn with_related_ids(mut self, related_ids: Vec<Id>) -> Self {
        self.related_ids = related_ids;
        self
    }
}

/// Explicit rule proposal used by a completion detector.
///
/// The MVP rule surface carries a concrete candidate proposal rather than a
/// domain-specific algorithm. Engines can decide whether the rule applies, then
/// materialize it as an unreviewed [`CompletionCandidate`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionRule {
    /// Rule identifier.
    pub id: Id,
    /// Candidate identifier to assign when this rule produces a proposal.
    pub candidate_id: Id,
    /// Kind of missing structure proposed by the rule.
    pub missing_type: MissingType,
    /// Proposed structure payload.
    pub suggested_structure: SuggestedStructure,
    /// Context identifiers that must be present for the rule to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Source structures used to infer the candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_from: Vec<Id>,
    /// Explanation for why the structure is proposed.
    pub rationale: String,
    /// Confidence in the completion inference.
    pub confidence: Confidence,
}

impl CompletionRule {
    /// Creates an explicit completion rule proposal.
    pub fn new(
        id: Id,
        candidate_id: Id,
        missing_type: MissingType,
        suggested_structure: SuggestedStructure,
        rationale: impl Into<String>,
        confidence: Confidence,
    ) -> Result<Self> {
        Ok(Self {
            id,
            candidate_id,
            missing_type,
            suggested_structure,
            context_ids: Vec::new(),
            inferred_from: Vec::new(),
            rationale: required_text("rationale", rationale)?,
            confidence,
        })
    }

    /// Returns this rule with the context IDs required for applicability.
    #[must_use]
    pub fn with_context_ids(mut self, context_ids: Vec<Id>) -> Self {
        self.context_ids = context_ids;
        self
    }

    /// Returns this rule with source IDs used to infer the candidate.
    #[must_use]
    pub fn with_inferred_from(mut self, inferred_from: Vec<Id>) -> Self {
        self.inferred_from = inferred_from;
        self
    }

    fn applies_to(&self, input_context_ids: &[Id]) -> bool {
        self.context_ids.is_empty()
            || self
                .context_ids
                .iter()
                .all(|context_id| input_context_ids.contains(context_id))
    }

    fn to_candidate(&self, space_id: &Id) -> Result<CompletionCandidate> {
        CompletionCandidate::new(
            self.candidate_id.clone(),
            space_id.clone(),
            self.missing_type,
            self.suggested_structure.clone(),
            self.inferred_from.clone(),
            self.rationale.clone(),
            self.confidence,
        )
    }
}

/// Input for candidate detection against explicit completion rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionDetectionInput {
    /// Space in which missing structure is being detected.
    pub space_id: Id,
    /// Context identifiers available to rule matching.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Explicit rule proposals to evaluate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<CompletionRule>,
}

impl CompletionDetectionInput {
    /// Creates detection input for a space and explicit rule set.
    pub fn new(space_id: Id, rules: Vec<CompletionRule>) -> Self {
        Self {
            space_id,
            context_ids: Vec::new(),
            rules,
        }
    }

    /// Returns this input with context IDs used by rule matching.
    #[must_use]
    pub fn with_context_ids(mut self, context_ids: Vec<Id>) -> Self {
        self.context_ids = context_ids;
        self
    }
}

/// Candidate detection output.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionDetectionResult {
    /// Space in which missing structure was detected.
    pub space_id: Id,
    /// Context identifiers used during rule matching.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Reviewable candidates produced by matching rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CompletionCandidate>,
}

impl CompletionDetectionResult {
    /// Creates a detection result and verifies candidates are still unreviewed.
    pub fn new(
        space_id: Id,
        context_ids: Vec<Id>,
        candidates: Vec<CompletionCandidate>,
    ) -> Result<Self> {
        ensure_unreviewed_candidates(&candidates)?;

        Ok(Self {
            space_id,
            context_ids,
            candidates,
        })
    }
}

impl<'de> Deserialize<'de> for CompletionDetectionResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            space_id: Id,
            #[serde(default)]
            context_ids: Vec<Id>,
            #[serde(default)]
            candidates: Vec<CompletionCandidate>,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.space_id, wire.context_ids, wire.candidates)
            .map_err(serde::de::Error::custom)
    }
}

/// Explicit accept/reject decision for a completion candidate.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionReviewDecision {
    /// Accept the proposed structure for downstream creation or promotion.
    Accepted,
    /// Reject the proposed structure and keep it out of accepted facts.
    Rejected,
}

impl CompletionReviewDecision {
    /// Returns the review status represented by this explicit decision.
    #[must_use]
    pub fn review_status(self) -> ReviewStatus {
        match self {
            Self::Accepted => ReviewStatus::Accepted,
            Self::Rejected => ReviewStatus::Rejected,
        }
    }
}

/// Reviewer-supplied request to explicitly accept or reject one candidate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewRequest {
    /// Candidate identifier the request applies to.
    pub candidate_id: Id,
    /// Explicit accept/reject decision.
    pub decision: CompletionReviewDecision,
    /// Human or workflow reviewer identifier.
    pub reviewer_id: Id,
    /// Reviewer-supplied rationale for the decision.
    pub reason: String,
    /// Optional externally supplied review time, such as RFC 3339 text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
}

impl CompletionReviewRequest {
    /// Creates a validated explicit completion review request.
    pub fn new(
        candidate_id: Id,
        decision: CompletionReviewDecision,
        reviewer_id: Id,
        reason: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            candidate_id,
            decision,
            reviewer_id,
            reason: required_text("reason", reason)?,
            reviewed_at: None,
        })
    }

    /// Creates a validated explicit acceptance request.
    pub fn accepted(candidate_id: Id, reviewer_id: Id, reason: impl Into<String>) -> Result<Self> {
        Self::new(
            candidate_id,
            CompletionReviewDecision::Accepted,
            reviewer_id,
            reason,
        )
    }

    /// Creates a validated explicit rejection request.
    pub fn rejected(candidate_id: Id, reviewer_id: Id, reason: impl Into<String>) -> Result<Self> {
        Self::new(
            candidate_id,
            CompletionReviewDecision::Rejected,
            reviewer_id,
            reason,
        )
    }

    /// Returns this request with externally supplied review time metadata.
    pub fn with_reviewed_at(mut self, reviewed_at: impl Into<String>) -> Result<Self> {
        self.reviewed_at = Some(required_text("reviewed_at", reviewed_at)?);
        Ok(self)
    }
}

/// Auditable result of an explicit completion review action.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReviewRecord {
    /// Original review request supplied by the reviewer or workflow.
    pub request: CompletionReviewRequest,
    /// Source candidate snapshot preserved before any downstream action.
    pub candidate: CompletionCandidate,
    /// Resulting review status from the explicit decision.
    pub outcome_review_status: ReviewStatus,
    /// Accepted completion payload when the decision is accepted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_completion: Option<AcceptedCompletion>,
    /// Rejected completion payload when the decision is rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_completion: Option<RejectedCompletion>,
}

impl CompletionReviewRecord {
    /// Returns the explicit decision represented by this record.
    #[must_use]
    pub fn decision(&self) -> CompletionReviewDecision {
        self.request.decision
    }
}

/// Stateless MVP completion engine for explicit in-memory rule proposals.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimpleCompletionEngine;

impl SimpleCompletionEngine {
    /// Detects missing structure by materializing matching explicit rules.
    pub fn detect_candidates(
        &self,
        input: CompletionDetectionInput,
    ) -> Result<CompletionDetectionResult> {
        detect_completion_candidates(input)
    }

    /// Accepts a completion candidate through the existing review helper.
    pub fn accept_candidate(
        &self,
        candidate: &CompletionCandidate,
        reviewer_id: Id,
        reason: impl Into<String>,
    ) -> Result<AcceptedCompletion> {
        accept_completion(candidate, reviewer_id, reason)
    }

    /// Rejects a completion candidate through the existing review helper.
    pub fn reject_candidate(
        &self,
        candidate: &CompletionCandidate,
        reviewer_id: Id,
        reason: impl Into<String>,
    ) -> Result<RejectedCompletion> {
        reject_completion(candidate, reviewer_id, reason)
    }

    /// Reviews a completion candidate through the existing review helper.
    pub fn review_candidate(
        &self,
        candidate: &CompletionCandidate,
        request: CompletionReviewRequest,
    ) -> Result<CompletionReviewRecord> {
        review_completion(candidate, request)
    }
}

/// Detects reviewable completion candidates from explicit rule proposals.
pub fn detect_completion_candidates(
    input: CompletionDetectionInput,
) -> Result<CompletionDetectionResult> {
    let CompletionDetectionInput {
        space_id,
        context_ids,
        rules,
    } = input;

    let candidates = rules
        .iter()
        .filter(|rule| rule.applies_to(&context_ids))
        .map(|rule| rule.to_candidate(&space_id))
        .collect::<Result<Vec<_>>>()?;

    CompletionDetectionResult::new(space_id, context_ids, candidates)
}

/// Reviewable proposal for missing HigherGraphen structure.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionCandidate {
    /// Candidate identifier.
    pub id: Id,
    /// Space in which the missing structure was detected.
    pub space_id: Id,
    /// Kind of missing structure.
    pub missing_type: MissingType,
    /// Proposed structure payload.
    pub suggested_structure: SuggestedStructure,
    /// Source structures used to infer the candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_from: Vec<Id>,
    /// Explanation for why the structure is proposed.
    pub rationale: String,
    /// Confidence in the completion inference.
    pub confidence: Confidence,
    /// Review state of the candidate.
    pub review_status: ReviewStatus,
}

impl CompletionCandidate {
    /// Creates an unreviewed completion candidate.
    pub fn new(
        id: Id,
        space_id: Id,
        missing_type: MissingType,
        suggested_structure: SuggestedStructure,
        inferred_from: Vec<Id>,
        rationale: impl Into<String>,
        confidence: Confidence,
    ) -> Result<Self> {
        Ok(Self {
            id,
            space_id,
            missing_type,
            suggested_structure,
            inferred_from,
            rationale: required_text("rationale", rationale)?,
            confidence,
            review_status: ReviewStatus::Unreviewed,
        })
    }

    /// Accepts this candidate through an explicit reviewer action.
    pub fn accept(&self, reviewer_id: Id, reason: impl Into<String>) -> Result<AcceptedCompletion> {
        accept_completion(self, reviewer_id, reason)
    }

    /// Rejects this candidate through an explicit reviewer action.
    pub fn reject(&self, reviewer_id: Id, reason: impl Into<String>) -> Result<RejectedCompletion> {
        reject_completion(self, reviewer_id, reason)
    }
}

/// Separate accepted result created from an explicit completion review.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedCompletion {
    /// Candidate that was accepted.
    pub candidate_id: Id,
    /// Space in which the accepted structure belongs.
    pub space_id: Id,
    /// Kind of missing structure that was accepted.
    pub missing_type: MissingType,
    /// Structure accepted for downstream creation or promotion.
    pub accepted_structure: SuggestedStructure,
    /// Source structures used to infer the accepted candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_from: Vec<Id>,
    /// Candidate rationale captured at the time of acceptance.
    pub rationale: String,
    /// Confidence captured at the time of acceptance.
    pub confidence: Confidence,
    /// Reviewer who explicitly accepted the candidate.
    pub reviewer_id: Id,
    /// Reviewer-supplied acceptance reason.
    pub reason: String,
    /// Review status assigned to the accepted result.
    pub review_status: ReviewStatus,
}

/// Rejection record created from an explicit completion review.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedCompletion {
    /// Candidate that was rejected.
    pub candidate_id: Id,
    /// Space in which the rejected structure had been proposed.
    pub space_id: Id,
    /// Kind of missing structure that was rejected.
    pub missing_type: MissingType,
    /// Structure that was rejected.
    pub rejected_structure: SuggestedStructure,
    /// Source structures used to infer the rejected candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inferred_from: Vec<Id>,
    /// Candidate rationale captured at the time of rejection.
    pub rationale: String,
    /// Confidence captured at the time of rejection.
    pub confidence: Confidence,
    /// Reviewer who explicitly rejected the candidate.
    pub reviewer_id: Id,
    /// Reviewer-supplied rejection reason.
    pub reason: String,
    /// Review status assigned to the rejection record.
    pub review_status: ReviewStatus,
}

/// Accepts a completion candidate and returns a separate accepted result.
///
/// This helper never mutates or silently promotes the source candidate. The
/// caller must provide a validated reviewer identifier and a non-empty reason.
pub fn accept_completion(
    candidate: &CompletionCandidate,
    reviewer_id: Id,
    reason: impl Into<String>,
) -> Result<AcceptedCompletion> {
    if candidate.review_status.is_rejected() {
        return Err(malformed_field(
            "review_status",
            "rejected completion candidates cannot be accepted",
        ));
    }

    Ok(AcceptedCompletion {
        candidate_id: candidate.id.clone(),
        space_id: candidate.space_id.clone(),
        missing_type: candidate.missing_type,
        accepted_structure: candidate.suggested_structure.clone(),
        inferred_from: candidate.inferred_from.clone(),
        rationale: candidate.rationale.clone(),
        confidence: candidate.confidence,
        reviewer_id,
        reason: required_text("reason", reason)?,
        review_status: ReviewStatus::Accepted,
    })
}

/// Rejects a completion candidate and returns a separate rejection record.
///
/// This helper never mutates the source candidate. The caller must provide a
/// validated reviewer identifier and a non-empty reason.
pub fn reject_completion(
    candidate: &CompletionCandidate,
    reviewer_id: Id,
    reason: impl Into<String>,
) -> Result<RejectedCompletion> {
    if candidate.review_status.is_accepted() {
        return Err(malformed_field(
            "review_status",
            "accepted completion candidates cannot be rejected",
        ));
    }

    Ok(RejectedCompletion {
        candidate_id: candidate.id.clone(),
        space_id: candidate.space_id.clone(),
        missing_type: candidate.missing_type,
        rejected_structure: candidate.suggested_structure.clone(),
        inferred_from: candidate.inferred_from.clone(),
        rationale: candidate.rationale.clone(),
        confidence: candidate.confidence,
        reviewer_id,
        reason: required_text("reason", reason)?,
        review_status: ReviewStatus::Rejected,
    })
}

/// Reviews a completion candidate with an explicit accept/reject request.
///
/// The source candidate is cloned into the returned audit record and is never
/// mutated or silently promoted by this helper.
pub fn review_completion(
    candidate: &CompletionCandidate,
    request: CompletionReviewRequest,
) -> Result<CompletionReviewRecord> {
    if request.candidate_id != candidate.id {
        return Err(malformed_field(
            "candidate_id",
            format!(
                "review request targets {}, but candidate snapshot is {}",
                request.candidate_id, candidate.id
            ),
        ));
    }

    let outcome_review_status = request.decision.review_status();
    let (accepted_completion, rejected_completion) = match request.decision {
        CompletionReviewDecision::Accepted => (
            Some(accept_completion(
                candidate,
                request.reviewer_id.clone(),
                request.reason.clone(),
            )?),
            None,
        ),
        CompletionReviewDecision::Rejected => (
            None,
            Some(reject_completion(
                candidate,
                request.reviewer_id.clone(),
                request.reason.clone(),
            )?),
        ),
    };

    Ok(CompletionReviewRecord {
        request,
        candidate: candidate.clone(),
        outcome_review_status,
        accepted_completion,
        rejected_completion,
    })
}

fn ensure_unreviewed_candidates(candidates: &[CompletionCandidate]) -> Result<()> {
    let mut candidate_ids = BTreeSet::new();

    for candidate in candidates {
        if candidate.review_status != ReviewStatus::Unreviewed {
            return Err(malformed_field(
                "candidates.review_status",
                "detected completion candidates must remain unreviewed",
            ));
        }

        if !candidate_ids.insert(&candidate.id) {
            return Err(malformed_field(
                "candidates.id",
                format!("duplicate completion candidate id {}", candidate.id),
            ));
        }
    }

    Ok(())
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(malformed_field(
            field,
            "field must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
