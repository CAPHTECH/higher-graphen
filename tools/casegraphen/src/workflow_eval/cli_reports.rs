use super::{
    evaluate_completion_candidates, evaluate_correspondence, evaluate_evidence_findings,
    evaluate_evolution, evaluate_obstructions, evaluate_projection, evaluate_readiness,
    validate_workflow_graph, WorkflowValidationError,
};
use crate::{
    model::ProjectionDefinition,
    store::{read_projection, read_workflow_graph, StoreError},
    workflow_model::WorkflowCaseGraph,
    workflow_report,
};
use serde_json::Value;
use std::{fmt, path::Path};

pub fn workflow_reason_json(input: &Path) -> WorkflowCommandResult<String> {
    let graph = read_workflow_graph(input)?;
    serialize(&workflow_report::reason_workflow(&graph))
}

pub fn workflow_validate_json(input: &Path) -> WorkflowCommandResult<String> {
    let graph = read_workflow_graph_unchecked(input)?;
    let violations = validate_workflow_graph(&graph)
        .err()
        .map(|error| error.violations)
        .unwrap_or_default();
    let valid = violations.is_empty();
    let violation_count = violations.len();
    serialize(&workflow_report::workflow_operation_report(
        "casegraphen workflow validate",
        "validate",
        workflow_report::workflow_input_with_paths(&graph, Some(input), None),
        serde_json::json!({ "valid": valid, "violations": violations }),
        workflow_report::validation_projection(valid, violation_count),
    ))
}

pub fn workflow_readiness_json(
    input: &Path,
    projection: Option<&Path>,
) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        projection,
        "casegraphen workflow readiness",
        "readiness",
        |graph| Ok(serde_json::to_value(evaluate_readiness(graph)?)?),
    )
}

pub fn workflow_obstructions_json(input: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        None,
        "casegraphen workflow obstructions",
        "obstructions",
        |graph| Ok(serde_json::json!({ "obstructions": evaluate_obstructions(graph)? })),
    )
}

pub fn workflow_completions_json(input: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        None,
        "casegraphen workflow completions",
        "completions",
        |graph| {
            Ok(serde_json::json!({
                "completion_candidates": evaluate_completion_candidates(graph)?
            }))
        },
    )
}

pub fn workflow_evidence_json(input: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        None,
        "casegraphen workflow evidence",
        "evidence",
        |graph| Ok(serde_json::to_value(evaluate_evidence_findings(graph)?)?),
    )
}

pub fn workflow_topology_json(input: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        None,
        "casegraphen workflow history topology",
        "topology",
        |graph| {
            Ok(serde_json::to_value(crate::topology::workflow_topology(
                graph,
            )?)?)
        },
    )
}

pub fn workflow_project_json(input: &Path, projection: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        Some(projection),
        "casegraphen workflow project",
        "project",
        |graph| Ok(serde_json::to_value(evaluate_projection(graph)?)?),
    )
}

pub fn workflow_correspond_json(left: &Path, right: &Path) -> WorkflowCommandResult<String> {
    let left_graph = read_workflow_graph(left)?;
    let right_graph = read_workflow_graph(right)?;
    let left_result = evaluate_correspondence(&left_graph)?;
    let right_result = evaluate_correspondence(&right_graph)?;
    let mut combined_result = left_result.clone();
    combined_result.extend(right_result.clone());
    combined_result.sort_by(|left, right| left.id.cmp(&right.id));
    serialize(&workflow_report::workflow_operation_report(
        "casegraphen workflow correspond",
        "correspond",
        serde_json::json!({
            "left": workflow_report::workflow_input_with_paths(&left_graph, Some(left), None),
            "right": workflow_report::workflow_input_with_paths(&right_graph, Some(right), None)
        }),
        serde_json::json!({
            "left_correspondence": left_result,
            "right_correspondence": right_result,
            "combined_correspondence": combined_result
        }),
        workflow_report::focused_projection(&left_graph, "correspond"),
    ))
}

pub fn workflow_evolution_json(input: &Path) -> WorkflowCommandResult<String> {
    workflow_section_json(
        input,
        None,
        "casegraphen workflow evolution",
        "evolution",
        |graph| Ok(serde_json::to_value(evaluate_evolution(graph)?)?),
    )
}

fn workflow_section_json(
    input: &Path,
    projection: Option<&Path>,
    command: &str,
    operation: &str,
    evaluator: impl FnOnce(&WorkflowCaseGraph) -> WorkflowCommandResult<Value>,
) -> WorkflowCommandResult<String> {
    let graph = read_workflow_graph(input)?;
    validate_optional_projection(projection)?;
    let result = evaluator(&graph)?;
    serialize(&workflow_report::workflow_operation_report(
        command,
        operation,
        workflow_report::workflow_input_with_paths(&graph, Some(input), projection),
        result,
        workflow_report::focused_projection(&graph, operation),
    ))
}

fn validate_optional_projection(projection: Option<&Path>) -> WorkflowCommandResult<()> {
    if let Some(path) = projection {
        let _definition: ProjectionDefinition = read_projection(path)?;
    }
    Ok(())
}

fn read_workflow_graph_unchecked(input: &Path) -> WorkflowCommandResult<WorkflowCaseGraph> {
    let text = std::fs::read_to_string(input).map_err(|source| StoreError::Io {
        path: input.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(WorkflowCommandError::from)
}

fn serialize(report: &impl serde::Serialize) -> WorkflowCommandResult<String> {
    serde_json::to_string(report).map_err(WorkflowCommandError::from)
}

pub type WorkflowCommandResult<T> = Result<T, WorkflowCommandError>;

#[derive(Debug)]
pub enum WorkflowCommandError {
    Store(StoreError),
    Validation(WorkflowValidationError),
    Core(higher_graphen_core::CoreError),
    Json(serde_json::Error),
}

impl From<StoreError> for WorkflowCommandError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<WorkflowValidationError> for WorkflowCommandError {
    fn from(error: WorkflowValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<higher_graphen_core::CoreError> for WorkflowCommandError {
    fn from(error: higher_graphen_core::CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<serde_json::Error> for WorkflowCommandError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for WorkflowCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => write!(formatter, "{error}"),
            Self::Validation(error) => write!(formatter, "{error}"),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for WorkflowCommandError {}
