use crate::text::normalize_required_text;
use crate::{Id, Result};
use serde::{Deserialize, Serialize};

/// Reference to a HigherGraphen object encoded by its stable identifier.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRef {
    /// Target object reference, such as `cell:customer` or `derivation:proof`.
    #[serde(rename = "ref")]
    pub reference: Id,
}

impl ObjectRef {
    /// Creates a reference from a stable object identifier.
    pub fn new(reference: Id) -> Self {
        Self { reference }
    }
}

/// Free-text structural note retained in machine-readable objects.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Description {
    /// Human-readable description.
    pub description: String,
}

impl Description {
    /// Creates a validated description.
    pub fn new(description: impl Into<String>) -> Result<Self> {
        Ok(Self {
            description: normalize_required_text("description", description)?,
        })
    }
}

/// Candidate lifecycle shared by reviewable core extension objects.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStatus {
    /// Proposed by an engine or actor, not accepted fact.
    Candidate,
    /// Awaiting or undergoing explicit review.
    UnderReview,
    /// Explicitly accepted for its declared scope.
    Accepted,
    /// Explicitly rejected.
    Rejected,
    /// Replaced by a later object.
    Superseded,
}

impl LifecycleStatus {
    /// Returns true when the object is accepted for its declared scope.
    pub fn is_accepted(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

/// Explicit review requirement for a promotion or operation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRequirement {
    /// Whether explicit review is required.
    pub required: bool,
    /// Reviewer identity when a specific reviewer is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<Id>,
    /// Review decision or requirement rationale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reason: Option<String>,
}

impl ReviewRequirement {
    /// Creates a review requirement.
    pub fn new(required: bool) -> Self {
        Self {
            required,
            reviewer: None,
            decision_reason: None,
        }
    }

    /// Returns this requirement with a validated decision reason.
    pub fn with_decision_reason(mut self, decision_reason: impl Into<String>) -> Result<Self> {
        self.decision_reason = Some(normalize_required_text("decision_reason", decision_reason)?);
        Ok(self)
    }
}
