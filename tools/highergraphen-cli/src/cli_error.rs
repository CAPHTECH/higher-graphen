use crate::USAGE;
use higher_graphen_runtime::RuntimeError;
use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub(crate) enum CliError {
    Usage(String),
    Runtime(RuntimeError),
    InputRead {
        path: PathBuf,
        source: std::io::Error,
    },
    InputParse {
        path: PathBuf,
        source: serde_json::Error,
    },
    InputContract {
        path: PathBuf,
        reason: String,
    },
    GitInput(String),
    TestGapEvidence(String),
    TestSemanticsInterpretation(String),
    TestSemanticsReview(String),
    TestSemanticsVerification(String),
    TestSemanticsGap(String),
    RustTestSemantics(String),
    SemanticProofArtifact(String),
    Output(std::io::Error),
}

impl CliError {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }

    pub(crate) fn write_output(error: std::io::Error) -> Self {
        Self::Output(error)
    }
}

impl From<RuntimeError> for CliError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

impl From<higher_graphen_core::CoreError> for CliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Runtime(RuntimeError::from(error))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Runtime(error) => write!(formatter, "{error}"),
            Self::InputRead { path, source } => {
                write!(
                    formatter,
                    "failed to read input {}: {source}",
                    path.display()
                )
            }
            Self::InputParse { path, source } => {
                write!(
                    formatter,
                    "failed to parse input {}: {source}",
                    path.display()
                )
            }
            Self::InputContract { path, reason } => {
                write!(formatter, "invalid input {}: {reason}", path.display())
            }
            Self::GitInput(message) => write!(formatter, "failed to build git input: {message}"),
            Self::TestGapEvidence(message) => {
                write!(formatter, "failed to build test-gap evidence: {message}")
            }
            Self::TestSemanticsInterpretation(message) => {
                write!(
                    formatter,
                    "failed to build test semantics interpretation: {message}"
                )
            }
            Self::TestSemanticsReview(message) => {
                write!(
                    formatter,
                    "failed to build test semantics interpretation review: {message}"
                )
            }
            Self::TestSemanticsVerification(message) => {
                write!(
                    formatter,
                    "failed to build test semantics verification: {message}"
                )
            }
            Self::TestSemanticsGap(message) => {
                write!(formatter, "failed to detect test semantics gaps: {message}")
            }
            Self::RustTestSemantics(message) => {
                write!(formatter, "failed to build rust test semantics: {message}")
            }
            Self::SemanticProofArtifact(message) => {
                write!(formatter, "failed to build semantic proof input: {message}")
            }
            Self::Output(error) => write!(formatter, "failed to write output: {error}"),
        }
    }
}

impl std::error::Error for CliError {}
