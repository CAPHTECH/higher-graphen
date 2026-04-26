use super::*;
use higher_graphen_core::Id;
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

#[test]
fn workflow_validation_rejects_dangling_internal_relation_endpoints() {
    let mut graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
    graph.workflow_relations[0].from_id = id("task:missing-work-item");

    let error = validate_workflow_graph(&graph).expect_err("dangling relation endpoint");

    assert!(error.violations.iter().any(|violation| {
        violation.code == WorkflowValidationCode::DanglingReference
            && violation
                .record_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "relation:engine-depends-on-contract")
            && violation.field == "from_id"
    }));
}

#[test]
fn workflow_validation_rejects_malformed_required_strings() {
    let mut graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
    graph.work_items[0].title = "   ".to_owned();

    let error = validate_workflow_graph(&graph).expect_err("empty title");

    assert!(error.violations.iter().any(|violation| {
        violation.code == WorkflowValidationCode::EmptyRequiredField
            && violation
                .record_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "task:define-workflow-reasoning-contract")
            && violation.field == "title"
    }));
}

#[test]
fn workflow_checked_section_helpers_validate_before_deriving_sections() {
    let graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

    let readiness = evaluate_readiness(&graph).expect("readiness section");
    let obstructions = evaluate_obstructions(&graph).expect("obstruction section");
    let candidates = evaluate_completion_candidates(&graph).expect("completion candidate section");
    let evidence = evaluate_evidence_findings(&graph).expect("evidence section");
    let projection = evaluate_projection(&graph).expect("projection section");
    let correspondence = evaluate_correspondence(&graph).expect("correspondence section");
    let evolution = evaluate_evolution(&graph).expect("evolution section");

    assert_eq!(
        readiness.ready_item_ids,
        evaluate_workflow(&graph).readiness.ready_item_ids
    );
    assert_eq!(
        obstructions.len(),
        evaluate_workflow(&graph).obstructions.len()
    );
    assert_eq!(
        candidates.len(),
        evaluate_workflow(&graph).completion_candidates.len()
    );
    assert_eq!(
        evidence.inference_record_ids,
        evaluate_workflow(&graph)
            .evidence_findings
            .inference_record_ids
    );
    assert_eq!(
        projection.projection_profile_id.as_str(),
        "projection:workflow-ai-review"
    );
    assert_eq!(correspondence.len(), 1);
    assert_eq!(
        evolution.transition_ids,
        vec![id("transition:foundation-docs-to-workflow-contract")]
    );
}

#[test]
fn ai_inference_does_not_satisfy_evidence_requirements() {
    let mut graph: WorkflowCaseGraph =
        serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
    graph
        .evidence_records
        .iter_mut()
        .find(|record| record.id.as_str() == "evidence:workflow-gap-inference")
        .expect("inference record")
        .supports_ids
        .push(id("evidence:json-parse-check-output"));

    let evaluation = evaluate_workflow_checked(&graph).expect("valid workflow");

    assert!(evaluation
        .obstructions
        .iter()
        .any(|record| record.obstruction_type == ObstructionType::MissingEvidence));
    assert!(!evaluation
        .evidence_findings
        .accepted_evidence_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-gap-inference"));
    assert!(evaluation
        .evidence_findings
        .inference_record_ids
        .iter()
        .any(|id| id.as_str() == "evidence:workflow-gap-inference"));
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id")
}
