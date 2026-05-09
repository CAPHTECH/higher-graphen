use super::*;
use higher_graphen_structure::space::{Incidence, IncidenceOrientation, PathStep};

pub(super) fn append_path(path: &GraphPath, step: PathStep) -> GraphPath {
    let mut next_path = path.clone();
    next_path.end_cell_id = step.to_cell_id.clone();
    next_path.steps.push(step);
    next_path
}

pub(super) fn next_cell_id(
    current_cell_id: &Id,
    incidence: &Incidence,
    direction: TraversalDirection,
) -> Option<Id> {
    match incidence.orientation {
        IncidenceOrientation::Directed => {
            directed_next_cell_id(current_cell_id, incidence, direction)
        }
        IncidenceOrientation::Undirected => undirected_next_cell_id(current_cell_id, incidence),
    }
}

pub(super) fn directed_next_cell_id(
    current_cell_id: &Id,
    incidence: &Incidence,
    direction: TraversalDirection,
) -> Option<Id> {
    match direction {
        TraversalDirection::Outgoing if &incidence.from_cell_id == current_cell_id => {
            Some(incidence.to_cell_id.clone())
        }
        TraversalDirection::Incoming if &incidence.to_cell_id == current_cell_id => {
            Some(incidence.from_cell_id.clone())
        }
        TraversalDirection::Both => undirected_next_cell_id(current_cell_id, incidence),
        _ => None,
    }
}

pub(super) fn undirected_next_cell_id(current_cell_id: &Id, incidence: &Incidence) -> Option<Id> {
    if &incidence.from_cell_id == current_cell_id {
        Some(incidence.to_cell_id.clone())
    } else if &incidence.to_cell_id == current_cell_id {
        Some(incidence.from_cell_id.clone())
    } else {
        None
    }
}

pub(super) fn require_space(store: &InMemorySpaceStore, space_id: &Id) -> Result<()> {
    if store.space(space_id).is_some() {
        Ok(())
    } else {
        Err(malformed(
            "space_id",
            format!("identifier {space_id} does not exist in the store"),
        ))
    }
}

pub(super) fn require_cell_in_space(
    store: &InMemorySpaceStore,
    field: &str,
    cell_id: &Id,
    space_id: &Id,
) -> Result<()> {
    let cell = store
        .cell(cell_id)
        .ok_or_else(|| malformed(field, format!("identifier {cell_id} does not exist")))?;
    if &cell.space_id == space_id {
        Ok(())
    } else {
        Err(malformed(
            field,
            format!("identifier {cell_id} belongs to space {}", cell.space_id),
        ))
    }
}

pub(super) fn normalized_ids(field: &str, values: &[Id]) -> Result<Vec<Id>> {
    if values.is_empty() {
        return Err(malformed(
            field,
            "value must include at least one cell identifier",
        ));
    }
    Ok(values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}

pub(super) fn normalize_relation_types(values: &[String]) -> Result<BTreeSet<String>> {
    values
        .iter()
        .map(|value| {
            let normalized = value.trim().to_owned();
            if normalized.is_empty() {
                Err(malformed(
                    "relation_types",
                    "value must not be empty after trimming",
                ))
            } else {
                Ok(normalized)
            }
        })
        .collect()
}
