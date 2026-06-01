use super::helpers::*;
use super::*;

/// Constructs a deterministic finite pullback candidate for a binary cospan.
///
/// The construction pairs left and right source cells whose morphism mappings
/// agree in the shared target. Blocking obstructions return
/// [`PullbackOutcome::Blocked`] and do not materialize cells, incidences, or a
/// complex.
pub fn construct_explicit_pullback(inputs: PullbackInputs) -> PullbackOutcome {
    let mut report = explicit_pullback_candidate(&inputs.left, &inputs.right);

    if inputs.left.target_space_id != inputs.right.target_space_id {
        report.review_status = ReviewStatus::Rejected;
        return PullbackOutcome::Blocked { report };
    }

    let left_cells = indexed_cells(&inputs.left_source_cells);
    let right_cells = indexed_cells(&inputs.right_source_cells);
    let left_incidences = indexed_incidences(&inputs.left_source_incidences);
    let right_incidences = indexed_incidences(&inputs.right_source_incidences);
    let mut information_loss = report.information_loss.clone();

    let cell_id_by_pair = build_pullback_cell_ids(
        &inputs.candidate_space_id,
        &report.cell_matches,
        &mut report.obstructions,
    );
    let cells = build_pullback_cells(
        &inputs,
        &report.cell_matches,
        &left_cells,
        &right_cells,
        &cell_id_by_pair,
        &mut report.obstructions,
        &mut information_loss,
    );
    let incidences = build_pullback_incidences(
        &inputs,
        &report.relation_matches,
        &left_incidences,
        &right_incidences,
        &cell_id_by_pair,
        &mut report.obstructions,
        &mut information_loss,
    );

    finalize_pullback_report(&mut report, information_loss);
    if pullback_has_blocking_obstruction(&report) {
        return PullbackOutcome::Blocked { report };
    }

    construct_pullback_outcome(inputs, cells, incidences, report)
}

fn indexed_cells(cells: &[Cell]) -> BTreeMap<Id, &Cell> {
    cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>()
}

fn indexed_incidences(incidences: &[Incidence]) -> BTreeMap<Id, &Incidence> {
    incidences
        .iter()
        .map(|incidence| (incidence.id.clone(), incidence))
        .collect::<BTreeMap<_, _>>()
}

fn build_pullback_cell_ids(
    candidate_space_id: &Id,
    cell_matches: &[PullbackCellMatch],
    obstructions: &mut Vec<PullbackObstruction>,
) -> BTreeMap<(Id, Id), Id> {
    let mut cell_id_by_pair = BTreeMap::<(Id, Id), Id>::new();
    for matched in cell_matches {
        if let Some(canonical_id) = pullback_cell_id(
            candidate_space_id,
            &matched.left_cell_id,
            &matched.right_cell_id,
        ) {
            cell_id_by_pair.insert(
                (matched.left_cell_id.clone(), matched.right_cell_id.clone()),
                canonical_id,
            );
        } else {
            obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "could not derive a valid canonical cell identifier for pair ({}, {})",
                    matched.left_cell_id, matched.right_cell_id
                ),
            });
        }
    }
    cell_id_by_pair
}

fn build_pullback_cells(
    inputs: &PullbackInputs,
    cell_matches: &[PullbackCellMatch],
    left_cells: &BTreeMap<Id, &Cell>,
    right_cells: &BTreeMap<Id, &Cell>,
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
    obstructions: &mut Vec<PullbackObstruction>,
    information_loss: &mut Vec<String>,
) -> Vec<Cell> {
    let mut cells = cell_matches
        .iter()
        .filter_map(|matched| {
            pullback_cell(
                inputs,
                matched,
                left_cells,
                right_cells,
                cell_id_by_pair,
                obstructions,
                information_loss,
            )
        })
        .collect::<Vec<_>>();
    cells.sort_by(|left, right| left.id.cmp(&right.id));
    cells
}

fn pullback_cell(
    inputs: &PullbackInputs,
    matched: &PullbackCellMatch,
    left_cells: &BTreeMap<Id, &Cell>,
    right_cells: &BTreeMap<Id, &Cell>,
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
    obstructions: &mut Vec<PullbackObstruction>,
    information_loss: &mut Vec<String>,
) -> Option<Cell> {
    let canonical_id = cell_id_by_pair.get(&cell_pair_key(matched))?;
    let (left_cell, right_cell) = matched_cells(matched, left_cells, right_cells, obstructions)?;
    if !cell_pair_is_compatible(matched, left_cell, right_cell, obstructions) {
        return None;
    }
    let label = [left_cell.label.clone(), right_cell.label.clone()]
        .into_iter()
        .flatten()
        .min();
    let boundary = paired_cell_refs(&left_cell.boundary, &right_cell.boundary, cell_id_by_pair);
    let coboundary = paired_cell_refs(
        &left_cell.coboundary,
        &right_cell.coboundary,
        cell_id_by_pair,
    );
    let context_ids = merged_context_ids(left_cell, right_cell);
    push_cell_provenance_loss(matched, left_cell, right_cell, information_loss);

    Some(Cell {
        id: canonical_id.clone(),
        space_id: inputs.candidate_space_id.clone(),
        dimension: left_cell.dimension,
        cell_type: left_cell.cell_type.clone(),
        label,
        boundary,
        coboundary,
        context_ids,
        provenance: None,
    })
}

fn cell_pair_key(matched: &PullbackCellMatch) -> (Id, Id) {
    (matched.left_cell_id.clone(), matched.right_cell_id.clone())
}

fn matched_cells<'a>(
    matched: &PullbackCellMatch,
    left_cells: &'a BTreeMap<Id, &Cell>,
    right_cells: &'a BTreeMap<Id, &Cell>,
    obstructions: &mut Vec<PullbackObstruction>,
) -> Option<(&'a Cell, &'a Cell)> {
    let cells = (
        left_cells.get(&matched.left_cell_id).copied(),
        right_cells.get(&matched.right_cell_id).copied(),
    );
    let (Some(left_cell), Some(right_cell)) = cells else {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::IncompatibleFiber,
            reason: format!(
                "matched cell pair ({}, {}) is not present in the finite source inputs",
                matched.left_cell_id, matched.right_cell_id
            ),
        });
        return None;
    };
    Some((left_cell, right_cell))
}

fn cell_pair_is_compatible(
    matched: &PullbackCellMatch,
    left_cell: &Cell,
    right_cell: &Cell,
    obstructions: &mut Vec<PullbackObstruction>,
) -> bool {
    if left_cell.dimension == right_cell.dimension && left_cell.cell_type == right_cell.cell_type {
        return true;
    }
    obstructions.push(PullbackObstruction {
        obstruction_type: PullbackObstructionType::IncompatibleFiber,
        reason: format!(
            "matched cell pair ({}, {}) has incompatible dimensions {} vs {} or cell types {:?} vs {:?}",
            matched.left_cell_id,
            matched.right_cell_id,
            left_cell.dimension,
            right_cell.dimension,
            left_cell.cell_type,
            right_cell.cell_type
        ),
    });
    false
}

fn paired_cell_refs(
    left_refs: &[Id],
    right_refs: &[Id],
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
) -> Vec<Id> {
    let mut refs = BTreeSet::new();
    for left_ref in left_refs {
        for right_ref in right_refs {
            if let Some(cell_id) = cell_id_by_pair.get(&(left_ref.clone(), right_ref.clone())) {
                refs.insert(cell_id.clone());
            }
        }
    }
    refs.into_iter().collect()
}

fn merged_context_ids(left_cell: &Cell, right_cell: &Cell) -> Vec<Id> {
    left_cell
        .context_ids
        .iter()
        .chain(right_cell.context_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn push_cell_provenance_loss(
    matched: &PullbackCellMatch,
    left_cell: &Cell,
    right_cell: &Cell,
    information_loss: &mut Vec<String>,
) {
    if left_cell.provenance.is_some() || right_cell.provenance.is_some() {
        information_loss.push(format!(
            "cell pair ({}, {}) provenance dropped because the pullback cell has two sources",
            matched.left_cell_id, matched.right_cell_id
        ));
    }
}

fn build_pullback_incidences(
    inputs: &PullbackInputs,
    relation_matches: &[PullbackRelationMatch],
    left_incidences: &BTreeMap<Id, &Incidence>,
    right_incidences: &BTreeMap<Id, &Incidence>,
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
    obstructions: &mut Vec<PullbackObstruction>,
    information_loss: &mut Vec<String>,
) -> Vec<Incidence> {
    let incidence_seeds = relation_matches
        .iter()
        .filter_map(|matched| {
            pullback_incidence_seed(
                inputs,
                matched,
                left_incidences,
                right_incidences,
                cell_id_by_pair,
                obstructions,
                information_loss,
            )
        })
        .collect::<Vec<_>>();
    let mut incidences = deduplicate_incidences(incidence_seeds, information_loss);
    incidences.sort_by(|left, right| left.id.cmp(&right.id));
    incidences
}

fn pullback_incidence_seed(
    inputs: &PullbackInputs,
    matched: &PullbackRelationMatch,
    left_incidences: &BTreeMap<Id, &Incidence>,
    right_incidences: &BTreeMap<Id, &Incidence>,
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
    obstructions: &mut Vec<PullbackObstruction>,
    information_loss: &mut Vec<String>,
) -> Option<IncidenceSeed> {
    let canonical_id = canonical_pullback_incidence_id(inputs, matched, obstructions)?;
    let (left_incidence, right_incidence) =
        matched_incidences(matched, left_incidences, right_incidences, obstructions)?;
    if !incidence_pair_is_compatible(matched, left_incidence, right_incidence, obstructions) {
        return None;
    }
    let (from_cell_id, to_cell_id) = incidence_endpoint_ids(
        matched,
        left_incidence,
        right_incidence,
        cell_id_by_pair,
        information_loss,
    )?;
    push_incidence_provenance_loss(matched, left_incidence, right_incidence, information_loss);

    Some(IncidenceSeed {
        signature: IncidenceSignature {
            from_cell_id: from_cell_id.clone(),
            to_cell_id: to_cell_id.clone(),
            relation_type: left_incidence.relation_type.clone(),
            orientation: left_incidence.orientation,
        },
        incidence: Incidence {
            id: canonical_id,
            space_id: inputs.candidate_space_id.clone(),
            from_cell_id,
            to_cell_id,
            relation_type: left_incidence.relation_type.clone(),
            orientation: left_incidence.orientation,
            weight: left_incidence.weight,
            provenance: None,
        },
    })
}

fn canonical_pullback_incidence_id(
    inputs: &PullbackInputs,
    matched: &PullbackRelationMatch,
    obstructions: &mut Vec<PullbackObstruction>,
) -> Option<Id> {
    let canonical_id = pullback_incidence_id(
        &inputs.candidate_space_id,
        &matched.left_relation_id,
        &matched.right_relation_id,
    );
    if canonical_id.is_none() {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::IncompatibleFiber,
            reason: format!(
                "could not derive a valid canonical incidence identifier for pair ({}, {})",
                matched.left_relation_id, matched.right_relation_id
            ),
        });
    }
    canonical_id
}

fn matched_incidences<'a>(
    matched: &PullbackRelationMatch,
    left_incidences: &'a BTreeMap<Id, &Incidence>,
    right_incidences: &'a BTreeMap<Id, &Incidence>,
    obstructions: &mut Vec<PullbackObstruction>,
) -> Option<(&'a Incidence, &'a Incidence)> {
    let incidences = (
        left_incidences.get(&matched.left_relation_id).copied(),
        right_incidences.get(&matched.right_relation_id).copied(),
    );
    let (Some(left_incidence), Some(right_incidence)) = incidences else {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::IncompatibleFiber,
            reason: format!(
                "matched relation pair ({}, {}) is not present in the finite source inputs",
                matched.left_relation_id, matched.right_relation_id
            ),
        });
        return None;
    };
    Some((left_incidence, right_incidence))
}

fn incidence_pair_is_compatible(
    matched: &PullbackRelationMatch,
    left_incidence: &Incidence,
    right_incidence: &Incidence,
    obstructions: &mut Vec<PullbackObstruction>,
) -> bool {
    if left_incidence.relation_type == right_incidence.relation_type
        && left_incidence.orientation == right_incidence.orientation
    {
        return true;
    }
    obstructions.push(PullbackObstruction {
        obstruction_type: PullbackObstructionType::IncompatibleFiber,
        reason: format!(
            "matched relation pair ({}, {}) has incompatible relation types {:?} vs {:?} or orientations {:?} vs {:?}",
            matched.left_relation_id,
            matched.right_relation_id,
            left_incidence.relation_type,
            right_incidence.relation_type,
            left_incidence.orientation,
            right_incidence.orientation
        ),
    });
    false
}

fn incidence_endpoint_ids(
    matched: &PullbackRelationMatch,
    left_incidence: &Incidence,
    right_incidence: &Incidence,
    cell_id_by_pair: &BTreeMap<(Id, Id), Id>,
    information_loss: &mut Vec<String>,
) -> Option<(Id, Id)> {
    let from_pair = (
        left_incidence.from_cell_id.clone(),
        right_incidence.from_cell_id.clone(),
    );
    let to_pair = (
        left_incidence.to_cell_id.clone(),
        right_incidence.to_cell_id.clone(),
    );
    let endpoints = (
        cell_id_by_pair.get(&from_pair),
        cell_id_by_pair.get(&to_pair),
    );
    let (Some(from_cell_id), Some(to_cell_id)) = endpoints else {
        information_loss.push(format!(
            "relation pair ({}, {}) dropped because one or both endpoint pairs are outside the pullback cells",
            matched.left_relation_id, matched.right_relation_id
        ));
        return None;
    };
    Some((from_cell_id.clone(), to_cell_id.clone()))
}

fn push_incidence_provenance_loss(
    matched: &PullbackRelationMatch,
    left_incidence: &Incidence,
    right_incidence: &Incidence,
    information_loss: &mut Vec<String>,
) {
    if left_incidence.provenance.is_some() || right_incidence.provenance.is_some() {
        information_loss.push(format!(
            "incidence pair ({}, {}) provenance dropped because the pullback incidence has two sources",
            matched.left_relation_id, matched.right_relation_id
        ));
    }
}

fn finalize_pullback_report(report: &mut ExplicitPullbackReport, information_loss: Vec<String>) {
    report.information_loss = information_loss;
    report.review_status = if pullback_has_blocking_obstruction(report) {
        ReviewStatus::Rejected
    } else {
        ReviewStatus::Candidate
    };
}

fn pullback_has_blocking_obstruction(report: &ExplicitPullbackReport) -> bool {
    report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
}

fn construct_pullback_outcome(
    inputs: PullbackInputs,
    cells: Vec<Cell>,
    incidences: Vec<Incidence>,
    report: ExplicitPullbackReport,
) -> PullbackOutcome {
    let (mut space, mut complex) = assemble_candidate(
        &inputs.candidate_space_id,
        "pullback",
        inputs.complex_type,
        &cells,
        &incidences,
    );
    space.name = inputs.candidate_space_name.trim().to_owned();
    complex.name = inputs.candidate_space_name.trim().to_owned();

    PullbackOutcome::Constructed {
        construction: Box::new(PullbackConstruction {
            space,
            complex,
            cells,
            incidences,
            cell_matches: report.cell_matches.clone(),
            relation_matches: report.relation_matches.clone(),
        }),
        report,
    }
}

fn pullback_cell_id(candidate_space_id: &Id, left_cell_id: &Id, right_cell_id: &Id) -> Option<Id> {
    Id::new(format!(
        "{}/pullback/cell/{}+{}",
        candidate_space_id.as_str(),
        encode_pushout_fragment(left_cell_id.as_str()),
        encode_pushout_fragment(right_cell_id.as_str())
    ))
    .ok()
}

fn pullback_incidence_id(
    candidate_space_id: &Id,
    left_relation_id: &Id,
    right_relation_id: &Id,
) -> Option<Id> {
    Id::new(format!(
        "{}/pullback/incidence/{}+{}",
        candidate_space_id.as_str(),
        encode_pushout_fragment(left_relation_id.as_str()),
        encode_pushout_fragment(right_relation_id.as_str())
    ))
    .ok()
}
