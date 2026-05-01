use higher_graphen_core::{Id, Provenance};
use higher_graphen_structure::space::IncidenceOrientation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedStructure {
    pub space: TestGapLiftedSpace,
    pub structural_summary: TestGapStructuralSummary,
    pub contexts: Vec<TestGapLiftedContext>,
    pub cells: Vec<TestGapLiftedCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidences: Vec<TestGapLiftedIncidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedSpace {
    pub id: Id,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub cell_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
    pub context_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapStructuralSummary {
    pub accepted_cell_count: usize,
    pub accepted_incidence_count: usize,
    pub context_count: usize,
    pub branch_count: usize,
    pub requirement_count: usize,
    pub test_count: usize,
    pub coverage_record_count: usize,
    pub higher_order_cell_count: usize,
    pub higher_order_incidence_count: usize,
    pub morphism_count: usize,
    pub law_count: usize,
    pub verification_cell_count: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedContext {
    pub id: Id,
    pub space_id: Id,
    pub name: String,
    pub context_type: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedCell {
    pub id: Id,
    pub space_id: Id,
    pub dimension: u32,
    pub cell_type: String,
    pub label: String,
    pub context_ids: Vec<Id>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedIncidence {
    pub id: Id,
    pub space_id: Id,
    pub from_cell_id: Id,
    pub to_cell_id: Id,
    pub relation_type: String,
    pub orientation: IncidenceOrientation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub provenance: Provenance,
}
