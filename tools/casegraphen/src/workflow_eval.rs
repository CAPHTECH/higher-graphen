use crate::workflow_model::{
    EvidenceRecord, ReadinessRule, ReadinessRuleType, WorkItem, WorkItemState, WorkflowCaseGraph,
    WorkflowProvenance, WorkflowRelationType, WorkflowSeverity,
};
use higher_graphen_core::{Confidence, Id, ReviewStatus};
use std::collections::{BTreeMap, BTreeSet};

mod completion;
mod evidence;
mod result_sections;
#[cfg(test)]
mod tests;
mod types;

use completion::completion_candidates;
use evidence::{acceptable_evidence, evidence_findings};
pub use result_sections::projection_profile_for;
use result_sections::{
    correspondence_results, evaluation_status, evolution_result, projection_result,
};
pub use types::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ReadinessCheck {
    State,
    Dependencies,
    ExternalWaits,
    Evidence,
    Proof,
    Contradictions,
}

pub fn evaluate_workflow(graph: &WorkflowCaseGraph) -> WorkflowEvaluation {
    let context = EvaluationContext::new(graph);
    let item_results = graph
        .work_items
        .iter()
        .map(|item| context.evaluate_item(item))
        .collect::<Vec<_>>();

    let obstructions = merge_obstructions(&item_results);
    let readiness = readiness_result(graph, &item_results);
    let completion_candidates = completion_candidates(&obstructions);
    let evidence_findings = evidence_findings(graph, &obstructions);
    let projection = projection_result(graph);
    let correspondence = correspondence_results(graph);
    let evolution = evolution_result(graph, &obstructions, &completion_candidates);
    let status = evaluation_status(graph, &readiness, &obstructions, &evidence_findings);

    WorkflowEvaluation {
        status,
        readiness,
        obstructions,
        completion_candidates,
        evidence_findings,
        projection,
        correspondence,
        evolution,
    }
}

struct EvaluationContext<'a> {
    graph: &'a WorkflowCaseGraph,
    work_items: BTreeMap<&'a str, &'a WorkItem>,
}

struct ItemEvaluation {
    item_id: Id,
    state: WorkItemState,
    hard_dependency_ids: Vec<Id>,
    external_wait_ids: Vec<Id>,
    evidence_requirement_ids: Vec<Id>,
    proof_requirement_ids: Vec<Id>,
    obstructions: Vec<ObstructionRecord>,
    rule_results: Vec<ReadinessRuleResult>,
}

struct RequirementIds {
    hard_dependency_ids: Vec<Id>,
    external_wait_ids: Vec<Id>,
    evidence_requirement_ids: Vec<Id>,
    proof_requirement_ids: Vec<Id>,
}

impl RequirementIds {
    fn collect(graph: &WorkflowCaseGraph, item: &WorkItem) -> Self {
        Self {
            hard_dependency_ids: collect_requirement_ids(
                graph,
                item,
                WorkflowRelationType::DependsOn,
            ),
            external_wait_ids: collect_requirement_ids(graph, item, WorkflowRelationType::WaitsFor),
            evidence_requirement_ids: collect_requirement_ids(
                graph,
                item,
                WorkflowRelationType::RequiresEvidence,
            ),
            proof_requirement_ids: collect_requirement_ids(
                graph,
                item,
                WorkflowRelationType::RequiresProof,
            ),
        }
    }
}

impl<'a> EvaluationContext<'a> {
    fn new(graph: &'a WorkflowCaseGraph) -> Self {
        let work_items = graph
            .work_items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        Self { graph, work_items }
    }

    fn evaluate_item(&self, item: &WorkItem) -> ItemEvaluation {
        let requirements = RequirementIds::collect(self.graph, item);
        let by_check = self.obstructions_by_check(item, &requirements);
        let obstructions = sorted_obstructions(&by_check);
        let rule_results = self.rule_results_for(item, &by_check);

        ItemEvaluation {
            item_id: item.id.clone(),
            state: item.state,
            hard_dependency_ids: requirements.hard_dependency_ids,
            external_wait_ids: requirements.external_wait_ids,
            evidence_requirement_ids: requirements.evidence_requirement_ids,
            proof_requirement_ids: requirements.proof_requirement_ids,
            obstructions,
            rule_results,
        }
    }

    fn obstructions_by_check(
        &self,
        item: &WorkItem,
        requirements: &RequirementIds,
    ) -> BTreeMap<ReadinessCheck, Vec<ObstructionRecord>> {
        let mut by_check: BTreeMap<ReadinessCheck, Vec<ObstructionRecord>> = BTreeMap::new();

        if let Some(obstruction) = self.state_obstruction(item) {
            by_check
                .entry(ReadinessCheck::State)
                .or_default()
                .push(obstruction);
        }
        for dependency_id in &requirements.hard_dependency_ids {
            if !self.dependency_satisfied(dependency_id) {
                by_check
                    .entry(ReadinessCheck::Dependencies)
                    .or_default()
                    .push(self.dependency_obstruction(item, dependency_id));
            }
        }
        for wait_id in &requirements.external_wait_ids {
            if !self.external_wait_resolved(wait_id) {
                by_check
                    .entry(ReadinessCheck::ExternalWaits)
                    .or_default()
                    .push(self.external_wait_obstruction(item, wait_id));
            }
        }
        for requirement_id in &requirements.evidence_requirement_ids {
            if !self.evidence_requirement_satisfied(requirement_id, &item.id) {
                by_check
                    .entry(ReadinessCheck::Evidence)
                    .or_default()
                    .push(self.evidence_obstruction(item, requirement_id));
            }
        }
        for requirement_id in &requirements.proof_requirement_ids {
            if !self.proof_requirement_satisfied(requirement_id, &item.id) {
                by_check
                    .entry(ReadinessCheck::Proof)
                    .or_default()
                    .push(self.proof_obstruction(item, requirement_id));
            }
        }
        for relation in self
            .graph
            .workflow_relations
            .iter()
            .filter(|relation| relation.relation_type == WorkflowRelationType::Contradicts)
            .filter(|relation| relation.from_id == item.id || relation.to_id == item.id)
        {
            by_check
                .entry(ReadinessCheck::Contradictions)
                .or_default()
                .push(self.contradiction_obstruction(item, &relation.id));
        }

        by_check
    }

    fn rule_results_for(
        &self,
        item: &WorkItem,
        by_check: &BTreeMap<ReadinessCheck, Vec<ObstructionRecord>>,
    ) -> Vec<ReadinessRuleResult> {
        let mut rule_results = by_check
            .iter()
            .map(|(check, records)| {
                self.rule_result(
                    item,
                    *check,
                    records.iter().map(|record| record.id.clone()).collect(),
                )
            })
            .collect::<Vec<_>>();
        if rule_results.is_empty() {
            rule_results.push(self.rule_result(item, ReadinessCheck::State, Vec::new()));
        }
        rule_results.sort_by(|left, right| left.id.cmp(&right.id));
        rule_results
    }

    fn state_obstruction(&self, item: &WorkItem) -> Option<ObstructionRecord> {
        let (obstruction_type, explanation, resolution, severity) = match item.state {
            WorkItemState::Blocked => (
                ObstructionType::ReviewRequired,
                format!("{} is explicitly in blocked state.", item.id),
                "Review the blocking condition or move the work item out of blocked state.",
                WorkflowSeverity::High,
            ),
            WorkItemState::Waiting => (
                ObstructionType::ExternalWait,
                format!("{} is waiting for an external condition.", item.id),
                "Resolve the external wait or record an accepted wait-resolution witness.",
                WorkflowSeverity::Medium,
            ),
            WorkItemState::Failed => (
                ObstructionType::ReviewRequired,
                format!("{} is failed and needs review before readiness.", item.id),
                "Review the failure and add a follow-up task, proof, or accepted resolution.",
                WorkflowSeverity::High,
            ),
            WorkItemState::Rejected => (
                ObstructionType::ReviewRequired,
                format!("{} is rejected and cannot be treated as ready.", item.id),
                "Create a reviewed replacement or explicitly accept a revised item.",
                WorkflowSeverity::High,
            ),
            WorkItemState::Cancelled => (
                ObstructionType::ReviewRequired,
                format!("{} is cancelled and cannot be treated as ready.", item.id),
                "Create a replacement work item if the workflow still requires this structure.",
                WorkflowSeverity::Medium,
            ),
            _ => return None,
        };

        Some(obstruction(ObstructionDraft {
            id: obstruction_id(obstruction_type, &item.id, &item.id),
            obstruction_type,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: default_constraint_id(ReadinessCheck::State),
            witness_ids: vec![item.id.clone()],
            explanation,
            severity,
            required_resolution: resolution.to_owned(),
            blocking: true,
        }))
    }

    fn dependency_satisfied(&self, dependency_id: &Id) -> bool {
        self.work_items
            .get(dependency_id.as_str())
            .is_some_and(|item| terminal_success_state(item.state))
    }

    fn external_wait_resolved(&self, wait_id: &Id) -> bool {
        self.work_items
            .get(wait_id.as_str())
            .is_some_and(|item| terminal_success_state(item.state))
            || self
                .acceptable_evidence_for(wait_id, wait_id)
                .next()
                .is_some()
    }

    fn evidence_requirement_satisfied(&self, requirement_id: &Id, item_id: &Id) -> bool {
        self.acceptable_evidence_for(requirement_id, item_id)
            .next()
            .is_some()
    }

    fn proof_requirement_satisfied(&self, requirement_id: &Id, item_id: &Id) -> bool {
        self.acceptable_evidence_for(requirement_id, item_id)
            .next()
            .is_some()
            || self
                .work_items
                .get(requirement_id.as_str())
                .is_some_and(|item| terminal_success_state(item.state))
    }

    fn acceptable_evidence_for(
        &'a self,
        requirement_id: &'a Id,
        item_id: &'a Id,
    ) -> impl Iterator<Item = &'a EvidenceRecord> + 'a {
        self.graph
            .evidence_records
            .iter()
            .filter(|record| acceptable_evidence(record))
            .filter(move |record| {
                record.id == *requirement_id
                    || record.supports_ids.contains(requirement_id)
                    || record.supports_ids.contains(item_id)
            })
    }

    fn dependency_obstruction(&self, item: &WorkItem, dependency_id: &Id) -> ObstructionRecord {
        let explanation = if self.work_items.contains_key(dependency_id.as_str()) {
            format!(
                "{} depends on {}, which has not reached done or accepted state.",
                item.id, dependency_id
            )
        } else {
            format!(
                "{} depends on missing work item {}.",
                item.id, dependency_id
            )
        };
        obstruction(ObstructionDraft {
            id: obstruction_id(
                ObstructionType::UnresolvedDependency,
                &item.id,
                dependency_id,
            ),
            obstruction_type: ObstructionType::UnresolvedDependency,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: source_constraint_id(
                self.rule_for(ReadinessCheck::Dependencies),
                ReadinessCheck::Dependencies,
            ),
            witness_ids: vec![dependency_id.clone()],
            explanation,
            severity: severity_for(
                self.rule_for(ReadinessCheck::Dependencies),
                WorkflowSeverity::High,
            ),
            required_resolution: "Complete, accept, or add the required dependency work item."
                .to_owned(),
            blocking: true,
        })
    }

    fn external_wait_obstruction(&self, item: &WorkItem, wait_id: &Id) -> ObstructionRecord {
        obstruction(ObstructionDraft {
            id: obstruction_id(ObstructionType::ExternalWait, &item.id, wait_id),
            obstruction_type: ObstructionType::ExternalWait,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: source_constraint_id(
                self.rule_for(ReadinessCheck::ExternalWaits),
                ReadinessCheck::ExternalWaits,
            ),
            witness_ids: vec![wait_id.clone()],
            explanation: format!(
                "{} is waiting on unresolved external wait {}.",
                item.id, wait_id
            ),
            severity: severity_for(
                self.rule_for(ReadinessCheck::ExternalWaits),
                WorkflowSeverity::Medium,
            ),
            required_resolution: "Record accepted evidence that the external wait has resolved."
                .to_owned(),
            blocking: true,
        })
    }

    fn evidence_obstruction(&self, item: &WorkItem, requirement_id: &Id) -> ObstructionRecord {
        let witness_ids = witness_ids_for_requirement(self.graph, requirement_id, &item.id);
        obstruction(ObstructionDraft {
            id: obstruction_id(ObstructionType::MissingEvidence, &item.id, requirement_id),
            obstruction_type: ObstructionType::MissingEvidence,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: source_constraint_id(
                self.rule_for(ReadinessCheck::Evidence),
                ReadinessCheck::Evidence,
            ),
            witness_ids,
            explanation: format!(
                "{} requires source-backed or accepted evidence {}, but none is available.",
                item.id, requirement_id
            ),
            severity: severity_for(
                self.rule_for(ReadinessCheck::Evidence),
                WorkflowSeverity::Medium,
            ),
            required_resolution:
                "Attach source-backed evidence or promote reviewed evidence for this requirement."
                    .to_owned(),
            blocking: true,
        })
    }

    fn proof_obstruction(&self, item: &WorkItem, requirement_id: &Id) -> ObstructionRecord {
        let witness_ids = witness_ids_for_requirement(self.graph, requirement_id, &item.id);
        obstruction(ObstructionDraft {
            id: obstruction_id(ObstructionType::MissingProof, &item.id, requirement_id),
            obstruction_type: ObstructionType::MissingProof,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: source_constraint_id(
                self.rule_for(ReadinessCheck::Proof),
                ReadinessCheck::Proof,
            ),
            witness_ids,
            explanation: format!(
                "{} requires proof {}, but no accepted proof or completed proof item is available.",
                item.id, requirement_id
            ),
            severity: severity_for(
                self.rule_for(ReadinessCheck::Proof),
                WorkflowSeverity::Medium,
            ),
            required_resolution: "Complete the proof work item or attach accepted proof evidence."
                .to_owned(),
            blocking: true,
        })
    }

    fn contradiction_obstruction(&self, item: &WorkItem, relation_id: &Id) -> ObstructionRecord {
        obstruction(ObstructionDraft {
            id: obstruction_id(ObstructionType::Contradiction, &item.id, relation_id),
            obstruction_type: ObstructionType::Contradiction,
            affected_ids: vec![item.id.clone()],
            source_constraint_id: Id::new("constraint:workflow-contradiction").expect("static id"),
            witness_ids: vec![relation_id.clone()],
            explanation: format!(
                "{} participates in contradictory workflow relation {}.",
                item.id, relation_id
            ),
            severity: WorkflowSeverity::High,
            required_resolution:
                "Resolve or review the contradictory relation before treating this item as ready."
                    .to_owned(),
            blocking: true,
        })
    }

    fn rule_result(
        &self,
        item: &WorkItem,
        check: ReadinessCheck,
        obstruction_ids: Vec<Id>,
    ) -> ReadinessRuleResult {
        let rule_id = self
            .rule_for(check)
            .map(|rule| rule.id.clone())
            .unwrap_or_else(|| default_rule_id(check));
        ReadinessRuleResult {
            id: Id::new(format!(
                "readiness_result:{}:{}",
                sanitize(item.id.as_str()),
                sanitize(rule_id.as_str())
            ))
            .expect("generated readiness result id"),
            rule_id,
            target_work_item_id: item.id.clone(),
            ready: obstruction_ids.is_empty(),
            obstruction_ids,
        }
    }

    fn rule_for(&self, check: ReadinessCheck) -> Option<&ReadinessRule> {
        self.graph
            .readiness_rules
            .iter()
            .find(|rule| rule_type_for_check(check) == Some(rule.rule_type))
    }
}

fn collect_requirement_ids(
    graph: &WorkflowCaseGraph,
    item: &WorkItem,
    relation_type: WorkflowRelationType,
) -> Vec<Id> {
    let mut ids = match relation_type {
        WorkflowRelationType::DependsOn => item.hard_dependency_ids.clone(),
        WorkflowRelationType::WaitsFor => item.external_wait_ids.clone(),
        WorkflowRelationType::RequiresEvidence => item.evidence_requirement_ids.clone(),
        WorkflowRelationType::RequiresProof => item.proof_requirement_ids.clone(),
        _ => Vec::new(),
    };
    ids.extend(
        graph
            .workflow_relations
            .iter()
            .filter(|relation| {
                relation.from_id == item.id && relation.relation_type == relation_type
            })
            .map(|relation| relation.to_id.clone()),
    );
    dedupe_ids(ids)
}

fn sorted_obstructions(
    by_check: &BTreeMap<ReadinessCheck, Vec<ObstructionRecord>>,
) -> Vec<ObstructionRecord> {
    let mut obstructions = by_check
        .values()
        .flat_map(|records| records.iter().cloned())
        .collect::<Vec<_>>();
    obstructions.sort_by(|left, right| left.id.cmp(&right.id));
    obstructions
}

fn readiness_result(graph: &WorkflowCaseGraph, results: &[ItemEvaluation]) -> ReadinessResult {
    let evaluated_work_item_ids = graph
        .work_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let ready_item_ids = results
        .iter()
        .filter(|result| {
            result
                .obstructions
                .iter()
                .all(|obstruction| !obstruction.blocking)
        })
        .map(|result| result.item_id.clone())
        .collect::<Vec<_>>();
    let not_ready_items = results
        .iter()
        .filter(|result| {
            result
                .obstructions
                .iter()
                .any(|obstruction| obstruction.blocking)
        })
        .map(|result| NotReadyItem {
            work_item_id: result.item_id.clone(),
            state: state_text(result.state),
            hard_dependency_ids: result.hard_dependency_ids.clone(),
            external_wait_ids: result.external_wait_ids.clone(),
            evidence_requirement_ids: result.evidence_requirement_ids.clone(),
            proof_requirement_ids: result.proof_requirement_ids.clone(),
            obstruction_ids: result
                .obstructions
                .iter()
                .filter(|obstruction| obstruction.blocking)
                .map(|obstruction| obstruction.id.clone())
                .collect(),
        })
        .collect::<Vec<_>>();
    let rule_results = results
        .iter()
        .flat_map(|result| result.rule_results.iter().cloned())
        .collect::<Vec<_>>();

    ReadinessResult {
        evaluated_work_item_ids,
        ready_item_ids,
        not_ready_items,
        rule_results,
    }
}

fn merge_obstructions(results: &[ItemEvaluation]) -> Vec<ObstructionRecord> {
    let mut by_id = BTreeMap::new();
    for obstruction in results
        .iter()
        .flat_map(|result| result.obstructions.iter().cloned())
    {
        by_id.entry(obstruction.id.clone()).or_insert(obstruction);
    }
    by_id.into_values().collect()
}

fn terminal_success_state(state: WorkItemState) -> bool {
    matches!(state, WorkItemState::Done | WorkItemState::Accepted)
}

fn witness_ids_for_requirement(
    graph: &WorkflowCaseGraph,
    requirement_id: &Id,
    item_id: &Id,
) -> Vec<Id> {
    let mut ids = graph
        .evidence_records
        .iter()
        .filter(|record| {
            record.id == *requirement_id
                || record.supports_ids.contains(requirement_id)
                || record.supports_ids.contains(item_id)
        })
        .map(|record| record.id.clone())
        .collect::<Vec<_>>();
    if ids.is_empty() {
        ids.push(requirement_id.clone());
    }
    dedupe_ids(ids)
}

struct ObstructionDraft {
    id: Id,
    obstruction_type: ObstructionType,
    affected_ids: Vec<Id>,
    source_constraint_id: Id,
    witness_ids: Vec<Id>,
    explanation: String,
    severity: WorkflowSeverity,
    required_resolution: String,
    blocking: bool,
}

fn obstruction(draft: ObstructionDraft) -> ObstructionRecord {
    ObstructionRecord {
        id: draft.id,
        obstruction_type: draft.obstruction_type,
        affected_ids: dedupe_ids(draft.affected_ids),
        source_constraint_id: draft.source_constraint_id,
        witness_ids: dedupe_ids(draft.witness_ids),
        explanation: draft.explanation,
        severity: draft.severity,
        required_resolution: draft.required_resolution,
        blocking: draft.blocking,
        provenance: generated_provenance("Workflow readiness obstruction", 0.84),
    }
}

fn obstruction_id(obstruction_type: ObstructionType, item_id: &Id, witness_id: &Id) -> Id {
    Id::new(format!(
        "obstruction:{}:{}:{}",
        obstruction_type_stem(obstruction_type),
        sanitize(item_id.as_str()),
        sanitize(witness_id.as_str())
    ))
    .expect("generated obstruction id")
}

fn source_constraint_id(rule: Option<&ReadinessRule>, check: ReadinessCheck) -> Id {
    rule.and_then(|rule| rule.source_constraint_id.clone())
        .unwrap_or_else(|| default_constraint_id(check))
}

fn severity_for(rule: Option<&ReadinessRule>, fallback: WorkflowSeverity) -> WorkflowSeverity {
    rule.map(|rule| rule.severity).unwrap_or(fallback)
}

fn default_rule_id(check: ReadinessCheck) -> Id {
    let id = match check {
        ReadinessCheck::State => "readiness:state-allows-work",
        ReadinessCheck::Dependencies => "readiness:dependencies-done",
        ReadinessCheck::ExternalWaits => "readiness:external-waits-resolved",
        ReadinessCheck::Evidence => "readiness:evidence-available",
        ReadinessCheck::Proof => "readiness:proof-available",
        ReadinessCheck::Contradictions => "readiness:no-contradictions",
    };
    Id::new(id).expect("static id")
}

fn default_constraint_id(check: ReadinessCheck) -> Id {
    let id = match check {
        ReadinessCheck::State => "constraint:workflow-state",
        ReadinessCheck::Dependencies => "constraint:workflow-dependencies",
        ReadinessCheck::ExternalWaits => "constraint:workflow-external-waits",
        ReadinessCheck::Evidence => "constraint:workflow-evidence",
        ReadinessCheck::Proof => "constraint:workflow-proof",
        ReadinessCheck::Contradictions => "constraint:workflow-contradiction",
    };
    Id::new(id).expect("static id")
}

fn rule_type_for_check(check: ReadinessCheck) -> Option<ReadinessRuleType> {
    match check {
        ReadinessCheck::Dependencies => Some(ReadinessRuleType::DependencyClosure),
        ReadinessCheck::ExternalWaits => Some(ReadinessRuleType::ExternalWaitResolved),
        ReadinessCheck::Evidence => Some(ReadinessRuleType::EvidenceAvailable),
        ReadinessCheck::Proof => Some(ReadinessRuleType::ProofAvailable),
        ReadinessCheck::State => Some(ReadinessRuleType::ObstructionAbsent),
        ReadinessCheck::Contradictions => Some(ReadinessRuleType::ObstructionAbsent),
    }
}

fn generated_provenance(title: &str, value: f64) -> WorkflowProvenance {
    WorkflowProvenance {
        source: crate::workflow_model::WorkflowSourceRef {
            kind: "agent_inference".to_owned(),
            uri: None,
            title: Some(title.to_owned()),
            captured_at: None,
            source_local_id: None,
        },
        confidence: confidence(value),
        review_status: ReviewStatus::Unreviewed,
        recorded_at: None,
        actor_id: None,
        extraction_method: Some("casegraphen.workflow_eval.v1".to_owned()),
    }
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("static confidence")
}

fn dedupe_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

fn state_text(state: WorkItemState) -> String {
    serde_json::to_value(state)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn obstruction_type_stem(obstruction_type: ObstructionType) -> &'static str {
    match obstruction_type {
        ObstructionType::UnresolvedDependency => "unresolved-dependency",
        ObstructionType::ExternalWait => "external-wait",
        ObstructionType::MissingEvidence => "missing-evidence",
        ObstructionType::MissingProof => "missing-proof",
        ObstructionType::InvalidTransition => "invalid-transition",
        ObstructionType::Contradiction => "contradiction",
        ObstructionType::ImpossibleClosure => "impossible-closure",
        ObstructionType::ProjectionLoss => "projection-loss",
        ObstructionType::CorrespondenceMismatch => "correspondence-mismatch",
        ObstructionType::ReviewRequired => "review-required",
    }
}
