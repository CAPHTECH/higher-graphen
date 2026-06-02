//! Deterministic consistency checks over finite incidence views.

use crate::obstruction::{
    Counterexample, Obstruction, ObstructionExplanation, ObstructionType, RequiredResolution,
};
use higher_graphen_core::{CoreError, Id, Provenance, Result, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A declared cell in a lightweight incidence view.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidenceCell {
    /// Cell identifier.
    pub id: Id,
    /// Context labels explicitly declared for this cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
}

impl IncidenceCell {
    /// Creates a declared incidence-view cell.
    #[must_use]
    pub fn new(id: Id) -> Self {
        Self {
            id,
            context_ids: Vec::new(),
        }
    }

    /// Returns this cell with declared context labels.
    #[must_use]
    pub fn with_contexts<I>(mut self, context_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.context_ids = sorted_unique_ids(context_ids);
        self
    }
}

/// A non-numeric incidence relation between two declared cell identifiers.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidenceRelation {
    /// Incidence identifier.
    pub id: Id,
    /// Source endpoint cell identifier.
    pub source_cell_id: Id,
    /// Target endpoint cell identifier.
    pub target_cell_id: Id,
    /// Context labels explicitly declared for this incidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Optional obstruction type override for dangling endpoint findings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dangling_obstruction_type: Option<ObstructionType>,
    /// Optional resolution hint supplied by the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_resolution: Option<RequiredResolution>,
}

impl IncidenceRelation {
    /// Creates an incidence relation between two endpoint identifiers.
    #[must_use]
    pub fn new(id: Id, source_cell_id: Id, target_cell_id: Id) -> Self {
        Self {
            id,
            source_cell_id,
            target_cell_id,
            context_ids: Vec::new(),
            dangling_obstruction_type: None,
            required_resolution: None,
        }
    }

    /// Returns this incidence with declared context labels.
    #[must_use]
    pub fn with_contexts<I>(mut self, context_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.context_ids = sorted_unique_ids(context_ids);
        self
    }

    /// Returns this incidence with a dangling-endpoint obstruction override.
    #[must_use]
    pub fn with_dangling_obstruction_type(mut self, obstruction_type: ObstructionType) -> Self {
        self.dangling_obstruction_type = Some(obstruction_type);
        self
    }

    /// Returns this incidence with an input-supplied resolution hint.
    #[must_use]
    pub fn with_required_resolution(mut self, required_resolution: RequiredResolution) -> Self {
        self.required_resolution = Some(required_resolution);
        self
    }
}

/// A required cell set or context that must be covered by at least one incidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredRegion {
    /// Required-region identifier.
    pub id: Id,
    /// Cell identifiers that define the required region.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_ids: Vec<Id>,
    /// Context identifiers that define the required region.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Optional resolution hint supplied by the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_resolution: Option<RequiredResolution>,
}

impl RequiredRegion {
    /// Creates a required region from an explicit cell set or context set.
    pub fn new<I, C>(id: Id, cell_ids: I, context_ids: C) -> Result<Self>
    where
        I: IntoIterator<Item = Id>,
        C: IntoIterator<Item = Id>,
    {
        let cell_ids = sorted_unique_ids(cell_ids);
        let context_ids = sorted_unique_ids(context_ids);
        if cell_ids.is_empty() && context_ids.is_empty() {
            return Err(malformed_field(
                "required_region",
                "at least one cell id or context id is required",
            ));
        }
        Ok(Self {
            id,
            cell_ids,
            context_ids,
            required_resolution: None,
        })
    }

    /// Returns this required region with an input-supplied resolution hint.
    #[must_use]
    pub fn with_required_resolution(mut self, required_resolution: RequiredResolution) -> Self {
        self.required_resolution = Some(required_resolution);
        self
    }
}

/// Input to the incidence consistency engine.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidenceConsistencyInput {
    /// Space identifier for all emitted obstructions.
    pub space_id: Id,
    /// Cells explicitly supplied by the incidence view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cells: Vec<IncidenceCell>,
    /// Incidences explicitly supplied by the incidence view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidences: Vec<IncidenceRelation>,
    /// Required regions explicitly supplied by the incidence view.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_regions: Vec<RequiredRegion>,
    /// Source metadata copied to emitted obstructions.
    pub provenance: Provenance,
}

impl IncidenceConsistencyInput {
    /// Creates an empty incidence consistency input.
    #[must_use]
    pub fn new(space_id: Id, provenance: Provenance) -> Self {
        Self {
            space_id,
            cells: Vec::new(),
            incidences: Vec::new(),
            required_regions: Vec::new(),
            provenance,
        }
    }

    /// Returns this input with explicit incidence-view cells.
    #[must_use]
    pub fn with_cells<I>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = IncidenceCell>,
    {
        self.cells = cells.into_iter().collect();
        self
    }

    /// Returns this input with explicit incidence relations.
    #[must_use]
    pub fn with_incidences<I>(mut self, incidences: I) -> Self
    where
        I: IntoIterator<Item = IncidenceRelation>,
    {
        self.incidences = incidences.into_iter().collect();
        self
    }

    /// Returns this input with explicit required regions.
    #[must_use]
    pub fn with_required_regions<I>(mut self, required_regions: I) -> Self
    where
        I: IntoIterator<Item = RequiredRegion>,
    {
        self.required_regions = required_regions.into_iter().collect();
        self
    }
}

/// Structured output of an incidence consistency check.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncidenceConsistencyReport {
    /// Space checked by the engine.
    pub space_id: Id,
    /// Number of input cells observed by the engine.
    pub input_cell_count: usize,
    /// Number of input incidences observed by the engine.
    pub input_incidence_count: usize,
    /// Obstructions derivable from the explicit incidence input.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<Obstruction>,
}

/// Deterministic consistency engine for lightweight incidence views.
#[derive(Clone, Copy, Debug, Default)]
pub struct IncidenceConsistencyEngine;

impl IncidenceConsistencyEngine {
    /// Checks the supplied incidence view and emits unreviewed obstructions.
    pub fn check(&self, input: IncidenceConsistencyInput) -> Result<IncidenceConsistencyReport> {
        let cell_map = cell_map(&input.cells)?;
        let incidence_ids = unique_incidence_ids(&input.incidences)?;
        let mut obstructions = Vec::new();

        for incidence in &input.incidences {
            append_dangling_obstruction(incidence, &cell_map, &input, &mut obstructions)?;
            append_context_mismatch(incidence, &cell_map, &input, &mut obstructions)?;
        }
        for region in &input.required_regions {
            append_uncovered_region(region, &input, &cell_map, &mut obstructions)?;
        }

        obstructions.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(IncidenceConsistencyReport {
            space_id: input.space_id,
            input_cell_count: cell_map.len(),
            input_incidence_count: incidence_ids.len(),
            obstructions,
        })
    }
}

fn append_dangling_obstruction(
    incidence: &IncidenceRelation,
    cell_map: &BTreeMap<Id, IncidenceCell>,
    input: &IncidenceConsistencyInput,
    obstructions: &mut Vec<Obstruction>,
) -> Result<()> {
    let missing = missing_endpoint_ids(incidence, cell_map);
    if missing.is_empty() {
        return Ok(());
    }

    let counterexample = Counterexample::new("incidence references absent cell")?
        .with_assignment("incidence_id", incidence.id.as_str())?
        .with_assignment("missing_cell_ids", join_ids(&missing))?;
    let mut obstruction = base_obstruction(
        obstruction_id(&incidence.id, "dangling")?,
        input,
        incidence
            .dangling_obstruction_type
            .clone()
            .unwrap_or(ObstructionType::MissingMorphism),
        "incidence references absent cell",
    )?
    .with_counterexample(counterexample);

    for cell_id in endpoint_ids(incidence) {
        obstruction = obstruction.with_location_cell(cell_id);
    }
    if let Some(required_resolution) = &incidence.required_resolution {
        obstruction = obstruction.with_required_resolution(required_resolution.clone());
    }
    obstructions.push(obstruction);
    Ok(())
}

fn append_context_mismatch(
    incidence: &IncidenceRelation,
    cell_map: &BTreeMap<Id, IncidenceCell>,
    input: &IncidenceConsistencyInput,
    obstructions: &mut Vec<Obstruction>,
) -> Result<()> {
    let Some((source, target)) = present_endpoints(incidence, cell_map) else {
        return Ok(());
    };
    if contexts_compatible(source, target) {
        return Ok(());
    }

    let mut obstruction = base_obstruction(
        obstruction_id(&incidence.id, "context_mismatch")?,
        input,
        ObstructionType::ContextMismatch,
        "incidence joins incompatible contexts",
    )?
    .with_counterexample(context_counterexample(incidence, source, target)?)
    .with_location_cell(source.id.clone())
    .with_location_cell(target.id.clone());
    for context_id in context_union(source, target, incidence) {
        obstruction = obstruction.with_location_context(context_id);
    }
    if let Some(required_resolution) = &incidence.required_resolution {
        obstruction = obstruction.with_required_resolution(required_resolution.clone());
    }
    obstructions.push(obstruction);
    Ok(())
}

fn append_uncovered_region(
    region: &RequiredRegion,
    input: &IncidenceConsistencyInput,
    cell_map: &BTreeMap<Id, IncidenceCell>,
    obstructions: &mut Vec<Obstruction>,
) -> Result<()> {
    if input
        .incidences
        .iter()
        .any(|incidence| covers_region(incidence, region, cell_map))
    {
        return Ok(());
    }

    let mut obstruction = base_obstruction(
        obstruction_id(&region.id, "uncovered")?,
        input,
        ObstructionType::UncoveredRegion,
        "required region has no covering incidence",
    )?
    .with_counterexample(region_counterexample(region)?);
    for cell_id in &region.cell_ids {
        obstruction = obstruction.with_location_cell(cell_id.clone());
    }
    for context_id in &region.context_ids {
        obstruction = obstruction.with_location_context(context_id.clone());
    }
    if let Some(required_resolution) = &region.required_resolution {
        obstruction = obstruction.with_required_resolution(required_resolution.clone());
    }
    obstructions.push(obstruction);
    Ok(())
}

fn base_obstruction(
    id: Id,
    input: &IncidenceConsistencyInput,
    obstruction_type: ObstructionType,
    summary: &'static str,
) -> Result<Obstruction> {
    Ok(Obstruction::new(
        id,
        input.space_id.clone(),
        obstruction_type,
        ObstructionExplanation::new(summary)?,
        Severity::Medium,
        input.provenance.clone(),
    ))
}

fn cell_map(cells: &[IncidenceCell]) -> Result<BTreeMap<Id, IncidenceCell>> {
    let mut map = BTreeMap::new();
    for cell in cells {
        if map.insert(cell.id.clone(), cell.clone()).is_some() {
            return Err(malformed_field("cells.id", "duplicate cell id"));
        }
    }
    Ok(map)
}

fn unique_incidence_ids(incidences: &[IncidenceRelation]) -> Result<BTreeSet<Id>> {
    let mut ids = BTreeSet::new();
    for incidence in incidences {
        if !ids.insert(incidence.id.clone()) {
            return Err(malformed_field("incidences.id", "duplicate incidence id"));
        }
    }
    Ok(ids)
}

fn missing_endpoint_ids(
    incidence: &IncidenceRelation,
    cell_map: &BTreeMap<Id, IncidenceCell>,
) -> Vec<Id> {
    endpoint_ids(incidence)
        .into_iter()
        .filter(|cell_id| !cell_map.contains_key(cell_id))
        .collect()
}

fn endpoint_ids(incidence: &IncidenceRelation) -> Vec<Id> {
    sorted_unique_ids([
        incidence.source_cell_id.clone(),
        incidence.target_cell_id.clone(),
    ])
}

fn present_endpoints<'a>(
    incidence: &IncidenceRelation,
    cell_map: &'a BTreeMap<Id, IncidenceCell>,
) -> Option<(&'a IncidenceCell, &'a IncidenceCell)> {
    Some((
        cell_map.get(&incidence.source_cell_id)?,
        cell_map.get(&incidence.target_cell_id)?,
    ))
}

fn contexts_compatible(source: &IncidenceCell, target: &IncidenceCell) -> bool {
    if source.context_ids.is_empty() || target.context_ids.is_empty() {
        return true;
    }
    source
        .context_ids
        .iter()
        .any(|context_id| target.context_ids.contains(context_id))
}

fn context_counterexample(
    incidence: &IncidenceRelation,
    source: &IncidenceCell,
    target: &IncidenceCell,
) -> Result<Counterexample> {
    Counterexample::new("incidence endpoints have disjoint contexts")?
        .with_assignment("incidence_id", incidence.id.as_str())?
        .with_assignment("source_context_ids", join_ids(&source.context_ids))?
        .with_assignment("target_context_ids", join_ids(&target.context_ids))
}

fn region_counterexample(region: &RequiredRegion) -> Result<Counterexample> {
    let mut counterexample = Counterexample::new("required region has no covering incidence")?
        .with_assignment("required_region_id", region.id.as_str())?;
    if !region.cell_ids.is_empty() {
        counterexample = counterexample.with_assignment("cell_ids", join_ids(&region.cell_ids))?;
    }
    if !region.context_ids.is_empty() {
        counterexample =
            counterexample.with_assignment("context_ids", join_ids(&region.context_ids))?;
    }
    Ok(counterexample)
}

fn context_union(
    source: &IncidenceCell,
    target: &IncidenceCell,
    incidence: &IncidenceRelation,
) -> Vec<Id> {
    sorted_unique_ids(
        source
            .context_ids
            .iter()
            .chain(target.context_ids.iter())
            .chain(incidence.context_ids.iter())
            .cloned(),
    )
}

fn covers_region(
    incidence: &IncidenceRelation,
    region: &RequiredRegion,
    cell_map: &BTreeMap<Id, IncidenceCell>,
) -> bool {
    if present_endpoints(incidence, cell_map).is_none() {
        return false;
    }
    covers_cells(incidence, region) && covers_contexts(incidence, region)
}

fn covers_cells(incidence: &IncidenceRelation, region: &RequiredRegion) -> bool {
    region.cell_ids.is_empty()
        || endpoint_ids(incidence)
            .iter()
            .any(|cell_id| region.cell_ids.contains(cell_id))
}

fn covers_contexts(incidence: &IncidenceRelation, region: &RequiredRegion) -> bool {
    region.context_ids.is_empty()
        || incidence
            .context_ids
            .iter()
            .any(|context_id| region.context_ids.contains(context_id))
}

fn obstruction_id(source_id: &Id, suffix: &'static str) -> Result<Id> {
    Id::new(format!(
        "obstruction:incidence:{}:{suffix}",
        source_id.as_str()
    ))
}

fn sorted_unique_ids<I>(ids: I) -> Vec<Id>
where
    I: IntoIterator<Item = Id>,
{
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn join_ids(ids: &[Id]) -> String {
    ids.iter().map(Id::as_str).collect::<Vec<_>>().join(",")
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
