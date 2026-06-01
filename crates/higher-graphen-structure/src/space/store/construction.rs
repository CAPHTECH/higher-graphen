use super::super::*;
use super::InMemorySpaceStore;
use crate::morphism::{
    construct_explicit_pullback, construct_explicit_pushout, Morphism, PullbackInputs,
    PullbackOutcome, PushoutInputs, PushoutOutcome,
};

impl InMemorySpaceStore {
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
        let (left_cells, left_incidences) = self
            .gather_space_elements(&left.target_space_id)
            .map_err(|_| {
                malformed(
                    "left",
                    format!(
                        "target space identifier {} does not exist in the store",
                        left.target_space_id
                    ),
                )
            })?;
        let (right_cells, right_incidences) = self
            .gather_space_elements(&right.target_space_id)
            .map_err(|_| {
                malformed(
                    "right",
                    format!(
                        "target space identifier {} does not exist in the store",
                        right.target_space_id
                    ),
                )
            })?;

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
        let (left_source_cells, left_source_incidences) = self
            .gather_space_elements(&left.source_space_id)
            .map_err(|_| {
                malformed(
                    "left",
                    format!(
                        "source space identifier {} does not exist in the store",
                        left.source_space_id
                    ),
                )
            })?;
        let (right_source_cells, right_source_incidences) = self
            .gather_space_elements(&right.source_space_id)
            .map_err(|_| {
                malformed(
                    "right",
                    format!(
                        "source space identifier {} does not exist in the store",
                        right.source_space_id
                    ),
                )
            })?;

        Ok(construct_explicit_pullback(PullbackInputs {
            left: left.clone(),
            right: right.clone(),
            candidate_space_id,
            candidate_space_name,
            complex_type,
            left_source_cells,
            right_source_cells,
            left_source_incidences,
            right_source_incidences,
        }))
    }

    fn gather_space_elements(
        &self,
        space_id: &Id,
    ) -> std::result::Result<(Vec<Cell>, Vec<Incidence>), CoreError> {
        if !self.spaces.contains_key(space_id) {
            return Err(malformed(
                "space_id",
                format!("space identifier {space_id} does not exist in the store"),
            ));
        }

        let mut cells = self
            .cells
            .values()
            .filter(|cell| &cell.space_id == space_id)
            .cloned()
            .collect::<Vec<_>>();
        cells.sort_by(|left, right| left.id.cmp(&right.id));

        let mut incidences = self
            .incidences
            .values()
            .filter(|incidence| &incidence.space_id == space_id)
            .cloned()
            .collect::<Vec<_>>();
        incidences.sort_by(|left, right| left.id.cmp(&right.id));

        Ok((cells, incidences))
    }
}
