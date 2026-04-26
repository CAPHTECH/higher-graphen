//! Space, cell, incidence, complex, boundary, and storage abstractions for HigherGraphen.

pub mod traversal;

use higher_graphen_core::{CoreError, Id, Provenance, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub use traversal::*;

/// Non-negative cell dimension.
pub type Dimension = u32;

/// Directionality for an incidence between cells.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidenceOrientation {
    /// The incidence has a source and target direction.
    Directed,
    /// The incidence connects cells without source-target direction.
    Undirected,
}

/// Structural kind represented by a complex.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplexType {
    /// A typed graph complex.
    TypedGraph,
    /// A hypergraph complex.
    Hypergraph,
    /// A simplicial complex.
    SimplicialComplex,
    /// A cell complex.
    CellComplex,
    /// A downstream structural kind that is not product-specific.
    Custom(String),
}

/// Top-level structural container for cells, incidences, complexes, and contexts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Space {
    /// Stable space identifier.
    pub id: Id,
    /// Human-readable space name.
    pub name: String,
    /// Optional space description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Cells owned by the space.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_ids: Vec<Id>,
    /// Incidences owned by the space.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
    /// Complexes owned by the space.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub complex_ids: Vec<Id>,
    /// Context identifiers referenced by cells in the space.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
}

impl Space {
    /// Creates a space with empty structural membership lists.
    #[must_use]
    pub fn new(id: Id, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into().trim().to_owned(),
            description: None,
            cell_ids: Vec::new(),
            incidence_ids: Vec::new(),
            complex_ids: Vec::new(),
            context_ids: Vec::new(),
        }
    }

    /// Returns this space with an optional description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Typed structural element inside a space.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    /// Stable cell identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Cell dimension.
    pub dimension: Dimension,
    /// Abstract or downstream-owned cell type.
    pub cell_type: String,
    /// Optional display label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Lower-dimensional cells on this cell's boundary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary: Vec<Id>,
    /// Higher-dimensional cells that include this cell on their boundary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coboundary: Vec<Id>,
    /// Contexts in which this cell participates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    /// Source and review metadata for this cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Cell {
    /// Creates a cell with empty boundary, coboundary, and context membership.
    #[must_use]
    pub fn new(id: Id, space_id: Id, dimension: Dimension, cell_type: impl Into<String>) -> Self {
        Self {
            id,
            space_id,
            dimension,
            cell_type: cell_type.into().trim().to_owned(),
            label: None,
            boundary: Vec::new(),
            coboundary: Vec::new(),
            context_ids: Vec::new(),
            provenance: None,
        }
    }

    /// Returns this cell with a display label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Returns this cell with a boundary cell identifier appended.
    #[must_use]
    pub fn with_boundary_cell(mut self, cell_id: Id) -> Self {
        push_unique(&mut self.boundary, cell_id);
        self
    }

    /// Returns this cell with a context identifier appended.
    #[must_use]
    pub fn with_context(mut self, context_id: Id) -> Self {
        push_unique(&mut self.context_ids, context_id);
        self
    }

    /// Returns this cell with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Relation record between two cells in a space.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Incidence {
    /// Stable incidence identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Source cell identifier.
    pub from_cell_id: Id,
    /// Target cell identifier.
    pub to_cell_id: Id,
    /// Abstract relation type.
    pub relation_type: String,
    /// Relation directionality.
    pub orientation: IncidenceOrientation,
    /// Optional relation weight. Insertion rejects non-finite values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Source and review metadata for this incidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Incidence {
    /// Creates an incidence without weight or provenance.
    #[must_use]
    pub fn new(
        id: Id,
        space_id: Id,
        from_cell_id: Id,
        to_cell_id: Id,
        relation_type: impl Into<String>,
        orientation: IncidenceOrientation,
    ) -> Self {
        Self {
            id,
            space_id,
            from_cell_id,
            to_cell_id,
            relation_type: relation_type.into().trim().to_owned(),
            orientation,
            weight: None,
            provenance: None,
        }
    }

    /// Returns this incidence with a finite weight.
    #[must_use]
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Returns this incidence with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Organized collection of cells and incidences inside one space.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Complex {
    /// Stable complex identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Human-readable complex name.
    pub name: String,
    /// Cells included in the complex.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_ids: Vec<Id>,
    /// Incidences included in the complex.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
    /// Highest dimension across included cells.
    pub max_dimension: Dimension,
    /// Structural complex kind.
    pub complex_type: ComplexType,
}

impl Complex {
    /// Creates an empty complex with max dimension zero.
    #[must_use]
    pub fn new(id: Id, space_id: Id, name: impl Into<String>, complex_type: ComplexType) -> Self {
        Self {
            id,
            space_id,
            name: name.into().trim().to_owned(),
            cell_ids: Vec::new(),
            incidence_ids: Vec::new(),
            max_dimension: 0,
            complex_type,
        }
    }
}

impl ComplexType {
    /// Creates a downstream-owned complex type after trimming surrounding whitespace.
    pub fn custom(extension: impl Into<String>) -> Result<Self> {
        let normalized = normalize_required("complex_type", extension.into())?;
        Ok(Self::Custom(normalized))
    }
}

/// Boundary closure of a complex's cells.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexClosure {
    /// Complex whose closure was computed.
    pub complex_id: Id,
    /// Complex cells plus every recursively reachable boundary cell.
    pub cell_ids: Vec<Id>,
}

/// A complex cell whose direct boundary is not fully included in the complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexClosureViolation {
    /// Cell with a missing direct boundary member.
    pub cell_id: Id,
    /// Boundary cells referenced by `cell_id` but absent from the complex.
    pub missing_boundary_cell_ids: Vec<Id>,
}

/// Result of checking whether a complex contains the full boundary closure of its cells.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexClosureValidation {
    /// Complex whose closure was checked.
    pub complex_id: Id,
    /// Missing boundary cells across all checked cells.
    pub missing_boundary_cell_ids: Vec<Id>,
    /// Per-cell closure violations.
    pub violations: Vec<ComplexClosureViolation>,
}

impl ComplexClosureValidation {
    /// Returns true when every direct boundary reference is included in the complex.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Boundary cells directly referenced by cells in a complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexBoundary {
    /// Complex whose boundary was computed.
    pub complex_id: Id,
    /// Boundary cells that are also included in the complex.
    pub cell_ids: Vec<Id>,
    /// Boundary cells that exist in the same space but are outside the complex.
    pub external_cell_ids: Vec<Id>,
}

/// Coboundary cells that directly include cells from a complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexCoboundary {
    /// Complex whose coboundary was computed.
    pub complex_id: Id,
    /// Coboundary cells that are also included in the complex.
    pub cell_ids: Vec<Id>,
    /// Coboundary cells that exist in the same space but are outside the complex.
    pub external_cell_ids: Vec<Id>,
}

/// Star and link-style neighborhood around seed cells inside a complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexNeighborhood {
    /// Complex that bounds the neighborhood.
    pub complex_id: Id,
    /// Unique seed cells used to compute the neighborhood.
    pub seed_cell_ids: Vec<Id>,
    /// Boundary closure of the seed cells.
    pub seed_closure_cell_ids: Vec<Id>,
    /// Complex cells whose closure contains at least one seed cell.
    pub coface_cell_ids: Vec<Id>,
    /// Closed star: coface cells plus their boundary closure, constrained to complex cells.
    pub star_cell_ids: Vec<Id>,
    /// Link-style shell: closed-star cells whose own closure does not intersect the seed closure.
    pub link_cell_ids: Vec<Id>,
}

/// Coverage report for a finite set of cells over a complex.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegionCoverage {
    /// Complex whose coverage was computed.
    pub complex_id: Id,
    /// Unique requested cover cells after duplicate removal.
    pub requested_cell_ids: Vec<Id>,
    /// Requested cover cells that were repeated.
    pub duplicate_cell_ids: Vec<Id>,
    /// Requested cells that exist in the same space but are outside the complex.
    pub external_cell_ids: Vec<Id>,
    /// Complex cells covered by requested cells and their boundary closure.
    pub covered_cell_ids: Vec<Id>,
    /// Complex cells not covered by requested cells and their boundary closure.
    pub uncovered_cell_ids: Vec<Id>,
}

impl RegionCoverage {
    /// Returns true when the requested region covers every complex cell and has no external cells.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.external_cell_ids.is_empty() && self.uncovered_cell_ids.is_empty()
    }
}

/// Cell query selectors supported by the MVP in-memory store.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CellQuery {
    /// Optional owning space filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<Id>,
    /// Optional cell type filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_type: Option<String>,
    /// Optional cell dimension filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<Dimension>,
    /// Optional context membership filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<Id>,
}

impl CellQuery {
    /// Creates an unconstrained cell query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this query with a space filter.
    #[must_use]
    pub fn in_space(mut self, space_id: Id) -> Self {
        self.space_id = Some(space_id);
        self
    }

    /// Returns this query with a cell type filter.
    #[must_use]
    pub fn of_type(mut self, cell_type: impl Into<String>) -> Self {
        self.cell_type = Some(cell_type.into().trim().to_owned());
        self
    }

    /// Returns this query with a dimension filter.
    #[must_use]
    pub fn with_dimension(mut self, dimension: Dimension) -> Self {
        self.dimension = Some(dimension);
        self
    }

    /// Returns this query with a context membership filter.
    #[must_use]
    pub fn in_context(mut self, context_id: Id) -> Self {
        self.context_id = Some(context_id);
        self
    }

    fn matches(&self, cell: &Cell) -> bool {
        if let Some(space_id) = &self.space_id {
            if &cell.space_id != space_id {
                return false;
            }
        }
        if let Some(cell_type) = &self.cell_type {
            if &cell.cell_type != cell_type {
                return false;
            }
        }
        if let Some(dimension) = self.dimension {
            if cell.dimension != dimension {
                return false;
            }
        }
        if let Some(context_id) = &self.context_id {
            if !cell.context_ids.contains(context_id) {
                return false;
            }
        }
        true
    }
}

mod store;
pub use store::InMemorySpaceStore;

fn normalize_required(field: &str, value: String) -> Result<String> {
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        Err(malformed(field, "value must not be empty after trimming"))
    } else {
        Ok(normalized)
    }
}

fn unique_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    let mut unique = Vec::new();
    for id in ids {
        push_unique(&mut unique, id);
    }
    unique
}

fn id_set(ids: &[Id]) -> BTreeSet<Id> {
    ids.iter().cloned().collect()
}

fn ids_from_set(ids: BTreeSet<Id>) -> Vec<Id> {
    ids.into_iter().collect()
}

fn insert_by_membership(
    id: &Id,
    members: &BTreeSet<Id>,
    included: &mut BTreeSet<Id>,
    external: &mut BTreeSet<Id>,
) {
    if members.contains(id) {
        included.insert(id.clone());
    } else {
        external.insert(id.clone());
    }
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn ensure_absent(occupied: bool, field: &str, id: &Id) -> Result<()> {
    if occupied {
        Err(malformed(
            field,
            format!("identifier {id} already exists in the store"),
        ))
    } else {
        Ok(())
    }
}

fn ensure_empty(field: &str, ids: &[Id]) -> Result<()> {
    if ids.is_empty() {
        Ok(())
    } else {
        Err(malformed(
            field,
            "store-owned membership lists must be populated through insert operations",
        ))
    }
}

fn normalize_complex_type(complex_type: ComplexType) -> Result<ComplexType> {
    match complex_type {
        ComplexType::Custom(extension) => ComplexType::custom(extension),
        built_in => Ok(built_in),
    }
}

fn missing(field: &str, id: &Id) -> CoreError {
    malformed(
        field,
        format!("identifier {id} does not exist in the store"),
    )
}

fn wrong_space(field: &str, id: &Id, expected: &Id, actual: &Id) -> CoreError {
    malformed(
        field,
        format!("identifier {id} belongs to space {actual}, expected {expected}"),
    )
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
