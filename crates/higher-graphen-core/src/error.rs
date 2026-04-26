use serde::{Deserialize, Serialize};
use std::fmt;

/// Core-owned result type for fallible primitive APIs.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Structured, machine-readable errors for core primitive boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum CoreError {
    /// An identifier was empty or malformed after normalization.
    InvalidId {
        /// The value supplied by the caller or serialized input.
        value: String,
        /// A diagnostic explanation for humans.
        reason: String,
    },
    /// A confidence score was NaN, infinite, or outside the valid range.
    InvalidConfidence {
        /// The value supplied by the caller or serialized input.
        value: String,
        /// A diagnostic explanation for humans.
        reason: String,
    },
    /// A source kind was not one of the stable core categories.
    InvalidSourceKind {
        /// The value supplied by the caller or serialized input.
        value: String,
        /// A diagnostic explanation for humans.
        reason: String,
    },
    /// A required field was absent or malformed at a primitive boundary.
    MalformedField {
        /// The stable lower snake case field name.
        field: String,
        /// A diagnostic explanation for humans.
        reason: String,
    },
    /// A primitive parser could not convert input into the requested type.
    ParseFailure {
        /// The primitive type or target schema being parsed.
        target: String,
        /// The value supplied by the caller or serialized input.
        value: String,
        /// A diagnostic explanation for humans.
        reason: String,
    },
    /// Serialized data used a version unsupported by this crate.
    UnsupportedVersion {
        /// The unsupported serialized version.
        version: String,
        /// The version or version range supported by this crate.
        supported: String,
    },
}

impl CoreError {
    /// Returns the stable error code for bindings and tests.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidId { .. } => "invalid_id",
            Self::InvalidConfidence { .. } => "invalid_confidence",
            Self::InvalidSourceKind { .. } => "invalid_source_kind",
            Self::MalformedField { .. } => "malformed_field",
            Self::ParseFailure { .. } => "parse_failure",
            Self::UnsupportedVersion { .. } => "unsupported_version",
        }
    }

    pub(crate) fn invalid_id(value: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidId {
            value: value.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn invalid_confidence(value: f64, reason: impl Into<String>) -> Self {
        Self::InvalidConfidence {
            value: value.to_string(),
            reason: reason.into(),
        }
    }

    pub(crate) fn invalid_source_kind(value: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidSourceKind {
            value: value.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::MalformedField {
            field: field.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn parse_failure(
        target: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ParseFailure {
            target: target.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId { value, reason } => {
                write!(formatter, "{}: invalid id {value:?}: {reason}", self.code())
            }
            Self::InvalidConfidence { value, reason } => write!(
                formatter,
                "{}: invalid confidence {value:?}: {reason}",
                self.code()
            ),
            Self::InvalidSourceKind { value, reason } => write!(
                formatter,
                "{}: invalid source kind {value:?}: {reason}",
                self.code()
            ),
            Self::MalformedField { field, reason } => {
                write!(
                    formatter,
                    "{}: malformed field {field:?}: {reason}",
                    self.code()
                )
            }
            Self::ParseFailure {
                target,
                value,
                reason,
            } => write!(
                formatter,
                "{}: could not parse {value:?} as {target}: {reason}",
                self.code()
            ),
            Self::UnsupportedVersion { version, supported } => write!(
                formatter,
                "{}: unsupported version {version:?}; supported {supported}",
                self.code()
            ),
        }
    }
}

impl std::error::Error for CoreError {}
