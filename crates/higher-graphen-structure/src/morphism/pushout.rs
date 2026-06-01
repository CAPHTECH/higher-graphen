use super::helpers::*;
use super::*;

/// Constructs a deterministic finite pushout candidate for a binary cospan.
///
/// The construction quotients the disjoint union of the left and right target
/// cells and incidences by the identifications induced from shared source
/// mappings. Blocking obstructions return [`PushoutOutcome::Blocked`] and do
/// not materialize cells, incidences, or a complex.
pub fn construct_explicit_pushout(inputs: PushoutInputs<'_>) -> PushoutOutcome {
    let mut report =
        explicit_pushout_candidate(inputs.left, inputs.right, inputs.candidate_space_id.clone());
    let mut quotient_losses = Vec::new();
    let left_cells = indexed_cells(inputs.left_cells);
    let right_cells = indexed_cells(inputs.right_cells);

    let Some((cell_classes, cell_id_by_key)) = build_cell_classes(&inputs, &mut report) else {
        return blocked_pushout(report, quotient_losses);
    };
    report.obstructions.extend(ambiguous_class_obstructions(
        "cell",
        &cell_classes,
        &mut quotient_losses,
    ));
    let cells = build_merged_cells(
        &inputs,
        &left_cells,
        &right_cells,
        &cell_classes,
        &cell_id_by_key,
        &mut report,
        &mut quotient_losses,
    );

    let left_incidences = indexed_incidences(inputs.left_incidences);
    let right_incidences = indexed_incidences(inputs.right_incidences);
    let Some(relation_classes) = build_relation_classes(&inputs, &mut report) else {
        return blocked_pushout(report, quotient_losses);
    };
    report.obstructions.extend(ambiguous_class_obstructions(
        "relation",
        &relation_classes,
        &mut quotient_losses,
    ));
    let incidences = build_merged_incidences(
        &inputs,
        &left_incidences,
        &right_incidences,
        &relation_classes,
        &cell_id_by_key,
        &mut report,
        &mut quotient_losses,
    );

    finalize_pushout_report(&mut report, quotient_losses);
    if pushout_has_blocking_obstruction(&report) {
        return PushoutOutcome::Blocked { report };
    }

    construct_pushout_outcome(inputs, cells, incidences, report)
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

fn build_cell_classes(
    inputs: &PushoutInputs<'_>,
    report: &mut ExplicitPushoutReport,
) -> Option<(
    Vec<PushoutEquivalenceClass>,
    BTreeMap<PushoutElementKey, Id>,
)> {
    let mut cell_union = PushoutUnionFind::default();
    for cell in inputs.left_cells {
        cell_union.insert(PushoutElementKey::left(cell.id.clone()));
    }
    for cell in inputs.right_cells {
        cell_union.insert(PushoutElementKey::right(cell.id.clone()));
    }
    for group in &report.identified_cell_groups {
        cell_union.union(
            PushoutElementKey::left(group.left_target_id.clone()),
            PushoutElementKey::right(group.right_target_id.clone()),
        );
    }

    let Some(cell_classes) = cell_union.classes(&inputs.candidate_space_id, "cell") else {
        report.obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: "could not derive a valid canonical cell identifier".to_owned(),
        });
        return None;
    };
    let cell_id_by_key = class_id_map(&cell_classes);
    Some((cell_classes, cell_id_by_key))
}

fn build_relation_classes(
    inputs: &PushoutInputs<'_>,
    report: &mut ExplicitPushoutReport,
) -> Option<Vec<PushoutEquivalenceClass>> {
    let mut relation_union = PushoutUnionFind::default();
    for incidence in inputs.left_incidences {
        relation_union.insert(PushoutElementKey::left(incidence.id.clone()));
    }
    for incidence in inputs.right_incidences {
        relation_union.insert(PushoutElementKey::right(incidence.id.clone()));
    }
    for group in &report.identified_relation_groups {
        relation_union.union(
            PushoutElementKey::left(group.left_target_id.clone()),
            PushoutElementKey::right(group.right_target_id.clone()),
        );
    }

    let classes = relation_union.classes(&inputs.candidate_space_id, "incidence");
    if classes.is_none() {
        report.obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: "could not derive a valid canonical incidence identifier".to_owned(),
        });
    }
    classes
}

fn class_id_map(classes: &[PushoutEquivalenceClass]) -> BTreeMap<PushoutElementKey, Id> {
    let mut id_by_key = BTreeMap::new();
    for class in classes {
        for member in &class.members {
            id_by_key.insert(member.clone(), class.canonical_id.clone());
        }
    }
    id_by_key
}

fn build_merged_cells(
    inputs: &PushoutInputs<'_>,
    left_cells: &BTreeMap<Id, &Cell>,
    right_cells: &BTreeMap<Id, &Cell>,
    cell_classes: &[PushoutEquivalenceClass],
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    report: &mut ExplicitPushoutReport,
    quotient_losses: &mut Vec<String>,
) -> Vec<Cell> {
    let mut cells = cell_classes
        .iter()
        .filter_map(|class| {
            merged_cell(
                class,
                &inputs.candidate_space_id,
                left_cells,
                right_cells,
                cell_id_by_key,
                &mut report.obstructions,
                quotient_losses,
            )
        })
        .collect::<Vec<_>>();
    cells.sort_by(|left, right| left.id.cmp(&right.id));
    cells
}

fn build_merged_incidences(
    inputs: &PushoutInputs<'_>,
    left_incidences: &BTreeMap<Id, &Incidence>,
    right_incidences: &BTreeMap<Id, &Incidence>,
    relation_classes: &[PushoutEquivalenceClass],
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    report: &mut ExplicitPushoutReport,
    quotient_losses: &mut Vec<String>,
) -> Vec<Incidence> {
    let incidence_seeds = relation_classes
        .iter()
        .filter_map(|class| {
            merged_incidence_seed(
                class,
                &inputs.candidate_space_id,
                left_incidences,
                right_incidences,
                cell_id_by_key,
                &mut report.obstructions,
                quotient_losses,
            )
        })
        .collect::<Vec<_>>();
    let mut incidences = deduplicate_incidences(incidence_seeds, quotient_losses);
    incidences.sort_by(|left, right| left.id.cmp(&right.id));
    incidences
}

fn blocked_pushout(
    mut report: ExplicitPushoutReport,
    quotient_losses: Vec<String>,
) -> PushoutOutcome {
    finalize_pushout_report(&mut report, quotient_losses);
    PushoutOutcome::Blocked { report }
}

fn finalize_pushout_report(report: &mut ExplicitPushoutReport, quotient_losses: Vec<String>) {
    report.quotient_losses = quotient_losses;
    report.review_status = pushout_review_status(&report.obstructions);
}

fn pushout_has_blocking_obstruction(report: &ExplicitPushoutReport) -> bool {
    report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
}

fn construct_pushout_outcome(
    inputs: PushoutInputs<'_>,
    cells: Vec<Cell>,
    incidences: Vec<Incidence>,
    report: ExplicitPushoutReport,
) -> PushoutOutcome {
    let review_status = report.review_status;
    let (mut space, mut complex) = assemble_candidate(
        &inputs.candidate_space_id,
        "pushout",
        inputs.complex_type,
        &cells,
        &incidences,
    );
    space.name = inputs.candidate_space_name.trim().to_owned();
    complex.name = inputs.candidate_space_name.trim().to_owned();

    PushoutOutcome::Constructed {
        construction: Box::new(PushoutConstruction {
            space,
            complex,
            cells,
            incidences,
            review_status,
        }),
        report,
    }
}

fn pushout_review_status(obstructions: &[PushoutObstruction]) -> ReviewStatus {
    if obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
    {
        ReviewStatus::Rejected
    } else if obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == PushoutObstructionType::AmbiguousIdentification
    }) {
        ReviewStatus::Candidate
    } else {
        ReviewStatus::Unreviewed
    }
}

fn class_member_cell<'a>(
    key: &PushoutElementKey,
    left_cells: &'a BTreeMap<Id, &'a Cell>,
    right_cells: &'a BTreeMap<Id, &'a Cell>,
) -> Option<&'a Cell> {
    match key.side {
        PushoutSide::Left => left_cells.get(&key.id).copied(),
        PushoutSide::Right => right_cells.get(&key.id).copied(),
    }
}

fn class_member_incidence<'a>(
    key: &PushoutElementKey,
    left_incidences: &'a BTreeMap<Id, &'a Incidence>,
    right_incidences: &'a BTreeMap<Id, &'a Incidence>,
) -> Option<&'a Incidence> {
    match key.side {
        PushoutSide::Left => left_incidences.get(&key.id).copied(),
        PushoutSide::Right => right_incidences.get(&key.id).copied(),
    }
}

fn ambiguous_class_obstructions(
    element_kind: &str,
    classes: &[PushoutEquivalenceClass],
    quotient_losses: &mut Vec<String>,
) -> Vec<PushoutObstruction> {
    let mut obstructions = Vec::new();
    for class in classes {
        let left_ids = class
            .members
            .iter()
            .filter(|member| member.side == PushoutSide::Left)
            .map(|member| member.id.clone())
            .collect::<Vec<_>>();
        let right_ids = class
            .members
            .iter()
            .filter(|member| member.side == PushoutSide::Right)
            .map(|member| member.id.clone())
            .collect::<Vec<_>>();

        if left_ids.len() > 1 || right_ids.len() > 1 {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::AmbiguousIdentification,
                reason: format!(
                    "{element_kind} class {} collapses multiple same-side elements",
                    class.canonical_id
                ),
            });
            quotient_losses.push(format!(
                "ambiguous_identification: {element_kind} class {} collapses left {:?} and right {:?}",
                class.canonical_id, left_ids, right_ids
            ));
        }
    }
    obstructions
}

fn remap_cell_reference(
    side: &PushoutSide,
    cell_id: &Id,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
) -> Option<Id> {
    let key = match side {
        PushoutSide::Left => PushoutElementKey::left(cell_id.clone()),
        PushoutSide::Right => PushoutElementKey::right(cell_id.clone()),
    };
    cell_id_by_key.get(&key).cloned()
}

fn remap_cell_refs(
    owner: &PushoutElementKey,
    attribute: &str,
    refs: &[Id],
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    quotient_losses: &mut Vec<String>,
) -> BTreeSet<Id> {
    let mut remapped = BTreeSet::new();
    for cell_id in refs {
        if let Some(mapped_id) = remap_cell_reference(&owner.side, cell_id, cell_id_by_key) {
            remapped.insert(mapped_id);
        } else {
            quotient_losses.push(format!(
                "cell {}:{} {attribute} reference {} dropped because it is outside the pushout input",
                owner.side.as_str(),
                owner.id,
                cell_id
            ));
        }
    }
    remapped
}

type CellMember<'a> = (&'a PushoutElementKey, &'a Cell);

fn merged_cell(
    class: &PushoutEquivalenceClass,
    candidate_space_id: &Id,
    left_cells: &BTreeMap<Id, &Cell>,
    right_cells: &BTreeMap<Id, &Cell>,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    obstructions: &mut Vec<PushoutObstruction>,
    quotient_losses: &mut Vec<String>,
) -> Option<Cell> {
    let members = collect_cell_members(class, left_cells, right_cells, obstructions);
    let (dimension, cell_type) = cell_member_shape(class, &members, obstructions)?;
    let canonical_label = canonical_cell_label(&members);
    push_cell_label_losses(class, &members, &canonical_label, quotient_losses);
    let (boundary, coboundary, context_ids) =
        merged_cell_refs(&members, cell_id_by_key, quotient_losses);
    let provenance = merged_cell_provenance(class, &members, quotient_losses);

    Some(Cell {
        id: class.canonical_id.clone(),
        space_id: candidate_space_id.clone(),
        dimension,
        cell_type,
        label: canonical_label,
        boundary: boundary.into_iter().collect(),
        coboundary: coboundary.into_iter().collect(),
        context_ids: context_ids.into_iter().collect(),
        provenance,
    })
}

fn collect_cell_members<'a>(
    class: &'a PushoutEquivalenceClass,
    left_cells: &'a BTreeMap<Id, &Cell>,
    right_cells: &'a BTreeMap<Id, &Cell>,
    obstructions: &mut Vec<PushoutObstruction>,
) -> Vec<CellMember<'a>> {
    let mut members = Vec::new();
    for key in &class.members {
        if let Some(cell) = class_member_cell(key, left_cells, right_cells) {
            members.push((key, cell));
        } else {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::IncompatibleIdentification,
                reason: format!(
                    "identified cell {}:{} is not present in the finite input",
                    key.side.as_str(),
                    key.id
                ),
            });
        }
    }
    members
}

fn cell_member_shape(
    class: &PushoutEquivalenceClass,
    members: &[CellMember<'_>],
    obstructions: &mut Vec<PushoutObstruction>,
) -> Option<(Dimension, String)> {
    let dimensions = members
        .iter()
        .map(|(_, cell)| cell.dimension)
        .collect::<BTreeSet<_>>();
    let cell_types = members
        .iter()
        .map(|(_, cell)| cell.cell_type.clone())
        .collect::<BTreeSet<_>>();

    if dimensions.len() > 1 || cell_types.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: format!(
                "cell class {} has incompatible dimensions {:?} or cell types {:?}",
                class.canonical_id, dimensions, cell_types
            ),
        });
        return None;
    }

    let dimension = dimensions.iter().next().copied()?;
    let cell_type = cell_types.iter().next().cloned()?;
    Some((dimension, cell_type))
}

fn canonical_cell_label(members: &[CellMember<'_>]) -> Option<String> {
    members
        .iter()
        .filter_map(|(_, cell)| cell.label.clone())
        .min()
}

fn push_cell_label_losses(
    class: &PushoutEquivalenceClass,
    members: &[CellMember<'_>],
    canonical_label: &Option<String>,
    quotient_losses: &mut Vec<String>,
) {
    let distinct_labels = members
        .iter()
        .map(|(_, cell)| cell.label.clone())
        .collect::<BTreeSet<_>>();
    if distinct_labels.len() > 1 {
        for (key, cell) in members {
            if &cell.label != canonical_label {
                quotient_losses.push(format!(
                    "cell {}:{} label {:?} dropped; class {} keeps {:?}",
                    key.side.as_str(),
                    key.id,
                    cell.label,
                    class.canonical_id,
                    canonical_label
                ));
            }
        }
    }
}

fn merged_cell_refs(
    members: &[CellMember<'_>],
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    quotient_losses: &mut Vec<String>,
) -> (BTreeSet<Id>, BTreeSet<Id>, BTreeSet<Id>) {
    let mut boundary = BTreeSet::new();
    let mut coboundary = BTreeSet::new();
    let mut context_ids = BTreeSet::new();
    for (key, cell) in members {
        boundary.extend(remap_cell_refs(
            key,
            "boundary",
            &cell.boundary,
            cell_id_by_key,
            quotient_losses,
        ));
        coboundary.extend(remap_cell_refs(
            key,
            "coboundary",
            &cell.coboundary,
            cell_id_by_key,
            quotient_losses,
        ));
        context_ids.extend(cell.context_ids.iter().cloned());
    }
    (boundary, coboundary, context_ids)
}

fn merged_cell_provenance(
    class: &PushoutEquivalenceClass,
    members: &[CellMember<'_>],
    quotient_losses: &mut Vec<String>,
) -> Option<Provenance> {
    if members.len() == 1 {
        members
            .first()
            .and_then(|(_, cell)| cell.provenance.clone())
    } else {
        for (key, cell) in members {
            if cell.provenance.is_some() {
                quotient_losses.push(format!(
                    "cell {}:{} provenance dropped; merged class {} has multiple sources",
                    key.side.as_str(),
                    key.id,
                    class.canonical_id
                ));
            }
        }
        None
    }
}

fn merged_incidence_seed(
    class: &PushoutEquivalenceClass,
    candidate_space_id: &Id,
    left_incidences: &BTreeMap<Id, &Incidence>,
    right_incidences: &BTreeMap<Id, &Incidence>,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    obstructions: &mut Vec<PushoutObstruction>,
    quotient_losses: &mut Vec<String>,
) -> Option<IncidenceSeed> {
    let members = collect_incidence_members(
        class,
        left_incidences,
        right_incidences,
        cell_id_by_key,
        obstructions,
    );
    let (relation_type, orientation) = relation_member_shape(class, &members, obstructions)?;
    let (from_cell_id, to_cell_id) = relation_endpoints(class, &members, obstructions)?;
    let (_, first, _, _) = members.first()?;
    let weight = first.weight;
    let provenance = merged_incidence_provenance(class, &members, weight, quotient_losses);
    let signature = IncidenceSignature {
        from_cell_id,
        to_cell_id,
        relation_type,
        orientation,
    };

    Some(IncidenceSeed {
        incidence: Incidence {
            id: class.canonical_id.clone(),
            space_id: candidate_space_id.clone(),
            from_cell_id: signature.from_cell_id.clone(),
            to_cell_id: signature.to_cell_id.clone(),
            relation_type: signature.relation_type.clone(),
            orientation: signature.orientation,
            weight,
            provenance,
        },
        signature,
    })
}

type IncidenceMember<'a> = (&'a PushoutElementKey, &'a Incidence, Id, Id);

fn collect_incidence_members<'a>(
    class: &'a PushoutEquivalenceClass,
    left_incidences: &'a BTreeMap<Id, &Incidence>,
    right_incidences: &'a BTreeMap<Id, &Incidence>,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    obstructions: &mut Vec<PushoutObstruction>,
) -> Vec<IncidenceMember<'a>> {
    let mut members = Vec::new();
    for key in &class.members {
        if let Some(incidence) = class_member_incidence(key, left_incidences, right_incidences) {
            let from_cell_id =
                remap_cell_reference(&key.side, &incidence.from_cell_id, cell_id_by_key);
            let to_cell_id = remap_cell_reference(&key.side, &incidence.to_cell_id, cell_id_by_key);
            if from_cell_id.is_none() || to_cell_id.is_none() {
                obstructions.push(PushoutObstruction {
                    obstruction_type: PushoutObstructionType::RelationEndpointConflict,
                    reason: format!(
                        "incidence {}:{} references an endpoint outside the pushout cells",
                        key.side.as_str(),
                        key.id
                    ),
                });
            }
            if let (Some(from_cell_id), Some(to_cell_id)) = (from_cell_id, to_cell_id) {
                members.push((key, incidence, from_cell_id, to_cell_id));
            }
        } else {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::IncompatibleIdentification,
                reason: format!(
                    "identified relation {}:{} is not present in the finite input",
                    key.side.as_str(),
                    key.id
                ),
            });
        }
    }
    members
}

fn relation_member_shape(
    class: &PushoutEquivalenceClass,
    members: &[IncidenceMember<'_>],
    obstructions: &mut Vec<PushoutObstruction>,
) -> Option<(String, IncidenceOrientation)> {
    let relation_types = members
        .iter()
        .map(|(_, incidence, _, _)| incidence.relation_type.clone())
        .collect::<BTreeSet<_>>();
    let orientations = members
        .iter()
        .map(|(_, incidence, _, _)| incidence.orientation)
        .collect::<BTreeSet<_>>();
    if relation_types.len() > 1 || orientations.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: format!(
                "relation class {} has incompatible relation types {:?} or orientations {:?}",
                class.canonical_id, relation_types, orientations
            ),
        });
        return None;
    }
    let relation_type = relation_types.iter().next().cloned()?;
    let orientation = orientations.iter().next().copied()?;
    Some((relation_type, orientation))
}

fn relation_endpoints(
    class: &PushoutEquivalenceClass,
    members: &[IncidenceMember<'_>],
    obstructions: &mut Vec<PushoutObstruction>,
) -> Option<(Id, Id)> {
    let endpoints = members
        .iter()
        .map(|(_, _, from_cell_id, to_cell_id)| (from_cell_id.clone(), to_cell_id.clone()))
        .collect::<BTreeSet<_>>();
    if endpoints.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::RelationEndpointConflict,
            reason: format!(
                "relation class {} has conflicting remapped endpoints {:?}",
                class.canonical_id, endpoints
            ),
        });
        return None;
    }
    endpoints.into_iter().next()
}

fn merged_incidence_provenance(
    class: &PushoutEquivalenceClass,
    members: &[IncidenceMember<'_>],
    weight: Option<f64>,
    quotient_losses: &mut Vec<String>,
) -> Option<Provenance> {
    if members.len() == 1 {
        members
            .first()
            .and_then(|(_, incidence, _, _)| incidence.provenance.clone())
    } else {
        for (key, incidence, _, _) in members {
            if incidence.weight != weight {
                quotient_losses.push(format!(
                    "incidence {}:{} weight {:?} dropped; class {} keeps {:?}",
                    key.side.as_str(),
                    key.id,
                    incidence.weight,
                    class.canonical_id,
                    weight
                ));
            }
            if incidence.provenance.is_some() {
                quotient_losses.push(format!(
                    "incidence {}:{} provenance dropped; merged class {} has multiple sources",
                    key.side.as_str(),
                    key.id,
                    class.canonical_id
                ));
            }
        }
        None
    }
}
