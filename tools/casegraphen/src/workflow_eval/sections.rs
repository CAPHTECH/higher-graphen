use super::{
    evaluate_workflow, validate_workflow_graph, CompletionCandidate, CorrespondenceResult,
    EvidenceFindings, EvolutionResult, ObstructionRecord, ProjectionResult, ReadinessResult,
    WorkflowEvaluation, WorkflowResult,
};
use crate::workflow_model::WorkflowCaseGraph;

pub fn evaluate_workflow_checked(graph: &WorkflowCaseGraph) -> WorkflowResult<WorkflowEvaluation> {
    validate_workflow_graph(graph)?;
    Ok(evaluate_workflow(graph))
}

pub fn evaluate_readiness(graph: &WorkflowCaseGraph) -> WorkflowResult<ReadinessResult> {
    Ok(evaluate_workflow_checked(graph)?.readiness)
}

pub fn evaluate_obstructions(graph: &WorkflowCaseGraph) -> WorkflowResult<Vec<ObstructionRecord>> {
    Ok(evaluate_workflow_checked(graph)?.obstructions)
}

pub fn evaluate_completion_candidates(
    graph: &WorkflowCaseGraph,
) -> WorkflowResult<Vec<CompletionCandidate>> {
    Ok(evaluate_workflow_checked(graph)?.completion_candidates)
}

pub fn evaluate_evidence_findings(graph: &WorkflowCaseGraph) -> WorkflowResult<EvidenceFindings> {
    Ok(evaluate_workflow_checked(graph)?.evidence_findings)
}

pub fn evaluate_projection(graph: &WorkflowCaseGraph) -> WorkflowResult<ProjectionResult> {
    Ok(evaluate_workflow_checked(graph)?.projection)
}

pub fn evaluate_correspondence(
    graph: &WorkflowCaseGraph,
) -> WorkflowResult<Vec<CorrespondenceResult>> {
    Ok(evaluate_workflow_checked(graph)?.correspondence)
}

pub fn evaluate_evolution(graph: &WorkflowCaseGraph) -> WorkflowResult<EvolutionResult> {
    Ok(evaluate_workflow_checked(graph)?.evolution)
}
