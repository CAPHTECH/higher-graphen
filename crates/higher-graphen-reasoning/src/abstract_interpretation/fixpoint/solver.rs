use super::types::{
    AbstractElement, AbstractGraph, AbstractInterpretationReport, AbstractJoin, AbstractMembership,
    FixpointObstruction, FixpointObstructionType, FixpointOptions, MembershipCheck,
    NodeAbstractState, SoundnessStatus, WideningEvent,
};
use higher_graphen_core::{Id, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Mutable solver state shared across the worklist loop.
pub(super) struct Solver {
    pub(super) states: BTreeMap<Id, AbstractElement>,
    pub(super) revisits: BTreeMap<Id, usize>,
    pub(super) widening_events: Vec<WideningEvent>,
    pub(super) obstructions: Vec<FixpointObstruction>,
    pub(super) information_loss: BTreeSet<String>,
}

/// Computes the least fixpoint of `graph` and evaluates the selected `checks`.
///
/// Propagates seed states along edges with the monotone `gen`-only transfer,
/// joins at merge nodes via [`AbstractJoin`], and iterates in deterministic
/// (sorted) worklist order. Widening is applied per
/// [`FixpointOptions::widen_threshold`] to guarantee termination on cyclic
/// graphs. The returned report is a reviewable candidate; membership results
/// default to "possibly" and never assert an exact violation without a witness.
pub fn run_fixpoint(
    analysis_id: Id,
    graph: &AbstractGraph,
    options: &FixpointOptions,
    checks: &[MembershipCheck],
) -> Result<AbstractInterpretationReport> {
    let normalized = graph.normalize()?;

    let mut solver = Solver {
        states: normalized.seed_states.clone(),
        revisits: BTreeMap::new(),
        widening_events: Vec::new(),
        obstructions: Vec::new(),
        information_loss: BTreeSet::new(),
    };

    // Seed worklist: every node that has a state. Sorted ascending by id via BTreeSet.
    let mut worklist: BTreeSet<Id> = normalized.seed_states.keys().cloned().collect();

    let mut iterations = 0usize;
    let mut reached_fixpoint = true;

    while let Some(node_id) = worklist.iter().next().cloned() {
        worklist.remove(&node_id);

        if iterations >= options.max_iterations {
            reached_fixpoint = false;
            solver.obstructions.push(FixpointObstruction {
                obstruction_type: FixpointObstructionType::IterationLimitExceeded,
                reason: format!(
                    "iteration bound {} reached before a fixed point",
                    options.max_iterations
                ),
                node_id: None,
            });
            solver.information_loss.insert(
                "iteration bound reached; node states are a partial under-iterated result"
                    .to_owned(),
            );
            break;
        }
        iterations += 1;

        let Some(current) = solver.states.get(&node_id).cloned() else {
            continue;
        };
        if matches!(current.soundness, SoundnessStatus::Unsound) {
            push_unique_obstruction(
                &mut solver.obstructions,
                FixpointObstruction {
                    obstruction_type: FixpointObstructionType::UnsoundAbstraction,
                    reason: "node state is known unsound; absence cannot be treated as excluded"
                        .to_owned(),
                    node_id: Some(node_id.clone()),
                },
            );
        }

        let Some(edges) = normalized.adjacency.get(&node_id) else {
            continue;
        };

        for edge in edges {
            let transferred = edge.transfer(edge.target_node_id.clone(), &current);
            propagate(
                &mut solver,
                &mut worklist,
                edge.target_node_id.clone(),
                transferred,
                options,
                iterations,
            )?;
        }
    }

    Ok(build_report(
        analysis_id,
        solver,
        iterations,
        reached_fixpoint,
        checks,
    ))
}

/// Joins `incoming` into the target node's state, widening when revisited too often.
pub(super) fn propagate(
    solver: &mut Solver,
    worklist: &mut BTreeSet<Id>,
    target: Id,
    incoming: AbstractElement,
    options: &FixpointOptions,
    iteration: usize,
) -> Result<()> {
    let Some(existing) = solver.states.get(&target).cloned() else {
        // First arrival: install the transferred state directly.
        solver.states.insert(target.clone(), incoming);
        worklist.insert(target);
        return Ok(());
    };

    if existing.domain != incoming.domain {
        push_unique_obstruction(
            &mut solver.obstructions,
            FixpointObstruction {
                obstruction_type: FixpointObstructionType::MissingJoinOperation,
                reason: "cannot join states from different domains at a merge node".to_owned(),
                node_id: Some(target),
            },
        );
        return Ok(());
    }

    let revisit_count = solver.revisits.entry(target.clone()).or_insert(0);
    *revisit_count += 1;
    let should_widen = *revisit_count > options.widen_threshold;

    // The join record id is internal and discarded; only `result` is used, so the
    // target id doubles as the throwaway join id (no synthesized id, no fallback).
    let join = AbstractJoin::new(target.clone(), target.clone(), &existing, &incoming)?;
    let joined = join.result;

    let next = if should_widen {
        widen(solver, &target, iteration, joined, options)
    } else {
        joined
    };

    if next != existing {
        solver.states.insert(target.clone(), next);
        worklist.insert(target);
    }

    Ok(())
}

/// Relaxes a revisited node's may-set toward the widening universe.
///
/// Only the may-set is widened: it is the single monotone-increasing direction
/// on this lattice and is therefore the only source of non-termination. The
/// must-set is monotone decreasing under join and converges on its own. When a
/// universe is configured, every universe id is unioned into the may-set, which
/// caps growth at a fixed finite set and guarantees a fixed point on the next
/// pass. The added ids are recorded as a [`WideningEvent`].
fn widen(
    solver: &mut Solver,
    target: &Id,
    iteration: usize,
    joined: AbstractElement,
    options: &FixpointOptions,
) -> AbstractElement {
    if options.widen_top.is_empty() {
        return joined;
    }

    let widened_possible_ids: BTreeSet<Id> = options
        .widen_top
        .difference(&joined.possible_concrete_ids)
        .cloned()
        .collect();
    if widened_possible_ids.is_empty() {
        return joined;
    }

    let mut widened = joined;
    widened
        .possible_concrete_ids
        .extend(widened_possible_ids.iter().cloned());

    solver.information_loss.insert(format!(
        "node {} widened toward top after exceeding the revisit threshold",
        target.as_str()
    ));
    solver.widening_events.push(WideningEvent {
        node_id: target.clone(),
        iteration,
        widened_possible_ids,
    });

    widened
}

/// Builds the final report from the converged (or bounded) solver state.
fn build_report(
    analysis_id: Id,
    mut solver: Solver,
    iterations: usize,
    reached_fixpoint: bool,
    checks: &[MembershipCheck],
) -> AbstractInterpretationReport {
    let widened_ids_by_node = widened_ids_by_node(&solver.widening_events);
    let node_states = sorted_node_states(&solver.states);
    let unknown_region_node_ids = unknown_region_node_ids(&node_states);
    let (definitely_satisfied, possibly_violated) =
        evaluate_checks(&mut solver, checks, &widened_ids_by_node);

    solver.obstructions.sort();
    solver.obstructions.dedup();
    // Derived `Ord` on `WideningEvent` orders by (node_id, iteration, added ids); a node
    // produces at most one event per iteration, so this is the (node_id, iteration) order.
    solver.widening_events.sort();

    AbstractInterpretationReport {
        analysis_id,
        node_states,
        reached_fixpoint,
        iterations,
        definitely_satisfied_check_ids: definitely_satisfied.into_iter().collect(),
        possibly_violated_check_ids: possibly_violated.into_iter().collect(),
        unknown_region_node_ids: unknown_region_node_ids.into_iter().collect(),
        widening_events: solver.widening_events,
        obstructions: solver.obstructions,
        information_loss: solver.information_loss.into_iter().collect(),
    }
}

fn widened_ids_by_node(events: &[WideningEvent]) -> BTreeMap<Id, BTreeSet<Id>> {
    let mut ids_by_node: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
    for event in events {
        ids_by_node
            .entry(event.node_id.clone())
            .or_default()
            .extend(event.widened_possible_ids.iter().cloned());
    }
    ids_by_node
}

fn sorted_node_states(states: &BTreeMap<Id, AbstractElement>) -> Vec<NodeAbstractState> {
    let mut node_states: Vec<NodeAbstractState> = states
        .iter()
        .map(|(node_id, state)| NodeAbstractState {
            node_id: node_id.clone(),
            state: state.clone(),
        })
        .collect();
    node_states.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    node_states
}

fn unknown_region_node_ids(node_states: &[NodeAbstractState]) -> BTreeSet<Id> {
    node_states
        .iter()
        .filter(|entry| {
            matches!(
                entry.state.soundness,
                SoundnessStatus::Unsound | SoundnessStatus::Unknown
            ) || !entry.state.regions.is_empty()
        })
        .map(|entry| entry.node_id.clone())
        .collect()
}

fn evaluate_checks(
    solver: &mut Solver,
    checks: &[MembershipCheck],
    widened_ids_by_node: &BTreeMap<Id, BTreeSet<Id>>,
) -> (BTreeSet<Id>, BTreeSet<Id>) {
    let mut definitely_satisfied = BTreeSet::new();
    let mut possibly_violated = BTreeSet::new();
    for check in checks {
        evaluate_check(
            solver,
            check,
            widened_ids_by_node,
            &mut definitely_satisfied,
            &mut possibly_violated,
        );
    }
    (definitely_satisfied, possibly_violated)
}

fn evaluate_check(
    solver: &mut Solver,
    check: &MembershipCheck,
    widened_ids_by_node: &BTreeMap<Id, BTreeSet<Id>>,
    definitely_satisfied: &mut BTreeSet<Id>,
    possibly_violated: &mut BTreeSet<Id>,
) {
    let Some(state) = solver.states.get(&check.node_id) else {
        possibly_violated.insert(check.id.clone());
        push_check_obstruction(
            solver,
            check,
            FixpointObstructionType::UnknownRegionRequiresWitness,
        );
        return;
    };

    match state.classify(&check.concrete_id) {
        AbstractMembership::Definite => {
            definitely_satisfied.insert(check.id.clone());
        }
        AbstractMembership::Possible => {
            possibly_violated.insert(check.id.clone());
            push_widening_distinction_obstruction(solver, check, widened_ids_by_node);
        }
        AbstractMembership::UnknownRegion | AbstractMembership::Unknown => {
            possibly_violated.insert(check.id.clone());
            push_unknown_region_obstruction(solver, check);
        }
        AbstractMembership::Excluded => {}
        AbstractMembership::Unsound => {
            possibly_violated.insert(check.id.clone());
            push_unsound_check_obstruction(solver, check);
        }
    }
}

fn push_check_obstruction(
    solver: &mut Solver,
    check: &MembershipCheck,
    obstruction_type: FixpointObstructionType,
) {
    push_unique_obstruction(
        &mut solver.obstructions,
        FixpointObstruction {
            obstruction_type,
            reason: format!(
                "check {} targets node {} that no state reached",
                check.id.as_str(),
                check.node_id.as_str()
            ),
            node_id: Some(check.node_id.clone()),
        },
    );
}

fn push_widening_distinction_obstruction(
    solver: &mut Solver,
    check: &MembershipCheck,
    widened_ids_by_node: &BTreeMap<Id, BTreeSet<Id>>,
) {
    let widening_added = widened_ids_by_node
        .get(&check.node_id)
        .is_some_and(|ids| ids.contains(&check.concrete_id));
    if !widening_added {
        return;
    }
    push_unique_obstruction(
        &mut solver.obstructions,
        FixpointObstruction {
            obstruction_type: FixpointObstructionType::WideningLostRequiredDistinction,
            reason: format!(
                "widening at node {} added concrete id {}, so check {} \
                 lost its exclusion and is only possible",
                check.node_id.as_str(),
                check.concrete_id.as_str(),
                check.id.as_str()
            ),
            node_id: Some(check.node_id.clone()),
        },
    );
}

fn push_unknown_region_obstruction(solver: &mut Solver, check: &MembershipCheck) {
    push_unique_obstruction(
        &mut solver.obstructions,
        FixpointObstruction {
            obstruction_type: FixpointObstructionType::UnknownRegionRequiresWitness,
            reason: format!(
                "check {} is unknown at node {} and requires a concrete witness",
                check.id.as_str(),
                check.node_id.as_str()
            ),
            node_id: Some(check.node_id.clone()),
        },
    );
}

fn push_unsound_check_obstruction(solver: &mut Solver, check: &MembershipCheck) {
    push_unique_obstruction(
        &mut solver.obstructions,
        FixpointObstruction {
            obstruction_type: FixpointObstructionType::UnsoundAbstraction,
            reason: format!(
                "check {} cannot be classified at unsound node {}",
                check.id.as_str(),
                check.node_id.as_str()
            ),
            node_id: Some(check.node_id.clone()),
        },
    );
}

fn push_unique_obstruction(
    obstructions: &mut Vec<FixpointObstruction>,
    obstruction: FixpointObstruction,
) {
    if !obstructions.contains(&obstruction) {
        obstructions.push(obstruction);
    }
}
