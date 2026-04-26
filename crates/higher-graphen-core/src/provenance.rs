use crate::text::{normalize_optional_text, normalize_optional_text_ref};
use crate::Result;
use crate::{Confidence, ReviewStatus, SourceRef};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Source, extraction, confidence, and review metadata for a structure.
#[derive(Clone, Debug, PartialEq)]
pub struct Provenance {
    /// Source reference for the observed or inferred structure.
    pub source: SourceRef,
    /// Confidence in extraction or inference.
    pub confidence: Confidence,
    /// Human or workflow review state.
    pub review_status: ReviewStatus,
    /// Optional extraction method name or description.
    pub extraction_method: Option<String>,
    /// Optional extractor identity.
    pub extractor_id: Option<String>,
    /// Optional reviewer identity.
    pub reviewer_id: Option<String>,
    /// Optional stable text review time, such as RFC 3339.
    pub reviewed_at: Option<String>,
    /// Optional review or extraction notes.
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

    /// Returns this provenance with a validated extraction method.
    pub fn with_extraction_method(mut self, extraction_method: impl Into<String>) -> Result<Self> {
        self.extraction_method =
            normalize_optional_text("extraction_method", Some(extraction_method.into()))?;
        Ok(self)
    }

    /// Returns this provenance with a validated extractor identity.
    pub fn with_extractor_id(mut self, extractor_id: impl Into<String>) -> Result<Self> {
        self.extractor_id = normalize_optional_text("extractor_id", Some(extractor_id.into()))?;
        Ok(self)
    }

    /// Returns this provenance with a validated reviewer identity.
    pub fn with_reviewer_id(mut self, reviewer_id: impl Into<String>) -> Result<Self> {
        self.reviewer_id = normalize_optional_text("reviewer_id", Some(reviewer_id.into()))?;
        Ok(self)
    }

    /// Returns this provenance with a validated review timestamp payload.
    pub fn with_reviewed_at(mut self, reviewed_at: impl Into<String>) -> Result<Self> {
        self.reviewed_at = normalize_optional_text("reviewed_at", Some(reviewed_at.into()))?;
        Ok(self)
    }

    /// Returns this provenance with validated notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Result<Self> {
        self.notes = normalize_optional_text("notes", Some(notes.into()))?;
        Ok(self)
    }

    /// Validates source and optional provenance payload fields.
    pub fn validate(&self) -> Result<()> {
        self.to_wire().map(|_| ())
    }

    fn from_wire(wire: ProvenanceWire) -> Result<Self> {
        wire.source.validate()?;

        Ok(Self {
            source: wire.source,
            confidence: wire.confidence,
            review_status: wire.review_status,
            extraction_method: normalize_optional_text(
                "extraction_method",
                wire.extraction_method,
            )?,
            extractor_id: normalize_optional_text("extractor_id", wire.extractor_id)?,
            reviewer_id: normalize_optional_text("reviewer_id", wire.reviewer_id)?,
            reviewed_at: normalize_optional_text("reviewed_at", wire.reviewed_at)?,
            notes: normalize_optional_text("notes", wire.notes)?,
        })
    }

    fn to_wire(&self) -> Result<ProvenanceWire> {
        self.source.validate()?;

        Ok(ProvenanceWire {
            source: self.source.clone(),
            confidence: self.confidence,
            review_status: self.review_status,
            extraction_method: normalize_optional_text_ref(
                "extraction_method",
                self.extraction_method.as_ref(),
            )?,
            extractor_id: normalize_optional_text_ref("extractor_id", self.extractor_id.as_ref())?,
            reviewer_id: normalize_optional_text_ref("reviewer_id", self.reviewer_id.as_ref())?,
            reviewed_at: normalize_optional_text_ref("reviewed_at", self.reviewed_at.as_ref())?,
            notes: normalize_optional_text_ref("notes", self.notes.as_ref())?,
        })
    }
}

impl Serialize for Provenance {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_wire()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Provenance {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProvenanceWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProvenanceWire {
    source: SourceRef,
    confidence: Confidence,
    review_status: ReviewStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    extraction_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extractor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviewer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviewed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
}
