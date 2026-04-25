use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const CASE_GRAPH_SCHEMA: &str = "highergraphen.case.graph.v1";
pub const COVERAGE_POLICY_SCHEMA: &str = "highergraphen.case.coverage_policy.v1";
pub const PROJECTION_SCHEMA: &str = "highergraphen.case.projection.v1";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseGraph {
    pub schema: String,
    pub case_graph_id: Id,
    pub space_id: Id,
    pub cases: Vec<CaseRecord>,
    pub scenarios: Vec<Scenario>,
    pub coverage_goals: Vec<CoverageGoal>,
    pub relations: Vec<CaseRelation>,
    pub review_records: Vec<ReviewRecord>,
    pub metadata: Map<String, Value>,
}

impl CaseGraph {
    pub fn empty(case_graph_id: Id, space_id: Id) -> Self {
        Self {
            schema: CASE_GRAPH_SCHEMA.to_owned(),
            case_graph_id,
            space_id,
            cases: Vec::new(),
            scenarios: Vec::new(),
            coverage_goals: Vec::new(),
            relations: Vec::new(),
            review_records: Vec::new(),
            metadata: Map::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseRecord {
    pub id: Id,
    pub space_id: Id,
    pub title: String,
    pub case_type: String,
    pub situation_summary: String,
    pub scenario_ids: Vec<Id>,
    pub cell_ids: Vec<Id>,
    pub incidence_ids: Vec<Id>,
    pub context_ids: Vec<Id>,
    pub expected_outcomes: Vec<Outcome>,
    pub observed_outcomes: Vec<Outcome>,
    pub source_ids: Vec<Id>,
    pub tags: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub id: Id,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub id: Id,
    pub space_id: Id,
    pub title: String,
    pub scenario_type: String,
    pub parameters: Map<String, Value>,
    pub required_context_ids: Vec<Id>,
    pub required_cell_types: Vec<String>,
    pub coverage_target_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageGoal {
    pub id: Id,
    pub space_id: Id,
    pub coverage_type: String,
    pub required_ids: Vec<Id>,
    pub dimensions: Vec<u32>,
    pub min_cases_per_target: u32,
    pub severity_if_uncovered: Severity,
    pub source_ids: Vec<Id>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseRelation {
    pub id: Id,
    pub relation_type: String,
    pub from_id: Id,
    pub to_id: Id,
    pub evidence_ids: Vec<Id>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRecord {
    pub id: Id,
    pub target_id: Id,
    pub review_status: ReviewStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoveragePolicy {
    pub schema: String,
    pub policy_id: Id,
    #[serde(default)]
    pub coverage_goal_ids: Vec<Id>,
    #[serde(default)]
    pub require_explicit_relations: bool,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionDefinition {
    pub schema: String,
    pub projection_id: Id,
    pub audience: String,
    #[serde(default = "default_include_sources")]
    pub include_sources: bool,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MissingCase {
    pub id: Id,
    pub missing_type: String,
    pub coverage_goal_id: Id,
    pub target_ids: Vec<Id>,
    pub rationale: String,
    pub confidence: Confidence,
    pub severity: Severity,
    pub provenance: Provenance,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictingCase {
    pub id: Id,
    pub conflict_type: String,
    pub case_ids: Vec<Id>,
    pub scenario_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub evidence_ids: Vec<Id>,
    pub severity: Severity,
    pub explanation: String,
    pub provenance: Provenance,
}

pub fn id_text(id: &Id) -> String {
    id.as_str().to_owned()
}

fn default_include_sources() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_core::{Confidence, SourceKind, SourceRef};

    #[test]
    fn empty_graph_uses_case_graph_schema() {
        let graph = CaseGraph::empty(
            Id::new("case_graph:demo").expect("case graph id"),
            Id::new("space:demo").expect("space id"),
        );

        assert_eq!(graph.schema, CASE_GRAPH_SCHEMA);
        assert!(graph.cases.is_empty());
    }

    #[test]
    fn primitives_validate_during_deserialization() {
        let json = serde_json::json!({
            "schema": CASE_GRAPH_SCHEMA,
            "case_graph_id": " ",
            "space_id": "space:demo",
            "cases": [],
            "scenarios": [],
            "coverage_goals": [],
            "relations": [],
            "review_records": [],
            "metadata": {}
        });

        assert!(serde_json::from_value::<CaseGraph>(json).is_err());
    }

    #[test]
    fn missing_case_preserves_review_boundary() {
        let provenance = Provenance::new(
            SourceRef::new(SourceKind::Ai),
            Confidence::new(0.8).expect("confidence"),
        );
        let missing = MissingCase {
            id: Id::new("missing:demo").expect("missing id"),
            missing_type: "boundary".to_owned(),
            coverage_goal_id: Id::new("coverage:demo").expect("coverage id"),
            target_ids: vec![Id::new("cell:demo").expect("target id")],
            rationale: "Target is uncovered.".to_owned(),
            confidence: Confidence::new(0.8).expect("confidence"),
            severity: Severity::High,
            provenance,
            review_status: ReviewStatus::Unreviewed,
        };

        assert_eq!(missing.review_status, ReviewStatus::Unreviewed);
    }
}
