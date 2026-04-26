use crate::{CoreError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

impl Severity {
    /// Stable lower snake case representation used by serde and text protocols.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Returns true when this severity is at least the supplied minimum.
    pub fn is_at_least(self, minimum: Self) -> bool {
        self >= minimum
    }
}

impl FromStr for Severity {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            unknown => Err(CoreError::parse_failure(
                "severity",
                unknown,
                "expected low, medium, high, or critical",
            )),
        }
    }
}

impl TryFrom<&str> for Severity {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for Severity {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self> {
        Self::from_str(&value)
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
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
    /// Stable lower snake case representation used by serde and text protocols.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unreviewed => "unreviewed",
            Self::Reviewed => "reviewed",
            Self::Rejected => "rejected",
            Self::Accepted => "accepted",
        }
    }

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

impl FromStr for ReviewStatus {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "unreviewed" => Ok(Self::Unreviewed),
            "reviewed" => Ok(Self::Reviewed),
            "rejected" => Ok(Self::Rejected),
            "accepted" => Ok(Self::Accepted),
            unknown => Err(CoreError::parse_failure(
                "review_status",
                unknown,
                "expected unreviewed, reviewed, rejected, or accepted",
            )),
        }
    }
}

impl TryFrom<&str> for ReviewStatus {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self> {
        Self::from_str(value)
    }
}

impl TryFrom<String> for ReviewStatus {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self> {
        Self::from_str(&value)
    }
}

impl fmt::Display for ReviewStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
