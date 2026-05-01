use super::{
    confidence, finalize_validation, generated_id, generated_provenance, metadata_extensions,
    payload_ref, source_uri, witness_type_for_workflow, workflow_provenance,
    CaseGraphenCoreExtensions,
};
use crate::{
    workflow_eval::{CorrespondenceResult, ReadinessRuleResult, WorkflowEvaluation},
    workflow_model::{EvidenceRecord, ReadinessRule, WorkflowCaseGraph},
};
use higher_graphen_core::{
    CriterionDirection, CriterionValue, Derivation, DerivationFailureMode, EquivalenceClaim,
    EquivalenceCriterion, EquivalenceKind, EquivalenceScope, Id, InferenceRule, LifecycleStatus,
    ObjectRef, OrderType, Policy, PolicyApplicability, PolicyKind, PolicyRule, PolicyStatus,
    Provenance, Reachability, ReviewStatus, Scenario, ScenarioChanges, ScenarioKind,
    ScenarioStatus, Valuation, ValuationCriterion, ValuationValue, VerificationStatus, Verifier,
    VerifierKind, Witness, WitnessStatus,
};

pub fn workflow_reason_extensions(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
) -> CaseGraphenCoreExtensions {
    let provenance = workflow_reason_provenance(graph, evaluation);
    let generated = CaseGraphenCoreExtensions {
        witnesses: workflow_witnesses(graph),
        derivations: workflow_derivations(graph, evaluation, &provenance),
        policies: workflow_policies(graph),
        scenarios: vec![workflow_reason_scenario(graph, evaluation, &provenance)],
        equivalence_claims: workflow_equivalence_claims(graph, evaluation, &provenance),
        ..CaseGraphenCoreExtensions::default()
    };

    let valuation_evidence = workflow_valuation_evidence(graph, &generated.witnesses);
    let mut extensions = generated;
    extensions.valuations.push(workflow_readiness_valuation(
        graph,
        evaluation,
        valuation_evidence,
        provenance,
    ));
    let mut merged = metadata_extensions(&graph.metadata);
    merged.append(extensions);
    let mut extensions = merged;
    finalize_validation(&mut extensions);
    extensions
}

fn workflow_reason_provenance(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
) -> Provenance {
    generated_provenance(
        source_uri(
            "workflow",
            &graph.workflow_graph_id,
            "reason",
            &evaluation.evolution.revision_id,
        ),
        "CaseGraphen workflow reason",
        ReviewStatus::Reviewed,
        0.89,
    )
}

fn workflow_witnesses(graph: &WorkflowCaseGraph) -> Vec<Witness> {
    graph
        .evidence_records
        .iter()
        .map(|evidence| workflow_evidence_witness(graph, evidence))
        .collect()
}

fn workflow_derivations(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
    provenance: &Provenance,
) -> Vec<Derivation> {
    evaluation
        .readiness
        .rule_results
        .iter()
        .map(|rule_result| workflow_derivation(graph, evaluation, rule_result, provenance))
        .collect()
}

fn workflow_policies(graph: &WorkflowCaseGraph) -> Vec<Policy> {
    graph
        .readiness_rules
        .iter()
        .map(|rule| workflow_policy(graph, rule))
        .collect()
}

fn workflow_equivalence_claims(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
    provenance: &Provenance,
) -> Vec<EquivalenceClaim> {
    evaluation
        .correspondence
        .iter()
        .filter_map(|correspondence| workflow_equivalence_claim(graph, correspondence, provenance))
        .collect()
}

fn workflow_evidence_witness(graph: &WorkflowCaseGraph, evidence: &EvidenceRecord) -> Witness {
    let mut witness = Witness::candidate(
        generated_id("witness", &[evidence.id.as_str()]),
        witness_type_for_workflow(evidence.evidence_type),
        payload_ref(
            &format!("workflow_{:?}", evidence.evidence_boundary).to_ascii_lowercase(),
            source_uri(
                "workflow",
                &graph.workflow_graph_id,
                "evidence",
                &evidence.id,
            ),
        ),
        evidence
            .provenance
            .recorded_at
            .as_deref()
            .unwrap_or("unrecorded"),
        workflow_provenance(&evidence.provenance, &evidence.summary),
        evidence.provenance.confidence,
    )
    .expect("workflow evidence witness is valid");
    witness.supports = evidence
        .supports_ids
        .iter()
        .cloned()
        .map(ObjectRef::new)
        .collect();
    witness.contradicts = evidence
        .contradicts_ids
        .iter()
        .cloned()
        .map(ObjectRef::new)
        .collect();
    witness.validity_contexts = vec![graph.workflow_graph_id.clone()];
    witness.review_status = workflow_witness_status(evidence.provenance.review_status);
    witness
}

fn workflow_witness_status(review_status: ReviewStatus) -> WitnessStatus {
    match review_status {
        ReviewStatus::Accepted => WitnessStatus::Accepted,
        ReviewStatus::Rejected => WitnessStatus::Rejected,
        ReviewStatus::Unreviewed | ReviewStatus::Reviewed => WitnessStatus::Candidate,
    }
}

fn workflow_derivation(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
    rule_result: &ReadinessRuleResult,
    provenance: &Provenance,
) -> Derivation {
    let mut rule = InferenceRule::new(
        rule_result.rule_id.clone(),
        "Workflow readiness rule evaluation",
    )
    .expect("workflow rule id is valid");
    rule.rule_scope_contexts = vec![graph.workflow_graph_id.clone()];

    let mut derivation = Derivation::candidate(
        rule_result.id.clone(),
        rule_result.target_work_item_id.clone(),
        workflow_derivation_premises(rule_result),
        rule,
        provenance.clone(),
    );
    derivation.warrants = evaluation.evidence_findings.accepted_evidence_ids.clone();
    derivation.verifier = Some(
        Verifier::new(VerifierKind::CustomEngine, "casegraphen workflow reason")
            .expect("generated verifier is valid"),
    );
    derivation.verification_status = VerificationStatus::MachineChecked;
    derivation.failure_mode = if rule_result.ready {
        DerivationFailureMode::None
    } else {
        DerivationFailureMode::MissingPremise
    };
    derivation
}

fn workflow_derivation_premises(rule_result: &ReadinessRuleResult) -> Vec<Id> {
    rule_result
        .obstruction_ids
        .iter()
        .cloned()
        .chain(std::iter::once(rule_result.rule_id.clone()))
        .collect()
}

fn workflow_policy(graph: &WorkflowCaseGraph, rule: &ReadinessRule) -> Policy {
    Policy {
        id: generated_id("policy", &[rule.id.as_str()]),
        policy_kind: PolicyKind::Obligation,
        applies_to: PolicyApplicability {
            target_types: vec!["workflow_work_item".to_owned()],
            contexts: vec![graph.workflow_graph_id.clone()],
            operations: vec![
                "workflow_reason".to_owned(),
                format!("{:?}", rule.rule_type),
            ],
        },
        rule: PolicyRule::new(
            rule.description
                .as_deref()
                .unwrap_or("Workflow readiness rule must hold before a work item is ready."),
        )
        .expect("workflow policy rule is valid"),
        required_witnesses: rule.source_constraint_id.iter().cloned().collect(),
        required_derivations: Vec::new(),
        escalation_path: Vec::new(),
        violation_obstruction_template: Some(generated_id(
            "obstruction_template",
            &[&rule.obstruction_type],
        )),
        status: PolicyStatus::Draft,
        provenance: workflow_provenance(&rule.provenance, "Workflow readiness rule"),
        review_status: rule.provenance.review_status,
    }
}

fn workflow_reason_scenario(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
    provenance: &Provenance,
) -> Scenario {
    Scenario {
        id: generated_id("scenario", &[graph.workflow_graph_id.as_str(), "reasoning"]),
        base_space: graph.space_id.clone(),
        scenario_kind: workflow_scenario_kind(evaluation),
        assumptions: graph
            .work_items
            .iter()
            .map(|item| item.id.clone())
            .collect(),
        changed_structures: ScenarioChanges::default(),
        reachable_from: Some(Reachability {
            reference: graph.space_id.clone(),
            via_morphisms: evaluation.evolution.transition_ids.clone(),
        }),
        affected_invariants: evaluation
            .readiness
            .rule_results
            .iter()
            .map(|result| result.rule_id.clone())
            .collect(),
        expected_obstructions: evaluation
            .obstructions
            .iter()
            .map(|obstruction| obstruction.id.clone())
            .collect(),
        required_witnesses: evaluation.evidence_findings.accepted_evidence_ids.clone(),
        valuations: vec![generated_id(
            "valuation",
            &[graph.workflow_graph_id.as_str(), "readiness"],
        )],
        status: workflow_scenario_status(evaluation),
        provenance: provenance.clone(),
        review_status: LifecycleStatus::Candidate,
    }
}

fn workflow_scenario_kind(evaluation: &WorkflowEvaluation) -> ScenarioKind {
    if evaluation.obstructions.is_empty() {
        ScenarioKind::Reachable
    } else {
        ScenarioKind::Blocked
    }
}

fn workflow_scenario_status(evaluation: &WorkflowEvaluation) -> ScenarioStatus {
    if evaluation.obstructions.is_empty() {
        ScenarioStatus::Reachable
    } else {
        ScenarioStatus::Blocked
    }
}

fn workflow_equivalence_claim(
    graph: &WorkflowCaseGraph,
    correspondence: &CorrespondenceResult,
    provenance: &Provenance,
) -> Option<EquivalenceClaim> {
    if correspondence.left_ids.is_empty() || correspondence.right_ids.is_empty() {
        return None;
    }
    let mut subjects = Vec::new();
    subjects.extend(correspondence.left_ids.iter().cloned().map(ObjectRef::new));
    subjects.extend(correspondence.right_ids.iter().cloned().map(ObjectRef::new));

    let mut claim = EquivalenceClaim::candidate(
        generated_id("equivalence", &[correspondence.id.as_str()]),
        subjects,
        EquivalenceKind::ContextualEquivalence,
        correspondence.confidence,
        provenance.clone(),
    );
    claim.scope = Some(EquivalenceScope {
        contexts: vec![graph.workflow_graph_id.clone()],
        valid_under_morphisms: evaluation_transition_scope(graph),
    });
    claim.criterion = Some(
        EquivalenceCriterion::new("Workflow correspondence result projects compatible structures into the reasoning report.")
            .expect("workflow equivalence criterion is valid"),
    );
    claim.witnesses = correspondence.mismatch_evidence_ids.clone();
    Some(claim)
}

fn workflow_valuation_evidence(graph: &WorkflowCaseGraph, witnesses: &[Witness]) -> Id {
    witnesses
        .first()
        .map(|witness| witness.id.clone())
        .unwrap_or_else(|| {
            generated_id("witness", &[graph.workflow_graph_id.as_str(), "synthetic"])
        })
}

fn workflow_readiness_valuation(
    graph: &WorkflowCaseGraph,
    evaluation: &WorkflowEvaluation,
    evidence: Id,
    provenance: Provenance,
) -> Valuation {
    Valuation {
        id: generated_id(
            "valuation",
            &[graph.workflow_graph_id.as_str(), "readiness"],
        ),
        target: ObjectRef::new(graph.workflow_graph_id.clone()),
        valuation_context: Some(graph.workflow_graph_id.clone()),
        criteria: workflow_readiness_criteria(),
        order_type: OrderType::ThresholdAcceptance,
        values: workflow_readiness_values(evaluation, &evidence),
        tradeoffs: Vec::new(),
        incomparable_with: Vec::new(),
        confidence: confidence(0.84),
        provenance,
        review_status: LifecycleStatus::Candidate,
    }
}

fn workflow_readiness_criteria() -> Vec<ValuationCriterion> {
    vec![
        ValuationCriterion {
            criterion_id: "ready_items".to_owned(),
            name: "Ready work items".to_owned(),
            direction: CriterionDirection::Maximize,
            weight: Some(0.6),
            required: true,
        },
        ValuationCriterion {
            criterion_id: "blocking_obstructions".to_owned(),
            name: "Blocking obstructions".to_owned(),
            direction: CriterionDirection::Avoid,
            weight: Some(0.4),
            required: true,
        },
    ]
}

fn workflow_readiness_values(
    evaluation: &WorkflowEvaluation,
    evidence: &Id,
) -> Vec<CriterionValue> {
    vec![
        CriterionValue {
            criterion_id: "ready_items".to_owned(),
            value: ValuationValue::Number(evaluation.readiness.ready_item_ids.len() as f64),
            evidence: evidence.clone(),
        },
        CriterionValue {
            criterion_id: "blocking_obstructions".to_owned(),
            value: ValuationValue::Number(blocking_obstruction_count(evaluation) as f64),
            evidence: evidence.clone(),
        },
    ]
}

fn blocking_obstruction_count(evaluation: &WorkflowEvaluation) -> usize {
    evaluation
        .obstructions
        .iter()
        .filter(|obstruction| obstruction.blocking)
        .count()
}

fn evaluation_transition_scope(graph: &WorkflowCaseGraph) -> Vec<Id> {
    graph
        .transition_records
        .iter()
        .map(|transition| transition.morphism_id.clone())
        .collect()
}
