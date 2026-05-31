use super::*;
use crate::morphism::{
    construct_explicit_pullback, construct_explicit_pushout, Morphism, PullbackInputs,
    PullbackOutcome, PushoutInputs, PushoutOutcome,
};
use crate::space::ComplexType;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// In-memory MVP store for spaces, cells, incidences, complexes, and basic cell queries.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InMemorySpaceStore {
    pub(crate) spaces: BTreeMap<Id, Space>,
    pub(crate) cells: BTreeMap<Id, Cell>,
    pub(crate) incidences: BTreeMap<Id, Incidence>,
    pub(crate) complexes: BTreeMap<Id, Complex>,
}

impl InMemorySpaceStore {
    /// Creates an empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a space and returns the normalized stored value.
    pub fn insert_space(&mut self, space: Space) -> Result<Space> {
        let mut space = space;
        space.name = normalize_required("name", space.name)?;
        ensure_empty("cell_ids", &space.cell_ids)?;
        ensure_empty("incidence_ids", &space.incidence_ids)?;
        ensure_empty("complex_ids", &space.complex_ids)?;
        space.cell_ids = unique_ids(space.cell_ids);
        space.incidence_ids = unique_ids(space.incidence_ids);
        space.complex_ids = unique_ids(space.complex_ids);
        space.context_ids = unique_ids(space.context_ids);
        self.ensure_space_absent(&space.id)?;

        self.spaces.insert(space.id.clone(), space.clone());
        Ok(space)
    }

    /// Inserts a cell, registers it with its space, and updates boundary/coboundary inverses.
    pub fn insert_cell(&mut self, cell: Cell) -> Result<Cell> {
        let mut cell = cell;
        cell.cell_type = normalize_required("cell_type", cell.cell_type)?;
        cell.boundary = unique_ids(cell.boundary);
        cell.coboundary = unique_ids(cell.coboundary);
        cell.context_ids = unique_ids(cell.context_ids);
        self.ensure_cell_absent(&cell.id)?;
        self.validate_cell_references(&cell)?;

        self.cells.insert(cell.id.clone(), cell.clone());
        self.register_cell_in_space(&cell);
        self.register_boundary_inverses(&cell);
        Ok(cell)
    }

    /// Inserts an incidence after validating both endpoint cells are in the same space.
    pub fn insert_incidence(&mut self, incidence: Incidence) -> Result<Incidence> {
        let mut incidence = incidence;
        incidence.relation_type = normalize_required("relation_type", incidence.relation_type)?;
        self.ensure_incidence_absent(&incidence.id)?;
        self.validate_incidence_references(&incidence)?;

        self.incidences
            .insert(incidence.id.clone(), incidence.clone());
        let space = self
            .spaces
            .get_mut(&incidence.space_id)
            .expect("validated incidence space should exist");
        push_unique(&mut space.incidence_ids, incidence.id.clone());
        Ok(incidence)
    }

    /// Inserts a complex after validating membership and recomputing max dimension.
    pub fn insert_complex(&mut self, complex: Complex) -> Result<Complex> {
        let mut complex = complex;
        complex.name = normalize_required("name", complex.name)?;
        complex.complex_type = normalize_complex_type(complex.complex_type)?;
        complex.cell_ids = unique_ids(complex.cell_ids);
        complex.incidence_ids = unique_ids(complex.incidence_ids);
        self.ensure_complex_absent(&complex.id)?;
        complex.max_dimension = self.validate_complex_references(&complex)?;

        self.complexes.insert(complex.id.clone(), complex.clone());
        let space = self
            .spaces
            .get_mut(&complex.space_id)
            .expect("validated complex space should exist");
        push_unique(&mut space.complex_ids, complex.id.clone());
        Ok(complex)
    }

    /// Constructs, inserts, and returns a complex from existing cells and incidences.
    pub fn construct_complex(
        &mut self,
        id: Id,
        space_id: Id,
        name: impl Into<String>,
        complex_type: ComplexType,
        cell_ids: impl IntoIterator<Item = Id>,
        incidence_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Complex> {
        let mut complex = Complex::new(id, space_id, name, complex_type);
        complex.cell_ids = unique_ids(cell_ids);
        complex.incidence_ids = unique_ids(incidence_ids);
        self.insert_complex(complex)
    }

    /// Constructs a finite pushout candidate over a cospan whose legs are the
    /// two morphisms, gathering target-space cells and incidences from this
    /// store. Read-only: the constructed candidate is returned, never inserted.
    pub fn construct_pushout(
        &self,
        left: &Morphism,
        right: &Morphism,
        candidate_space_id: Id,
        candidate_space_name: String,
        complex_type: ComplexType,
    ) -> std::result::Result<PushoutOutcome, CoreError> {
        if !self.spaces.contains_key(&left.target_space_id) {
            return Err(malformed(
                "left",
                format!(
                    "target space identifier {} does not exist in the store",
                    left.target_space_id
                ),
            ));
        }
        if !self.spaces.contains_key(&right.target_space_id) {
            return Err(malformed(
                "right",
                format!(
                    "target space identifier {} does not exist in the store",
                    right.target_space_id
                ),
            ));
        }

        let mut left_cells = self
            .cells
            .values()
            .filter(|cell| cell.space_id == left.target_space_id)
            .cloned()
            .collect::<Vec<_>>();
        left_cells.sort_by(|left, right| left.id.cmp(&right.id));

        let mut right_cells = self
            .cells
            .values()
            .filter(|cell| cell.space_id == right.target_space_id)
            .cloned()
            .collect::<Vec<_>>();
        right_cells.sort_by(|left, right| left.id.cmp(&right.id));

        let mut left_incidences = self
            .incidences
            .values()
            .filter(|incidence| incidence.space_id == left.target_space_id)
            .cloned()
            .collect::<Vec<_>>();
        left_incidences.sort_by(|left, right| left.id.cmp(&right.id));

        let mut right_incidences = self
            .incidences
            .values()
            .filter(|incidence| incidence.space_id == right.target_space_id)
            .cloned()
            .collect::<Vec<_>>();
        right_incidences.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(construct_explicit_pushout(PushoutInputs {
            left,
            right,
            candidate_space_id,
            candidate_space_name,
            complex_type,
            left_cells: &left_cells,
            right_cells: &right_cells,
            left_incidences: &left_incidences,
            right_incidences: &right_incidences,
        }))
    }

    /// Constructs a finite pullback candidate over a cospan whose legs are the
    /// two morphisms, gathering source-space cells and incidences from this
    /// store. Read-only: the constructed candidate is returned, never inserted.
    pub fn construct_pullback(
        &self,
        left: &Morphism,
        right: &Morphism,
        candidate_space_id: Id,
        candidate_space_name: String,
        complex_type: ComplexType,
    ) -> std::result::Result<PullbackOutcome, CoreError> {
        if !self.spaces.contains_key(&left.source_space_id) {
            return Err(malformed(
                "left",
                format!(
                    "source space identifier {} does not exist in the store",
                    left.source_space_id
                ),
            ));
        }
        if !self.spaces.contains_key(&right.source_space_id) {
            return Err(malformed(
                "right",
                format!(
                    "source space identifier {} does not exist in the store",
                    right.source_space_id
                ),
            ));
        }

        let mut left_source_cells = self
            .cells
            .values()
            .filter(|cell| cell.space_id == left.source_space_id)
            .cloned()
            .collect::<Vec<_>>();
        left_source_cells.sort_by(|left, right| left.id.cmp(&right.id));

        let mut right_source_cells = self
            .cells
            .values()
            .filter(|cell| cell.space_id == right.source_space_id)
            .cloned()
            .collect::<Vec<_>>();
        right_source_cells.sort_by(|left, right| left.id.cmp(&right.id));

        let mut left_source_incidences = self
            .incidences
            .values()
            .filter(|incidence| incidence.space_id == left.source_space_id)
            .cloned()
            .collect::<Vec<_>>();
        left_source_incidences.sort_by(|left, right| left.id.cmp(&right.id));

        let mut right_source_incidences = self
            .incidences
            .values()
            .filter(|incidence| incidence.space_id == right.source_space_id)
            .cloned()
            .collect::<Vec<_>>();
        right_source_incidences.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(construct_explicit_pullback(
            PullbackInputs {
                left: left.clone(),
                right: right.clone(),
                left_source_cells,
                right_source_cells,
                left_source_incidences,
                right_source_incidences,
            },
            candidate_space_id,
            candidate_space_name,
            complex_type,
        ))
    }

    /// Returns a space by identifier.
    #[must_use]
    pub fn space(&self, id: &Id) -> Option<&Space> {
        self.spaces.get(id)
    }

    /// Returns a cell by identifier.
    #[must_use]
    pub fn cell(&self, id: &Id) -> Option<&Cell> {
        self.cells.get(id)
    }

    /// Returns an incidence by identifier.
    #[must_use]
    pub fn incidence(&self, id: &Id) -> Option<&Incidence> {
        self.incidences.get(id)
    }

    /// Returns a complex by identifier.
    #[must_use]
    pub fn complex(&self, id: &Id) -> Option<&Complex> {
        self.complexes.get(id)
    }

    /// Returns cells matching all supplied query selectors.
    #[must_use]
    pub fn query_cells(&self, query: &CellQuery) -> Vec<Cell> {
        self.cells
            .values()
            .filter(|cell| query.matches(cell))
            .cloned()
            .collect()
    }

    /// Returns the recursive boundary closure for a complex.
    pub fn complex_closure(&self, complex_id: &Id) -> Result<ComplexClosure> {
        let complex = self.complex_by_id(complex_id)?;
        let closure = self.closure_for_cells(
            "cell_ids",
            &complex.space_id,
            complex.cell_ids.iter().cloned(),
        )?;

        Ok(ComplexClosure {
            complex_id: complex.id.clone(),
            cell_ids: ids_from_set(closure),
        })
    }

    /// Validates that every complex cell's direct boundary is included in the complex.
    pub fn validate_complex_closure(&self, complex_id: &Id) -> Result<ComplexClosureValidation> {
        let complex = self.complex_by_id(complex_id)?;
        let complex_cell_ids = id_set(&complex.cell_ids);
        let mut missing_boundary_cell_ids = BTreeSet::new();
        let mut violations = Vec::new();

        for cell_id in &complex.cell_ids {
            let cell = self.cell_in_space("cell_ids", cell_id, &complex.space_id)?;
            let missing = cell
                .boundary
                .iter()
                .filter(|boundary_id| !complex_cell_ids.contains(*boundary_id))
                .cloned()
                .collect::<BTreeSet<_>>();
            if missing.is_empty() {
                continue;
            }

            missing_boundary_cell_ids.extend(missing.iter().cloned());
            violations.push(ComplexClosureViolation {
                cell_id: cell_id.clone(),
                missing_boundary_cell_ids: ids_from_set(missing),
            });
        }

        Ok(ComplexClosureValidation {
            complex_id: complex.id.clone(),
            missing_boundary_cell_ids: ids_from_set(missing_boundary_cell_ids),
            violations,
        })
    }

    /// Returns direct boundary cells referenced by cells in a complex.
    pub fn complex_boundary(&self, complex_id: &Id) -> Result<ComplexBoundary> {
        let complex = self.complex_by_id(complex_id)?;
        let complex_cell_ids = id_set(&complex.cell_ids);
        let mut boundary_cell_ids = BTreeSet::new();
        let mut external_cell_ids = BTreeSet::new();

        for cell_id in &complex.cell_ids {
            let cell = self.cell_in_space("cell_ids", cell_id, &complex.space_id)?;
            for boundary_id in &cell.boundary {
                insert_by_membership(
                    boundary_id,
                    &complex_cell_ids,
                    &mut boundary_cell_ids,
                    &mut external_cell_ids,
                );
            }
        }

        Ok(ComplexBoundary {
            complex_id: complex.id.clone(),
            cell_ids: ids_from_set(boundary_cell_ids),
            external_cell_ids: ids_from_set(external_cell_ids),
        })
    }

    /// Returns direct coboundary cells that include cells from a complex.
    pub fn complex_coboundary(&self, complex_id: &Id) -> Result<ComplexCoboundary> {
        let complex = self.complex_by_id(complex_id)?;
        let complex_cell_ids = id_set(&complex.cell_ids);
        let mut coboundary_cell_ids = BTreeSet::new();
        let mut external_cell_ids = BTreeSet::new();

        for cell_id in &complex.cell_ids {
            let cell = self.cell_in_space("cell_ids", cell_id, &complex.space_id)?;
            for coboundary_id in &cell.coboundary {
                insert_by_membership(
                    coboundary_id,
                    &complex_cell_ids,
                    &mut coboundary_cell_ids,
                    &mut external_cell_ids,
                );
            }
        }

        Ok(ComplexCoboundary {
            complex_id: complex.id.clone(),
            cell_ids: ids_from_set(coboundary_cell_ids),
            external_cell_ids: ids_from_set(external_cell_ids),
        })
    }

    /// Computes a closed-star and link-style neighborhood for seed cells inside a complex.
    pub fn complex_neighborhood(
        &self,
        complex_id: &Id,
        seed_cell_ids: impl IntoIterator<Item = Id>,
    ) -> Result<ComplexNeighborhood> {
        let complex = self.complex_by_id(complex_id)?;
        let complex_cell_ids = id_set(&complex.cell_ids);
        let seed_cell_ids =
            self.normalize_complex_member_ids(complex, "seed_cell_ids", seed_cell_ids)?;
        if seed_cell_ids.is_empty() {
            return Err(malformed(
                "seed_cell_ids",
                "at least one seed cell is required",
            ));
        }

        let seed_closure =
            self.closure_for_cells("seed_cell_ids", &complex.space_id, seed_cell_ids.clone())?;
        let mut coface_cell_ids = BTreeSet::new();
        let mut star_cell_ids = BTreeSet::new();

        for cell_id in &complex.cell_ids {
            let candidate_closure =
                self.closure_for_cells("cell_ids", &complex.space_id, [cell_id.clone()])?;
            if candidate_closure.is_disjoint(&seed_cell_ids) {
                continue;
            }

            coface_cell_ids.insert(cell_id.clone());
            for closure_id in candidate_closure {
                if complex_cell_ids.contains(&closure_id) {
                    star_cell_ids.insert(closure_id);
                }
            }
        }

        let mut link_cell_ids = BTreeSet::new();
        for cell_id in &star_cell_ids {
            let candidate_closure =
                self.closure_for_cells("cell_ids", &complex.space_id, [cell_id.clone()])?;
            if candidate_closure.is_disjoint(&seed_closure) {
                link_cell_ids.insert(cell_id.clone());
            }
        }

        Ok(ComplexNeighborhood {
            complex_id: complex.id.clone(),
            seed_cell_ids: ids_from_set(seed_cell_ids),
            seed_closure_cell_ids: ids_from_set(seed_closure),
            coface_cell_ids: ids_from_set(coface_cell_ids),
            star_cell_ids: ids_from_set(star_cell_ids),
            link_cell_ids: ids_from_set(link_cell_ids),
        })
    }

    /// Computes covered and uncovered cells for a requested region over a complex.
    pub fn covered_region(
        &self,
        complex_id: &Id,
        covered_cell_ids: impl IntoIterator<Item = Id>,
    ) -> Result<RegionCoverage> {
        let complex = self.complex_by_id(complex_id)?;
        let complex_cell_ids = id_set(&complex.cell_ids);
        let mut requested_cell_ids = BTreeSet::new();
        let mut duplicate_cell_ids = BTreeSet::new();
        let mut external_cell_ids = BTreeSet::new();
        let mut cover_seed_ids = Vec::new();

        for cell_id in covered_cell_ids {
            self.cell_in_space("covered_cell_ids", &cell_id, &complex.space_id)?;
            if !requested_cell_ids.insert(cell_id.clone()) {
                duplicate_cell_ids.insert(cell_id);
                continue;
            }

            if complex_cell_ids.contains(&cell_id) {
                cover_seed_ids.push(cell_id);
            } else {
                external_cell_ids.insert(cell_id);
            }
        }

        let mut covered_cell_ids = BTreeSet::new();
        for cell_id in cover_seed_ids {
            let closure =
                self.closure_for_cells("covered_cell_ids", &complex.space_id, [cell_id])?;
            covered_cell_ids.extend(
                closure
                    .into_iter()
                    .filter(|closure_id| complex_cell_ids.contains(closure_id)),
            );
        }

        let uncovered_cell_ids = complex_cell_ids
            .difference(&covered_cell_ids)
            .cloned()
            .collect();

        Ok(RegionCoverage {
            complex_id: complex.id.clone(),
            requested_cell_ids: ids_from_set(requested_cell_ids),
            duplicate_cell_ids: ids_from_set(duplicate_cell_ids),
            external_cell_ids: ids_from_set(external_cell_ids),
            covered_cell_ids: ids_from_set(covered_cell_ids),
            uncovered_cell_ids: ids_from_set(uncovered_cell_ids),
        })
    }

    /// Returns only the uncovered cells from a requested region over a complex.
    pub fn uncovered_region(
        &self,
        complex_id: &Id,
        covered_cell_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Vec<Id>> {
        self.covered_region(complex_id, covered_cell_ids)
            .map(|coverage| coverage.uncovered_cell_ids)
    }

    fn ensure_space_absent(&self, id: &Id) -> Result<()> {
        ensure_absent(self.spaces.contains_key(id), "space_id", id)
    }

    fn ensure_cell_absent(&self, id: &Id) -> Result<()> {
        ensure_absent(self.cells.contains_key(id), "cell_id", id)
    }

    fn ensure_incidence_absent(&self, id: &Id) -> Result<()> {
        ensure_absent(self.incidences.contains_key(id), "incidence_id", id)
    }

    fn ensure_complex_absent(&self, id: &Id) -> Result<()> {
        ensure_absent(self.complexes.contains_key(id), "complex_id", id)
    }

    fn complex_by_id(&self, complex_id: &Id) -> Result<&Complex> {
        self.complexes
            .get(complex_id)
            .ok_or_else(|| missing("complex_id", complex_id))
    }

    fn closure_for_cells(
        &self,
        field: &str,
        space_id: &Id,
        cell_ids: impl IntoIterator<Item = Id>,
    ) -> Result<BTreeSet<Id>> {
        let mut closure = BTreeSet::new();
        let mut frontier = cell_ids.into_iter().collect::<Vec<_>>();

        while let Some(cell_id) = frontier.pop() {
            if !closure.insert(cell_id.clone()) {
                continue;
            }

            let cell = self.cell_in_space(field, &cell_id, space_id)?;
            frontier.extend(cell.boundary.iter().cloned());
        }

        Ok(closure)
    }

    fn normalize_complex_member_ids(
        &self,
        complex: &Complex,
        field: &str,
        cell_ids: impl IntoIterator<Item = Id>,
    ) -> Result<BTreeSet<Id>> {
        let complex_cell_ids = id_set(&complex.cell_ids);
        let mut normalized = BTreeSet::new();
        for cell_id in cell_ids {
            self.cell_in_space(field, &cell_id, &complex.space_id)?;
            if !complex_cell_ids.contains(&cell_id) {
                return Err(malformed(
                    field,
                    format!(
                        "identifier {cell_id} is not included in complex {}",
                        complex.id
                    ),
                ));
            }
            normalized.insert(cell_id);
        }
        Ok(normalized)
    }

    fn validate_cell_references(&self, cell: &Cell) -> Result<()> {
        self.ensure_space_exists(&cell.space_id)?;
        for boundary_id in &cell.boundary {
            let boundary = self.cell_in_space("boundary", boundary_id, &cell.space_id)?;
            if boundary.dimension >= cell.dimension {
                return Err(malformed(
                    "boundary",
                    "boundary cells must have lower dimension than the owning cell",
                ));
            }
        }
        for coboundary_id in &cell.coboundary {
            let coboundary = self.cell_in_space("coboundary", coboundary_id, &cell.space_id)?;
            if coboundary.dimension <= cell.dimension {
                return Err(malformed(
                    "coboundary",
                    "coboundary cells must have higher dimension than the owning cell",
                ));
            }
        }
        Ok(())
    }

    fn validate_incidence_references(&self, incidence: &Incidence) -> Result<()> {
        self.ensure_space_exists(&incidence.space_id)?;
        self.cell_in_space("from_cell_id", &incidence.from_cell_id, &incidence.space_id)?;
        self.cell_in_space("to_cell_id", &incidence.to_cell_id, &incidence.space_id)?;
        if let Some(weight) = incidence.weight {
            if !weight.is_finite() {
                return Err(malformed("weight", "incidence weight must be finite"));
            }
        }
        Ok(())
    }

    fn validate_complex_references(&self, complex: &Complex) -> Result<Dimension> {
        self.ensure_space_exists(&complex.space_id)?;
        for incidence_id in &complex.incidence_ids {
            let incidence = self.incidence_in_space(incidence_id, &complex.space_id)?;
            if !complex.cell_ids.contains(&incidence.from_cell_id)
                || !complex.cell_ids.contains(&incidence.to_cell_id)
            {
                return Err(malformed(
                    "incidence_ids",
                    format!(
                        "incidence {incidence_id} endpoints must both be included in complex cell_ids"
                    ),
                ));
            }
        }

        let mut max_dimension = 0;
        for cell_id in &complex.cell_ids {
            let cell = self.cell_in_space("cell_ids", cell_id, &complex.space_id)?;
            max_dimension = max_dimension.max(cell.dimension);
        }
        Ok(max_dimension)
    }

    fn ensure_space_exists(&self, space_id: &Id) -> Result<()> {
        if self.spaces.contains_key(space_id) {
            Ok(())
        } else {
            Err(missing("space_id", space_id))
        }
    }

    fn cell_in_space(&self, field: &str, cell_id: &Id, space_id: &Id) -> Result<&Cell> {
        let cell = self
            .cells
            .get(cell_id)
            .ok_or_else(|| missing(field, cell_id))?;
        if &cell.space_id == space_id {
            Ok(cell)
        } else {
            Err(wrong_space(field, cell_id, space_id, &cell.space_id))
        }
    }

    fn incidence_in_space(&self, incidence_id: &Id, space_id: &Id) -> Result<&Incidence> {
        let incidence = self
            .incidences
            .get(incidence_id)
            .ok_or_else(|| missing("incidence_ids", incidence_id))?;
        if &incidence.space_id == space_id {
            Ok(incidence)
        } else {
            Err(wrong_space(
                "incidence_ids",
                incidence_id,
                space_id,
                &incidence.space_id,
            ))
        }
    }

    fn register_cell_in_space(&mut self, cell: &Cell) {
        let space = self
            .spaces
            .get_mut(&cell.space_id)
            .expect("validated cell space should exist");
        push_unique(&mut space.cell_ids, cell.id.clone());
        for context_id in &cell.context_ids {
            push_unique(&mut space.context_ids, context_id.clone());
        }
    }

    fn register_boundary_inverses(&mut self, cell: &Cell) {
        for boundary_id in &cell.boundary {
            let boundary = self
                .cells
                .get_mut(boundary_id)
                .expect("validated boundary cell should exist");
            push_unique(&mut boundary.coboundary, cell.id.clone());
        }
        for coboundary_id in &cell.coboundary {
            let coboundary = self
                .cells
                .get_mut(coboundary_id)
                .expect("validated coboundary cell should exist");
            push_unique(&mut coboundary.boundary, cell.id.clone());
        }
    }
}

#[cfg(test)]
mod pushout_store_adapter_tests {
    use super::*;
    use crate::morphism::{MorphismType, PushoutConstruction, PushoutObstructionType};
    use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};
    use std::collections::BTreeMap;

    #[test]
    fn construct_pushout_matches_direct_explicit_construction() {
        let (store, left, right) = clean_pushout_store();
        let candidate_space_id = id("space/pushout");
        let store_outcome = store
            .construct_pushout(
                &left,
                &right,
                candidate_space_id.clone(),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect("store pushout should construct");

        let left_cells = gathered_cells(&store, &left.target_space_id);
        let right_cells = gathered_cells(&store, &right.target_space_id);
        let left_incidences = gathered_incidences(&store, &left.target_space_id);
        let right_incidences = gathered_incidences(&store, &right.target_space_id);
        let direct_outcome = construct_explicit_pushout(PushoutInputs {
            left: &left,
            right: &right,
            candidate_space_id,
            candidate_space_name: "Pushout".to_owned(),
            complex_type: ComplexType::CellComplex,
            left_cells: &left_cells,
            right_cells: &right_cells,
            left_incidences: &left_incidences,
            right_incidences: &right_incidences,
        });

        let store_construction = constructed(&store_outcome);
        let direct_construction = constructed(&direct_outcome);
        assert_eq!(store_construction.cells, direct_construction.cells);
        assert_eq!(
            store_construction.incidences,
            direct_construction.incidences
        );
        assert_eq!(
            store_construction.complex.max_dimension,
            direct_construction.complex.max_dimension
        );
        assert_eq!(
            store_construction.space.cell_ids,
            direct_construction.space.cell_ids
        );
        assert_eq!(
            store_construction.space.incidence_ids,
            direct_construction.space.incidence_ids
        );
        assert_eq!(
            store_construction.space.complex_ids,
            direct_construction.space.complex_ids
        );
        assert_eq!(
            store_construction.space.context_ids,
            direct_construction.space.context_ids
        );
    }

    #[test]
    fn construct_pushout_errors_when_target_space_is_missing() {
        let mut store = InMemorySpaceStore::new();
        store
            .insert_space(Space::new(id("space/source"), "Source"))
            .expect("insert source space");
        store
            .insert_space(Space::new(id("space/right"), "Right"))
            .expect("insert right space");
        let left = fixture_morphism("left", "space/source", "space/missing-left", [], []);
        let right = fixture_morphism("right", "space/source", "space/right", [], []);

        let error = store
            .construct_pushout(
                &left,
                &right,
                id("space/pushout"),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect_err("missing target space should fail");

        assert!(matches!(
            error,
            CoreError::MalformedField { ref field, ref reason }
                if field == "left" && reason.contains("space/missing-left")
        ));
    }

    #[test]
    fn construct_pushout_passes_blocked_outcome_through() {
        let mut store = InMemorySpaceStore::new();
        for (space_id, name) in [
            ("space/source-left", "Source left"),
            ("space/source-right", "Source right"),
            ("space/left", "Left"),
            ("space/right", "Right"),
        ] {
            store
                .insert_space(Space::new(id(space_id), name))
                .expect("insert space");
        }
        let left = fixture_morphism("left", "space/source-left", "space/left", [], []);
        let right = fixture_morphism("right", "space/source-right", "space/right", [], []);

        let outcome = store
            .construct_pushout(
                &left,
                &right,
                id("space/pushout"),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect("blocked pushout should be returned as outcome");

        let PushoutOutcome::Blocked { report } = outcome else {
            panic!("expected blocked pushout outcome");
        };
        assert!(report.obstructions.iter().any(|obstruction| {
            obstruction.obstruction_type == PushoutObstructionType::IncompatibleSourceSpace
        }));
    }

    #[test]
    fn construct_pushout_is_deterministic_across_insertion_orders() {
        let (ordered, left, right) = deterministic_store(false);
        let (reversed, _, _) = deterministic_store(true);

        let ordered_json = serde_json::to_string(
            &ordered
                .construct_pushout(
                    &left,
                    &right,
                    id("space/pushout"),
                    "Pushout".to_owned(),
                    ComplexType::CellComplex,
                )
                .expect("ordered store pushout"),
        )
        .expect("serialize ordered outcome");
        let reversed_json = serde_json::to_string(
            &reversed
                .construct_pushout(
                    &left,
                    &right,
                    id("space/pushout"),
                    "Pushout".to_owned(),
                    ComplexType::CellComplex,
                )
                .expect("reversed store pushout"),
        )
        .expect("serialize reversed outcome");

        assert_eq!(ordered_json, reversed_json);
    }

    #[test]
    fn construct_pushout_leaves_store_unchanged() {
        let (store, left, right) = clean_pushout_store();
        let counts = (
            store.cells.len(),
            store.incidences.len(),
            store.spaces.len(),
            store.complexes.len(),
        );

        let outcome = store
            .construct_pushout(
                &left,
                &right,
                id("space/pushout"),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect("store pushout should construct");

        assert!(matches!(outcome, PushoutOutcome::Constructed { .. }));
        assert_eq!(
            counts,
            (
                store.cells.len(),
                store.incidences.len(),
                store.spaces.len(),
                store.complexes.len(),
            )
        );
    }

    fn clean_pushout_store() -> (InMemorySpaceStore, Morphism, Morphism) {
        let left = fixture_morphism(
            "left",
            "space/source",
            "space/left",
            [
                ("cell/source-a", "cell/left-a"),
                ("cell/source-b", "cell/left-b"),
            ],
            [("rel/source-a", "rel/left-a")],
        );
        let right = fixture_morphism(
            "right",
            "space/source",
            "space/right",
            [
                ("cell/source-a", "cell/right-a"),
                ("cell/source-b", "cell/right-b"),
            ],
            [("rel/source-a", "rel/right-a")],
        );
        let mut store = base_spaces();
        insert_cell(
            &mut store,
            Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex").with_label("shared-a"),
        );
        insert_cell(
            &mut store,
            Cell::new(id("cell/left-b"), id("space/left"), 0, "vertex").with_context(id("ctx/b")),
        );
        insert_cell(
            &mut store,
            Cell::new(id("cell/left-private"), id("space/left"), 1, "edge")
                .with_boundary_cell(id("cell/left-a")),
        );
        insert_cell(
            &mut store,
            Cell::new(id("cell/right-b"), id("space/right"), 0, "vertex").with_context(id("ctx/b")),
        );
        insert_cell(
            &mut store,
            Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex").with_label("shared-a"),
        );
        insert_incidence(
            &mut store,
            Incidence::new(
                id("rel/left-a"),
                id("space/left"),
                id("cell/left-a"),
                id("cell/left-b"),
                "attaches",
                IncidenceOrientation::Directed,
            ),
        );
        insert_incidence(
            &mut store,
            Incidence::new(
                id("rel/right-a"),
                id("space/right"),
                id("cell/right-a"),
                id("cell/right-b"),
                "attaches",
                IncidenceOrientation::Directed,
            ),
        );

        (store, left, right)
    }

    fn deterministic_store(reverse: bool) -> (InMemorySpaceStore, Morphism, Morphism) {
        let left = fixture_morphism(
            "left",
            "space/source",
            "space/left",
            [("cell/source-a", "cell/left-a")],
            [("rel/source-a", "rel/left-a")],
        );
        let right = fixture_morphism(
            "right",
            "space/source",
            "space/right",
            [("cell/source-a", "cell/right-a")],
            [("rel/source-a", "rel/right-a")],
        );
        let mut store = base_spaces();
        let left_cells = [
            Cell::new(id("cell/left-z"), id("space/left"), 0, "vertex"),
            Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex"),
        ];
        let right_cells = [
            Cell::new(id("cell/right-z"), id("space/right"), 0, "vertex"),
            Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex"),
        ];
        if reverse {
            for cell in left_cells
                .into_iter()
                .rev()
                .chain(right_cells.into_iter().rev())
            {
                insert_cell(&mut store, cell);
            }
        } else {
            for cell in left_cells.into_iter().chain(right_cells) {
                insert_cell(&mut store, cell);
            }
        }

        let left_incidence = Incidence::new(
            id("rel/left-a"),
            id("space/left"),
            id("cell/left-a"),
            id("cell/left-z"),
            "attaches",
            IncidenceOrientation::Directed,
        )
        .with_weight(2.0);
        let right_incidence = Incidence::new(
            id("rel/right-a"),
            id("space/right"),
            id("cell/right-a"),
            id("cell/right-z"),
            "attaches",
            IncidenceOrientation::Directed,
        )
        .with_weight(2.0);
        if reverse {
            insert_incidence(&mut store, right_incidence);
            insert_incidence(&mut store, left_incidence);
        } else {
            insert_incidence(&mut store, left_incidence);
            insert_incidence(&mut store, right_incidence);
        }

        (store, left, right)
    }

    fn base_spaces() -> InMemorySpaceStore {
        let mut store = InMemorySpaceStore::new();
        for (space_id, name) in [
            ("space/source", "Source"),
            ("space/left", "Left"),
            ("space/right", "Right"),
        ] {
            store
                .insert_space(Space::new(id(space_id), name))
                .expect("insert space");
        }
        store
    }

    fn fixture_morphism<const C: usize, const R: usize>(
        morphism_id: &str,
        source_space_id: &str,
        target_space_id: &str,
        cell_pairs: [(&str, &str); C],
        relation_pairs: [(&str, &str); R],
    ) -> Morphism {
        Morphism {
            id: id(morphism_id),
            source_space_id: id(source_space_id),
            target_space_id: id(target_space_id),
            name: morphism_id.to_owned(),
            morphism_type: MorphismType::Translation,
            cell_mapping: mapping(cell_pairs),
            relation_mapping: mapping(relation_pairs),
            preserved_invariant_ids: Vec::new(),
            lost_structure: Vec::new(),
            distortion: Vec::new(),
            composable_with: Vec::new(),
            provenance: provenance(),
        }
    }

    fn mapping<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<Id, Id> {
        pairs
            .into_iter()
            .map(|(source_id, target_id)| (id(source_id), id(target_id)))
            .collect()
    }

    fn gathered_cells(store: &InMemorySpaceStore, space_id: &Id) -> Vec<Cell> {
        let mut cells = store
            .cells
            .values()
            .filter(|cell| &cell.space_id == space_id)
            .cloned()
            .collect::<Vec<_>>();
        cells.sort_by(|left, right| left.id.cmp(&right.id));
        cells
    }

    fn gathered_incidences(store: &InMemorySpaceStore, space_id: &Id) -> Vec<Incidence> {
        let mut incidences = store
            .incidences
            .values()
            .filter(|incidence| &incidence.space_id == space_id)
            .cloned()
            .collect::<Vec<_>>();
        incidences.sort_by(|left, right| left.id.cmp(&right.id));
        incidences
    }

    fn constructed(outcome: &PushoutOutcome) -> &PushoutConstruction {
        let PushoutOutcome::Constructed { construction, .. } = outcome else {
            panic!("expected constructed pushout");
        };
        construction
    }

    fn insert_cell(store: &mut InMemorySpaceStore, cell: Cell) {
        store.insert_cell(cell).expect("insert cell");
    }

    fn insert_incidence(store: &mut InMemorySpaceStore, incidence: Incidence) {
        store.insert_incidence(incidence).expect("insert incidence");
    }

    fn provenance() -> Provenance {
        Provenance::new(
            SourceRef::new(SourceKind::custom("pushout-store-test").expect("valid source kind")),
            Confidence::ONE,
        )
        .with_review_status(ReviewStatus::Accepted)
    }

    fn id(value: impl AsRef<str>) -> Id {
        Id::new(value.as_ref()).expect("valid id")
    }
}
