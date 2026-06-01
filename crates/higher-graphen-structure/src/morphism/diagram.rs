use super::helpers::*;
use super::*;

/// Checks whether two explicit morphism paths commute.
///
/// This finite MVP compares only explicit cell and relation mappings produced
/// by path composition. Missing mappings and incompatible path boundaries are
/// retained as structured obstructions.
pub fn check_diagram_commutativity(
    left_path: &[Morphism],
    right_path: &[Morphism],
) -> DiagramCommutativityReport {
    let left = compose_path_summary(left_path);
    let right = compose_path_summary(right_path);
    let mut obstructions = Vec::new();
    obstructions.extend(path_boundary_obstructions("left", left_path));
    obstructions.extend(path_boundary_obstructions("right", right_path));
    obstructions.extend(path_obstructions("left", &left));
    obstructions.extend(path_obstructions("right", &right));

    if left.source_space_id != right.source_space_id
        || left.target_space_id != right.target_space_id
    {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompatibleBoundary,
            reason: "left and right paths do not share the same source and target spaces"
                .to_owned(),
        });
    }

    let non_commutative_witnesses = mapping_witnesses(
        DiagramElementKind::Cell,
        &left.cell_mapping,
        &right.cell_mapping,
    )
    .into_iter()
    .chain(mapping_witnesses(
        DiagramElementKind::Relation,
        &left.relation_mapping,
        &right.relation_mapping,
    ))
    .collect::<Vec<_>>();

    if !non_commutative_witnesses.is_empty() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::NonCommutativeDiagram,
            reason: "left and right path mappings disagree on explicit source elements".to_owned(),
        });
    }

    let commutes = obstructions.is_empty();

    DiagramCommutativityReport {
        left_path: left,
        right_path: right,
        commutes,
        non_commutative_witnesses,
        obstructions,
        information_loss: vec![
            "only explicit morphism mappings are compared".to_owned(),
            "unmapped source elements are reported as incomplete path coverage".to_owned(),
        ],
    }
}

/// Checks multiple commutativity requirements for one finite diagram.
pub fn check_diagram_requirements(
    diagram_id: Id,
    requirements: &[DiagramCommutativityRequirement],
) -> DiagramCheckReport {
    let requirement_reports = requirements
        .iter()
        .map(|requirement| DiagramRequirementReport {
            requirement_id: requirement.id.clone(),
            report: check_diagram_commutativity(&requirement.left_path, &requirement.right_path),
        })
        .collect::<Vec<_>>();
    let commutes = requirement_reports
        .iter()
        .all(|requirement| requirement.report.commutes);

    DiagramCheckReport {
        diagram_id,
        commutes,
        requirement_reports,
    }
}
