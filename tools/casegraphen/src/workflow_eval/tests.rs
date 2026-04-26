use super::*;
use serde_json::json;
use std::collections::BTreeSet;

const WORKFLOW_EXAMPLE: &str =
    include_str!("../../../../schemas/casegraphen/workflow.graph.example.json");

#[test]
fn workflow_evaluation_emits_readiness_obstructions_and_candidates() {
    let graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

    let evaluation = evaluate_workflow(&graph);
    let obstruction_types = evaluation
        .obstructions
        .iter()
        .map(|record| record.obstruction_type)
        .collect::<BTreeSet<_>>();
    let candidate_types = evaluation
        .completion_candidates
        .iter()
        .map(|candidate| candidate.candidate_type)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        evaluation.readiness.evaluated_work_item_ids.len(),
        graph.work_items.len()
    );
    assert!(evaluation
        .readiness
        .not_ready_items
        .iter()
        .any(|item| item.work_item_id.as_str() == "proof:workflow-schema-parse-check"));
    assert!(obstruction_types.contains(&ObstructionType::MissingEvidence));
    assert!(obstruction_types.contains(&ObstructionType::MissingProof));
    assert!(obstruction_types.contains(&ObstructionType::UnresolvedDependency));
    assert!(obstruction_types.contains(&ObstructionType::ReviewRequired));
    assert!(candidate_types.contains(&CompletionCandidateType::MissingEvidence));
    assert!(candidate_types.contains(&CompletionCandidateType::MissingProof));
    assert!(candidate_types.contains(&CompletionCandidateType::MissingTask));
}

#[test]
fn workflow_evaluation_preserves_evidence_boundary_and_trace_sections() {
    let graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

    let evaluation = evaluate_workflow(&graph);

    assert!(evaluation
        .evidence_findings
        .accepted_evidence_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-target-doc"));
    assert!(evaluation
        .evidence_findings
        .source_backed_evidence_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-target-doc"));
    assert!(evaluation
        .evidence_findings
        .inference_record_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-gap-inference"));
    assert!(!evaluation
        .evidence_findings
        .accepted_evidence_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-gap-inference"));
    assert_eq!(
        evaluation.projection.projection_profile_id.as_str(),
        "projection:workflow-ai-review"
    );
    assert!(!evaluation.projection.information_loss.is_empty());
    assert_eq!(evaluation.correspondence.len(), 1);
    assert_eq!(
        evaluation.evolution.transition_ids[0].as_str(),
        "transition:foundation-docs-to-workflow-contract"
    );
}

#[test]
fn workflow_evaluation_is_machine_readable_json() {
    let graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
    let evaluation = evaluate_workflow(&graph);

    let value = serde_json::to_value(&evaluation).expect("serialize evaluation");

    assert_eq!(value["status"], json!("obstructions_detected"));
    assert!(value["readiness"]["rule_results"].is_array());
    assert!(value["obstructions"].is_array());
    assert!(value["completion_candidates"].is_array());
    assert!(value["evidence_findings"]["findings"].is_array());
    assert!(value["projection"]["information_loss"].is_array());
    assert!(value["correspondence"].is_array());
    assert!(value["evolution"]["transition_ids"].is_array());
}
