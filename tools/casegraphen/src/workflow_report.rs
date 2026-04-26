use crate::workflow_eval::{
    evaluate_workflow, projection_profile_for, CompletionCandidate, CorrespondenceResult,
    EvidenceFinding, ObstructionRecord, ProjectionResult, ReadinessRuleResult, WorkflowEvaluation,
};
use crate::workflow_model::{
    InformationLoss, ProjectionAudience, ProjectionProfile, WorkItem, WorkflowCaseGraph,
    WorkflowProvenance, WORKFLOW_GRAPH_SCHEMA,
};
use higher_graphen_core::{Confidence, Id, ReviewStatus};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};

pub const WORKFLOW_REASONING_REPORT_SCHEMA: &str = "highergraphen.case.workflow.report.v1";
pub const WORKFLOW_REASONING_REPORT_TYPE: &str = "case_workflow_reasoning";
pub const WORKFLOW_REASONING_REPORT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReasoningReport {
    pub schema: String,
    pub report_type: String,
    pub report_version: u32,
    pub metadata: WorkflowReportMetadata,
    pub input: WorkflowReportInput,
    pub result: WorkflowEvaluation,
    pub projection: ProjectionBundle,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowOperationReport {
    pub schema: String,
    pub report_type: String,
    pub report_version: u32,
    pub metadata: WorkflowReportMetadata,
    pub input: Value,
    pub result: Value,
    pub projection: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReportMetadata {
    pub command: String,
    pub tool_package: String,
    pub core_packages: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReportInput {
    pub workflow_graph_schema: String,
    pub workflow_graph_id: Id,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub projection_profile_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionBundle {
    pub human_review: ProjectionView,
    pub ai_view: ProjectionView,
    pub audit_trace: ProjectionView,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionView {
    pub audience: ProjectionAudience,
    pub purpose: String,
    pub summary: String,
    pub records: Vec<ProjectionRecord>,
    pub source_ids: Vec<Id>,
    pub information_loss: Vec<InformationLoss>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionRecord {
    pub id: Id,
    pub record_type: ProjectionRecordType,
    pub summary: String,
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
    pub review_status: ReviewStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<crate::workflow_model::WorkflowSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<WorkflowProvenance>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionRecordType {
    WorkItem,
    Readiness,
    Obstruction,
    CompletionCandidate,
    EvidenceFinding,
    Projection,
    Correspondence,
    Evolution,
}

pub fn reason_workflow(graph: &WorkflowCaseGraph) -> WorkflowReasoningReport {
    workflow_reasoning_report(
        "casegraphen workflow reason",
        graph,
        evaluate_workflow(graph),
    )
}

pub fn workflow_reasoning_report(
    command: &str,
    graph: &WorkflowCaseGraph,
    result: WorkflowEvaluation,
) -> WorkflowReasoningReport {
    let projection = projection_bundle(graph, &result);
    WorkflowReasoningReport {
        schema: WORKFLOW_REASONING_REPORT_SCHEMA.to_owned(),
        report_type: WORKFLOW_REASONING_REPORT_TYPE.to_owned(),
        report_version: WORKFLOW_REASONING_REPORT_VERSION,
        metadata: workflow_metadata(command),
        input: workflow_report_input(graph),
        result,
        projection,
    }
}

pub fn workflow_operation_report(
    command: &str,
    operation: &str,
    input: Value,
    result: Value,
    projection: Value,
) -> WorkflowOperationReport {
    WorkflowOperationReport {
        schema: format!("highergraphen.case.workflow.{operation}.report.v1"),
        report_type: format!("case_workflow_{operation}"),
        report_version: WORKFLOW_REASONING_REPORT_VERSION,
        metadata: workflow_metadata(command),
        input,
        result,
        projection,
    }
}

pub fn workflow_input_with_paths(
    graph: &WorkflowCaseGraph,
    input_path: Option<&Path>,
    projection_path: Option<&Path>,
) -> Value {
    let mut value =
        serde_json::to_value(workflow_report_input(graph)).expect("workflow input serializes");
    if let Value::Object(object) = &mut value {
        if let Some(path) = input_path {
            object.insert("path".to_owned(), json!(path.display().to_string()));
        }
        if let Some(path) = projection_path {
            object.insert("projection".to_owned(), json!(path.display().to_string()));
        }
    }
    value
}

pub fn validation_projection(valid: bool, violation_count: usize) -> Value {
    json!({
        "human_review": {
            "summary": if valid { "Workflow graph validation passed." } else { "Workflow graph validation failed." },
            "violation_count": violation_count
        },
        "ai_view": { "valid": valid, "violation_count": violation_count },
        "audit_trace": {
            "source_ids": [],
            "information_loss": ["Validation reports schema and semantic violations without running workflow reasoning."]
        }
    })
}

pub fn focused_projection(graph: &WorkflowCaseGraph, operation: &str) -> Value {
    json!({
        "human_review": {
            "summary": format!("Focused workflow {operation} report."),
            "workflow_graph_id": graph.workflow_graph_id
        },
        "ai_view": {
            "section": operation,
            "workflow_graph_id": graph.workflow_graph_id,
            "case_graph_id": graph.case_graph_id
        },
        "audit_trace": {
            "source_ids": graph.projection_profiles.iter().flat_map(|profile| profile.source_ids.iter()).collect::<Vec<_>>(),
            "information_loss": ["Focused report contains the requested section; use workflow reason for the aggregate projection."]
        }
    })
}

fn workflow_metadata(command: &str) -> WorkflowReportMetadata {
    WorkflowReportMetadata {
        command: command.to_owned(),
        tool_package: "tools/casegraphen".to_owned(),
        core_packages: vec![
            "higher-graphen-core".to_owned(),
            "higher-graphen-space".to_owned(),
            "higher-graphen-projection".to_owned(),
        ],
    }
}

fn workflow_report_input(graph: &WorkflowCaseGraph) -> WorkflowReportInput {
    WorkflowReportInput {
        workflow_graph_schema: WORKFLOW_GRAPH_SCHEMA.to_owned(),
        workflow_graph_id: graph.workflow_graph_id.clone(),
        case_graph_id: graph.case_graph_id.clone(),
        space_id: graph.space_id.clone(),
        projection_profile_ids: graph
            .projection_profiles
            .iter()
            .map(|profile| profile.id.clone())
            .collect(),
    }
}

fn projection_bundle(graph: &WorkflowCaseGraph, result: &WorkflowEvaluation) -> ProjectionBundle {
    ProjectionBundle {
        human_review: projection_view(graph, result, ProjectionAudience::HumanReview),
        ai_view: projection_view(graph, result, ProjectionAudience::AiAgent),
        audit_trace: projection_view(graph, result, ProjectionAudience::Audit),
    }
}

fn projection_view(
    graph: &WorkflowCaseGraph,
    result: &WorkflowEvaluation,
    audience: ProjectionAudience,
) -> ProjectionView {
    let profile = projection_profile_for(graph, audience);
    let records = projection_records(graph, result, audience);
    let source_ids = source_ids(profile, &records);
    ProjectionView {
        audience,
        purpose: profile
            .map(|profile| profile.purpose.clone())
            .unwrap_or_else(|| default_purpose(audience).to_owned()),
        summary: projection_summary(result, audience),
        records,
        source_ids,
        information_loss: projection_loss(graph, result, profile, audience),
    }
}

fn projection_records(
    graph: &WorkflowCaseGraph,
    result: &WorkflowEvaluation,
    audience: ProjectionAudience,
) -> Vec<ProjectionRecord> {
    match audience {
        ProjectionAudience::HumanReview => human_records(graph, result),
        ProjectionAudience::AiAgent | ProjectionAudience::System => ai_records(graph, result),
        ProjectionAudience::Audit => audit_records(graph, result),
    }
}

fn human_records(graph: &WorkflowCaseGraph, result: &WorkflowEvaluation) -> Vec<ProjectionRecord> {
    let mut records = graph
        .work_items
        .iter()
        .map(work_item_record)
        .collect::<Vec<_>>();
    records.extend(
        result
            .obstructions
            .iter()
            .filter(|record| record.blocking)
            .map(obstruction_record),
    );
    records
}

fn ai_records(graph: &WorkflowCaseGraph, result: &WorkflowEvaluation) -> Vec<ProjectionRecord> {
    let mut records = graph
        .work_items
        .iter()
        .map(work_item_record)
        .collect::<Vec<_>>();
    records.extend(result.readiness.rule_results.iter().map(readiness_record));
    records.extend(result.obstructions.iter().map(obstruction_record));
    records.extend(
        result
            .completion_candidates
            .iter()
            .map(completion_candidate_record),
    );
    records.extend(
        result
            .evidence_findings
            .findings
            .iter()
            .map(evidence_finding_record),
    );
    records.extend(result.correspondence.iter().map(correspondence_record));
    records.push(evolution_record(result));
    records.push(projection_result_record(&result.projection));
    records
}

fn audit_records(graph: &WorkflowCaseGraph, result: &WorkflowEvaluation) -> Vec<ProjectionRecord> {
    let mut records = graph
        .evidence_records
        .iter()
        .map(|evidence| ProjectionRecord {
            id: evidence.id.clone(),
            record_type: ProjectionRecordType::EvidenceFinding,
            summary: evidence.summary.clone(),
            source_ids: evidence.source_ids.clone(),
            confidence: Some(evidence.provenance.confidence),
            review_status: evidence.provenance.review_status,
            severity: None,
            provenance: Some(evidence.provenance.clone()),
        })
        .collect::<Vec<_>>();
    records.extend(graph.transition_records.iter().map(|transition| {
        ProjectionRecord {
            id: transition.id.clone(),
            record_type: ProjectionRecordType::Evolution,
            summary: format!(
                "{:?} transition from {} to {}.",
                transition.transition_type, transition.from_revision_id, transition.to_revision_id
            ),
            source_ids: transition.source_ids.clone(),
            confidence: Some(transition.provenance.confidence),
            review_status: transition.provenance.review_status,
            severity: (!transition.violated_invariant_ids.is_empty())
                .then_some(crate::workflow_model::WorkflowSeverity::High),
            provenance: Some(transition.provenance.clone()),
        }
    }));
    records.push(projection_result_record(&result.projection));
    records
}

fn work_item_record(item: &WorkItem) -> ProjectionRecord {
    ProjectionRecord {
        id: item.id.clone(),
        record_type: ProjectionRecordType::WorkItem,
        summary: item.title.clone(),
        source_ids: item.source_ids.clone(),
        confidence: Some(item.provenance.confidence),
        review_status: item.provenance.review_status,
        severity: None,
        provenance: Some(item.provenance.clone()),
    }
}

fn readiness_record(record: &ReadinessRuleResult) -> ProjectionRecord {
    ProjectionRecord {
        id: record.id.clone(),
        record_type: ProjectionRecordType::Readiness,
        summary: if record.ready {
            format!(
                "{} satisfied {}.",
                record.target_work_item_id, record.rule_id
            )
        } else {
            format!(
                "{} failed {} with {} obstruction(s).",
                record.target_work_item_id,
                record.rule_id,
                record.obstruction_ids.len()
            )
        },
        source_ids: record
            .obstruction_ids
            .iter()
            .chain(std::iter::once(&record.target_work_item_id))
            .cloned()
            .collect(),
        confidence: Some(confidence(0.84)),
        review_status: ReviewStatus::Unreviewed,
        severity: (!record.ready).then_some(crate::workflow_model::WorkflowSeverity::Medium),
        provenance: None,
    }
}

fn obstruction_record(record: &ObstructionRecord) -> ProjectionRecord {
    ProjectionRecord {
        id: record.id.clone(),
        record_type: ProjectionRecordType::Obstruction,
        summary: record.explanation.clone(),
        source_ids: record
            .affected_ids
            .iter()
            .chain(&record.witness_ids)
            .cloned()
            .collect(),
        confidence: Some(record.provenance.confidence),
        review_status: record.provenance.review_status,
        severity: Some(record.severity),
        provenance: Some(record.provenance.clone()),
    }
}

fn completion_candidate_record(candidate: &CompletionCandidate) -> ProjectionRecord {
    ProjectionRecord {
        id: candidate.id.clone(),
        record_type: ProjectionRecordType::CompletionCandidate,
        summary: candidate.rationale.clone(),
        source_ids: candidate
            .target_ids
            .iter()
            .chain(&candidate.inferred_from)
            .cloned()
            .collect(),
        confidence: Some(candidate.confidence),
        review_status: candidate.review_status,
        severity: None,
        provenance: Some(candidate.provenance.clone()),
    }
}

fn evidence_finding_record(finding: &EvidenceFinding) -> ProjectionRecord {
    ProjectionRecord {
        id: finding.id.clone(),
        record_type: ProjectionRecordType::EvidenceFinding,
        summary: finding.summary.clone(),
        source_ids: finding.evidence_ids.clone(),
        confidence: Some(confidence(0.8)),
        review_status: finding.review_status,
        severity: None,
        provenance: None,
    }
}

fn correspondence_record(record: &CorrespondenceResult) -> ProjectionRecord {
    ProjectionRecord {
        id: record.id.clone(),
        record_type: ProjectionRecordType::Correspondence,
        summary: format!(
            "{:?} correspondence between {} left and {} right identifier(s).",
            record.correspondence_type,
            record.left_ids.len(),
            record.right_ids.len()
        ),
        source_ids: record
            .mismatch_evidence_ids
            .iter()
            .chain(&record.transferable_pattern_ids)
            .cloned()
            .collect(),
        confidence: Some(record.confidence),
        review_status: record.review_status,
        severity: None,
        provenance: None,
    }
}

fn evolution_record(result: &WorkflowEvaluation) -> ProjectionRecord {
    ProjectionRecord {
        id: result.evolution.revision_id.clone(),
        record_type: ProjectionRecordType::Evolution,
        summary: format!(
            "{} transition(s), {} appeared obstruction(s), {} persisted shape id(s).",
            result.evolution.transition_ids.len(),
            result.evolution.appeared_obstruction_ids.len(),
            result.evolution.persisted_shape_ids.len()
        ),
        source_ids: result
            .evolution
            .transition_ids
            .iter()
            .chain(&result.evolution.persisted_shape_ids)
            .cloned()
            .collect(),
        confidence: Some(confidence(0.82)),
        review_status: ReviewStatus::Unreviewed,
        severity: (!result.evolution.invariant_breaks.is_empty())
            .then_some(crate::workflow_model::WorkflowSeverity::High),
        provenance: None,
    }
}

fn projection_result_record(result: &ProjectionResult) -> ProjectionRecord {
    ProjectionRecord {
        id: result.projection_profile_id.clone(),
        record_type: ProjectionRecordType::Projection,
        summary: format!(
            "{:?} projection represents {} id(s) and omits {} id(s).",
            result.audience,
            result.represented_ids.len(),
            result.omitted_ids.len()
        ),
        source_ids: result
            .represented_ids
            .iter()
            .chain(&result.omitted_ids)
            .cloned()
            .collect(),
        confidence: Some(confidence(0.86)),
        review_status: ReviewStatus::Unreviewed,
        severity: (!result.omitted_ids.is_empty())
            .then_some(crate::workflow_model::WorkflowSeverity::Info),
        provenance: None,
    }
}

fn projection_summary(result: &WorkflowEvaluation, audience: ProjectionAudience) -> String {
    match audience {
        ProjectionAudience::HumanReview => format!(
            "{} ready item(s), {} not-ready item(s), {} obstruction(s).",
            result.readiness.ready_item_ids.len(),
            result.readiness.not_ready_items.len(),
            result.obstructions.len()
        ),
        ProjectionAudience::AiAgent => {
            "Machine view preserves readiness, obstruction, completion, evidence, correspondence, and evolution identifiers.".to_owned()
        }
        ProjectionAudience::Audit => {
            "Audit view preserves evidence boundary, transition, and projection loss identifiers.".to_owned()
        }
        ProjectionAudience::System => "System projection over workflow reasoning records.".to_owned(),
    }
}

fn projection_loss(
    graph: &WorkflowCaseGraph,
    result: &WorkflowEvaluation,
    profile: Option<&ProjectionProfile>,
    audience: ProjectionAudience,
) -> Vec<InformationLoss> {
    if let Some(profile) = profile {
        if !profile.information_loss.is_empty() {
            return profile.information_loss.clone();
        }
    }
    vec![InformationLoss {
        description: format!(
            "{:?} projection omits full source payloads and preserves stable workflow identifiers.",
            audience
        ),
        represented_ids: result.projection.represented_ids.clone(),
        omitted_ids: graph
            .work_items
            .iter()
            .flat_map(|item| item.source_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    }]
}

fn source_ids(profile: Option<&ProjectionProfile>, records: &[ProjectionRecord]) -> Vec<Id> {
    let mut ids = profile
        .map(|profile| profile.included_ids.clone())
        .unwrap_or_default();
    ids.extend(records.iter().map(|record| record.id.clone()));
    ids.into_iter()
        .chain(
            records
                .iter()
                .flat_map(|record| record.source_ids.iter().cloned()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn default_purpose(audience: ProjectionAudience) -> &'static str {
    match audience {
        ProjectionAudience::HumanReview => "workflow_human_review",
        ProjectionAudience::AiAgent => "workflow_ai_reasoning",
        ProjectionAudience::Audit => "workflow_audit_trace",
        ProjectionAudience::System => "workflow_system_projection",
    }
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("static confidence")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const WORKFLOW_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/workflow.graph.example.json");

    #[test]
    fn workflow_reasoning_report_matches_report_envelope() {
        let graph: WorkflowCaseGraph =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

        let report = reason_workflow(&graph);
        let value = serde_json::to_value(&report).expect("serialize workflow report");

        assert_eq!(value["schema"], json!(WORKFLOW_REASONING_REPORT_SCHEMA));
        assert_eq!(value["report_type"], json!(WORKFLOW_REASONING_REPORT_TYPE));
        assert_eq!(
            value["report_version"],
            json!(WORKFLOW_REASONING_REPORT_VERSION)
        );
        assert_eq!(
            value["input"]["workflow_graph_schema"],
            json!(WORKFLOW_GRAPH_SCHEMA)
        );
        assert!(value["result"]["readiness"]["not_ready_items"].is_array());
        assert!(value["result"]["obstructions"].is_array());
        assert!(value["result"]["completion_candidates"].is_array());
        assert!(value["result"]["evidence_findings"]["findings"].is_array());
        assert!(value["result"]["projection"]["information_loss"].is_array());
        assert!(value["result"]["correspondence"].is_array());
        assert!(value["result"]["evolution"]["transition_ids"].is_array());
    }

    #[test]
    fn workflow_reasoning_report_emits_projection_views() {
        let graph: WorkflowCaseGraph =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");

        let report = reason_workflow(&graph);
        let value = serde_json::to_value(&report).expect("serialize workflow report");

        assert_eq!(
            value["projection"]["human_review"]["audience"],
            json!("human_review")
        );
        assert_eq!(
            value["projection"]["ai_view"]["audience"],
            json!("ai_agent")
        );
        assert_eq!(
            value["projection"]["audit_trace"]["audience"],
            json!("audit")
        );
        assert!(value["projection"]["ai_view"]["records"].is_array());
        assert!(value["projection"]["audit_trace"]["information_loss"].is_array());
        let round_trip: WorkflowReasoningReport =
            serde_json::from_value(value).expect("deserialize report");
        assert_eq!(round_trip.schema, WORKFLOW_REASONING_REPORT_SCHEMA);
    }
}
