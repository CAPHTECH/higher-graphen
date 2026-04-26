use super::*;

pub(super) fn build_persistence_intervals(
    store: &InMemorySpaceStore,
    complex: &Complex,
    stages: &[FiltrationStage],
    birth_stage_by_cell_id: &BTreeMap<Id, usize>,
) -> Result<Vec<PersistenceInterval>> {
    let mut entries = birth_stage_by_cell_id
        .iter()
        .map(|(cell_id, stage_index)| {
            let cell = require_cell_in_complex(store, complex, cell_id)?;
            Ok((
                *stage_index,
                dimension_rank(cell.dimension),
                cell.id.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    entries.sort();

    let mut intervals = Vec::new();
    let mut components = PersistentUnionFind::new();

    for (stage_index, _, cell_id) in entries {
        let cell = require_cell_in_complex(store, complex, &cell_id)?;
        match cell.dimension {
            0 => {
                let interval_index = intervals.len();
                intervals.push(PersistenceInterval {
                    dimension: 0,
                    birth_stage_id: stages[stage_index].id.clone(),
                    birth_stage_index: stage_index,
                    death_stage_id: None,
                    death_stage_index: None,
                    generator_cell_ids: vec![cell.id.clone()],
                });
                components.add(cell.id.clone(), interval_index);
            }
            1 => {
                if let Some((source, target)) =
                    graph_edge_endpoints_for_persistence(store, complex, cell)?
                {
                    if !components.contains(&source) || !components.contains(&target) {
                        continue;
                    }
                    if components.find(&source) == components.find(&target) {
                        intervals.push(PersistenceInterval {
                            dimension: 1,
                            birth_stage_id: stages[stage_index].id.clone(),
                            birth_stage_index: stage_index,
                            death_stage_id: None,
                            death_stage_index: None,
                            generator_cell_ids: vec![cell.id.clone()],
                        });
                    } else {
                        components.union_by_birth(
                            &source,
                            &target,
                            stage_index,
                            &stages[stage_index].id,
                            &mut intervals,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    intervals.sort_by(compare_intervals);
    Ok(intervals)
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

fn graph_edge_endpoints_for_persistence(
    store: &InMemorySpaceStore,
    complex: &Complex,
    cell: &Cell,
) -> Result<Option<(Id, Id)>> {
    let complex_cell_ids = id_set(&complex.cell_ids);
    let mut endpoint_ids = BTreeSet::new();
    for boundary_id in &cell.boundary {
        if !complex_cell_ids.contains(boundary_id) {
            return Ok(None);
        }
        let boundary = require_cell_in_complex(store, complex, boundary_id)?;
        if boundary.dimension != 0 {
            return Ok(None);
        }
        endpoint_ids.insert(boundary_id.clone());
    }

    let endpoints = ids_from_set(endpoint_ids);
    if endpoints.len() == 2 {
        Ok(Some((endpoints[0].clone(), endpoints[1].clone())))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Debug, Default)]
struct PersistentUnionFind {
    parent: BTreeMap<Id, Id>,
    root_interval_index: BTreeMap<Id, usize>,
}

impl PersistentUnionFind {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, id: Id, interval_index: usize) {
        self.parent.entry(id.clone()).or_insert_with(|| id.clone());
        self.root_interval_index.insert(id, interval_index);
    }

    fn contains(&self, id: &Id) -> bool {
        self.parent.contains_key(id)
    }

    fn find(&mut self, id: &Id) -> Id {
        let parent = self
            .parent
            .get(id)
            .cloned()
            .expect("persistent union-find contains vertex");
        if &parent == id {
            return parent;
        }
        let root = self.find(&parent);
        self.parent.insert(id.clone(), root.clone());
        root
    }

    fn union_by_birth(
        &mut self,
        left: &Id,
        right: &Id,
        death_stage_index: usize,
        death_stage_id: &Id,
        intervals: &mut [PersistenceInterval],
    ) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return;
        }

        let left_interval_index = *self
            .root_interval_index
            .get(&left_root)
            .expect("root interval exists");
        let right_interval_index = *self
            .root_interval_index
            .get(&right_root)
            .expect("root interval exists");
        let left_birth = intervals[left_interval_index].birth_stage_index;
        let right_birth = intervals[right_interval_index].birth_stage_index;

        let left_survives =
            left_birth < right_birth || (left_birth == right_birth && left_root <= right_root);
        let (survivor_root, loser_root, survivor_interval_index, loser_interval_index) =
            if left_survives {
                (
                    left_root,
                    right_root,
                    left_interval_index,
                    right_interval_index,
                )
            } else {
                (
                    right_root,
                    left_root,
                    right_interval_index,
                    left_interval_index,
                )
            };

        intervals[loser_interval_index].death_stage_id = Some(death_stage_id.clone());
        intervals[loser_interval_index].death_stage_index = Some(death_stage_index);
        self.parent
            .insert(loser_root.clone(), survivor_root.clone());
        self.root_interval_index.remove(&loser_root);
        self.root_interval_index
            .insert(survivor_root, survivor_interval_index);
    }
}
