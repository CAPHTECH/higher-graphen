//! Shared topology diagnostics for CaseGraphen data models.

use crate::{model, native_model, workflow_model::WorkflowCaseGraph};
use higher_graphen_core::{CoreError, Id};
use higher_graphen_space::{Cell, ComplexType, InMemorySpaceStore, Space};
use higher_graphen_topology::{summarize_complex, TopologySummary};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Topology report for a lifted CaseGraphen graph.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseTopologyReport {
    /// Space summarized by the topology engine.
    pub space_id: Id,
    /// Complex summarized by the topology engine.
    pub complex_id: Id,
    /// Shared finite topology summary over the lifted complex.
    pub topology: TopologySummary,
    /// Deterministic mapping from source records to generated cells.
    pub source_mapping: TopologyLiftSummary,
}

/// Deterministic source-to-cell mapping created during lift.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyLiftSummary {
    /// Source records lifted as dimension-0 cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<SourceCellMapping>,
    /// Source relation records lifted as dimension-1 cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<SourceCellMapping>,
    /// Relation-like records skipped because endpoints were not present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_relations: Vec<SkippedRelationMapping>,
}

/// Mapping between one source record and one generated topology cell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCellMapping {
    /// Stable source category.
    pub source_type: String,
    /// Original CaseGraphen identifier.
    pub source_id: Id,
    /// Generated cell identifier in the lifted complex.
    pub cell_id: Id,
    /// Generated cell dimension.
    pub dimension: u32,
}

/// Deterministic diagnostic for a relation skipped during lift.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedRelationMapping {
    /// Stable source relation category.
    pub source_type: String,
    /// Original relation identifier.
    pub source_id: Id,
    /// Missing original endpoint identifiers.
    pub missing_endpoint_ids: Vec<Id>,
    /// Human-readable deterministic reason.
    pub reason: String,
}

/// Error returned while building or summarizing a topology report.
pub type TopologyReportError = CoreError;

/// Lifts a structured case graph into a finite complex and summarizes it.
pub fn case_graph_topology(
    graph: &model::CaseGraph,
) -> Result<CaseTopologyReport, TopologyReportError> {
    let mut lift = LiftBuilder::new(
        graph.space_id.clone(),
        cell_id("complex", "case_graph", &graph.case_graph_id)?,
        "CaseGraphen case graph topology",
    )?;

    for case in &graph.cases {
        lift.add_node("case", &case.id, &case.title)?;
    }
    for scenario in &graph.scenarios {
        lift.add_node("scenario", &scenario.id, &scenario.title)?;
    }
    for goal in &graph.coverage_goals {
        lift.add_node("coverage_goal", &goal.id, &goal.coverage_type)?;
    }
    for review in &graph.review_records {
        lift.add_node("review_record", &review.id, "review record")?;
    }
    for relation in &graph.relations {
        lift.add_relation(
            "case_relation",
            &relation.id,
            &relation.relation_type,
            &relation.from_id,
            &relation.to_id,
        )?;
    }

    lift.finish()
}

/// Lifts a workflow case graph into a finite complex and summarizes it.
pub fn workflow_topology(
    graph: &WorkflowCaseGraph,
) -> Result<CaseTopologyReport, TopologyReportError> {
    let mut lift = LiftBuilder::new(
        graph.space_id.clone(),
        cell_id("complex", "workflow_graph", &graph.workflow_graph_id)?,
        "CaseGraphen workflow topology",
    )?;

    for item in &graph.work_items {
        lift.add_node("work_item", &item.id, &item.title)?;
    }
    for rule in &graph.readiness_rules {
        lift.add_node("readiness_rule", &rule.id, "readiness rule")?;
    }
    for evidence in &graph.evidence_records {
        lift.add_node("evidence_record", &evidence.id, &evidence.summary)?;
    }
    for review in &graph.completion_reviews {
        lift.add_node("completion_review", &review.id, &review.reason)?;
    }
    for transition in &graph.transition_records {
        lift.add_node("transition_record", &transition.id, "transition record")?;
    }
    for profile in &graph.projection_profiles {
        lift.add_node("projection_profile", &profile.id, &profile.purpose)?;
    }
    for correspondence in &graph.correspondence_records {
        lift.add_node(
            "correspondence_record",
            &correspondence.id,
            "correspondence record",
        )?;
    }
    for relation in &graph.workflow_relations {
        lift.add_relation(
            "workflow_relation",
            &relation.id,
            &format!("{:?}", relation.relation_type),
            &relation.from_id,
            &relation.to_id,
        )?;
    }

    lift.finish()
}

/// Lifts a native case space into a finite complex and summarizes it.
pub fn native_case_topology(
    case_space: &native_model::CaseSpace,
) -> Result<CaseTopologyReport, TopologyReportError> {
    let mut lift = LiftBuilder::new(
        case_space.space_id.clone(),
        cell_id("complex", "native_case_space", &case_space.case_space_id)?,
        "CaseGraphen native case topology",
    )?;

    for cell in &case_space.case_cells {
        lift.add_node("case_cell", &cell.id, &cell.title)?;
    }
    for projection in &case_space.projections {
        lift.add_node("projection", &projection.projection_id, "native projection")?;
    }
    lift.add_node(
        "revision",
        &case_space.revision.revision_id,
        "native revision",
    )?;
    for entry in &case_space.morphism_log {
        lift.add_node("morphism_log_entry", &entry.entry_id, "morphism log entry")?;
        lift.add_node("morphism", &entry.morphism_id, "morphism")?;
    }
    for relation in &case_space.case_relations {
        lift.add_relation(
            "case_relation",
            &relation.id,
            &relation.relation_type.to_string(),
            &relation.from_id,
            &relation.to_id,
        )?;
    }

    lift.finish()
}

struct LiftBuilder {
    store: InMemorySpaceStore,
    space_id: Id,
    complex_id: Id,
    name: String,
    cell_ids: Vec<Id>,
    source_cell_ids: BTreeMap<Id, Id>,
    source_mapping: TopologyLiftSummary,
}

impl LiftBuilder {
    fn new(space_id: Id, complex_id: Id, name: impl Into<String>) -> Result<Self, CoreError> {
        let mut store = InMemorySpaceStore::new();
        store.insert_space(Space::new(space_id.clone(), name.into()))?;

        Ok(Self {
            store,
            space_id,
            complex_id,
            name: "CaseGraphen lifted topology complex".to_owned(),
            cell_ids: Vec::new(),
            source_cell_ids: BTreeMap::new(),
            source_mapping: TopologyLiftSummary::default(),
        })
    }

    fn add_node(
        &mut self,
        source_type: &str,
        source_id: &Id,
        label: &str,
    ) -> Result<(), CoreError> {
        if self.source_cell_ids.contains_key(source_id) {
            return Ok(());
        }

        let generated_id = cell_id("cell", source_type, source_id)?;
        let cell = Cell::new(generated_id.clone(), self.space_id.clone(), 0, source_type)
            .with_label(label);
        self.store.insert_cell(cell)?;
        self.cell_ids.push(generated_id.clone());
        self.source_cell_ids
            .insert(source_id.clone(), generated_id.clone());
        self.source_mapping.nodes.push(SourceCellMapping {
            source_type: source_type.to_owned(),
            source_id: source_id.clone(),
            cell_id: generated_id,
            dimension: 0,
        });
        Ok(())
    }

    fn add_relation(
        &mut self,
        source_type: &str,
        source_id: &Id,
        relation_type: &str,
        from_id: &Id,
        to_id: &Id,
    ) -> Result<(), CoreError> {
        let from_cell_id = self.source_cell_ids.get(from_id);
        let to_cell_id = self.source_cell_ids.get(to_id);
        let mut missing_endpoint_ids = Vec::new();
        if from_cell_id.is_none() {
            missing_endpoint_ids.push(from_id.clone());
        }
        if to_cell_id.is_none() && to_id != from_id {
            missing_endpoint_ids.push(to_id.clone());
        }
        if !missing_endpoint_ids.is_empty() {
            self.source_mapping
                .skipped_relations
                .push(SkippedRelationMapping {
                    source_type: source_type.to_owned(),
                    source_id: source_id.clone(),
                    missing_endpoint_ids,
                    reason: "relation endpoint was not lifted as a dimension-0 cell".to_owned(),
                });
            return Ok(());
        }

        let generated_id = cell_id("cell", source_type, source_id)?;
        let cell = Cell::new(
            generated_id.clone(),
            self.space_id.clone(),
            1,
            relation_type,
        )
        .with_boundary_cell(from_cell_id.expect("checked from endpoint").clone())
        .with_boundary_cell(to_cell_id.expect("checked to endpoint").clone());
        self.store.insert_cell(cell)?;
        self.cell_ids.push(generated_id.clone());
        self.source_mapping.relations.push(SourceCellMapping {
            source_type: source_type.to_owned(),
            source_id: source_id.clone(),
            cell_id: generated_id,
            dimension: 1,
        });
        Ok(())
    }

    fn finish(mut self) -> Result<CaseTopologyReport, TopologyReportError> {
        self.source_mapping
            .nodes
            .sort_by(|left, right| left.source_id.cmp(&right.source_id));
        self.source_mapping
            .relations
            .sort_by(|left, right| left.source_id.cmp(&right.source_id));
        self.source_mapping
            .skipped_relations
            .sort_by(|left, right| left.source_id.cmp(&right.source_id));
        self.cell_ids.sort();

        self.store.construct_complex(
            self.complex_id.clone(),
            self.space_id.clone(),
            self.name,
            ComplexType::CellComplex,
            self.cell_ids,
            Vec::new(),
        )?;
        let topology = summarize_complex(&self.store, &self.complex_id)?;

        Ok(CaseTopologyReport {
            space_id: self.space_id,
            complex_id: self.complex_id,
            topology,
            source_mapping: self.source_mapping,
        })
    }
}

fn cell_id(prefix: &str, source_type: &str, source_id: &Id) -> Result<Id, CoreError> {
    Id::new(format!(
        "{prefix}:casegraphen:{source_type}:{}",
        source_id.as_str()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CaseGraph;

    const WORKFLOW_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/workflow.graph.example.json");
    const NATIVE_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/native.case.space.example.json");

    #[test]
    fn case_graph_topology_serializes_homology() {
        let graph = crate::fixtures::sample_graph();
        let report = case_graph_topology(&graph).expect("case graph topology");

        assert_eq!(report.space_id, graph.space_id);
        assert_eq!(report.topology.homology.betti_number(0), 2);
        assert_eq!(report.topology.homology.betti_number(1), 0);
        assert_eq!(report.source_mapping.nodes.len(), 3);
        assert_eq!(report.source_mapping.relations.len(), 1);

        let value = serde_json::to_value(&report).expect("serialize report");
        assert!(value.get("topology").is_some());
        assert!(value["topology"].get("homology").is_some());
        assert!(value.get("source_mapping").is_some());
    }

    #[test]
    fn workflow_topology_serializes_homology() {
        let graph: WorkflowCaseGraph =
            serde_json::from_str(WORKFLOW_EXAMPLE).expect("workflow graph example");
        let report = workflow_topology(&graph).expect("workflow topology");

        assert_eq!(report.space_id, graph.space_id);
        assert!(report.topology.homology.betti_number(0) > 0);
        assert_eq!(report.topology.homology.betti_number(1), 0);
        assert!(!report.source_mapping.nodes.is_empty());
        assert!(!report.source_mapping.relations.is_empty());

        serde_json::to_value(&report).expect("serialize workflow topology");
    }

    #[test]
    fn native_case_topology_serializes_homology() {
        let case_space: native_model::CaseSpace =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example");
        let report = native_case_topology(&case_space).expect("native case topology");

        assert_eq!(report.space_id, case_space.space_id);
        assert!(report.topology.homology.betti_number(0) > 0);
        assert_eq!(report.topology.homology.betti_number(1), 0);
        assert!(!report.source_mapping.nodes.is_empty());
        assert!(!report.source_mapping.relations.is_empty());

        serde_json::to_value(&report).expect("serialize native topology");
    }

    #[test]
    fn relation_with_missing_endpoint_is_skipped() {
        let mut graph = CaseGraph::empty(
            Id::new("case_graph:missing-endpoint").expect("case graph id"),
            Id::new("space:missing-endpoint").expect("space id"),
        );
        graph.relations.push(model::CaseRelation {
            id: Id::new("relation:missing").expect("relation id"),
            relation_type: "depends_on".to_owned(),
            from_id: Id::new("case:missing").expect("from id"),
            to_id: Id::new("coverage:missing").expect("to id"),
            evidence_ids: Vec::new(),
            provenance: crate::fixtures::sample_graph().relations[0]
                .provenance
                .clone(),
        });

        let report = case_graph_topology(&graph).expect("case graph topology");

        assert!(report.source_mapping.relations.is_empty());
        assert_eq!(report.source_mapping.skipped_relations.len(), 1);
        assert_eq!(report.topology.vertex_count, 0);
    }
}
