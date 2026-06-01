use super::helpers::*;
use super::*;

/// Extracts common mapped substructure for two morphisms with a shared target.
///
/// The construction is finite and explicit: a left source and right source
/// match only when both are mapped to the same target identifier. Missing
/// mappings remain visible as unmatched identifiers; the report is a candidate,
/// not an accepted categorical universal property.
pub fn explicit_pullback_candidate(left: &Morphism, right: &Morphism) -> ExplicitPullbackReport {
    let compatible_target = left.target_space_id == right.target_space_id;
    let (cell_matches, unmatched_left_cell_ids, unmatched_right_cell_ids) =
        pullback_matches(&left.cell_mapping, &right.cell_mapping).into_parts();
    let (relation_matches, unmatched_left_relation_ids, unmatched_right_relation_ids) =
        pullback_matches(&left.relation_mapping, &right.relation_mapping).into_parts();
    let mut obstructions = Vec::new();

    if !compatible_target {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::IncompatibleTargetSpace,
            reason: format!(
                "left target space {} differs from right target space {}",
                left.target_space_id, right.target_space_id
            ),
        });
    }
    if !unmatched_left_cell_ids.is_empty()
        || !unmatched_right_cell_ids.is_empty()
        || !unmatched_left_relation_ids.is_empty()
        || !unmatched_right_relation_ids.is_empty()
    {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::PullbackIncomplete,
            reason: "some explicit mappings have no partner with the same target".to_owned(),
        });
    }

    ExplicitPullbackReport {
        left_morphism_id: left.id.clone(),
        right_morphism_id: right.id.clone(),
        left_source_space_id: left.source_space_id.clone(),
        right_source_space_id: right.source_space_id.clone(),
        target_space_id: compatible_target.then(|| left.target_space_id.clone()),
        cell_matches: cell_matches
            .into_iter()
            .map(|matched| PullbackCellMatch {
                left_cell_id: matched.left_source_id,
                right_cell_id: matched.right_source_id,
                target_cell_id: matched.target_id,
            })
            .collect(),
        relation_matches: relation_matches
            .into_iter()
            .map(|matched| PullbackRelationMatch {
                left_relation_id: matched.left_source_id,
                right_relation_id: matched.right_source_id,
                target_relation_id: matched.target_id,
            })
            .collect(),
        unmatched_left_cell_ids,
        unmatched_right_cell_ids,
        unmatched_left_relation_ids,
        unmatched_right_relation_ids,
        information_loss: vec![
            "only explicit mapping equality is considered".to_owned(),
            "universal property is not proven by this finite candidate report".to_owned(),
        ],
        obstructions,
        review_status: higher_graphen_core::ReviewStatus::Unreviewed,
    }
}

/// Extracts an explicit pushout-style merge report for two morphisms sharing a source.
///
/// The candidate identifies left and right targets that come from the same
/// source element. Use [`construct_explicit_pushout`] with finite cells and
/// incidences to materialize the merged candidate structure.
pub fn explicit_pushout_candidate(
    left: &Morphism,
    right: &Morphism,
    candidate_space_id: Id,
) -> ExplicitPushoutReport {
    let compatible_source = left.source_space_id == right.source_space_id;
    let (identified_cell_groups, unmatched_left_cell_source_ids, unmatched_right_cell_source_ids) =
        pushout_groups(&left.cell_mapping, &right.cell_mapping).into_parts();
    let (
        identified_relation_groups,
        unmatched_left_relation_source_ids,
        unmatched_right_relation_source_ids,
    ) = pushout_groups(&left.relation_mapping, &right.relation_mapping).into_parts();
    let mut obstructions = Vec::new();

    if !compatible_source {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleSourceSpace,
            reason: format!(
                "left source space {} differs from right source space {}",
                left.source_space_id, right.source_space_id
            ),
        });
    }
    if !unmatched_left_cell_source_ids.is_empty()
        || !unmatched_right_cell_source_ids.is_empty()
        || !unmatched_left_relation_source_ids.is_empty()
        || !unmatched_right_relation_source_ids.is_empty()
    {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::PushoutIncomplete,
            reason: "some explicit source mappings have no partner on the other leg".to_owned(),
        });
    }

    ExplicitPushoutReport {
        candidate_space_id,
        left_morphism_id: left.id.clone(),
        right_morphism_id: right.id.clone(),
        source_space_id: compatible_source.then(|| left.source_space_id.clone()),
        left_target_space_id: left.target_space_id.clone(),
        right_target_space_id: right.target_space_id.clone(),
        identified_cell_groups,
        identified_relation_groups,
        unmatched_left_cell_source_ids,
        unmatched_right_cell_source_ids,
        unmatched_left_relation_source_ids,
        unmatched_right_relation_source_ids,
        quotient_losses: Vec::new(),
        obstructions,
        review_status: higher_graphen_core::ReviewStatus::Unreviewed,
    }
}
