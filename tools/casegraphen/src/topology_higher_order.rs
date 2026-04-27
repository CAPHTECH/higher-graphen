use crate::{
    native_model, topology::TopologyReportOptions, workflow_workspace::WorkflowHistoryEntry,
};
use higher_graphen_core::{CoreError, Id};
use higher_graphen_space::{Dimension, InMemorySpaceStore};
use higher_graphen_topology::{
    summarize_filtration_with_options, FiltrationStage, PersistenceOptions, PersistenceSummary,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::topology::{TopologyLiftSummary, TopologyReportError};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HigherOrderTopologyReport {
    pub options: TopologyReportOptions,
    pub filtration_source: HigherOrderFiltrationSource,
    pub cell_count: usize,
    pub stage_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_sources: Vec<HigherOrderFiltrationStageSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<HigherOrderTopologySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<PersistenceSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omitted_reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HigherOrderFiltrationSource {
    DeterministicCellOrder,
    WorkflowHistory,
    NativeMorphismLog,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HigherOrderFiltrationStageSource {
    pub stage_id: Id,
    pub source_type: String,
    pub source_id: Id,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_source_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_cell_ids: Vec<Id>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HigherOrderTopologySummary {
    pub interval_count_by_dimension: BTreeMap<Dimension, usize>,
    pub open_interval_count_by_dimension: BTreeMap<Dimension, usize>,
    pub persistent_interval_count_by_dimension: BTreeMap<Dimension, usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_lifetime_interval: Option<HigherOrderIntervalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_nonzero_betti_dimension: Option<Dimension>,
    pub max_betti_rank: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_betti_rank_dimension: Option<Dimension>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HigherOrderIntervalSummary {
    pub dimension: Dimension,
    pub birth_stage_id: Id,
    pub birth_stage_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_stage_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_stage_index: Option<usize>,
    pub lifetime_stages: usize,
    pub is_open: bool,
    pub generator_cell_ids: Vec<Id>,
}

#[derive(Clone, Copy)]
pub(crate) enum HigherOrderFiltrationInput<'a> {
    Deterministic,
    WorkflowHistory(&'a [WorkflowHistoryEntry]),
    NativeMorphismLog(&'a [native_model::MorphismLogEntry]),
}

pub(crate) struct HigherOrderFiltrationPlan {
    source: HigherOrderFiltrationSource,
    stages: Vec<FiltrationStage>,
    stage_sources: Vec<HigherOrderFiltrationStageSource>,
}

pub(crate) fn higher_order_topology(
    store: &InMemorySpaceStore,
    complex: &higher_graphen_space::Complex,
    options: TopologyReportOptions,
    filtration_plan: Option<HigherOrderFiltrationPlan>,
) -> Result<HigherOrderTopologyReport, TopologyReportError> {
    let cell_ids = filtration_cell_ids(store, complex, options.max_dimension)?;
    let filtration_source = filtration_plan
        .as_ref()
        .map(|plan| plan.source)
        .unwrap_or(HigherOrderFiltrationSource::DeterministicCellOrder);
    let stage_sources = filtration_plan
        .as_ref()
        .map(|plan| plan.stage_sources.clone())
        .unwrap_or_default();
    if cell_ids.is_empty() {
        return Ok(HigherOrderTopologyReport {
            options,
            filtration_source,
            cell_count: 0,
            stage_count: 0,
            stage_sources,
            summary: None,
            persistence: None,
            omitted_reason: Some("no cells selected for higher-order filtration".to_owned()),
        });
    }

    let stages = match filtration_plan {
        Some(plan) => plan.stages,
        None => cumulative_filtration_stages(&cell_ids)?,
    };
    let persistence = summarize_filtration_with_options(
        store,
        &complex.id,
        &stages,
        PersistenceOptions::new().with_min_lifetime_stages(options.min_persistence_stages),
    )?;
    let summary = summarize_higher_order_persistence(&persistence);

    Ok(HigherOrderTopologyReport {
        options,
        filtration_source,
        cell_count: cell_ids.len(),
        stage_count: stages.len(),
        stage_sources,
        summary: Some(summary),
        persistence: Some(persistence),
        omitted_reason: None,
    })
}

pub(crate) fn filtration_plan_from_input(
    input: HigherOrderFiltrationInput<'_>,
    store: &InMemorySpaceStore,
    complex: &higher_graphen_space::Complex,
    source_mapping: &TopologyLiftSummary,
    max_dimension: Option<Dimension>,
) -> Result<Option<HigherOrderFiltrationPlan>, TopologyReportError> {
    match input {
        HigherOrderFiltrationInput::Deterministic => Ok(None),
        HigherOrderFiltrationInput::WorkflowHistory(history) => history_filtration_plan(
            HigherOrderFiltrationSource::WorkflowHistory,
            history.iter().map(|entry| {
                (
                    "workflow_revision",
                    entry.revision_id.clone(),
                    entry.changed_ids.added_ids.clone(),
                )
            }),
            store,
            complex,
            source_mapping,
            max_dimension,
        ),
        HigherOrderFiltrationInput::NativeMorphismLog(history) => history_filtration_plan(
            HigherOrderFiltrationSource::NativeMorphismLog,
            history.iter().map(|entry| {
                let mut source_ids = vec![
                    entry.entry_id.clone(),
                    entry.morphism_id.clone(),
                    entry.target_revision_id.clone(),
                ];
                source_ids.extend(entry.morphism.added_ids.clone());
                (
                    "native_morphism_log_entry",
                    entry.entry_id.clone(),
                    source_ids,
                )
            }),
            store,
            complex,
            source_mapping,
            max_dimension,
        ),
    }
}

fn summarize_higher_order_persistence(
    persistence: &PersistenceSummary,
) -> HigherOrderTopologySummary {
    let mut interval_count_by_dimension = BTreeMap::new();
    let mut open_interval_count_by_dimension = BTreeMap::new();
    let mut persistent_interval_count_by_dimension = BTreeMap::new();
    let longest_lifetime_interval = longest_interval_summary(
        persistence,
        &mut interval_count_by_dimension,
        &mut open_interval_count_by_dimension,
    );

    for interval in &persistence.persistent_intervals {
        increment_count(
            &mut persistent_interval_count_by_dimension,
            interval.dimension,
        );
    }
    let betti = betti_extrema(persistence);

    HigherOrderTopologySummary {
        interval_count_by_dimension,
        open_interval_count_by_dimension,
        persistent_interval_count_by_dimension,
        longest_lifetime_interval,
        highest_nonzero_betti_dimension: betti.highest_nonzero_dimension,
        max_betti_rank: betti.max_rank,
        max_betti_rank_dimension: betti.max_rank_dimension,
    }
}

fn longest_interval_summary(
    persistence: &PersistenceSummary,
    interval_count_by_dimension: &mut BTreeMap<Dimension, usize>,
    open_interval_count_by_dimension: &mut BTreeMap<Dimension, usize>,
) -> Option<HigherOrderIntervalSummary> {
    let last_stage_index = persistence.stages.len().saturating_sub(1);
    let mut interval_summaries = persistence
        .intervals
        .iter()
        .map(|interval| {
            increment_count(interval_count_by_dimension, interval.dimension);
            if interval.is_open() {
                increment_count(open_interval_count_by_dimension, interval.dimension);
            }
            HigherOrderIntervalSummary {
                dimension: interval.dimension,
                birth_stage_id: interval.birth_stage_id.clone(),
                birth_stage_index: interval.birth_stage_index,
                death_stage_id: interval.death_stage_id.clone(),
                death_stage_index: interval.death_stage_index,
                lifetime_stages: interval.lifetime_stages(last_stage_index),
                is_open: interval.is_open(),
                generator_cell_ids: interval.generator_cell_ids.clone(),
            }
        })
        .collect::<Vec<_>>();
    interval_summaries.sort_by(interval_summary_order);
    interval_summaries.into_iter().next()
}

fn interval_summary_order(
    left: &HigherOrderIntervalSummary,
    right: &HigherOrderIntervalSummary,
) -> std::cmp::Ordering {
    right
        .lifetime_stages
        .cmp(&left.lifetime_stages)
        .then_with(|| left.dimension.cmp(&right.dimension))
        .then_with(|| left.birth_stage_index.cmp(&right.birth_stage_index))
        .then_with(|| left.birth_stage_id.cmp(&right.birth_stage_id))
        .then_with(|| left.death_stage_index.cmp(&right.death_stage_index))
        .then_with(|| left.death_stage_id.cmp(&right.death_stage_id))
        .then_with(|| left.generator_cell_ids.cmp(&right.generator_cell_ids))
}

struct BettiExtrema {
    highest_nonzero_dimension: Option<Dimension>,
    max_rank: usize,
    max_rank_dimension: Option<Dimension>,
}

fn betti_extrema(persistence: &PersistenceSummary) -> BettiExtrema {
    let mut extrema = BettiExtrema {
        highest_nonzero_dimension: None,
        max_rank: 0,
        max_rank_dimension: None,
    };
    for stage in &persistence.stages {
        for dimension in &stage.topology.homology.dimensions {
            update_betti_extrema(&mut extrema, dimension.dimension, dimension.homology_rank);
        }
    }
    extrema
}

fn update_betti_extrema(extrema: &mut BettiExtrema, dimension: Dimension, rank: usize) {
    if rank > 0 {
        extrema.highest_nonzero_dimension = Some(
            extrema
                .highest_nonzero_dimension
                .map_or(dimension, |current| current.max(dimension)),
        );
    }
    let is_lower_tie = extrema
        .max_rank_dimension
        .map_or(true, |current| dimension < current);
    if rank > extrema.max_rank || (rank == extrema.max_rank && rank > 0 && is_lower_tie) {
        extrema.max_rank = rank;
        extrema.max_rank_dimension = Some(dimension);
    }
}

fn history_filtration_plan(
    source: HigherOrderFiltrationSource,
    entries: impl Iterator<Item = (&'static str, Id, Vec<Id>)>,
    store: &InMemorySpaceStore,
    complex: &higher_graphen_space::Complex,
    source_mapping: &TopologyLiftSummary,
    max_dimension: Option<Dimension>,
) -> Result<Option<HigherOrderFiltrationPlan>, TopologyReportError> {
    let selected_cell_ids = filtration_cell_ids(store, complex, max_dimension)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if selected_cell_ids.is_empty() {
        return Ok(None);
    }

    let source_to_cell = source_to_cell_ids(source_mapping);
    let mut stages = Vec::new();
    let mut stage_sources = Vec::new();
    let mut active = BTreeSet::new();

    for entry in entries {
        append_history_stage(
            entry,
            store,
            &selected_cell_ids,
            &source_to_cell,
            &mut active,
            &mut stages,
            &mut stage_sources,
        )?;
    }
    append_remainder_stage(
        store,
        &selected_cell_ids,
        &mut active,
        &mut stages,
        &mut stage_sources,
    )?;
    filtration_plan(source, stages, stage_sources)
}

fn append_history_stage(
    entry: (&'static str, Id, Vec<Id>),
    store: &InMemorySpaceStore,
    selected_cell_ids: &BTreeSet<Id>,
    source_to_cell: &BTreeMap<Id, Id>,
    active: &mut BTreeSet<Id>,
    stages: &mut Vec<FiltrationStage>,
    stage_sources: &mut Vec<HigherOrderFiltrationStageSource>,
) -> Result<(), TopologyReportError> {
    let (source_type, source_id, requested_source_ids) = entry;
    let requested_cells = requested_source_ids
        .iter()
        .filter_map(|source_id| source_to_cell.get(source_id))
        .filter(|cell_id| selected_cell_ids.contains(*cell_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    let added_cell_ids =
        added_boundary_closed_cells(store, selected_cell_ids, active, requested_cells)?;
    if added_cell_ids.is_empty() {
        return Ok(());
    }
    active.extend(added_cell_ids.iter().cloned());
    let stage_id = Id::new(format!(
        "stage:casegraphen:{}:{}:{}",
        source_type,
        stages.len(),
        source_id.as_str()
    ))?;
    stages.push(FiltrationStage::new(
        stage_id.clone(),
        active.iter().cloned().collect::<Vec<_>>(),
    ));
    stage_sources.push(HigherOrderFiltrationStageSource {
        stage_id,
        source_type: source_type.to_owned(),
        source_id,
        added_source_ids: requested_source_ids,
        added_cell_ids,
    });
    Ok(())
}

fn append_remainder_stage(
    store: &InMemorySpaceStore,
    selected_cell_ids: &BTreeSet<Id>,
    active: &mut BTreeSet<Id>,
    stages: &mut Vec<FiltrationStage>,
    stage_sources: &mut Vec<HigherOrderFiltrationStageSource>,
) -> Result<(), TopologyReportError> {
    let remainder = selected_cell_ids
        .difference(active)
        .cloned()
        .collect::<BTreeSet<_>>();
    let added_cell_ids = added_boundary_closed_cells(store, selected_cell_ids, active, remainder)?;
    if added_cell_ids.is_empty() {
        return Ok(());
    }
    active.extend(added_cell_ids.iter().cloned());
    let stage_id = Id::new(format!(
        "stage:casegraphen:deterministic_remainder:{}",
        stages.len()
    ))?;
    stages.push(FiltrationStage::new(
        stage_id.clone(),
        active.iter().cloned().collect::<Vec<_>>(),
    ));
    stage_sources.push(HigherOrderFiltrationStageSource {
        stage_id,
        source_type: "deterministic_remainder".to_owned(),
        source_id: Id::new("filtration:deterministic_remainder")?,
        added_source_ids: Vec::new(),
        added_cell_ids,
    });
    Ok(())
}

fn added_boundary_closed_cells(
    store: &InMemorySpaceStore,
    selected_cell_ids: &BTreeSet<Id>,
    active: &BTreeSet<Id>,
    requested_cell_ids: BTreeSet<Id>,
) -> Result<Vec<Id>, TopologyReportError> {
    Ok(
        boundary_closed_cells(store, selected_cell_ids, requested_cell_ids)?
            .difference(active)
            .cloned()
            .collect(),
    )
}

fn filtration_plan(
    source: HigherOrderFiltrationSource,
    stages: Vec<FiltrationStage>,
    stage_sources: Vec<HigherOrderFiltrationStageSource>,
) -> Result<Option<HigherOrderFiltrationPlan>, TopologyReportError> {
    if stages.is_empty() {
        Ok(None)
    } else {
        Ok(Some(HigherOrderFiltrationPlan {
            source,
            stages,
            stage_sources,
        }))
    }
}

fn filtration_cell_ids(
    store: &InMemorySpaceStore,
    complex: &higher_graphen_space::Complex,
    max_dimension: Option<Dimension>,
) -> Result<Vec<Id>, TopologyReportError> {
    let mut cells = complex
        .cell_ids
        .iter()
        .map(|cell_id| {
            let cell = store
                .cell(cell_id)
                .ok_or_else(|| CoreError::MalformedField {
                    field: "cell_ids".to_owned(),
                    reason: format!("identifier {cell_id} does not exist"),
                })?;
            Ok((cell.dimension, cell.id.clone()))
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    cells.retain(|(dimension, _)| max_dimension.map_or(true, |max| *dimension <= max));
    cells.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(cells.into_iter().map(|(_, cell_id)| cell_id).collect())
}

fn cumulative_filtration_stages(
    cell_ids: &[Id],
) -> Result<Vec<FiltrationStage>, TopologyReportError> {
    let mut stages = Vec::with_capacity(cell_ids.len());
    let mut active_cell_ids = Vec::new();
    for cell_id in cell_ids {
        active_cell_ids.push(cell_id.clone());
        stages.push(FiltrationStage::new(
            Id::new(format!("stage:{}", cell_id.as_str()))?,
            active_cell_ids.clone(),
        ));
    }
    Ok(stages)
}

fn boundary_closed_cells(
    store: &InMemorySpaceStore,
    selected_cell_ids: &BTreeSet<Id>,
    requested_cell_ids: BTreeSet<Id>,
) -> Result<BTreeSet<Id>, TopologyReportError> {
    let mut closed = BTreeSet::new();
    let mut stack = requested_cell_ids.into_iter().collect::<Vec<_>>();
    while let Some(cell_id) = stack.pop() {
        if !selected_cell_ids.contains(&cell_id) || !closed.insert(cell_id.clone()) {
            continue;
        }
        let cell = store
            .cell(&cell_id)
            .ok_or_else(|| CoreError::MalformedField {
                field: "cell_ids".to_owned(),
                reason: format!("identifier {cell_id} does not exist"),
            })?;
        for boundary_cell_id in &cell.boundary {
            if selected_cell_ids.contains(boundary_cell_id) {
                stack.push(boundary_cell_id.clone());
            }
        }
    }
    Ok(closed)
}

fn source_to_cell_ids(source_mapping: &TopologyLiftSummary) -> BTreeMap<Id, Id> {
    source_mapping
        .nodes
        .iter()
        .chain(source_mapping.relations.iter())
        .map(|mapping| (mapping.source_id.clone(), mapping.cell_id.clone()))
        .collect()
}

fn increment_count(counts: &mut BTreeMap<Dimension, usize>, dimension: Dimension) {
    *counts.entry(dimension).or_insert(0) += 1;
}
