//! Mathematical diagnostics projected into CaseGraphen report sections.

use crate::native_model::{CaseMorphism, CaseSpace};
use higher_graphen_core::{CoreError, Id};
use higher_graphen_reasoning::model_checking::{
    check_dead_ends, check_required_event, DeadEndQuery, ModelCheckingOptions, RequiredEventQuery,
    TemporalCheckReport,
};
use higher_graphen_structure::space::{
    Cell, InMemorySpaceStore, Incidence, IncidenceOrientation, Space, TraversalDirection,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Native temporal diagnostics derived from a finite CaseGraphen state model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTemporalDiagnostics {
    /// Bounded temporal checks that were run.
    pub temporal_checks: Vec<NamedTemporalCheck>,
}

/// Named temporal check report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamedTemporalCheck {
    /// Stable diagnostic identifier.
    pub id: Id,
    /// Human-readable mathematical role.
    pub purpose: String,
    /// Underlying model-checking report.
    pub report: TemporalCheckReport,
}

/// Builds temporal diagnostics for native close-check reports.
pub fn native_close_temporal_diagnostics(
    case_space: &CaseSpace,
    validation_evidence_ids: &[Id],
) -> Result<NativeTemporalDiagnostics, CoreError> {
    if case_space.morphism_log.is_empty() {
        return Ok(NativeTemporalDiagnostics {
            temporal_checks: Vec::new(),
        });
    }
    let mut model = MorphismLogModel::new(case_space)?;
    for validation_evidence_id in validation_evidence_ids {
        model.add_validation_evidence_loop(validation_evidence_id)?;
    }

    let options = ModelCheckingOptions::new()
        .in_direction(TraversalDirection::Outgoing)
        .with_relation_type("morphism_log_next");
    let mut temporal_checks = vec![NamedTemporalCheck {
        id: diagnostic_id("temporal:no-dead-end-except-current-revision", case_space)?,
        purpose: "morphism log has no dead-end state except the current revision".to_owned(),
        report: check_dead_ends(
            &DeadEndQuery::new(
                model.space_id.clone(),
                [model.initial_cell_id.clone()],
                [model.terminal_cell_id.clone()],
            )
            .with_options(options),
            &model.store,
        )?,
    }];

    if !validation_evidence_ids.is_empty() {
        temporal_checks.push(NamedTemporalCheck {
            id: diagnostic_id("temporal:validation-evidence-eventual", case_space)?,
            purpose: "a declared validation evidence event appears on the morphism log trace"
                .to_owned(),
            report: check_required_event(
                &RequiredEventQuery::new(
                    model.space_id.clone(),
                    [model.initial_cell_id],
                    ["validation_evidence_named".to_owned()],
                )
                .with_options(
                    ModelCheckingOptions::new().in_direction(TraversalDirection::Outgoing),
                ),
                &model.store,
            )?,
        });
    }

    Ok(NativeTemporalDiagnostics { temporal_checks })
}

/// Builds temporal diagnostics for native morphism-check reports.
pub fn native_morphism_temporal_diagnostics(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
) -> Result<NativeTemporalDiagnostics, CoreError> {
    let space_id = diagnostic_id("space:morphism-check", case_space)?;
    let source_cell_id = morphism
        .source_revision_id
        .clone()
        .unwrap_or_else(|| case_space.revision.revision_id.clone());
    let target_cell_id = morphism.target_revision_id.clone();
    let mut store = InMemorySpaceStore::new();
    store.insert_space(Space::new(
        space_id.clone(),
        "Native morphism temporal check",
    ))?;
    store.insert_cell(Cell::new(
        source_cell_id.clone(),
        space_id.clone(),
        0,
        "source_revision",
    ))?;
    if target_cell_id != source_cell_id {
        store.insert_cell(Cell::new(
            target_cell_id.clone(),
            space_id.clone(),
            0,
            "target_revision",
        ))?;
    }
    let relation_type = morphism.morphism_type.serialized_value();
    store.insert_incidence(Incidence::new(
        diagnostic_id("incidence:morphism-transition", case_space)?,
        space_id.clone(),
        source_cell_id.clone(),
        target_cell_id.clone(),
        relation_type.clone(),
        IncidenceOrientation::Directed,
    ))?;

    let transition_report = check_required_event(
        &RequiredEventQuery::new(space_id.clone(), [source_cell_id.clone()], [relation_type])
            .with_options(ModelCheckingOptions::new().in_direction(TraversalDirection::Outgoing)),
        &store,
    )?;
    let terminal_report = check_dead_ends(
        &DeadEndQuery::new(space_id, [source_cell_id], [target_cell_id])
            .with_options(ModelCheckingOptions::new().in_direction(TraversalDirection::Outgoing)),
        &store,
    )?;

    Ok(NativeTemporalDiagnostics {
        temporal_checks: vec![
            NamedTemporalCheck {
                id: diagnostic_id("temporal:morphism-transition-eventual", case_space)?,
                purpose: "candidate morphism reaches its target revision".to_owned(),
                report: transition_report,
            },
            NamedTemporalCheck {
                id: diagnostic_id("temporal:morphism-target-terminal", case_space)?,
                purpose: "candidate morphism target revision is the terminal state".to_owned(),
                report: terminal_report,
            },
        ],
    })
}

struct MorphismLogModel {
    store: InMemorySpaceStore,
    space_id: Id,
    initial_cell_id: Id,
    terminal_cell_id: Id,
    event_cell_ids_by_id: BTreeMap<Id, Vec<Id>>,
}

impl MorphismLogModel {
    fn new(case_space: &CaseSpace) -> Result<Self, CoreError> {
        let space_id = diagnostic_id("space:morphism-log", case_space)?;
        let mut store = InMemorySpaceStore::new();
        store.insert_space(Space::new(
            space_id.clone(),
            "Native morphism log temporal model",
        ))?;
        let cell_ids = case_space
            .morphism_log
            .iter()
            .map(|entry| diagnostic_id(&format!("state:{}", entry.sequence), case_space))
            .collect::<Result<Vec<_>, _>>()?;
        let mut event_cell_ids_by_id: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        for (entry, cell_id) in case_space.morphism_log.iter().zip(&cell_ids) {
            store.insert_cell(Cell::new(
                cell_id.clone(),
                space_id.clone(),
                0,
                entry.morphism.morphism_type.serialized_value(),
            ))?;
            for event_id in morphism_event_ids(&entry.morphism) {
                event_cell_ids_by_id
                    .entry(event_id)
                    .or_default()
                    .push(cell_id.clone());
            }
        }
        for pair in cell_ids.windows(2) {
            let from = &pair[0];
            let to = &pair[1];
            store.insert_incidence(Incidence::new(
                diagnostic_id(&format!("incidence:{}:{}", from, to), case_space)?,
                space_id.clone(),
                from.clone(),
                to.clone(),
                "morphism_log_next",
                IncidenceOrientation::Directed,
            ))?;
        }
        Ok(Self {
            store,
            space_id,
            initial_cell_id: cell_ids[0].clone(),
            terminal_cell_id: cell_ids[cell_ids.len() - 1].clone(),
            event_cell_ids_by_id,
        })
    }

    fn add_validation_evidence_loop(
        &mut self,
        validation_evidence_id: &Id,
    ) -> Result<(), CoreError> {
        for cell_id in self
            .event_cell_ids_by_id
            .get(validation_evidence_id)
            .cloned()
            .unwrap_or_default()
        {
            self.store.insert_incidence(Incidence::new(
                Id::new(format!(
                    "incidence:validation-evidence:{}:{}",
                    sanitize_id(validation_evidence_id),
                    sanitize_id(&cell_id)
                ))?,
                self.space_id.clone(),
                cell_id.clone(),
                cell_id,
                "validation_evidence_named",
                IncidenceOrientation::Directed,
            ))?;
        }
        Ok(())
    }
}

fn morphism_event_ids(morphism: &CaseMorphism) -> BTreeSet<Id> {
    morphism
        .added_ids
        .iter()
        .chain(&morphism.updated_ids)
        .chain(&morphism.retired_ids)
        .chain(&morphism.preserved_ids)
        .chain(&morphism.evidence_ids)
        .chain(&morphism.source_ids)
        .cloned()
        .collect()
}

fn diagnostic_id(prefix: &str, case_space: &CaseSpace) -> Result<Id, CoreError> {
    Id::new(format!(
        "{prefix}:{}",
        sanitize_id(&case_space.case_space_id)
    ))
}

fn sanitize_id(id: &Id) -> String {
    id.as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':') {
                character
            } else {
                '-'
            }
        })
        .collect()
}
