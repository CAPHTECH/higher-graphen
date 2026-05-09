use super::USAGE;
use crate::{
    native_cli::NativeCliError, workflow_eval::cli_reports::WorkflowCommandError,
    workflow_workspace::cli_bridge::WorkflowBridgeError,
};
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Core(higher_graphen_core::CoreError),
    Store(crate::store::StoreError),
    WorkflowCommand(WorkflowCommandError),
    WorkflowBridge(WorkflowBridgeError),
    Native(NativeCliError),
    Json(serde_json::Error),
}

impl CliError {
    pub(crate) fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }
}

impl From<higher_graphen_core::CoreError> for CliError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<crate::store::StoreError> for CliError {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<WorkflowCommandError> for CliError {
    fn from(error: WorkflowCommandError) -> Self {
        Self::WorkflowCommand(error)
    }
}

impl From<WorkflowBridgeError> for CliError {
    fn from(error: WorkflowBridgeError) -> Self {
        Self::WorkflowBridge(error)
    }
}

impl From<NativeCliError> for CliError {
    fn from(error: NativeCliError) -> Self {
        Self::Native(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "{message}\n{USAGE}"),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "{error}"),
            Self::WorkflowCommand(error) => write!(formatter, "{error}"),
            Self::WorkflowBridge(error) => write!(formatter, "{error}"),
            Self::Native(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for CliError {}
