use super::*;

pub(super) fn evaluate_acyclicity(
    rule: &EvaluatorRule,
    check: &AcyclicityCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let relation_types = normalized_string_set("relation_types", &check.relation_types)?;
    let cycle = directed_cycle(
        context.space_store,
        &context.check_input.space_id,
        &relation_types,
    )?;

    Ok(cycle.map_or_else(
        || CheckResult::satisfied(rule.target_kind, rule.target_id.clone()),
        |cycle| {
            CheckResult::violated(
                rule.target_kind,
                rule.target_id.clone(),
                Violation::new(
                    format!("cycle detected through cells: {}", join_ids(&cycle)),
                    rule.severity,
                )
                .with_location_cells(cycle),
            )
        },
    ))
}

pub(super) fn evaluate_required_path(
    rule: &EvaluatorRule,
    check: &RequiredPathCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let query = ReachabilityQuery::new(
        context.check_input.space_id.clone(),
        check.from_cell_id.clone(),
        check.to_cell_id.clone(),
    )
    .with_options(check.options.clone());
    let result = context.space_store.reachable(&query)?;

    if result.reachable {
        return Ok(CheckResult::satisfied(
            rule.target_kind,
            rule.target_id.clone(),
        ));
    }

    let mut location_cell_ids = vec![check.from_cell_id.clone(), check.to_cell_id.clone()];
    location_cell_ids.extend(result.frontier_cell_ids);
    normalize_ids(&mut location_cell_ids);

    Ok(CheckResult::violated(
        rule.target_kind,
        rule.target_id.clone(),
        Violation::new(
            format!(
                "required path from {} to {} was not found",
                check.from_cell_id, check.to_cell_id
            ),
            rule.severity,
        )
        .with_location_cells(location_cell_ids),
    ))
}

pub(super) fn evaluate_reachability_safety(
    rule: &EvaluatorRule,
    check: &ReachabilitySafetyCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let from_cell_ids = normalized_ids(&check.from_cell_ids);
    let forbidden_cell_ids = normalized_ids(&check.forbidden_cell_ids);

    if from_cell_ids.is_empty() || forbidden_cell_ids.is_empty() {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            "reachability safety requires source and forbidden cell identifiers",
        ));
    }

    for from_cell_id in &from_cell_ids {
        for forbidden_cell_id in &forbidden_cell_ids {
            let query = ReachabilityQuery::new(
                context.check_input.space_id.clone(),
                from_cell_id.clone(),
                forbidden_cell_id.clone(),
            )
            .with_options(check.options.clone());
            let result = context.space_store.reachable(&query)?;
            if result.reachable {
                return Ok(forbidden_reachability_result(
                    rule,
                    from_cell_id,
                    forbidden_cell_id,
                    result.shortest_path.as_ref(),
                ));
            }
        }
    }

    Ok(CheckResult::satisfied(
        rule.target_kind,
        rule.target_id.clone(),
    ))
}

pub(super) fn evaluate_context_compatibility(
    rule: &EvaluatorRule,
    check: &ContextCompatibilityCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let cell_ids = if check.cell_ids.is_empty() {
        context.check_input.changed_cell_ids.clone()
    } else {
        check.cell_ids.clone()
    };
    let cell_ids = normalized_ids(&cell_ids);
    let required_context_ids = normalized_ids(&check.required_context_ids);

    if cell_ids.is_empty() {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            "context compatibility requires explicit cells or changed-cell input",
        ));
    }
    if required_context_ids.is_empty() {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            "context compatibility requires declared context identifiers",
        ));
    }

    for cell_id in &cell_ids {
        let cell = cell_in_space(context.space_store, cell_id, &context.check_input.space_id)?;
        let missing_context_ids = required_context_ids
            .iter()
            .filter(|context_id| !cell.context_ids.contains(*context_id))
            .cloned()
            .collect::<Vec<_>>();

        if !missing_context_ids.is_empty() {
            return Ok(CheckResult::violated(
                rule.target_kind,
                rule.target_id.clone(),
                Violation::new(
                    format!(
                        "cell {cell_id} is missing required contexts: {}",
                        join_ids(&missing_context_ids)
                    ),
                    rule.severity,
                )
                .with_location_cells(vec![cell_id.clone()])
                .with_location_contexts(missing_context_ids),
            ));
        }
    }

    Ok(CheckResult::satisfied(
        rule.target_kind,
        rule.target_id.clone(),
    ))
}

pub(super) fn evaluate_morphism_preservation(
    rule: &EvaluatorRule,
    check: &MorphismPreservationCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let Some(morphism) = context
        .morphisms
        .iter()
        .find(|morphism| morphism.id == check.morphism_id)
    else {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            format!("morphism {} was not supplied", check.morphism_id),
        ));
    };

    let invariant_ids = preservation_invariant_ids(rule, check, context.check_input);
    if invariant_ids.is_empty() {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            "morphism preservation requires invariant identifiers",
        ));
    }

    let report = morphism.check_preservation(invariant_ids);
    if report.violated.is_empty() {
        return Ok(CheckResult::satisfied(
            rule.target_kind,
            rule.target_id.clone(),
        ));
    }

    Ok(CheckResult::violated(
        rule.target_kind,
        rule.target_id.clone(),
        Violation::new(
            morphism_preservation_message(&check.morphism_id, &report.violated),
            rule.severity,
        )
        .with_related_morphisms(vec![check.morphism_id.clone()]),
    ))
}

pub(super) fn evaluate_projection_loss_declared(
    rule: &EvaluatorRule,
    check: &ProjectionLossDeclarationCheck,
    context: &EvaluatorContext<'_>,
) -> Result<CheckResult> {
    let Some(projection) = context
        .projections
        .iter()
        .find(|projection| projection.id == check.projection_id)
    else {
        return Ok(CheckResult::unsupported(
            rule.target_kind,
            rule.target_id.clone(),
            format!("projection {} was not supplied", check.projection_id),
        ));
    };

    if projection.information_loss().is_empty() {
        return Ok(projection_loss_violation(
            rule,
            &check.projection_id,
            "does not declare information loss",
        ));
    }

    let required_source_ids = normalized_ids(&check.required_source_ids);
    if required_source_ids.is_empty() {
        return Ok(CheckResult::satisfied(
            rule.target_kind,
            rule.target_id.clone(),
        ));
    }

    let declared_source_ids = projection
        .information_loss()
        .iter()
        .flat_map(|loss| loss.source_ids().iter().cloned())
        .collect::<BTreeSet<_>>();
    let missing_source_ids = required_source_ids
        .iter()
        .filter(|source_id| !declared_source_ids.contains(*source_id))
        .cloned()
        .collect::<Vec<_>>();

    if missing_source_ids.is_empty() {
        Ok(CheckResult::satisfied(
            rule.target_kind,
            rule.target_id.clone(),
        ))
    } else {
        Ok(projection_loss_violation(
            rule,
            &check.projection_id,
            format!(
                "does not declare information loss for sources: {}",
                join_ids(&missing_source_ids)
            ),
        ))
    }
}

fn directed_cycle(
    store: &InMemorySpaceStore,
    space_id: &Id,
    relation_types: &BTreeSet<String>,
) -> Result<Option<Vec<Id>>> {
    let space = store
        .space(space_id)
        .ok_or_else(|| malformed_field("space_id", format!("identifier {space_id} is absent")))?;
    let mut adjacency = space
        .cell_ids
        .iter()
        .cloned()
        .map(|cell_id| (cell_id, Vec::new()))
        .collect::<BTreeMap<_, _>>();

    for incidence_id in &space.incidence_ids {
        let incidence = store.incidence(incidence_id).ok_or_else(|| {
            malformed_field(
                "incidence_ids",
                format!("identifier {incidence_id} is absent"),
            )
        })?;
        if !relation_types.is_empty() && !relation_types.contains(&incidence.relation_type) {
            continue;
        }
        adjacency
            .entry(incidence.from_cell_id.clone())
            .or_default()
            .push(incidence.to_cell_id.clone());
        adjacency.entry(incidence.to_cell_id.clone()).or_default();
    }

    for neighbors in adjacency.values_mut() {
        normalize_ids(neighbors);
    }

    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    for cell_id in adjacency.keys().cloned().collect::<Vec<_>>() {
        if states.contains_key(&cell_id) {
            continue;
        }
        if let Some(cycle) = directed_cycle_from(&cell_id, &adjacency, &mut states, &mut stack) {
            return Ok(Some(cycle));
        }
    }

    Ok(None)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Visited,
}

fn directed_cycle_from(
    cell_id: &Id,
    adjacency: &BTreeMap<Id, Vec<Id>>,
    states: &mut BTreeMap<Id, VisitState>,
    stack: &mut Vec<Id>,
) -> Option<Vec<Id>> {
    states.insert(cell_id.clone(), VisitState::Visiting);
    stack.push(cell_id.clone());

    for neighbor_id in adjacency.get(cell_id).into_iter().flatten() {
        match states.get(neighbor_id).copied() {
            Some(VisitState::Visiting) => {
                let start = stack
                    .iter()
                    .position(|candidate| candidate == neighbor_id)
                    .unwrap_or(0);
                let mut cycle = stack[start..].to_vec();
                cycle.push(neighbor_id.clone());
                return Some(cycle);
            }
            Some(VisitState::Visited) => {}
            None => {
                if let Some(cycle) = directed_cycle_from(neighbor_id, adjacency, states, stack) {
                    return Some(cycle);
                }
            }
        }
    }

    stack.pop();
    states.insert(cell_id.clone(), VisitState::Visited);
    None
}

fn forbidden_reachability_result(
    rule: &EvaluatorRule,
    from_cell_id: &Id,
    forbidden_cell_id: &Id,
    path: Option<&GraphPath>,
) -> CheckResult {
    let location_cell_ids = path.map_or_else(
        || vec![from_cell_id.clone(), forbidden_cell_id.clone()],
        GraphPath::cell_ids,
    );
    CheckResult::violated(
        rule.target_kind,
        rule.target_id.clone(),
        Violation::new(
            format!("forbidden reachability from {from_cell_id} to {forbidden_cell_id} was found"),
            rule.severity,
        )
        .with_location_cells(location_cell_ids),
    )
}

fn cell_in_space<'a>(
    store: &'a InMemorySpaceStore,
    cell_id: &Id,
    expected_space_id: &Id,
) -> Result<&'a higher_graphen_structure::space::Cell> {
    let cell = store
        .cell(cell_id)
        .ok_or_else(|| malformed_field("cell_ids", format!("identifier {cell_id} is absent")))?;
    if &cell.space_id == expected_space_id {
        Ok(cell)
    } else {
        Err(malformed_field(
            "cell_ids",
            format!(
                "identifier {cell_id} belongs to space {}, expected {expected_space_id}",
                cell.space_id
            ),
        ))
    }
}

fn preservation_invariant_ids(
    rule: &EvaluatorRule,
    check: &MorphismPreservationCheck,
    input: &CheckInput,
) -> Vec<Id> {
    if !check.invariant_ids.is_empty() {
        return normalized_ids(&check.invariant_ids);
    }
    if rule.target_kind == CheckTargetKind::Invariant {
        return vec![rule.target_id.clone()];
    }
    normalized_ids(&input.invariant_ids)
}

fn morphism_preservation_message(morphism_id: &Id, violated_invariant_ids: &[Id]) -> String {
    format!(
        "morphism {morphism_id} does not preserve invariants: {}",
        join_ids(violated_invariant_ids)
    )
}

fn projection_loss_violation(
    rule: &EvaluatorRule,
    projection_id: &Id,
    detail: impl Into<String>,
) -> CheckResult {
    CheckResult::violated(
        rule.target_kind,
        rule.target_id.clone(),
        Violation::new(
            format!("projection {projection_id} {}", detail.into()),
            rule.severity,
        ),
    )
}
