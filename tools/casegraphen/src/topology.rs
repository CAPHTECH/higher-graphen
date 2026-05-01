//! Shared topology diagnostics for CaseGraphen data models.

use crate::{
    model, native_model, workflow_model::WorkflowCaseGraph,
    workflow_workspace::WorkflowHistoryEntry,
};
use higher_graphen_core::{CoreError, Id};
use higher_graphen_structure::space::Dimension;
use higher_graphen_structure::topology::TopologySummary;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[path = "topology_higher_order.rs"]
mod topology_higher_order;
#[path = "topology_lift.rs"]
mod topology_lift;

pub use self::topology_higher_order::{
    HigherOrderFiltrationSource, HigherOrderFiltrationStageSource, HigherOrderIntervalSummary,
    HigherOrderTopologyReport, HigherOrderTopologySummary,
};
pub use self::topology_lift::{SkippedRelationMapping, SourceCellMapping, TopologyLiftSummary};

use self::topology_higher_order::HigherOrderFiltrationInput;
use self::topology_lift::{cell_id, native_lift_builder, workflow_lift_builder, LiftBuilder};

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
    /// Optional higher-order persistence summary for opt-in CLI diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub higher_order: Option<HigherOrderTopologyReport>,
}

/// Options for opt-in higher-order topology diagnostics.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyReportOptions {
    /// Whether to include higher-order persistence diagnostics.
    pub include_higher_order: bool,
    /// Optional maximum cell dimension included in the filtration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_dimension: Option<Dimension>,
    /// Minimum interval lifetime in stage steps for persistent interval reporting.
    pub min_persistence_stages: usize,
}

impl TopologyReportOptions {
    /// Returns options that emit only the baseline static topology report.
    #[must_use]
    pub fn baseline() -> Self {
        Self::default()
    }

    /// Returns options that include opt-in higher-order persistence diagnostics.
    #[must_use]
    pub fn higher_order(max_dimension: Option<Dimension>, min_persistence_stages: usize) -> Self {
        Self {
            include_higher_order: true,
            max_dimension,
            min_persistence_stages,
        }
    }
}

/// File-to-file topology diff between two lifted CaseGraphen topology reports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyDiffReport {
    /// Space summarized by the left topology report.
    pub left_space_id: Id,
    /// Space summarized by the right topology report.
    pub right_space_id: Id,
    /// Complex summarized by the left topology report.
    pub left_complex_id: Id,
    /// Complex summarized by the right topology report.
    pub right_complex_id: Id,
    /// Scalar topology count changes from left to right.
    pub scalar_deltas: TopologyScalarDeltas,
    /// Source record additions and removals derived from source mappings.
    pub source_mapping: TopologySourceMappingDiff,
    /// Higher-order summary diff when both reports include higher-order summaries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub higher_order: Option<HigherOrderTopologyDiff>,
}

/// Scalar topology count changes from left to right.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyScalarDeltas {
    pub vertex_count: ScalarDelta,
    pub graph_edge_count: ScalarDelta,
    pub component_count: ScalarDelta,
    pub first_betti_number: ScalarDelta,
    pub simple_hole_count: ScalarDelta,
    pub euler_characteristic: ScalarDelta,
}

/// Numeric delta with left and right values retained for review.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarDelta {
    pub left: i64,
    pub right: i64,
    pub delta: i64,
}

/// Source record additions and removals derived from topology lift mappings.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopologySourceMappingDiff {
    pub added_source_node_ids: Vec<Id>,
    pub removed_source_node_ids: Vec<Id>,
    pub added_source_relation_ids: Vec<Id>,
    pub removed_source_relation_ids: Vec<Id>,
}

/// Compact higher-order topology summary diff.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HigherOrderTopologyDiff {
    pub interval_count_by_dimension: BTreeMap<Dimension, ScalarDelta>,
    pub open_interval_count_by_dimension: BTreeMap<Dimension, ScalarDelta>,
    pub persistent_interval_count_by_dimension: BTreeMap<Dimension, ScalarDelta>,
    pub max_betti_rank: ScalarDelta,
    pub max_betti_rank_dimension: OptionalDimensionDelta,
    pub highest_nonzero_betti_dimension: OptionalDimensionDelta,
}

/// Optional dimension change with left and right values retained for review.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OptionalDimensionDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<Dimension>,
    pub changed: bool,
}

/// Error returned while building or summarizing a topology report.
pub type TopologyReportError = CoreError;

/// Lifts a structured case graph into a finite complex and summarizes it.
pub fn case_graph_topology(
    graph: &model::CaseGraph,
) -> Result<CaseTopologyReport, TopologyReportError> {
    case_graph_topology_with_options(graph, TopologyReportOptions::baseline())
}

/// Lifts a structured case graph into a finite complex and summarizes it.
pub fn case_graph_topology_with_options(
    graph: &model::CaseGraph,
    options: TopologyReportOptions,
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

    lift.finish(options)
}

/// Lifts a workflow case graph into a finite complex and summarizes it.
pub fn workflow_topology(
    graph: &WorkflowCaseGraph,
) -> Result<CaseTopologyReport, TopologyReportError> {
    workflow_topology_with_options(graph, TopologyReportOptions::baseline())
}

/// Lifts a workflow case graph into a finite complex and summarizes it.
pub fn workflow_topology_with_options(
    graph: &WorkflowCaseGraph,
    options: TopologyReportOptions,
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

    lift.finish(options)
}

/// Lifts a workflow case graph and uses workspace history as the higher-order filtration source.
pub fn workflow_topology_with_history(
    graph: &WorkflowCaseGraph,
    history: &[WorkflowHistoryEntry],
    options: TopologyReportOptions,
) -> Result<CaseTopologyReport, TopologyReportError> {
    let lift = workflow_lift_builder(graph)?;
    lift.finish_with_filtration(
        options,
        HigherOrderFiltrationInput::WorkflowHistory(history),
    )
}

/// Lifts a native case space into a finite complex and summarizes it.
pub fn native_case_topology(
    case_space: &native_model::CaseSpace,
) -> Result<CaseTopologyReport, TopologyReportError> {
    native_case_topology_with_options(case_space, TopologyReportOptions::baseline())
}

/// Lifts a native case space into a finite complex and summarizes it.
pub fn native_case_topology_with_options(
    case_space: &native_model::CaseSpace,
    options: TopologyReportOptions,
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

    lift.finish(options)
}

/// Lifts a native case space and uses its morphism log as the higher-order filtration source.
pub fn native_case_topology_with_history(
    case_space: &native_model::CaseSpace,
    history: &[native_model::MorphismLogEntry],
    options: TopologyReportOptions,
) -> Result<CaseTopologyReport, TopologyReportError> {
    let lift = native_lift_builder(case_space)?;
    lift.finish_with_filtration(
        options,
        HigherOrderFiltrationInput::NativeMorphismLog(history),
    )
}

/// Compares two topology reports using deterministic scalar and source-mapping deltas.
#[must_use]
pub fn topology_diff(left: &CaseTopologyReport, right: &CaseTopologyReport) -> TopologyDiffReport {
    TopologyDiffReport {
        left_space_id: left.space_id.clone(),
        right_space_id: right.space_id.clone(),
        left_complex_id: left.complex_id.clone(),
        right_complex_id: right.complex_id.clone(),
        scalar_deltas: TopologyScalarDeltas {
            vertex_count: usize_delta(left.topology.vertex_count, right.topology.vertex_count),
            graph_edge_count: usize_delta(
                left.topology.graph_edge_count,
                right.topology.graph_edge_count,
            ),
            component_count: usize_delta(
                left.topology.component_count,
                right.topology.component_count,
            ),
            first_betti_number: usize_delta(
                left.topology.first_betti_number,
                right.topology.first_betti_number,
            ),
            simple_hole_count: usize_delta(
                left.topology.simple_hole_count,
                right.topology.simple_hole_count,
            ),
            euler_characteristic: i64_delta(
                left.topology.homology.euler_characteristic,
                right.topology.homology.euler_characteristic,
            ),
        },
        source_mapping: source_mapping_diff(&left.source_mapping, &right.source_mapping),
        higher_order: higher_order_diff(left, right),
    }
}

fn source_mapping_diff(
    left: &TopologyLiftSummary,
    right: &TopologyLiftSummary,
) -> TopologySourceMappingDiff {
    let (added_source_node_ids, removed_source_node_ids) = set_diff(
        &mapped_source_ids(&left.nodes),
        &mapped_source_ids(&right.nodes),
    );
    let (added_source_relation_ids, removed_source_relation_ids) = set_diff(
        &mapped_source_ids(&left.relations),
        &mapped_source_ids(&right.relations),
    );

    TopologySourceMappingDiff {
        added_source_node_ids,
        removed_source_node_ids,
        added_source_relation_ids,
        removed_source_relation_ids,
    }
}

fn higher_order_diff(
    left: &CaseTopologyReport,
    right: &CaseTopologyReport,
) -> Option<HigherOrderTopologyDiff> {
    let left_summary = left.higher_order.as_ref()?.summary.as_ref()?;
    let right_summary = right.higher_order.as_ref()?.summary.as_ref()?;

    Some(HigherOrderTopologyDiff {
        interval_count_by_dimension: count_map_delta(
            &left_summary.interval_count_by_dimension,
            &right_summary.interval_count_by_dimension,
        ),
        open_interval_count_by_dimension: count_map_delta(
            &left_summary.open_interval_count_by_dimension,
            &right_summary.open_interval_count_by_dimension,
        ),
        persistent_interval_count_by_dimension: count_map_delta(
            &left_summary.persistent_interval_count_by_dimension,
            &right_summary.persistent_interval_count_by_dimension,
        ),
        max_betti_rank: usize_delta(left_summary.max_betti_rank, right_summary.max_betti_rank),
        max_betti_rank_dimension: optional_dimension_delta(
            left_summary.max_betti_rank_dimension,
            right_summary.max_betti_rank_dimension,
        ),
        highest_nonzero_betti_dimension: optional_dimension_delta(
            left_summary.highest_nonzero_betti_dimension,
            right_summary.highest_nonzero_betti_dimension,
        ),
    })
}

fn mapped_source_ids(mappings: &[SourceCellMapping]) -> BTreeSet<Id> {
    mappings
        .iter()
        .map(|mapping| mapping.source_id.clone())
        .collect()
}

fn set_diff(left: &BTreeSet<Id>, right: &BTreeSet<Id>) -> (Vec<Id>, Vec<Id>) {
    (
        right.difference(left).cloned().collect(),
        left.difference(right).cloned().collect(),
    )
}

fn count_map_delta(
    left: &BTreeMap<Dimension, usize>,
    right: &BTreeMap<Dimension, usize>,
) -> BTreeMap<Dimension, ScalarDelta> {
    left.keys()
        .chain(right.keys())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|dimension| {
            (
                dimension,
                usize_delta(
                    left.get(&dimension).copied().unwrap_or_default(),
                    right.get(&dimension).copied().unwrap_or_default(),
                ),
            )
        })
        .collect()
}

fn usize_delta(left: usize, right: usize) -> ScalarDelta {
    i64_delta(left as i64, right as i64)
}

fn i64_delta(left: i64, right: i64) -> ScalarDelta {
    ScalarDelta {
        left,
        right,
        delta: right - left,
    }
}

fn optional_dimension_delta(
    left: Option<Dimension>,
    right: Option<Dimension>,
) -> OptionalDimensionDelta {
    OptionalDimensionDelta {
        left,
        right,
        changed: left != right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CaseGraph;
    use higher_graphen_structure::topology::PersistenceInterval;
    use std::collections::BTreeMap;

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
        assert!(value.get("higher_order").is_none());
    }

    #[test]
    fn case_graph_topology_can_include_higher_order_persistence() {
        let graph = crate::fixtures::sample_graph();
        let report = case_graph_topology_with_options(
            &graph,
            TopologyReportOptions::higher_order(Some(1), 2),
        )
        .expect("case graph higher-order topology");

        let higher_order = report.higher_order.expect("higher-order report");
        assert_eq!(higher_order.options.max_dimension, Some(1));
        assert_eq!(higher_order.options.min_persistence_stages, 2);
        assert!(higher_order.cell_count > 0);
        assert_eq!(higher_order.stage_count, higher_order.cell_count);
        assert!(higher_order.persistence.is_some());
        assert!(higher_order.summary.is_some());
    }

    #[test]
    fn higher_order_summary_reflects_persistence_intervals() {
        let graph = crate::fixtures::sample_graph();
        let report = case_graph_topology_with_options(
            &graph,
            TopologyReportOptions::higher_order(Some(1), 2),
        )
        .expect("case graph higher-order topology");
        let higher_order = report.higher_order.expect("higher-order report");
        let persistence = higher_order.persistence.as_ref().expect("persistence");
        let summary = higher_order.summary.as_ref().expect("summary");
        let last_stage_index = persistence.stages.len() - 1;

        assert_eq!(
            summary.interval_count_by_dimension,
            interval_counts_by_dimension(&persistence.intervals)
        );
        assert_eq!(
            summary.open_interval_count_by_dimension,
            interval_counts_by_dimension(
                &persistence
                    .intervals
                    .iter()
                    .filter(|interval| interval.is_open())
                    .cloned()
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(
            summary.persistent_interval_count_by_dimension,
            interval_counts_by_dimension(&persistence.persistent_intervals)
        );
        assert_eq!(
            summary
                .longest_lifetime_interval
                .as_ref()
                .expect("longest interval")
                .lifetime_stages,
            persistence
                .intervals
                .iter()
                .map(|interval| interval.lifetime_stages(last_stage_index))
                .max()
                .expect("interval lifetime")
        );

        let value = serde_json::to_value(&higher_order).expect("serialize higher-order report");
        assert!(value.get("summary").is_some());
        assert!(value["summary"]
            .get("persistent_interval_count_by_dimension")
            .is_some());
    }

    #[test]
    fn higher_order_summary_respects_dimension_and_persistence_options() {
        let graph = crate::fixtures::sample_graph();
        let report = case_graph_topology_with_options(
            &graph,
            TopologyReportOptions::higher_order(Some(0), 2),
        )
        .expect("case graph higher-order topology");
        let higher_order = report.higher_order.expect("higher-order report");
        let summary = higher_order.summary.expect("summary");

        assert_eq!(higher_order.cell_count, 3);
        assert_eq!(higher_order.stage_count, 3);
        assert_eq!(summary.interval_count_by_dimension, counts(&[(0, 3)]));
        assert_eq!(summary.open_interval_count_by_dimension, counts(&[(0, 3)]));
        assert_eq!(
            summary.persistent_interval_count_by_dimension,
            counts(&[(0, 2)])
        );
        assert_eq!(summary.highest_nonzero_betti_dimension, Some(0));
        assert_eq!(summary.max_betti_rank, 3);
        assert_eq!(summary.max_betti_rank_dimension, Some(0));

        let longest = summary
            .longest_lifetime_interval
            .expect("longest lifetime interval");
        assert_eq!(longest.dimension, 0);
        assert_eq!(longest.lifetime_stages, 3);
        assert!(longest.is_open);
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

    fn interval_counts_by_dimension(
        intervals: &[PersistenceInterval],
    ) -> BTreeMap<Dimension, usize> {
        let mut counts = BTreeMap::new();
        for interval in intervals {
            *counts.entry(interval.dimension).or_insert(0) += 1;
        }
        counts
    }

    fn counts(entries: &[(Dimension, usize)]) -> BTreeMap<Dimension, usize> {
        entries.iter().copied().collect()
    }
}
