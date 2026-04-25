use serde::{Deserialize, Serialize};

/// Impact classification used by downstream model and engine crates.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Lowest impact classification.
    Low,
    /// Moderate impact classification.
    Medium,
    /// High impact classification.
    High,
    /// Highest impact classification.
    Critical,
}

/// Human or workflow review state for observed or inferred structure.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    /// No review has occurred.
    #[default]
    Unreviewed,
    /// Review occurred without accepting or rejecting the structure.
    Reviewed,
    /// The structure must not be treated as accepted fact.
    Rejected,
    /// The structure may be treated as accepted fact.
    Accepted,
}

impl ReviewStatus {
    /// Returns true when the structure may be treated as accepted fact.
    pub fn is_accepted(self) -> bool {
        matches!(self, Self::Accepted)
    }

    /// Returns true when the structure must not be silently promoted.
    pub fn is_rejected(self) -> bool {
        matches!(self, Self::Rejected)
    }

    /// Returns true when a review action has occurred.
    pub fn has_review_action(self) -> bool {
        !matches!(self, Self::Unreviewed)
    }
}
