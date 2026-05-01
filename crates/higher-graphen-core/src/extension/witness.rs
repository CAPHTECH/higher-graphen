use super::common::ObjectRef;
use crate::text::normalize_required_text;
use crate::{Confidence, CoreError, Id, Provenance, Result};
use serde::{Deserialize, Serialize};

/// Witness payload category.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessType {
    /// Direct observation.
    Observation,
    /// Log entry.
    LogEntry,
    /// Metric point.
    MetricPoint,
    /// Test result.
    TestResult,
    /// Code location.
    CodeLocation,
    /// Document excerpt.
    DocumentExcerpt,
    /// Counterexample.
    Counterexample,
    /// Human review record.
    HumanReview,
    /// Machine check result.
    MachineCheckResult,
    /// External reference.
    ExternalReference,
}

/// Payload backing a witness.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadRef {
    /// Payload kind, such as `file`, `uri`, or `artifact`.
    pub kind: String,
    /// Stable URI for the payload.
    pub uri: String,
}

impl PayloadRef {
    /// Creates a validated payload reference.
    pub fn new(kind: impl Into<String>, uri: impl Into<String>) -> Result<Self> {
        Ok(Self {
            kind: normalize_required_text("payload_ref.kind", kind)?,
            uri: normalize_required_text("payload_ref.uri", uri)?,
        })
    }
}

/// Review status for a witness.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessStatus {
    /// Candidate witness, not accepted as support.
    Candidate,
    /// Accepted witness.
    Accepted,
    /// Rejected witness.
    Rejected,
    /// Deprecated witness retained for audit.
    Deprecated,
}

/// Observable support or counterexample for a structural judgment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    /// Witness identifier.
    pub id: Id,
    /// Witness category.
    pub witness_type: WitnessType,
    /// Supported objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supports: Vec<ObjectRef>,
    /// Contradicted objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicts: Vec<ObjectRef>,
    /// Payload backing the witness.
    pub payload_ref: PayloadRef,
    /// Contexts in which this witness is valid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validity_contexts: Vec<Id>,
    /// Stable observed timestamp string, such as RFC 3339.
    pub observed_at: String,
    /// Witness provenance.
    pub provenance: Provenance,
    /// Confidence in the witness.
    pub confidence: Confidence,
    /// Witness review status.
    pub review_status: WitnessStatus,
}

impl Witness {
    /// Creates a candidate witness.
    pub fn candidate(
        id: Id,
        witness_type: WitnessType,
        payload_ref: PayloadRef,
        observed_at: impl Into<String>,
        provenance: Provenance,
        confidence: Confidence,
    ) -> Result<Self> {
        Ok(Self {
            id,
            witness_type,
            supports: Vec::new(),
            contradicts: Vec::new(),
            payload_ref,
            validity_contexts: Vec::new(),
            observed_at: normalize_required_text("observed_at", observed_at)?,
            provenance,
            confidence,
            review_status: WitnessStatus::Candidate,
        })
    }

    /// Validates conditions required before using this witness as accepted support.
    pub fn validate_acceptance(&self) -> Result<()> {
        if self.validity_contexts.is_empty() {
            return Err(CoreError::malformed_field(
                "validity_contexts",
                "accepted witness requires explicit validity context",
            ));
        }
        if matches!(self.review_status, WitnessStatus::Rejected) {
            return Err(CoreError::malformed_field(
                "review_status",
                "rejected witness cannot be accepted support",
            ));
        }
        normalize_required_text("payload_ref.kind", &self.payload_ref.kind)?;
        normalize_required_text("payload_ref.uri", &self.payload_ref.uri)?;
        normalize_required_text("observed_at", &self.observed_at)?;
        Ok(())
    }
}
