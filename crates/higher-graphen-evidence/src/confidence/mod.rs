//! Bayesian-inspired confidence update records for HigherGraphen.
//!
//! Confidence updates are numerical belief updates only. They do not accept,
//! reject, or silently promote a structural claim; review acceptance remains an
//! explicit [`ReviewStatus`] value owned by the reviewer or workflow.

use higher_graphen_core::{Confidence, CoreError, Id, Result, ReviewStatus};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

const POSTERIOR_TOLERANCE: f64 = 1.0e-12;

/// Conditional likelihoods used by one evidence item.
///
/// Both probabilities must be strictly inside `0.0..1.0` so odds and
/// likelihood ratios stay finite.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceLikelihood {
    /// Probability of observing the evidence if the claim is true.
    pub given_claim: Confidence,
    /// Probability of observing the evidence if the claim is not true.
    pub given_not_claim: Confidence,
}

impl EvidenceLikelihood {
    /// Creates validated conditional likelihoods.
    pub fn new(given_claim: Confidence, given_not_claim: Confidence) -> Result<Self> {
        let likelihood = Self {
            given_claim,
            given_not_claim,
        };
        likelihood.validate()?;
        Ok(likelihood)
    }

    /// Returns the Bayes factor `P(evidence|claim) / P(evidence|not claim)`.
    #[must_use]
    pub fn likelihood_ratio(self) -> f64 {
        self.given_claim.value() / self.given_not_claim.value()
    }

    /// Returns the natural-log likelihood ratio.
    #[must_use]
    pub fn log_likelihood_ratio(self) -> f64 {
        self.likelihood_ratio().ln()
    }

    /// Returns true when this evidence increases posterior odds.
    #[must_use]
    pub fn supports_claim(self) -> bool {
        self.given_claim.value() > self.given_not_claim.value()
    }

    /// Returns true when this evidence decreases posterior odds.
    #[must_use]
    pub fn contradicts_claim(self) -> bool {
        self.given_claim.value() < self.given_not_claim.value()
    }

    /// Validates open-interval probability bounds.
    pub fn validate(&self) -> Result<()> {
        ensure_open_confidence("given_claim", self.given_claim)?;
        ensure_open_confidence("given_not_claim", self.given_not_claim)
    }
}

impl<'de> Deserialize<'de> for EvidenceLikelihood {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            given_claim: Confidence,
            given_not_claim: Confidence,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.given_claim, wire.given_not_claim).map_err(serde::de::Error::custom)
    }
}

/// Evidence item used by a confidence update.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceEvidence {
    /// Stable evidence identifier.
    pub evidence_id: Id,
    /// Human-readable explanation of the evidence.
    pub summary: String,
    /// Conditional likelihoods contributed by this evidence.
    pub likelihood: EvidenceLikelihood,
    /// Optional source structures or observations behind this evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

impl ConfidenceEvidence {
    /// Creates a validated evidence item with no source identifiers.
    pub fn new(
        evidence_id: Id,
        summary: impl Into<String>,
        likelihood: EvidenceLikelihood,
    ) -> Result<Self> {
        let evidence = Self {
            evidence_id,
            summary: required_text("summary", summary)?,
            likelihood,
            source_ids: Vec::new(),
        };
        evidence.validate()?;
        Ok(evidence)
    }

    /// Returns this evidence with deterministic, deduplicated source ids.
    #[must_use]
    pub fn with_source_ids(mut self, source_ids: Vec<Id>) -> Self {
        self.source_ids = normalize_ids(source_ids);
        self
    }

    /// Validates summary text and likelihood bounds.
    pub fn validate(&self) -> Result<()> {
        ensure_non_empty("summary", &self.summary)?;
        self.likelihood.validate()
    }
}

impl<'de> Deserialize<'de> for ConfidenceEvidence {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            evidence_id: Id,
            summary: String,
            likelihood: EvidenceLikelihood,
            #[serde(default)]
            source_ids: Vec<Id>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let evidence = Self {
            evidence_id: wire.evidence_id,
            summary: required_text("summary", wire.summary).map_err(serde::de::Error::custom)?,
            likelihood: wire.likelihood,
            source_ids: normalize_ids(wire.source_ids),
        };
        evidence.validate().map_err(serde::de::Error::custom)?;
        Ok(evidence)
    }
}

/// Input for a deterministic confidence update over one structural claim.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceUpdateInput {
    /// Structural claim being updated.
    pub claim_id: Id,
    /// Prior confidence before applying evidence.
    pub prior: Confidence,
    /// Evidence with likelihood ratio greater than one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence: Vec<ConfidenceEvidence>,
    /// Evidence with likelihood ratio less than one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicting_evidence: Vec<ConfidenceEvidence>,
}

impl ConfidenceUpdateInput {
    /// Creates an update input with no evidence.
    #[must_use]
    pub fn new(claim_id: Id, prior: Confidence) -> Self {
        Self {
            claim_id,
            prior,
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
        }
    }

    /// Returns this input with supporting evidence.
    #[must_use]
    pub fn with_supporting_evidence(
        mut self,
        supporting_evidence: Vec<ConfidenceEvidence>,
    ) -> Self {
        self.supporting_evidence = supporting_evidence;
        self
    }

    /// Returns this input with contradicting evidence.
    #[must_use]
    pub fn with_contradicting_evidence(
        mut self,
        contradicting_evidence: Vec<ConfidenceEvidence>,
    ) -> Self {
        self.contradicting_evidence = contradicting_evidence;
        self
    }

    /// Validates prior bounds, evidence bounds, evidence direction, and duplicate ids.
    pub fn validate(&self) -> Result<()> {
        ensure_open_confidence("prior", self.prior)?;
        ensure_evidence_set(EvidenceExpectation::Supporting, &self.supporting_evidence)?;
        ensure_evidence_set(
            EvidenceExpectation::Contradicting,
            &self.contradicting_evidence,
        )?;
        ensure_unique_evidence_ids(&self.supporting_evidence, &self.contradicting_evidence)
    }
}

impl<'de> Deserialize<'de> for ConfidenceUpdateInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            claim_id: Id,
            prior: Confidence,
            #[serde(default)]
            supporting_evidence: Vec<ConfidenceEvidence>,
            #[serde(default)]
            contradicting_evidence: Vec<ConfidenceEvidence>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let input = Self {
            claim_id: wire.claim_id,
            prior: wire.prior,
            supporting_evidence: wire.supporting_evidence,
            contradicting_evidence: wire.contradicting_evidence,
        };
        input.validate().map_err(serde::de::Error::custom)?;
        Ok(input)
    }
}

/// Auditable confidence update result.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceUpdateRecord {
    /// Structural claim that was updated.
    pub claim_id: Id,
    /// Prior confidence before applying evidence.
    pub prior: Confidence,
    /// Supporting evidence used for the update.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence: Vec<ConfidenceEvidence>,
    /// Contradicting evidence used for the update.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicting_evidence: Vec<ConfidenceEvidence>,
    /// Posterior confidence after applying evidence.
    pub posterior: Confidence,
    /// Explicit review state, separate from the numerical posterior.
    pub review_status: ReviewStatus,
}

impl ConfidenceUpdateRecord {
    /// Creates a record from validated update input.
    pub fn new(input: ConfidenceUpdateInput) -> Result<Self> {
        let posterior = posterior_confidence(&input)?;
        Ok(Self {
            claim_id: input.claim_id,
            prior: input.prior,
            supporting_evidence: input.supporting_evidence,
            contradicting_evidence: input.contradicting_evidence,
            posterior,
            review_status: ReviewStatus::Unreviewed,
        })
    }

    /// Returns a copy with an explicit review status supplied by a reviewer or workflow.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }

    /// Returns true only when the explicit review status accepts the claim.
    #[must_use]
    pub fn is_review_accepted(&self) -> bool {
        self.review_status.is_accepted()
    }

    /// Returns the record as update input without posterior or review state.
    #[must_use]
    pub fn to_input(&self) -> ConfidenceUpdateInput {
        ConfidenceUpdateInput {
            claim_id: self.claim_id.clone(),
            prior: self.prior,
            supporting_evidence: self.supporting_evidence.clone(),
            contradicting_evidence: self.contradicting_evidence.clone(),
        }
    }

    /// Validates the embedded input and confirms the stored posterior matches it.
    pub fn validate(&self) -> Result<()> {
        let input = self.to_input();
        let expected = posterior_confidence(&input)?;

        if !approximately_equal(expected.value(), self.posterior.value()) {
            return Err(malformed_field(
                "posterior",
                format!(
                    "posterior {} does not match recomputed posterior {}",
                    self.posterior, expected
                ),
            ));
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for ConfidenceUpdateRecord {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            claim_id: Id,
            prior: Confidence,
            #[serde(default)]
            supporting_evidence: Vec<ConfidenceEvidence>,
            #[serde(default)]
            contradicting_evidence: Vec<ConfidenceEvidence>,
            posterior: Confidence,
            review_status: ReviewStatus,
        }

        let wire = Wire::deserialize(deserializer)?;
        let record = Self {
            claim_id: wire.claim_id,
            prior: wire.prior,
            supporting_evidence: wire.supporting_evidence,
            contradicting_evidence: wire.contradicting_evidence,
            posterior: wire.posterior,
            review_status: wire.review_status,
        };
        record.validate().map_err(serde::de::Error::custom)?;
        Ok(record)
    }
}

/// Stateless confidence update engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct BayesianConfidenceEngine;

impl BayesianConfidenceEngine {
    /// Applies a deterministic Bayesian-inspired update.
    pub fn update(&self, input: ConfidenceUpdateInput) -> Result<ConfidenceUpdateRecord> {
        update_confidence(input)
    }
}

/// Applies a deterministic Bayesian-inspired confidence update.
pub fn update_confidence(input: ConfidenceUpdateInput) -> Result<ConfidenceUpdateRecord> {
    ConfidenceUpdateRecord::new(input)
}

fn posterior_confidence(input: &ConfidenceUpdateInput) -> Result<Confidence> {
    input.validate()?;

    let mut log_odds = logit(input.prior);

    for evidence in &input.supporting_evidence {
        log_odds += evidence.likelihood.log_likelihood_ratio();
    }

    for evidence in &input.contradicting_evidence {
        log_odds += evidence.likelihood.log_likelihood_ratio();
    }

    confidence_from_log_odds(log_odds)
}

fn logit(confidence: Confidence) -> f64 {
    let value = confidence.value();
    (value / (1.0 - value)).ln()
}

fn confidence_from_log_odds(log_odds: f64) -> Result<Confidence> {
    let value = if log_odds >= 0.0 {
        1.0 / (1.0 + (-log_odds).exp())
    } else {
        let exp_log_odds = log_odds.exp();
        exp_log_odds / (1.0 + exp_log_odds)
    };

    Confidence::new(value)
}

#[derive(Clone, Copy, Debug)]
enum EvidenceExpectation {
    Supporting,
    Contradicting,
}

impl EvidenceExpectation {
    fn field(self) -> &'static str {
        match self {
            Self::Supporting => "supporting_evidence",
            Self::Contradicting => "contradicting_evidence",
        }
    }

    fn matches(self, likelihood: EvidenceLikelihood) -> bool {
        match self {
            Self::Supporting => likelihood.supports_claim(),
            Self::Contradicting => likelihood.contradicts_claim(),
        }
    }

    fn violation_message(self) -> &'static str {
        match self {
            Self::Supporting => {
                "supporting evidence must have P(evidence|claim) > P(evidence|not claim)"
            }
            Self::Contradicting => {
                "contradicting evidence must have P(evidence|claim) < P(evidence|not claim)"
            }
        }
    }
}

fn ensure_evidence_set(
    expectation: EvidenceExpectation,
    evidence: &[ConfidenceEvidence],
) -> Result<()> {
    for item in evidence {
        item.validate()?;

        if !expectation.matches(item.likelihood) {
            return Err(malformed_field(
                expectation.field(),
                expectation.violation_message(),
            ));
        }
    }

    Ok(())
}

fn ensure_unique_evidence_ids(
    supporting_evidence: &[ConfidenceEvidence],
    contradicting_evidence: &[ConfidenceEvidence],
) -> Result<()> {
    let mut evidence_ids = BTreeSet::new();

    for evidence in supporting_evidence
        .iter()
        .chain(contradicting_evidence.iter())
    {
        if !evidence_ids.insert(&evidence.evidence_id) {
            return Err(malformed_field(
                "evidence_id",
                format!("duplicate evidence id {}", evidence.evidence_id),
            ));
        }
    }

    Ok(())
}

fn ensure_open_confidence(field: &'static str, confidence: Confidence) -> Result<()> {
    let value = confidence.value();
    if value <= Confidence::MIN || value >= Confidence::MAX {
        return Err(malformed_field(
            field,
            "probability must be strictly between 0.0 and 1.0",
        ));
    }

    Ok(())
}

fn ensure_non_empty(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(malformed_field(
            field,
            "field must not be empty after trimming",
        ));
    }

    Ok(())
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    ensure_non_empty(field, &normalized)?;
    Ok(normalized)
}

fn normalize_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= POSTERIOR_TOLERANCE
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
