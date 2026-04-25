use crate::model::{
    id_text, CaseGraph, CaseRecord, CaseRelation, ConflictingCase, CoverageGoal, CoveragePolicy,
    MissingCase,
};
use higher_graphen_core::{
    Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind, SourceRef,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GraphCounts {
    pub cases: usize,
    pub scenarios: usize,
    pub coverage_goals: usize,
    pub relations: usize,
    pub review_records: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub location: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub counts: GraphCounts,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CoverageEvaluation {
    pub coverage_status: String,
    pub goals: Vec<GoalCoverage>,
    pub represented_ids: Vec<String>,
    pub uncovered_ids: Vec<String>,
    pub partially_covered_ids: Vec<String>,
    pub boundary_coverage: BoundaryCoverage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GoalCoverage {
    pub coverage_goal_id: Id,
    pub coverage_type: String,
    pub status: String,
    pub represented_ids: Vec<Id>,
    pub uncovered_ids: Vec<Id>,
    pub covering_case_ids: Vec<Id>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoundaryCoverage {
    pub represented_context_ids: Vec<Id>,
    pub uncovered_context_ids: Vec<Id>,
    pub represented_boundary_ids: Vec<Id>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectionResult {
    pub projection_result: String,
    pub selected_source_ids: Vec<Id>,
    pub omitted_source_ids: Vec<Id>,
    pub information_loss: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CompareResult {
    pub equivalent: bool,
    pub added_case_ids: Vec<Id>,
    pub removed_case_ids: Vec<Id>,
    pub changed_case_ids: Vec<Id>,
    pub conflicting_case_ids: Vec<Id>,
    pub not_comparable: Vec<String>,
}

pub fn validate_case_graph(graph: &CaseGraph) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    validate_duplicate_ids(graph, &mut errors);
    validate_space_membership(graph, &mut errors);
    validate_relation_endpoints(graph, &mut errors);
    validate_review_targets(graph, &mut errors);

    if graph.cases.is_empty() {
        warnings.push(issue("no_cases", "case graph contains no cases", "$.cases"));
    }
    if graph.coverage_goals.is_empty() {
        warnings.push(issue(
            "no_coverage_goals",
            "case graph contains no coverage goals",
            "$.coverage_goals",
        ));
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        counts: graph_counts(graph),
    }
}

pub fn evaluate_coverage(graph: &CaseGraph, policy: &CoveragePolicy) -> CoverageEvaluation {
    let represented = represented_ids(graph);
    let goals = selected_goals(graph, policy)
        .map(|goal| evaluate_goal(graph, goal, &represented, policy))
        .collect::<Vec<_>>();
    let uncovered_ids = collect_goal_ids(&goals, |goal| !goal.uncovered_ids.is_empty());
    let partially_covered_ids = collect_goal_ids(&goals, |goal| goal.status == "partial");
    let represented_ids = represented.iter().cloned().collect::<Vec<_>>();
    let coverage_status = coverage_status(&goals);

    CoverageEvaluation {
        coverage_status,
        goals,
        represented_ids,
        uncovered_ids,
        partially_covered_ids,
        boundary_coverage: boundary_coverage(graph, &represented),
    }
}

pub fn detect_missing_cases(graph: &CaseGraph, policy: &CoveragePolicy) -> Vec<MissingCase> {
    let coverage = evaluate_coverage(graph, policy);
    coverage
        .goals
        .iter()
        .flat_map(|goal| missing_for_goal(goal, graph))
        .collect()
}

pub fn detect_conflicts(graph: &CaseGraph) -> Vec<ConflictingCase> {
    let relation_conflicts = graph
        .relations
        .iter()
        .filter(|relation| relation.relation_type == "contradicts")
        .filter_map(|relation| conflict_from_relation(graph, relation));
    let outcome_conflicts = graph.cases.iter().flat_map(conflicts_from_outcomes);
    relation_conflicts.chain(outcome_conflicts).collect()
}

pub fn compare_graphs(left: &CaseGraph, right: &CaseGraph) -> CompareResult {
    let mut not_comparable = Vec::new();
    if left.space_id != right.space_id {
        not_comparable.push("space_id differs".to_owned());
    }

    let left_cases = cases_by_id(&left.cases);
    let right_cases = cases_by_id(&right.cases);
    let added_case_ids = diff_keys(&right_cases, &left_cases);
    let removed_case_ids = diff_keys(&left_cases, &right_cases);
    let changed_case_ids = changed_cases(&left_cases, &right_cases);
    let conflicting_case_ids = conflicting_changed_cases(&left_cases, &right_cases);
    let equivalent = added_case_ids.is_empty()
        && removed_case_ids.is_empty()
        && changed_case_ids.is_empty()
        && conflicting_case_ids.is_empty()
        && not_comparable.is_empty();

    CompareResult {
        equivalent,
        added_case_ids,
        removed_case_ids,
        changed_case_ids,
        conflicting_case_ids,
        not_comparable,
    }
}

pub fn projection_result(graph: &CaseGraph) -> ProjectionResult {
    let selected_source_ids = source_ids(graph);
    ProjectionResult {
        projection_result: "projected".to_owned(),
        selected_source_ids,
        omitted_source_ids: Vec::new(),
        information_loss: vec![
            "projection summarizes case bodies and does not mutate source graph".to_owned(),
        ],
    }
}

pub fn graph_counts(graph: &CaseGraph) -> GraphCounts {
    GraphCounts {
        cases: graph.cases.len(),
        scenarios: graph.scenarios.len(),
        coverage_goals: graph.coverage_goals.len(),
        relations: graph.relations.len(),
        review_records: graph.review_records.len(),
    }
}

pub fn source_ids(graph: &CaseGraph) -> Vec<Id> {
    let mut ids = BTreeSet::new();
    for case_record in &graph.cases {
        ids.extend(case_record.source_ids.iter().cloned());
    }
    for scenario in &graph.scenarios {
        ids.extend(scenario.source_ids.iter().cloned());
    }
    for goal in &graph.coverage_goals {
        ids.extend(goal.source_ids.iter().cloned());
    }
    ids.into_iter().collect()
}

fn validate_duplicate_ids(graph: &CaseGraph, errors: &mut Vec<ValidationIssue>) {
    let mut seen = BTreeSet::new();
    for (location, id) in graph_record_ids(graph) {
        if !seen.insert(id.clone()) {
            errors.push(issue(
                "duplicate_id",
                format!("duplicate id {id}"),
                location,
            ));
        }
    }
}

fn validate_space_membership(graph: &CaseGraph, errors: &mut Vec<ValidationIssue>) {
    for (location, id, space_id) in graph_space_memberships(graph) {
        if space_id != graph.space_id {
            errors.push(issue(
                "cross_space_reference",
                format!("{id} belongs to {space_id}, expected {}", graph.space_id),
                location,
            ));
        }
    }
}

fn validate_relation_endpoints(graph: &CaseGraph, errors: &mut Vec<ValidationIssue>) {
    let graph_ids = graph_ids(graph);
    for relation in &graph.relations {
        if !endpoint_resolves(&relation.from_id, &graph_ids) {
            errors.push(dangling_relation_issue(relation, "from_id"));
        }
        if !endpoint_resolves(&relation.to_id, &graph_ids) {
            errors.push(dangling_relation_issue(relation, "to_id"));
        }
    }
}

fn validate_review_targets(graph: &CaseGraph, errors: &mut Vec<ValidationIssue>) {
    let graph_ids = graph_ids(graph);
    for record in &graph.review_records {
        if !graph_ids.contains(record.target_id.as_str()) {
            errors.push(issue(
                "dangling_review_target",
                format!("review target {} does not resolve", record.target_id),
                "$.review_records",
            ));
        }
    }
}

fn evaluate_goal(
    graph: &CaseGraph,
    goal: &CoverageGoal,
    represented: &BTreeSet<String>,
    policy: &CoveragePolicy,
) -> GoalCoverage {
    let covering_case_ids = covering_cases(graph, goal, policy);
    let represented_ids = goal
        .required_ids
        .iter()
        .filter(|id| represented.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let uncovered_ids = goal
        .required_ids
        .iter()
        .filter(|id| !represented.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    GoalCoverage {
        coverage_goal_id: goal.id.clone(),
        coverage_type: goal.coverage_type.clone(),
        status: goal_status(&uncovered_ids, &covering_case_ids, goal),
        represented_ids,
        uncovered_ids,
        covering_case_ids,
    }
}

fn represented_ids(graph: &CaseGraph) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for case_record in &graph.cases {
        ids.extend(case_record.cell_ids.iter().map(id_text));
        ids.extend(case_record.incidence_ids.iter().map(id_text));
        ids.extend(case_record.context_ids.iter().map(id_text));
        ids.extend(case_record.scenario_ids.iter().map(id_text));
    }
    for relation in &graph.relations {
        if matches!(relation.relation_type.as_str(), "covers" | "exercises") {
            ids.insert(id_text(&relation.to_id));
        }
    }
    ids
}

fn selected_goals<'a>(
    graph: &'a CaseGraph,
    policy: &'a CoveragePolicy,
) -> impl Iterator<Item = &'a CoverageGoal> + 'a {
    graph.coverage_goals.iter().filter(move |goal| {
        policy.coverage_goal_ids.is_empty() || policy.coverage_goal_ids.contains(&goal.id)
    })
}

fn covering_cases(graph: &CaseGraph, goal: &CoverageGoal, policy: &CoveragePolicy) -> Vec<Id> {
    graph
        .cases
        .iter()
        .filter(|case_record| case_covers_goal(case_record, goal, graph, policy))
        .map(|case_record| case_record.id.clone())
        .collect()
}

fn case_covers_goal(
    case_record: &CaseRecord,
    goal: &CoverageGoal,
    graph: &CaseGraph,
    policy: &CoveragePolicy,
) -> bool {
    let explicit = graph.relations.iter().any(|relation| {
        matches!(relation.relation_type.as_str(), "covers" | "exercises")
            && relation.from_id == case_record.id
            && relation.to_id == goal.id
    });
    let case_ids = case_exercised_ids(case_record);
    let covers_targets = goal
        .required_ids
        .iter()
        .all(|required| case_ids.contains(required.as_str()));

    (explicit || !policy.require_explicit_relations) && covers_targets
}

fn case_exercised_ids(case_record: &CaseRecord) -> BTreeSet<&str> {
    let mut ids = BTreeSet::new();
    ids.extend(case_record.cell_ids.iter().map(Id::as_str));
    ids.extend(case_record.incidence_ids.iter().map(Id::as_str));
    ids.extend(case_record.context_ids.iter().map(Id::as_str));
    ids.extend(case_record.scenario_ids.iter().map(Id::as_str));
    ids
}

fn goal_status(uncovered_ids: &[Id], covering_case_ids: &[Id], goal: &CoverageGoal) -> String {
    if uncovered_ids.is_empty()
        && covering_case_ids.len() >= goal.min_cases_per_target.try_into().unwrap_or(usize::MAX)
    {
        "covered".to_owned()
    } else if uncovered_ids.len() == goal.required_ids.len() {
        "uncovered".to_owned()
    } else {
        "partial".to_owned()
    }
}

fn coverage_status(goals: &[GoalCoverage]) -> String {
    if goals.is_empty() {
        "no_goals".to_owned()
    } else if goals.iter().all(|goal| goal.status == "covered") {
        "covered".to_owned()
    } else if goals.iter().any(|goal| goal.status == "partial") {
        "partial".to_owned()
    } else {
        "uncovered".to_owned()
    }
}

fn boundary_coverage(graph: &CaseGraph, represented: &BTreeSet<String>) -> BoundaryCoverage {
    let required_contexts = graph
        .coverage_goals
        .iter()
        .filter(|goal| goal.coverage_type == "boundary" || goal.coverage_type == "context")
        .flat_map(|goal| goal.required_ids.iter())
        .filter(|id| id.as_str().starts_with("context:"))
        .cloned()
        .collect::<BTreeSet<_>>();
    let represented_context_ids = required_contexts
        .iter()
        .filter(|id| represented.contains(id.as_str()))
        .cloned()
        .collect();
    let uncovered_context_ids = required_contexts
        .iter()
        .filter(|id| !represented.contains(id.as_str()))
        .cloned()
        .collect();
    let represented_boundary_ids = represented
        .iter()
        .filter(|id| id.starts_with("boundary:"))
        .filter_map(|id| Id::new(id.clone()).ok())
        .collect();

    BoundaryCoverage {
        represented_context_ids,
        uncovered_context_ids,
        represented_boundary_ids,
    }
}

fn missing_for_goal(goal: &GoalCoverage, graph: &CaseGraph) -> Vec<MissingCase> {
    let severity = graph
        .coverage_goals
        .iter()
        .find(|item| item.id == goal.coverage_goal_id)
        .map(|item| item.severity_if_uncovered)
        .unwrap_or(Severity::Medium);
    goal.uncovered_ids
        .iter()
        .map(|target| missing_case(goal, target, severity))
        .collect()
}

fn missing_case(goal: &GoalCoverage, target: &Id, severity: Severity) -> MissingCase {
    MissingCase {
        id: Id::new(format!(
            "missing:{}:{}",
            sanitize_id(&goal.coverage_goal_id),
            sanitize_id(target)
        ))
        .expect("generated missing case id is non-empty"),
        missing_type: goal.coverage_type.clone(),
        coverage_goal_id: goal.coverage_goal_id.clone(),
        target_ids: vec![target.clone()],
        rationale: format!("{target} is not represented by any accepted case."),
        confidence: Confidence::new(0.8).expect("static confidence is valid"),
        severity,
        provenance: Provenance::new(
            SourceRef::new(SourceKind::Ai),
            Confidence::new(0.8).expect("static confidence is valid"),
        ),
        review_status: ReviewStatus::Unreviewed,
    }
}

fn conflict_from_relation(graph: &CaseGraph, relation: &CaseRelation) -> Option<ConflictingCase> {
    let left = graph
        .cases
        .iter()
        .find(|case| case.id == relation.from_id)?;
    let right = graph.cases.iter().find(|case| case.id == relation.to_id)?;
    Some(ConflictingCase {
        id: Id::new(format!("conflict:{}", sanitize_id(&relation.id))).ok()?,
        conflict_type: "contradicts_relation".to_owned(),
        case_ids: vec![left.id.clone(), right.id.clone()],
        scenario_ids: merge_ids(&left.scenario_ids, &right.scenario_ids),
        source_ids: merge_ids(&left.source_ids, &right.source_ids),
        evidence_ids: relation.evidence_ids.clone(),
        severity: Severity::High,
        explanation: format!("{} contradicts {}", left.id, right.id),
        provenance: relation.provenance.clone(),
    })
}

fn conflicts_from_outcomes(case_record: &CaseRecord) -> Vec<ConflictingCase> {
    case_record
        .expected_outcomes
        .iter()
        .filter_map(|expected| {
            let observed = case_record
                .observed_outcomes
                .iter()
                .find(|item| item.id == expected.id)?;
            (observed.summary != expected.summary).then(|| outcome_conflict(case_record, expected))
        })
        .collect()
}

fn outcome_conflict(case_record: &CaseRecord, expected: &crate::model::Outcome) -> ConflictingCase {
    ConflictingCase {
        id: Id::new(format!(
            "conflict:{}:{}",
            sanitize_id(&case_record.id),
            sanitize_id(&expected.id)
        ))
        .expect("generated conflict id is non-empty"),
        conflict_type: "expected_observed_mismatch".to_owned(),
        case_ids: vec![case_record.id.clone()],
        scenario_ids: case_record.scenario_ids.clone(),
        source_ids: case_record.source_ids.clone(),
        evidence_ids: Vec::new(),
        severity: Severity::High,
        explanation: format!("observed outcome differs from expected {}", expected.id),
        provenance: case_record.provenance.clone(),
    }
}

fn graph_ids(graph: &CaseGraph) -> BTreeSet<String> {
    graph_record_ids(graph)
        .into_iter()
        .map(|(_, id)| id.as_str().to_owned())
        .chain(source_ids(graph).into_iter().map(|id| id.into_string()))
        .collect()
}

fn graph_record_ids(graph: &CaseGraph) -> Vec<(&'static str, Id)> {
    let mut ids = vec![("$.case_graph_id", graph.case_graph_id.clone())];
    ids.extend(graph.cases.iter().map(|item| ("$.cases", item.id.clone())));
    ids.extend(
        graph
            .scenarios
            .iter()
            .map(|item| ("$.scenarios", item.id.clone())),
    );
    ids.extend(
        graph
            .coverage_goals
            .iter()
            .map(|item| ("$.coverage_goals", item.id.clone())),
    );
    ids.extend(
        graph
            .relations
            .iter()
            .map(|item| ("$.relations", item.id.clone())),
    );
    ids.extend(
        graph
            .review_records
            .iter()
            .map(|item| ("$.review_records", item.id.clone())),
    );
    ids
}

fn graph_space_memberships(graph: &CaseGraph) -> Vec<(&'static str, Id, Id)> {
    let mut items = Vec::new();
    items.extend(
        graph
            .cases
            .iter()
            .map(|item| ("$.cases", item.id.clone(), item.space_id.clone())),
    );
    items.extend(
        graph
            .scenarios
            .iter()
            .map(|item| ("$.scenarios", item.id.clone(), item.space_id.clone())),
    );
    items.extend(
        graph
            .coverage_goals
            .iter()
            .map(|item| ("$.coverage_goals", item.id.clone(), item.space_id.clone())),
    );
    items
}

fn endpoint_resolves(id: &Id, graph_ids: &BTreeSet<String>) -> bool {
    graph_ids.contains(id.as_str()) || is_external_structure_id(id.as_str())
}

fn is_external_structure_id(id: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "cell:",
        "incidence:",
        "context:",
        "boundary:",
        "invariant:",
        "morphism:",
        "source:",
        "space:",
    ];
    PREFIXES.iter().any(|prefix| id.starts_with(prefix))
}

fn dangling_relation_issue(relation: &CaseRelation, field: &str) -> ValidationIssue {
    issue(
        "dangling_relation_endpoint",
        format!("relation {} has unresolved {field}", relation.id),
        "$.relations",
    )
}

fn cases_by_id(cases: &[CaseRecord]) -> BTreeMap<String, &CaseRecord> {
    cases
        .iter()
        .map(|case_record| (case_record.id.as_str().to_owned(), case_record))
        .collect()
}

fn diff_keys(
    left: &BTreeMap<String, &CaseRecord>,
    right: &BTreeMap<String, &CaseRecord>,
) -> Vec<Id> {
    left.keys()
        .filter(|id| !right.contains_key(*id))
        .filter_map(|id| Id::new(id.clone()).ok())
        .collect()
}

fn changed_cases(
    left: &BTreeMap<String, &CaseRecord>,
    right: &BTreeMap<String, &CaseRecord>,
) -> Vec<Id> {
    left.iter()
        .filter_map(|(id, left_case)| {
            let right_case = right.get(id)?;
            (serde_json::to_value(left_case).ok()? != serde_json::to_value(right_case).ok()?)
                .then(|| Id::new(id.clone()).ok())
                .flatten()
        })
        .collect()
}

fn conflicting_changed_cases(
    left: &BTreeMap<String, &CaseRecord>,
    right: &BTreeMap<String, &CaseRecord>,
) -> Vec<Id> {
    left.iter()
        .filter_map(|(id, left_case)| {
            let right_case = right.get(id)?;
            (left_case.expected_outcomes != right_case.expected_outcomes)
                .then(|| Id::new(id.clone()).ok())
                .flatten()
        })
        .collect()
}

fn collect_goal_ids(
    goals: &[GoalCoverage],
    predicate: impl Fn(&GoalCoverage) -> bool,
) -> Vec<String> {
    goals
        .iter()
        .filter(|goal| predicate(goal))
        .map(|goal| id_text(&goal.coverage_goal_id))
        .collect()
}

fn merge_ids(left: &[Id], right: &[Id]) -> Vec<Id> {
    left.iter()
        .chain(right)
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sanitize_id(id: &Id) -> String {
    id.as_str()
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

fn issue(
    code: impl Into<String>,
    message: impl Into<String>,
    location: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        message: message.into(),
        location: location.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{coverage_policy, sample_graph};

    #[test]
    fn coverage_reports_partial_and_missing_targets() {
        let graph = sample_graph();
        let policy = coverage_policy();
        let coverage = evaluate_coverage(&graph, &policy);
        let missing = detect_missing_cases(&graph, &policy);

        assert_eq!(coverage.coverage_status, "partial");
        assert_eq!(missing[0].target_ids[0].as_str(), "context:billing");
        assert_eq!(missing[0].review_status, ReviewStatus::Unreviewed);
    }

    #[test]
    fn contradictions_are_domain_findings() {
        let conflicts = detect_conflicts(&sample_graph());

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, "expected_observed_mismatch");
        assert_eq!(
            conflicts[0].source_ids[0].as_str(),
            "source:architecture-input"
        );
    }

    #[test]
    fn validation_rejects_dangling_case_relation() {
        let mut graph = sample_graph();
        graph.relations[0].to_id = Id::new("case:missing").expect("id");

        let result = validate_case_graph(&graph);

        assert!(!result.valid);
        assert_eq!(result.errors[0].code, "dangling_relation_endpoint");
    }
}
