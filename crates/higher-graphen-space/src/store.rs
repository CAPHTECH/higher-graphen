use super::*;
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
