//! Runtime-owned error types.

use higher_graphen_core::CoreError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Runtime-owned result type for workflow orchestration APIs.
pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;

/// Structured, machine-readable errors produced by runtime orchestration.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum RuntimeError {
    /// A lower-crate constructor or validator rejected deterministic input.
    Core {
        /// Structured lower-crate error.
        source: CoreError,
    },
    /// A deterministic workflow assembled an internally inconsistent result.
    WorkflowConstruction {
        /// Stable workflow name.
        workflow: String,
        /// Diagnostic explanation for humans.
        reason: String,
    },
    /// Runtime or consumers requested a report version that is not supported.
    UnsupportedReportVersion {
        /// Unsupported version.
        version: u32,
        /// Supported version or range.
        supported: String,
    },
    /// Report serialization failed.
    Serialization {
        /// Diagnostic explanation for humans.
        reason: String,
    },
}

impl RuntimeError {
    /// Returns the stable error code for bindings and tests.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Core { .. } => "core",
            Self::WorkflowConstruction { .. } => "workflow_construction",
            Self::UnsupportedReportVersion { .. } => "unsupported_report_version",
            Self::Serialization { .. } => "serialization",
        }
    }

    pub(crate) fn workflow_construction(
        workflow: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::WorkflowConstruction {
            workflow: workflow.into(),
            reason: reason.into(),
        }
    }

    /// Creates a serialization error from a serializer diagnostic.
    #[must_use]
    pub fn serialization(reason: impl Into<String>) -> Self {
        Self::Serialization {
            reason: reason.into(),
        }
    }
}

impl From<CoreError> for RuntimeError {
    fn from(source: CoreError) -> Self {
        Self::Core { source }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core { source } => write!(formatter, "{}: {source}", self.code()),
            Self::WorkflowConstruction { workflow, reason } => {
                write!(
                    formatter,
                    "{}: workflow {workflow:?} could not be constructed: {reason}",
                    self.code()
                )
            }
            Self::UnsupportedReportVersion { version, supported } => write!(
                formatter,
                "{}: unsupported report version {version}; supported {supported}",
                self.code()
            ),
            Self::Serialization { reason } => {
                write!(
                    formatter,
                    "{}: report serialization failed: {reason}",
                    self.code()
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}
