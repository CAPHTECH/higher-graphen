use super::*;

pub(super) fn build_persistence_intervals(
    store: &InMemorySpaceStore,
    complex: &Complex,
    stages: &[FiltrationStage],
    birth_stage_by_cell_id: &BTreeMap<Id, usize>,
) -> Result<Vec<PersistenceInterval>> {
    let cells = filtration_cells(store, complex, birth_stage_by_cell_id)?;
    let index_by_cell_id = cells
        .iter()
        .enumerate()
        .map(|(index, cell)| (cell.id.clone(), index))
        .collect::<BTreeMap<_, _>>();

    let mut reduced_columns = Vec::with_capacity(cells.len());
    let mut pivot_column_by_low = BTreeMap::<usize, usize>::new();
    let mut paired_birth_indices = BTreeSet::new();
    let mut intervals = Vec::new();

    for (column_index, cell) in cells.iter().enumerate() {
        let mut column = persistence_boundary_indices(store, complex, cell, &index_by_cell_id)?;
        column.retain(|row_index| *row_index < column_index);

        while let Some(low) = column.iter().next_back().copied() {
            if let Some(pivot_column_index) = pivot_column_by_low.get(&low).copied() {
                xor_indices(&mut column, &reduced_columns[pivot_column_index]);
            } else {
                pivot_column_by_low.insert(low, column_index);
                paired_birth_indices.insert(low);
                let birth = &cells[low];
                intervals.push(PersistenceInterval {
                    dimension: birth.dimension,
                    birth_stage_id: stages[birth.birth_stage_index].id.clone(),
                    birth_stage_index: birth.birth_stage_index,
                    death_stage_id: Some(stages[cell.birth_stage_index].id.clone()),
                    death_stage_index: Some(cell.birth_stage_index),
                    generator_cell_ids: vec![birth.id.clone()],
                });
                break;
            }
        }

        reduced_columns.push(column);
    }

    for (index, column) in reduced_columns.iter().enumerate() {
        if column.is_empty() && !paired_birth_indices.contains(&index) {
            let birth = &cells[index];
            intervals.push(PersistenceInterval {
                dimension: birth.dimension,
                birth_stage_id: stages[birth.birth_stage_index].id.clone(),
                birth_stage_index: birth.birth_stage_index,
                death_stage_id: None,
                death_stage_index: None,
                generator_cell_ids: vec![birth.id.clone()],
            });
        }
    }

    intervals.sort_by(compare_intervals);
    Ok(intervals)
}

fn filtration_cells(
    store: &InMemorySpaceStore,
    complex: &Complex,
    birth_stage_by_cell_id: &BTreeMap<Id, usize>,
) -> Result<Vec<FiltrationCell>> {
    let mut cells = birth_stage_by_cell_id
        .iter()
        .map(|(cell_id, birth_stage_index)| {
            let cell = require_cell_in_complex(store, complex, cell_id)?;
            Ok(FiltrationCell {
                id: cell.id.clone(),
                dimension: cell.dimension,
                birth_stage_index: *birth_stage_index,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    cells.sort();
    Ok(cells)
}

fn persistence_boundary_indices(
    store: &InMemorySpaceStore,
    complex: &Complex,
    cell: &FiltrationCell,
    index_by_cell_id: &BTreeMap<Id, usize>,
) -> Result<BTreeSet<usize>> {
    let stored_cell = require_cell_in_complex(store, complex, &cell.id)?;
    let mut boundary_indices = BTreeSet::new();

    for boundary_id in &stored_cell.boundary {
        let boundary = require_cell_in_space(store, complex, boundary_id)?;
        if boundary.dimension.checked_add(1) != Some(stored_cell.dimension) {
            continue;
        }
        if let Some(index) = index_by_cell_id.get(boundary_id) {
            boundary_indices.insert(*index);
        }
    }

    Ok(boundary_indices)
}

fn compare_intervals(
    left: &PersistenceInterval,
    right: &PersistenceInterval,
) -> std::cmp::Ordering {
    left.dimension
        .cmp(&right.dimension)
        .then_with(|| left.birth_stage_index.cmp(&right.birth_stage_index))
        .then_with(|| left.death_stage_index.cmp(&right.death_stage_index))
        .then_with(|| left.generator_cell_ids.cmp(&right.generator_cell_ids))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FiltrationCell {
    birth_stage_index: usize,
    dimension: Dimension,
    id: Id,
}
