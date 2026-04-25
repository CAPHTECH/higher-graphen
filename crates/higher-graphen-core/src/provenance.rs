use crate::{Confidence, ReviewStatus, SourceRef};
use serde::{Deserialize, Serialize};

/// Source, extraction, confidence, and review metadata for a structure.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Source reference for the observed or inferred structure.
    pub source: SourceRef,
    /// Confidence in extraction or inference.
    pub confidence: Confidence,
    /// Human or workflow review state.
    pub review_status: ReviewStatus,
    /// Optional extraction method name or description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_method: Option<String>,
    /// Optional extractor identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extractor_id: Option<String>,
    /// Optional reviewer identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<String>,
    /// Optional stable text review time, such as RFC 3339.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    /// Optional review or extraction notes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Provenance {
    /// Creates provenance with an explicit source and confidence.
    pub fn new(source: SourceRef, confidence: Confidence) -> Self {
        Self {
            source,
            confidence,
            review_status: ReviewStatus::default(),
            extraction_method: None,
            extractor_id: None,
            reviewer_id: None,
            reviewed_at: None,
            notes: None,
        }
    }

    /// Returns this provenance with a supplied review status.
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }
}
