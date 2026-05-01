use super::util::dedupe_ids;
use crate::native_model::{CaseRelationType, CaseSpace, RelationStrength};
use higher_graphen_core::Id;
use higher_graphen_structure::space::{
    Cell, CellPattern, InMemorySpaceStore, Incidence, IncidenceOrientation, PathPattern,
    PathPatternSegment, Space,
};
use std::collections::BTreeSet;

pub(super) struct NativeCaseTraversal {
    store: InMemorySpaceStore,
    space_id: Id,
}

impl NativeCaseTraversal {
    pub(super) fn from_case_space(case_space: &CaseSpace) -> Self {
        let mut store = InMemorySpaceStore::new();
        store
            .insert_space(space_for(case_space))
            .expect("validated native case space should build traversal space");
        for cell in &case_space.case_cells {
            store
                .insert_cell(
                    Cell::new(
                        cell.id.clone(),
                        case_space.space_id.clone(),
                        0,
                        cell.cell_type.serialized_value(),
                    )
                    .with_label(cell.title.clone())
                    .with_provenance(cell.provenance.clone()),
                )
                .expect("validated native case cell should build traversal cell");
        }
        for relation in hard_relations(case_space) {
            store
                .insert_incidence(
                    Incidence::new(
                        relation.id.clone(),
                        case_space.space_id.clone(),
                        relation.from_id.clone(),
                        relation.to_id.clone(),
                        relation.relation_type.serialized_value(),
                        IncidenceOrientation::Directed,
                    )
                    .with_provenance(relation.provenance.clone()),
                )
                .expect("validated native relation should build traversal incidence");
        }
        Self {
            store,
            space_id: case_space.space_id.clone(),
        }
    }

    pub(super) fn direct_targets(&self, cell_id: &Id, relation_type: CaseRelationType) -> Vec<Id> {
        let pattern = PathPattern::new(self.space_id.clone(), CellPattern::by_id(cell_id.clone()))
            .then(segment(relation_type, CellPattern::any()));
        self.target_ids(pattern)
    }

    pub(super) fn completed_targets(&self) -> BTreeSet<Id> {
        self.targets_for_relation(CaseRelationType::Completes)
            .into_iter()
            .chain(self.targets_for_relation(CaseRelationType::Supersedes))
            .collect()
    }

    fn targets_for_relation(&self, relation_type: CaseRelationType) -> Vec<Id> {
        let pattern = PathPattern::new(self.space_id.clone(), CellPattern::any())
            .then(segment(relation_type, CellPattern::any()));
        self.target_ids(pattern)
    }

    fn target_ids(&self, pattern: PathPattern) -> Vec<Id> {
        let targets = self
            .store
            .matches_path_pattern(&pattern)
            .expect("validated native traversal pattern should run")
            .into_iter()
            .filter_map(|record| record.matched_cell_ids.last().cloned())
            .collect();
        dedupe_ids(targets)
    }
}

fn space_for(case_space: &CaseSpace) -> Space {
    Space::new(
        case_space.space_id.clone(),
        format!("CaseGraphen {}", case_space.case_space_id),
    )
    .with_description("Native CaseGraphen traversal view")
}

fn hard_relations(
    case_space: &CaseSpace,
) -> impl Iterator<Item = &crate::native_model::CaseRelation> {
    case_space
        .case_relations
        .iter()
        .filter(|relation| relation.relation_strength == RelationStrength::Hard)
}

fn segment(relation_type: CaseRelationType, target: CellPattern) -> PathPatternSegment {
    PathPatternSegment::new(target).with_relation_type(relation_type.serialized_value())
}
