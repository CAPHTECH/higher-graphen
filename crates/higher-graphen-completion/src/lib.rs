//! Completion candidates, rules, engine, and review workflow for HigherGraphen.

use higher_graphen_core::{Confidence, CoreError, Id, Result, ReviewStatus};
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    if candidates
        .iter()
        .any(|candidate| candidate.review_status != ReviewStatus::Unreviewed)
    {
        return Err(malformed_field(
            "candidates.review_status",
            "detected completion candidates must remain unreviewed",
        ));
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
mod tests {
    use super::*;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn confidence(value: f64) -> Confidence {
        Confidence::new(value).expect("valid confidence")
    }

    fn candidate() -> CompletionCandidate {
        let suggestion = SuggestedStructure::new("cell", "Add a missing API contract cell")
            .expect("valid suggestion")
            .with_structure_id(id("cell.contract"))
            .with_related_ids(vec![id("cell.api")]);

        CompletionCandidate::new(
            id("candidate.contract"),
            id("space.architecture"),
            MissingType::Cell,
            suggestion,
            vec![id("cell.api")],
            "The API has behavior but no contract cell.",
            confidence(0.82),
        )
        .expect("valid candidate")
    }

    fn rule() -> CompletionRule {
        let suggestion = SuggestedStructure::new("cell", "Add a missing API contract cell")
            .expect("valid suggestion")
            .with_structure_id(id("cell.contract"))
            .with_related_ids(vec![id("cell.api")]);

        CompletionRule::new(
            id("rule.contract"),
            id("candidate.contract"),
            MissingType::Cell,
            suggestion,
            "The API has behavior but no contract cell.",
            confidence(0.82),
        )
        .expect("valid rule")
        .with_context_ids(vec![id("context.api")])
        .with_inferred_from(vec![id("cell.api")])
    }

    #[test]
    fn new_candidate_defaults_to_unreviewed() {
        let candidate = candidate();

        assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(
            candidate.rationale,
            "The API has behavior but no contract cell."
        );
    }

    #[test]
    fn detection_materializes_matching_rules_as_unreviewed_candidates() {
        let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
            .with_context_ids(vec![id("context.api"), id("context.review")]);

        let result =
            detect_completion_candidates(input).expect("completion detection should succeed");

        assert_eq!(result.candidates.len(), 1);
        let candidate = &result.candidates[0];
        assert_eq!(candidate.id, id("candidate.contract"));
        assert_eq!(candidate.space_id, id("space.architecture"));
        assert_eq!(candidate.inferred_from, vec![id("cell.api")]);
        assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    }

    #[test]
    fn detection_skips_rules_for_missing_contexts() {
        let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
            .with_context_ids(vec![id("context.other")]);

        let result =
            detect_completion_candidates(input).expect("completion detection should succeed");

        assert!(result.candidates.is_empty());
    }

    #[test]
    fn detection_result_rejects_reviewed_candidates() {
        let mut candidate = candidate();
        candidate.review_status = ReviewStatus::Accepted;

        let error =
            CompletionDetectionResult::new(id("space.architecture"), Vec::new(), vec![candidate])
                .expect_err("reviewed candidates should be rejected");

        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn simple_engine_wraps_detection_and_review_helpers() {
        let engine = SimpleCompletionEngine;
        let input = CompletionDetectionInput::new(id("space.architecture"), vec![rule()])
            .with_context_ids(vec![id("context.api")]);

        let result = engine
            .detect_candidates(input)
            .expect("completion detection should succeed");
        let accepted = engine
            .accept_candidate(&result.candidates[0], id("reviewer.architect"), "Reviewed")
            .expect("accepted completion");

        assert_eq!(result.candidates[0].review_status, ReviewStatus::Unreviewed);
        assert_eq!(accepted.review_status, ReviewStatus::Accepted);
    }

    #[test]
    fn review_request_records_explicit_acceptance_without_mutating_candidate() {
        let candidate = candidate();
        let request = CompletionReviewRequest::accepted(
            candidate.id.clone(),
            id("reviewer.architect"),
            "Reviewed plan",
        )
        .expect("valid request")
        .with_reviewed_at("2026-04-25T00:00:00Z")
        .expect("valid review time");

        let record = review_completion(&candidate, request).expect("review record");

        assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(record.decision(), CompletionReviewDecision::Accepted);
        assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(record.outcome_review_status, ReviewStatus::Accepted);
        assert_eq!(record.request.reason, "Reviewed plan");
        assert_eq!(
            record.request.reviewed_at.as_deref(),
            Some("2026-04-25T00:00:00Z")
        );
        let accepted = record
            .accepted_completion
            .expect("accepted completion payload");
        assert_eq!(accepted.review_status, ReviewStatus::Accepted);
        assert_eq!(
            accepted.accepted_structure.structure_id,
            Some(id("cell.contract"))
        );
        assert!(record.rejected_completion.is_none());
    }

    #[test]
    fn review_request_records_explicit_rejection() {
        let candidate = candidate();
        let request = CompletionReviewRequest::rejected(
            candidate.id.clone(),
            id("reviewer.architect"),
            "Duplicate of an existing contract",
        )
        .expect("valid request");

        let record = SimpleCompletionEngine
            .review_candidate(&candidate, request)
            .expect("review record");

        assert_eq!(record.decision(), CompletionReviewDecision::Rejected);
        assert_eq!(record.candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(record.outcome_review_status, ReviewStatus::Rejected);
        let rejected = record
            .rejected_completion
            .expect("rejected completion payload");
        assert_eq!(rejected.review_status, ReviewStatus::Rejected);
        assert_eq!(
            rejected.rejected_structure.structure_id,
            Some(id("cell.contract"))
        );
        assert!(record.accepted_completion.is_none());
    }

    #[test]
    fn review_request_rejects_candidate_mismatch_and_empty_metadata() {
        let candidate = candidate();
        let mismatch = CompletionReviewRequest::accepted(
            id("candidate.other"),
            id("reviewer.architect"),
            "Reviewed",
        )
        .expect("valid request");

        let error = review_completion(&candidate, mismatch).expect_err("candidate mismatch");

        assert_eq!(error.code(), "malformed_field");
        assert!(
            CompletionReviewRequest::accepted(candidate.id.clone(), id("reviewer"), "  ").is_err()
        );
        assert!(
            CompletionReviewRequest::rejected(candidate.id, id("reviewer"), "Rejected")
                .expect("valid request")
                .with_reviewed_at(" ")
                .is_err()
        );
    }

    #[test]
    fn accept_returns_separate_accepted_completion() {
        let candidate = candidate();

        let accepted = accept_completion(&candidate, id("reviewer.architect"), "Reviewed plan")
            .expect("accepted completion");

        assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(accepted.candidate_id, candidate.id);
        assert_eq!(accepted.review_status, ReviewStatus::Accepted);
        assert_eq!(accepted.reason, "Reviewed plan");
        assert_eq!(
            accepted.accepted_structure.structure_id,
            Some(id("cell.contract"))
        );
    }

    #[test]
    fn reject_returns_rejection_record() {
        let candidate = candidate();

        let rejected = candidate
            .reject(
                id("reviewer.architect"),
                "Duplicate of an existing contract",
            )
            .expect("rejected completion");

        assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
        assert_eq!(rejected.candidate_id, candidate.id);
        assert_eq!(rejected.review_status, ReviewStatus::Rejected);
        assert_eq!(rejected.reason, "Duplicate of an existing contract");
    }

    #[test]
    fn review_helpers_require_non_empty_reason() {
        let candidate = candidate();

        let error = candidate
            .accept(id("reviewer.architect"), "   ")
            .expect_err("empty reason rejected");

        assert_eq!(error.code(), "malformed_field");
    }

    #[test]
    fn accepted_and_rejected_statuses_cannot_be_cross_reviewed() {
        let mut rejected_candidate = candidate();
        rejected_candidate.review_status = ReviewStatus::Rejected;
        let accept_error = rejected_candidate
            .accept(id("reviewer.architect"), "Reconsidered")
            .expect_err("cannot accept rejected candidate");
        assert_eq!(accept_error.code(), "malformed_field");

        let mut accepted_candidate = candidate();
        accepted_candidate.review_status = ReviewStatus::Accepted;
        let reject_error = accepted_candidate
            .reject(id("reviewer.architect"), "Too late")
            .expect_err("cannot reject accepted candidate");
        assert_eq!(reject_error.code(), "malformed_field");
    }

    #[test]
    fn constructors_trim_and_validate_required_text() {
        let suggestion = SuggestedStructure::new("  invariant  ", "  Add a stability invariant  ")
            .expect("valid suggestion");

        assert_eq!(suggestion.structure_type, "invariant");
        assert_eq!(suggestion.summary, "Add a stability invariant");
        assert!(SuggestedStructure::new(" ", "summary").is_err());
    }
}
