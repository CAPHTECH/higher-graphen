use super::*;

pub(super) struct ComposedMorphismSpec {
    pub(super) composed_id: Id,
    pub(super) name: String,
    pub(super) morphism_type: MorphismType,
    pub(super) provenance: Provenance,
    pub(super) cell_mapping: CellMapping,
    pub(super) relation_mapping: RelationMapping,
}

pub(super) fn composed_morphism(
    first: &Morphism,
    second: &Morphism,
    spec: ComposedMorphismSpec,
) -> Morphism {
    Morphism {
        id: spec.composed_id,
        source_space_id: first.source_space_id.clone(),
        target_space_id: second.target_space_id.clone(),
        name: spec.name,
        morphism_type: spec.morphism_type,
        cell_mapping: spec.cell_mapping,
        relation_mapping: spec.relation_mapping,
        preserved_invariant_ids: intersect_ids(
            &first.preserved_invariant_ids,
            &second.preserved_invariant_ids,
        ),
        lost_structure: concat_records(&first.lost_structure, &second.lost_structure),
        distortion: concat_records(&first.distortion, &second.distortion),
        composable_with: Vec::new(),
        provenance: spec.provenance,
    }
}

pub(super) fn coverage_from_mapping_compositions(
    cell_composition: &MappingComposition,
    relation_composition: &MappingComposition,
) -> CompositionCoverage {
    CompositionCoverage {
        unmapped_cell_intermediate_ids: unmapped_intermediate_ids(cell_composition),
        unmapped_relation_intermediate_ids: unmapped_intermediate_ids(relation_composition),
    }
}

pub(super) fn findings_from_mapping_compositions(
    first: &Morphism,
    second: &Morphism,
    cell_composition: &MappingComposition,
    relation_composition: &MappingComposition,
) -> Vec<FailedCompositionFinding> {
    cell_composition
        .unmapped
        .iter()
        .map(|gap| {
            failed_composition_finding(
                first,
                second,
                FailedCompositionFindingKind::UnmappedIntermediateCell,
                gap,
            )
        })
        .chain(relation_composition.unmapped.iter().map(|gap| {
            failed_composition_finding(
                first,
                second,
                FailedCompositionFindingKind::UnmappedIntermediateRelation,
                gap,
            )
        }))
        .collect()
}

pub(super) fn failed_composition_finding(
    first: &Morphism,
    second: &Morphism,
    finding_type: FailedCompositionFindingKind,
    gap: &MappingGap,
) -> FailedCompositionFinding {
    FailedCompositionFinding {
        obstruction_type: FAILED_COMPOSITION_OBSTRUCTION_TYPE.to_owned(),
        finding_type,
        first_morphism_id: first.id.clone(),
        second_morphism_id: second.id.clone(),
        source_element_id: gap.source_element_id.clone(),
        intermediate_element_id: gap.intermediate_element_id.clone(),
    }
}

pub(super) fn unmapped_intermediate_ids(composition: &MappingComposition) -> Vec<Id> {
    composition
        .unmapped
        .iter()
        .map(|gap| gap.intermediate_element_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn compose_mapping_parts(
    first: &BTreeMap<Id, Id>,
    second: &BTreeMap<Id, Id>,
) -> MappingComposition {
    let mut mapping = BTreeMap::new();
    let mut unmapped = Vec::new();

    for (source_id, intermediate_id) in first {
        if let Some(target_id) = second.get(intermediate_id) {
            mapping.insert(source_id.clone(), target_id.clone());
        } else {
            unmapped.push(MappingGap {
                source_element_id: source_id.clone(),
                intermediate_element_id: intermediate_id.clone(),
            });
        }
    }

    MappingComposition { mapping, unmapped }
}

pub(super) fn pullback_matches(
    left: &BTreeMap<Id, Id>,
    right: &BTreeMap<Id, Id>,
) -> PullbackMatches {
    let mut right_by_target = BTreeMap::<Id, Vec<Id>>::new();
    for (right_source_id, target_id) in right {
        right_by_target
            .entry(target_id.clone())
            .or_default()
            .push(right_source_id.clone());
    }

    let mut left_by_target = BTreeMap::<Id, Vec<Id>>::new();
    for (left_source_id, target_id) in left {
        left_by_target
            .entry(target_id.clone())
            .or_default()
            .push(left_source_id.clone());
    }

    let mut matches = Vec::new();
    let mut unmatched_left_ids = BTreeSet::new();
    for (left_source_id, target_id) in left {
        if let Some(right_source_ids) = right_by_target.get(target_id) {
            for right_source_id in right_source_ids {
                matches.push(PullbackMappingMatch {
                    left_source_id: left_source_id.clone(),
                    right_source_id: right_source_id.clone(),
                    target_id: target_id.clone(),
                });
            }
        } else {
            unmatched_left_ids.insert(left_source_id.clone());
        }
    }

    let mut unmatched_right_ids = BTreeSet::new();
    for (right_source_id, target_id) in right {
        if !left_by_target.contains_key(target_id) {
            unmatched_right_ids.insert(right_source_id.clone());
        }
    }

    PullbackMatches {
        matches,
        unmatched_left_ids: unmatched_left_ids.into_iter().collect(),
        unmatched_right_ids: unmatched_right_ids.into_iter().collect(),
    }
}

pub(super) fn pushout_groups(left: &BTreeMap<Id, Id>, right: &BTreeMap<Id, Id>) -> PushoutGroups {
    let left_source_ids = left.keys().cloned().collect::<BTreeSet<_>>();
    let right_source_ids = right.keys().cloned().collect::<BTreeSet<_>>();
    let identified_groups = left_source_ids
        .intersection(&right_source_ids)
        .map(|source_element_id| IdentifiedSourceGroup {
            source_element_id: source_element_id.clone(),
            left_target_id: left[source_element_id].clone(),
            right_target_id: right[source_element_id].clone(),
        })
        .collect();

    PushoutGroups {
        identified_groups,
        unmatched_left_source_ids: left_source_ids
            .difference(&right_source_ids)
            .cloned()
            .collect(),
        unmatched_right_source_ids: right_source_ids
            .difference(&left_source_ids)
            .cloned()
            .collect(),
    }
}

pub(super) fn compose_path_summary(path: &[Morphism]) -> DiagramPathSummary {
    let morphism_ids = path.iter().map(|morphism| morphism.id.clone()).collect();
    let Some(first) = path.first() else {
        return DiagramPathSummary {
            morphism_ids,
            source_space_id: None,
            target_space_id: None,
            cell_mapping: BTreeMap::new(),
            relation_mapping: BTreeMap::new(),
            coverage: CompositionCoverage::default(),
        };
    };

    let mut cell_mapping = first.cell_mapping.clone();
    let mut relation_mapping = first.relation_mapping.clone();
    let mut unmapped_cell_intermediate_ids = BTreeSet::new();
    let mut unmapped_relation_intermediate_ids = BTreeSet::new();

    for next in path.iter().skip(1) {
        let cell_composition = compose_mapping_parts(&cell_mapping, &next.cell_mapping);
        let relation_composition = compose_mapping_parts(&relation_mapping, &next.relation_mapping);
        unmapped_cell_intermediate_ids.extend(unmapped_intermediate_ids(&cell_composition));
        unmapped_relation_intermediate_ids.extend(unmapped_intermediate_ids(&relation_composition));
        cell_mapping = cell_composition.mapping;
        relation_mapping = relation_composition.mapping;
    }

    DiagramPathSummary {
        morphism_ids,
        source_space_id: Some(first.source_space_id.clone()),
        target_space_id: path.last().map(|morphism| morphism.target_space_id.clone()),
        cell_mapping,
        relation_mapping,
        coverage: CompositionCoverage {
            unmapped_cell_intermediate_ids: unmapped_cell_intermediate_ids.into_iter().collect(),
            unmapped_relation_intermediate_ids: unmapped_relation_intermediate_ids
                .into_iter()
                .collect(),
        },
    }
}

pub(super) fn path_obstructions(label: &str, path: &DiagramPathSummary) -> Vec<DiagramObstruction> {
    let mut obstructions = Vec::new();
    if path.morphism_ids.is_empty() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompatiblePath,
            reason: format!("{label} path is empty"),
        });
    }
    if !path.coverage.is_complete() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompletePath,
            reason: format!("{label} path has explicit mappings that cannot continue"),
        });
    }
    obstructions
}

pub(super) fn path_boundary_obstructions(
    label: &str,
    path: &[Morphism],
) -> Vec<DiagramObstruction> {
    path.windows(2)
        .filter_map(|window| {
            let first = &window[0];
            let second = &window[1];
            (first.target_space_id != second.source_space_id).then(|| DiagramObstruction {
                obstruction_type: DiagramObstructionType::IncompatiblePath,
                reason: format!(
                    "{label} path morphism {} targets {}, but next morphism {} sources {}",
                    first.id, first.target_space_id, second.id, second.source_space_id
                ),
            })
        })
        .collect()
}

pub(super) fn mapping_witnesses(
    element_kind: DiagramElementKind,
    left: &BTreeMap<Id, Id>,
    right: &BTreeMap<Id, Id>,
) -> Vec<NonCommutativeWitness> {
    let source_ids = left
        .keys()
        .chain(right.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    source_ids
        .into_iter()
        .filter_map(|source_element_id| {
            let left_target_id = left.get(&source_element_id);
            let right_target_id = right.get(&source_element_id);
            (left_target_id != right_target_id).then(|| NonCommutativeWitness {
                element_kind,
                source_element_id,
                left_target_id: left_target_id.cloned(),
                right_target_id: right_target_id.cloned(),
            })
        })
        .collect()
}

#[derive(Debug)]
pub(super) struct MappingComposition {
    pub(super) mapping: BTreeMap<Id, Id>,
    pub(super) unmapped: Vec<MappingGap>,
}

#[derive(Debug)]
pub(super) struct MappingGap {
    source_element_id: Id,
    intermediate_element_id: Id,
}

#[derive(Debug)]
pub(super) struct PullbackMatches {
    matches: Vec<PullbackMappingMatch>,
    unmatched_left_ids: Vec<Id>,
    unmatched_right_ids: Vec<Id>,
}

impl PullbackMatches {
    pub(super) fn into_parts(self) -> (Vec<PullbackMappingMatch>, Vec<Id>, Vec<Id>) {
        (
            self.matches,
            self.unmatched_left_ids,
            self.unmatched_right_ids,
        )
    }
}

#[derive(Debug)]
pub(super) struct PullbackMappingMatch {
    pub(super) left_source_id: Id,
    pub(super) right_source_id: Id,
    pub(super) target_id: Id,
}

#[derive(Debug)]
pub(super) struct PushoutGroups {
    identified_groups: Vec<IdentifiedSourceGroup>,
    unmatched_left_source_ids: Vec<Id>,
    unmatched_right_source_ids: Vec<Id>,
}

impl PushoutGroups {
    pub(super) fn into_parts(self) -> (Vec<IdentifiedSourceGroup>, Vec<Id>, Vec<Id>) {
        (
            self.identified_groups,
            self.unmatched_left_source_ids,
            self.unmatched_right_source_ids,
        )
    }
}

pub(super) fn intersect_ids(first: &[Id], second: &[Id]) -> Vec<Id> {
    let first_ids: BTreeSet<Id> = first.iter().cloned().collect();
    let second_ids: BTreeSet<Id> = second.iter().cloned().collect();

    first_ids.intersection(&second_ids).cloned().collect()
}

pub(super) fn concat_records<T: Clone>(first: &[T], second: &[T]) -> Vec<T> {
    first.iter().chain(second.iter()).cloned().collect()
}

pub(super) fn partition_by_membership(
    selected: BTreeSet<Id>,
    known_preserved: &BTreeSet<Id>,
) -> (Vec<Id>, Vec<Id>) {
    selected
        .into_iter()
        .partition(|invariant_id| known_preserved.contains(invariant_id))
}
