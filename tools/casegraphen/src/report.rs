use crate::eval::{
    compare_graphs, detect_conflicts, detect_missing_cases, evaluate_coverage, graph_counts,
    projection_result, source_ids, BoundaryCoverage, CompareResult, CoverageEvaluation,
    ProjectionResult, ValidationResult,
};
use crate::model::{id_text, CaseGraph, ConflictingCase, CoveragePolicy, MissingCase};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OperationReport<T: Serialize> {
    pub schema: String,
    pub report_type: String,
    pub report_version: u32,
    pub metadata: ReportMetadata,
    pub input: Value,
    pub result: T,
    pub projection: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReportMetadata {
    pub command: String,
    pub tool_package: String,
    pub core_packages: Vec<String>,
}

pub fn create_report(command: &str, path: &Path, graph: &CaseGraph) -> OperationReport<Value> {
    OperationReport {
        schema: schema("create"),
        report_type: "case_create".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({
            "case_graph_id": graph.case_graph_id,
            "space_id": graph.space_id,
            "store_path": path_display(path)
        }),
        result: json!({
            "created": true,
            "path": path_display(path),
            "counts": graph_counts(graph)
        }),
        projection: basic_projection(graph),
    }
}

pub fn inspect_report(command: &str, input: &Path, graph: &CaseGraph) -> OperationReport<Value> {
    OperationReport {
        schema: schema("inspect"),
        report_type: "case_inspect".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: path_input(input),
        result: json!({
            "case_graph_id": graph.case_graph_id,
            "space_id": graph.space_id,
            "counts": graph_counts(graph),
            "case_ids": graph.cases.iter().map(|case| &case.id).collect::<Vec<_>>(),
            "scenario_ids": graph.scenarios.iter().map(|scenario| &scenario.id).collect::<Vec<_>>(),
            "coverage_goal_ids": graph.coverage_goals.iter().map(|goal| &goal.id).collect::<Vec<_>>()
        }),
        projection: basic_projection(graph),
    }
}

pub fn list_report(command: &str, store: &Path, entries: Vec<Value>) -> OperationReport<Value> {
    let projected_entries = entries.clone();
    OperationReport {
        schema: schema("list"),
        report_type: "case_list".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({ "store": path_display(store) }),
        result: json!({
            "graph_count": entries.len(),
            "case_graphs": entries
        }),
        projection: json!({
            "human_review": { "summaries": [] },
            "ai_view": { "case_graphs": projected_entries },
            "audit_trace": {
                "source_ids": [],
                "information_loss": ["list reads graph headers and paths only"]
            }
        }),
    }
}

pub fn validate_report(
    command: &str,
    input: &Path,
    graph: &CaseGraph,
    result: ValidationResult,
) -> OperationReport<ValidationResult> {
    OperationReport {
        schema: schema("validate"),
        report_type: "case_validate".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: path_input(input),
        result,
        projection: basic_projection(graph),
    }
}

pub fn coverage_report(
    command: &str,
    input: &Path,
    policy: &Path,
    graph: &CaseGraph,
    result: CoverageEvaluation,
) -> OperationReport<CoverageEvaluation> {
    OperationReport {
        schema: schema("coverage"),
        report_type: "case_coverage".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({ "path": path_display(input), "coverage": path_display(policy) }),
        projection: projection_with_findings(graph, &[], &[], result.boundary_coverage.clone()),
        result,
    }
}

pub fn missing_report(
    command: &str,
    input: &Path,
    policy: &Path,
    graph: &CaseGraph,
    result: Vec<MissingCase>,
) -> OperationReport<Value> {
    let coverage = evaluate_coverage(graph, &CoveragePolicy::for_all_goals());
    let projected_missing = result.clone();
    OperationReport {
        schema: schema("missing"),
        report_type: "case_missing".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({ "path": path_display(input), "coverage": path_display(policy) }),
        result: json!({ "missing_cases": result }),
        projection: projection_with_findings(
            graph,
            &projected_missing,
            &[],
            coverage.boundary_coverage,
        ),
    }
}

pub fn conflicts_report(
    command: &str,
    input: &Path,
    graph: &CaseGraph,
    result: Vec<ConflictingCase>,
) -> OperationReport<Value> {
    let coverage = evaluate_coverage(graph, &CoveragePolicy::for_all_goals());
    let projected_conflicts = result.clone();
    OperationReport {
        schema: schema("conflicts"),
        report_type: "case_conflicts".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: path_input(input),
        result: json!({ "conflicts": result }),
        projection: projection_with_findings(
            graph,
            &[],
            &projected_conflicts,
            coverage.boundary_coverage,
        ),
    }
}

pub fn project_report(
    command: &str,
    input: &Path,
    projection: &Path,
    graph: &CaseGraph,
    result: ProjectionResult,
) -> OperationReport<ProjectionResult> {
    let coverage = evaluate_coverage(graph, &CoveragePolicy::for_all_goals());
    let missing = detect_missing_cases(graph, &CoveragePolicy::for_all_goals());
    let conflicts = detect_conflicts(graph);
    let mut projected =
        projection_with_findings(graph, &missing, &conflicts, coverage.boundary_coverage);
    projected["audit_trace"]["information_loss"] = json!(result.information_loss);

    OperationReport {
        schema: schema("project"),
        report_type: "case_project".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({ "path": path_display(input), "projection": path_display(projection) }),
        result,
        projection: projected,
    }
}

pub fn compare_report(
    command: &str,
    left: &Path,
    right: &Path,
    left_graph: &CaseGraph,
    right_graph: &CaseGraph,
) -> OperationReport<CompareResult> {
    OperationReport {
        schema: schema("compare"),
        report_type: "case_compare".to_owned(),
        report_version: 1,
        metadata: metadata(command),
        input: json!({ "left": path_display(left), "right": path_display(right) }),
        result: compare_graphs(left_graph, right_graph),
        projection: json!({
            "human_review": { "recommended_review_actions": ["review changed and conflicting case records"] },
            "ai_view": {
                "left_case_graph_id": left_graph.case_graph_id,
                "right_case_graph_id": right_graph.case_graph_id
            },
            "audit_trace": {
                "source_ids": source_ids(left_graph),
                "information_loss": ["compare reports case-level differences, not a full JSON patch"]
            }
        }),
    }
}

pub fn schema(operation: &str) -> String {
    format!("highergraphen.case.{operation}.report.v1")
}

pub fn metadata(command: &str) -> ReportMetadata {
    ReportMetadata {
        command: command.to_owned(),
        tool_package: "tools/casegraphen".to_owned(),
        core_packages: vec![
            "higher-graphen-core".to_owned(),
            "higher-graphen-space".to_owned(),
        ],
    }
}

pub fn basic_projection(graph: &CaseGraph) -> Value {
    projection_with_findings(
        graph,
        &[],
        &[],
        BoundaryCoverage {
            represented_context_ids: Vec::new(),
            uncovered_context_ids: Vec::new(),
            represented_boundary_ids: Vec::new(),
        },
    )
}

fn projection_with_findings(
    graph: &CaseGraph,
    missing: &[MissingCase],
    conflicts: &[ConflictingCase],
    boundary: BoundaryCoverage,
) -> Value {
    json!({
        "human_review": {
            "case_summaries": human_case_summaries(graph),
            "missing_case_prompts": missing_case_prompts(missing),
            "conflict_explanations": conflict_explanations(conflicts),
            "recommended_review_actions": review_actions(missing, conflicts)
        },
        "ai_view": {
            "cases": graph.cases,
            "scenarios": graph.scenarios,
            "coverage_goals": graph.coverage_goals,
            "missing_cases": missing,
            "conflicts": conflicts
        },
        "audit_trace": {
            "source_ids": source_ids(graph),
            "per_source_case_ids": per_source_case_ids(graph),
            "boundary_coverage": boundary,
            "information_loss": ["projection is a deterministic summary of the source graph"]
        }
    })
}

fn human_case_summaries(graph: &CaseGraph) -> Vec<Value> {
    graph
        .cases
        .iter()
        .map(|case| {
            json!({
                "case_id": case.id,
                "title": case.title,
                "case_type": case.case_type,
                "summary": case.situation_summary,
                "source_ids": case.source_ids
            })
        })
        .collect()
}

fn missing_case_prompts(missing: &[MissingCase]) -> Vec<Value> {
    missing
        .iter()
        .map(|case| {
            json!({
                "missing_case_id": case.id,
                "target_ids": case.target_ids,
                "severity": case.severity,
                "review_status": case.review_status,
                "rationale": case.rationale
            })
        })
        .collect()
}

fn conflict_explanations(conflicts: &[ConflictingCase]) -> Vec<Value> {
    conflicts
        .iter()
        .map(|conflict| {
            json!({
                "conflict_id": conflict.id,
                "case_ids": conflict.case_ids,
                "severity": conflict.severity,
                "explanation": conflict.explanation,
                "source_ids": conflict.source_ids
            })
        })
        .collect()
}

fn review_actions(missing: &[MissingCase], conflicts: &[ConflictingCase]) -> Vec<String> {
    let mut actions = Vec::new();
    if !missing.is_empty() {
        actions.push("review missing cases before accepting coverage as complete".to_owned());
    }
    if !conflicts.is_empty() {
        actions.push(
            "resolve conflicting cases before using projection as an accepted fact set".to_owned(),
        );
    }
    if actions.is_empty() {
        actions.push("no review action required by this projection".to_owned());
    }
    actions
}

fn per_source_case_ids(graph: &CaseGraph) -> BTreeMap<String, Vec<String>> {
    let mut cases_by_source: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for case_record in &graph.cases {
        for source_id in &case_record.source_ids {
            cases_by_source
                .entry(id_text(source_id))
                .or_default()
                .push(id_text(&case_record.id));
        }
    }
    cases_by_source
}

fn path_input(path: &Path) -> Value {
    json!({ "path": path_display(path) })
}

fn path_display(path: &Path) -> String {
    path.display().to_string()
}

impl CoveragePolicy {
    fn for_all_goals() -> Self {
        Self {
            schema: crate::model::COVERAGE_POLICY_SCHEMA.to_owned(),
            policy_id: higher_graphen_core::Id::new("coverage-policy:all").expect("static id"),
            coverage_goal_ids: Vec::new(),
            require_explicit_relations: false,
            metadata: serde_json::Map::new(),
        }
    }
}

pub fn operation_projection(graph: &CaseGraph) -> ProjectionResult {
    projection_result(graph)
}
