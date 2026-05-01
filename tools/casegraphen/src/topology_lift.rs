use crate::{native_model, topology::TopologyReportOptions, workflow_model::WorkflowCaseGraph};
use higher_graphen_core::{CoreError, Id};
use higher_graphen_structure::space::{Cell, ComplexType, InMemorySpaceStore, Space};
use higher_graphen_structure::topology::summarize_complex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::topology::{
    topology_higher_order::{
        filtration_plan_from_input, higher_order_topology, HigherOrderFiltrationInput,
    },
    CaseTopologyReport, TopologyReportError,
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyLiftSummary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<SourceCellMapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<SourceCellMapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_relations: Vec<SkippedRelationMapping>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCellMapping {
    pub source_type: String,
    pub source_id: Id,
    pub cell_id: Id,
    pub dimension: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedRelationMapping {
    pub source_type: String,
    pub source_id: Id,
    pub missing_endpoint_ids: Vec<Id>,
    pub reason: String,
}

pub(super) struct LiftBuilder {
    store: InMemorySpaceStore,
    space_id: Id,
    complex_id: Id,
    name: String,
    cell_ids: Vec<Id>,
    source_cell_ids: BTreeMap<Id, Id>,
    source_mapping: TopologyLiftSummary,
}

impl LiftBuilder {
    pub(super) fn new(
        space_id: Id,
        complex_id: Id,
        name: impl Into<String>,
    ) -> Result<Self, CoreError> {
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

    pub(super) fn add_node(
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

    pub(super) fn add_relation(
        &mut self,
        source_type: &str,
        source_id: &Id,
        relation_type: &str,
        from_id: &Id,
        to_id: &Id,
    ) -> Result<(), CoreError> {
        if let Some(missing_endpoint_ids) = self.missing_endpoint_ids(from_id, to_id) {
            self.record_skipped_relation(source_type, source_id, missing_endpoint_ids);
            return Ok(());
        }

        let generated_id = cell_id("cell", source_type, source_id)?;
        let cell = Cell::new(
            generated_id.clone(),
            self.space_id.clone(),
            1,
            relation_type,
        )
        .with_boundary_cell(self.source_cell_ids[from_id].clone())
        .with_boundary_cell(self.source_cell_ids[to_id].clone());
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

    pub(super) fn finish(
        self,
        options: TopologyReportOptions,
    ) -> Result<CaseTopologyReport, TopologyReportError> {
        self.finish_with_filtration(options, HigherOrderFiltrationInput::Deterministic)
    }

    pub(super) fn finish_with_filtration(
        mut self,
        options: TopologyReportOptions,
        filtration_input: HigherOrderFiltrationInput<'_>,
    ) -> Result<CaseTopologyReport, TopologyReportError> {
        self.sort_mappings();
        self.store.construct_complex(
            self.complex_id.clone(),
            self.space_id.clone(),
            self.name.clone(),
            ComplexType::CellComplex,
            self.cell_ids.clone(),
            Vec::new(),
        )?;
        let topology = summarize_complex(&self.store, &self.complex_id)?;
        let complex = self.complex()?;
        let higher_order = if options.include_higher_order {
            let plan = filtration_plan_from_input(
                filtration_input,
                &self.store,
                complex,
                &self.source_mapping,
                options.max_dimension,
            )?;
            Some(higher_order_topology(&self.store, complex, options, plan)?)
        } else {
            None
        };

        Ok(CaseTopologyReport {
            space_id: self.space_id,
            complex_id: self.complex_id,
            topology,
            source_mapping: self.source_mapping,
            higher_order,
        })
    }

    fn missing_endpoint_ids(&self, from_id: &Id, to_id: &Id) -> Option<Vec<Id>> {
        let mut missing_endpoint_ids = Vec::new();
        if !self.source_cell_ids.contains_key(from_id) {
            missing_endpoint_ids.push(from_id.clone());
        }
        if !self.source_cell_ids.contains_key(to_id) && to_id != from_id {
            missing_endpoint_ids.push(to_id.clone());
        }
        (!missing_endpoint_ids.is_empty()).then_some(missing_endpoint_ids)
    }

    fn record_skipped_relation(
        &mut self,
        source_type: &str,
        source_id: &Id,
        missing_endpoint_ids: Vec<Id>,
    ) {
        self.source_mapping
            .skipped_relations
            .push(SkippedRelationMapping {
                source_type: source_type.to_owned(),
                source_id: source_id.clone(),
                missing_endpoint_ids,
                reason: "relation endpoint was not lifted as a dimension-0 cell".to_owned(),
            });
    }

    fn sort_mappings(&mut self) {
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
    }

    fn complex(&self) -> Result<&higher_graphen_structure::space::Complex, CoreError> {
        self.store
            .complex(&self.complex_id)
            .ok_or_else(|| CoreError::MalformedField {
                field: "complex_id".to_owned(),
                reason: format!("identifier {} does not exist", self.complex_id),
            })
    }
}

pub(super) fn workflow_lift_builder(
    graph: &WorkflowCaseGraph,
) -> Result<LiftBuilder, TopologyReportError> {
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

    Ok(lift)
}

pub(super) fn native_lift_builder(
    case_space: &native_model::CaseSpace,
) -> Result<LiftBuilder, TopologyReportError> {
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

    Ok(lift)
}

pub(super) fn cell_id(prefix: &str, source_type: &str, source_id: &Id) -> Result<Id, CoreError> {
    Id::new(format!(
        "{prefix}:casegraphen:{source_type}:{}",
        source_id.as_str()
    ))
}
