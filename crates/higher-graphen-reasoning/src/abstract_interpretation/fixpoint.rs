//! Monotone least-fixpoint engine over the membership lattice.
//!
//! This module adds the propagation loop that the data structures in the
//! parent [`super`] module describe but do not compute. It operates on an
//! explicit finite directed graph (`node_id` plus per-edge successors) and a
//! set of seed [`AbstractElement`] states, propagates each state along edges
//! with a monotone `gen`-only transfer, joins at merge nodes through the
//! existing [`AbstractJoin`], and iterates to a least fixpoint in deterministic
//! (sorted) worklist order. A revisit-count widening hook guarantees
//! termination on cyclic graphs. The result is a reviewable
//! [`AbstractInterpretationReport`]; it is never promoted to accepted fact.

use super::{AbstractDomain, AbstractElement, AbstractJoin, AbstractMembership, SoundnessStatus};
use higher_graphen_core::{CoreError, Id, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// One directed edge with a monotone `gen`-only transfer.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractEdge {
    /// Successor node reached by this edge.
    pub target_node_id: Id,
    /// Concrete identifiers introduced into both the must-set and may-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gen_definite: BTreeSet<Id>,
    /// Concrete identifiers introduced into the may-set only.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gen_possible: BTreeSet<Id>,
}

impl AbstractEdge {
    /// Creates an identity-transfer edge (no introduced identifiers).
    #[must_use]
    pub fn new(target_node_id: Id) -> Self {
        Self {
            target_node_id,
            gen_definite: BTreeSet::new(),
            gen_possible: BTreeSet::new(),
        }
    }

    /// Returns this edge with concrete identifiers added to the must-set and may-set.
    #[must_use]
    pub fn with_gen_definite(mut self, gen_definite: impl IntoIterator<Item = Id>) -> Self {
        self.gen_definite = gen_definite.into_iter().collect();
        self
    }

    /// Returns this edge with concrete identifiers added to the may-set only.
    #[must_use]
    pub fn with_gen_possible(mut self, gen_possible: impl IntoIterator<Item = Id>) -> Self {
        self.gen_possible = gen_possible.into_iter().collect();
        self
    }

    /// Applies the monotone transfer `f_edge` to an incoming state.
    ///
    /// `gen_definite` is unioned into both sets; `gen_possible` is unioned into
    /// the may-set only. The result keeps the must-set a subset of the may-set,
    /// so it is a valid [`AbstractElement`].
    fn transfer(&self, result_id: Id, incoming: &AbstractElement) -> AbstractElement {
        let mut definite = incoming.definite_concrete_ids.clone();
        definite.extend(self.gen_definite.iter().cloned());
        let mut possible = incoming.possible_concrete_ids.clone();
        possible.extend(self.gen_definite.iter().cloned());
        possible.extend(self.gen_possible.iter().cloned());

        let mut state = AbstractElement::new(
            result_id,
            incoming.domain.clone(),
            possible,
            incoming.soundness,
        );
        state.definite_concrete_ids = definite;
        state.source_ids = incoming.source_ids.clone();
        state
    }
}

/// One node of the explicit finite directed graph.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractGraphNode {
    /// Stable node identifier.
    pub node_id: Id,
    /// Outgoing edges. Deduplicated and sorted on graph construction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successors: Vec<AbstractEdge>,
}

impl AbstractGraphNode {
    /// Creates a node with the supplied outgoing edges.
    #[must_use]
    pub fn new(node_id: Id, successors: impl IntoIterator<Item = AbstractEdge>) -> Self {
        Self {
            node_id,
            successors: successors.into_iter().collect(),
        }
    }
}

/// Seed abstract state attached to a node before iteration starts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSeed {
    /// Node the seed applies to.
    pub node_id: Id,
    /// Initial may-set (possible concrete identifiers).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub possible_concrete_ids: BTreeSet<Id>,
    /// Initial must-set (definite concrete identifiers); must be a subset of the may-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub definite_concrete_ids: BTreeSet<Id>,
}

impl NodeSeed {
    /// Creates a seed with only possible (may-set) identifiers.
    #[must_use]
    pub fn possible(node_id: Id, possible_concrete_ids: impl IntoIterator<Item = Id>) -> Self {
        Self {
            node_id,
            possible_concrete_ids: possible_concrete_ids.into_iter().collect(),
            definite_concrete_ids: BTreeSet::new(),
        }
    }

    /// Creates an exact seed where every possible identifier is also definite.
    #[must_use]
    pub fn exact(node_id: Id, concrete_ids: impl IntoIterator<Item = Id>) -> Self {
        let concrete_ids: BTreeSet<Id> = concrete_ids.into_iter().collect();
        Self {
            node_id,
            possible_concrete_ids: concrete_ids.clone(),
            definite_concrete_ids: concrete_ids,
        }
    }

    /// Returns this seed with definite (must-set) identifiers attached.
    #[must_use]
    pub fn with_definite_concrete_ids(
        mut self,
        definite_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Self {
        self.definite_concrete_ids = definite_concrete_ids.into_iter().collect();
        self
    }
}

/// Explicit finite directed graph plus seed states for the fixpoint solver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractGraph {
    /// Single domain shared by every node state (joins require equal domains).
    pub domain: AbstractDomain,
    /// Nodes with their outgoing adjacency.
    pub nodes: Vec<AbstractGraphNode>,
    /// Seed abstract states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seeds: Vec<NodeSeed>,
    /// Soundness assumption joined into every node state.
    pub soundness: SoundnessStatus,
}

impl AbstractGraph {
    /// Creates a graph in the supplied domain.
    #[must_use]
    pub fn new(domain: AbstractDomain, soundness: SoundnessStatus) -> Self {
        Self {
            domain,
            nodes: Vec::new(),
            seeds: Vec::new(),
            soundness,
        }
    }

    /// Returns this graph with the supplied nodes appended.
    #[must_use]
    pub fn with_nodes(mut self, nodes: impl IntoIterator<Item = AbstractGraphNode>) -> Self {
        self.nodes.extend(nodes);
        self
    }

    /// Returns this graph with the supplied seeds appended.
    #[must_use]
    pub fn with_seeds(mut self, seeds: impl IntoIterator<Item = NodeSeed>) -> Self {
        self.seeds.extend(seeds);
        self
    }

    /// Normalizes nodes and validates that edges and seeds reference known nodes.
    ///
    /// Produces the deterministic adjacency map and the seed states. Returns a
    /// `MalformedField` error when an edge target or a seed node is unknown, or
    /// when a seed's must-set is not a subset of its may-set.
    fn normalize(&self) -> Result<NormalizedGraph> {
        let mut adjacency: BTreeMap<Id, BTreeSet<AbstractEdge>> = BTreeMap::new();
        for node in &self.nodes {
            adjacency
                .entry(node.node_id.clone())
                .or_default()
                .extend(node.successors.iter().cloned());
        }

        let node_ids: BTreeSet<Id> = adjacency.keys().cloned().collect();
        for edges in adjacency.values() {
            for edge in edges {
                if !node_ids.contains(&edge.target_node_id) {
                    return Err(malformed(
                        "edge.target_node_id",
                        format!(
                            "edge target {} is not a declared node",
                            edge.target_node_id.as_str()
                        ),
                    ));
                }
            }
        }

        let mut seed_states: BTreeMap<Id, AbstractElement> = BTreeMap::new();
        for seed in &self.seeds {
            if !node_ids.contains(&seed.node_id) {
                return Err(malformed(
                    "seed.node_id",
                    format!("seed node {} is not a declared node", seed.node_id.as_str()),
                ));
            }
            let mut state = AbstractElement::new(
                seed.node_id.clone(),
                self.domain.clone(),
                seed.possible_concrete_ids.iter().cloned(),
                self.soundness,
            );
            state.definite_concrete_ids = seed.definite_concrete_ids.clone();
            state.validate()?;
            seed_states.insert(seed.node_id.clone(), state);
        }

        Ok(NormalizedGraph {
            adjacency,
            seed_states,
        })
    }
}

/// Normalized, validated adjacency and seed states.
struct NormalizedGraph {
    adjacency: BTreeMap<Id, BTreeSet<AbstractEdge>>,
    seed_states: BTreeMap<Id, AbstractElement>,
}

/// Worklist and widening controls.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixpointOptions {
    /// A node is widened only after it is popped strictly more than this many times.
    pub widen_threshold: usize,
    /// May-set a widened node is relaxed to. Empty means widening only shrinks the must-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub widen_top: BTreeSet<Id>,
    /// Hard bound on total worklist pops before reporting `iteration_limit_exceeded`.
    pub max_iterations: usize,
}

impl Default for FixpointOptions {
    fn default() -> Self {
        Self {
            widen_threshold: 3,
            widen_top: BTreeSet::new(),
            max_iterations: 10_000,
        }
    }
}

impl FixpointOptions {
    /// Creates default options (widen after 3 revisits, no universe, 10000-pop bound).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns these options with a revisit threshold.
    #[must_use]
    pub fn with_widen_threshold(mut self, widen_threshold: usize) -> Self {
        self.widen_threshold = widen_threshold;
        self
    }

    /// Returns these options with an explicit widening universe (top may-set).
    #[must_use]
    pub fn with_widen_top(mut self, widen_top: impl IntoIterator<Item = Id>) -> Self {
        self.widen_top = widen_top.into_iter().collect();
        self
    }

    /// Returns these options with an iteration bound.
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }
}

/// A selected per-node membership check evaluated against the final state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipCheck {
    /// Stable check identifier.
    pub id: Id,
    /// Node whose final abstract state is queried.
    pub node_id: Id,
    /// Concrete identifier whose membership is checked.
    pub concrete_id: Id,
}

impl MembershipCheck {
    /// Creates a membership check.
    #[must_use]
    pub fn new(id: Id, node_id: Id, concrete_id: Id) -> Self {
        Self {
            id,
            node_id,
            concrete_id,
        }
    }
}

/// Stable obstruction category emitted by the fixpoint engine.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixpointObstructionType {
    /// Two states to merge are in different domains, so the join is undefined.
    MissingJoinOperation,
    /// A seed or derived state is known unsound; conservative absence reasoning is invalid.
    UnsoundAbstraction,
    /// A widening step moved a required distinction into imprecision.
    WideningLostRequiredDistinction,
    /// A selected check is unknown and requires a concrete witness before use.
    UnknownRegionRequiresWitness,
    /// The iteration bound was hit before a fixed point was reached.
    IterationLimitExceeded,
}

/// Structured fixpoint obstruction.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixpointObstruction {
    /// Obstruction category.
    pub obstruction_type: FixpointObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
    /// Node the obstruction is attached to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<Id>,
}

/// Record of one widening application at a node.
///
/// Widening relaxes only the may-set toward the configured universe; the
/// must-set is monotone decreasing under join and converges without widening,
/// so it is never widened. The added may-set ids are recorded so a reviewer can
/// see exactly which exclusions the widening sacrificed.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WideningEvent {
    /// Node that was widened.
    pub node_id: Id,
    /// Worklist iteration (total pop count) at which widening was applied.
    pub iteration: usize,
    /// May-set identifiers added by relaxing toward the widening universe.
    pub widened_possible_ids: BTreeSet<Id>,
}

/// Per-node final abstract state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeAbstractState {
    /// Node the state belongs to.
    pub node_id: Id,
    /// Final abstract state at the node. Its element id equals the node id.
    pub state: AbstractElement,
}

/// Structured abstract-interpretation report (see math-extension-kernels.md).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractInterpretationReport {
    /// Analysis identifier.
    pub analysis_id: Id,
    /// Per-node final abstract states, sorted by node id.
    pub node_states: Vec<NodeAbstractState>,
    /// True when the solver reached a fixed point within the iteration bound.
    pub reached_fixpoint: bool,
    /// Total worklist pops performed.
    pub iterations: usize,
    /// Checks whose concrete id is definitely a member at the node, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitely_satisfied_check_ids: Vec<Id>,
    /// Checks whose concrete id is only possibly a member (the safe default), sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possibly_violated_check_ids: Vec<Id>,
    /// Nodes whose final state is unsound or carries an unknown region, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_region_node_ids: Vec<Id>,
    /// Widening applications, sorted by (node id, iteration).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widening_events: Vec<WideningEvent>,
    /// Obstructions, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<FixpointObstruction>,
    /// Explicit information-loss declarations, sorted and deduplicated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
}

impl AbstractInterpretationReport {
    /// Returns the final abstract state at a node, when present.
    #[must_use]
    pub fn state(&self, node_id: &Id) -> Option<&AbstractElement> {
        self.node_states
            .iter()
            .find(|entry| &entry.node_id == node_id)
            .map(|entry| &entry.state)
    }

    /// Returns true when the solver reached a fixed point and no obstruction was raised.
    #[must_use]
    pub fn is_conclusive(&self) -> bool {
        self.reached_fixpoint && self.obstructions.is_empty()
    }
}

/// Mutable solver state shared across the worklist loop.
struct Solver {
    states: BTreeMap<Id, AbstractElement>,
    revisits: BTreeMap<Id, usize>,
    widening_events: Vec<WideningEvent>,
    obstructions: Vec<FixpointObstruction>,
    information_loss: BTreeSet<String>,
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
fn propagate(
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
    // Per node, the union of every concrete id widening pushed into its may-set.
    // A check that is only `Possible` because of such an id lost an exclusion.
    let mut widened_ids_by_node: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
    for event in &solver.widening_events {
        widened_ids_by_node
            .entry(event.node_id.clone())
            .or_default()
            .extend(event.widened_possible_ids.iter().cloned());
    }

    let mut node_states: Vec<NodeAbstractState> = solver
        .states
        .iter()
        .map(|(node_id, state)| NodeAbstractState {
            node_id: node_id.clone(),
            state: state.clone(),
        })
        .collect();
    node_states.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    let mut unknown_region_node_ids: BTreeSet<Id> = BTreeSet::new();
    for entry in &node_states {
        if matches!(
            entry.state.soundness,
            SoundnessStatus::Unsound | SoundnessStatus::Unknown
        ) || !entry.state.regions.is_empty()
        {
            unknown_region_node_ids.insert(entry.node_id.clone());
        }
    }

    let mut definitely_satisfied: BTreeSet<Id> = BTreeSet::new();
    let mut possibly_violated: BTreeSet<Id> = BTreeSet::new();
    for check in checks {
        let Some(state) = solver.states.get(&check.node_id) else {
            // No state ever reached this node; conservatively unknown.
            possibly_violated.insert(check.id.clone());
            push_unique_obstruction(
                &mut solver.obstructions,
                FixpointObstruction {
                    obstruction_type: FixpointObstructionType::UnknownRegionRequiresWitness,
                    reason: format!(
                        "check {} targets node {} that no state reached",
                        check.id.as_str(),
                        check.node_id.as_str()
                    ),
                    node_id: Some(check.node_id.clone()),
                },
            );
            continue;
        };

        match state.classify(&check.concrete_id) {
            AbstractMembership::Definite => {
                // A definite fact survives widening (the must-set is never widened).
                definitely_satisfied.insert(check.id.clone());
            }
            AbstractMembership::Possible => {
                possibly_violated.insert(check.id.clone());
                // Distinction loss only when widening itself introduced this id into
                // the may-set; otherwise `Possible` is an ordinary over-approximation.
                let widening_added = widened_ids_by_node
                    .get(&check.node_id)
                    .is_some_and(|ids| ids.contains(&check.concrete_id));
                if widening_added {
                    push_unique_obstruction(
                        &mut solver.obstructions,
                        FixpointObstruction {
                            obstruction_type:
                                FixpointObstructionType::WideningLostRequiredDistinction,
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
            }
            AbstractMembership::UnknownRegion | AbstractMembership::Unknown => {
                possibly_violated.insert(check.id.clone());
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
            AbstractMembership::Excluded => {
                // Sound exclusion: neither satisfied nor a possible violation.
            }
            AbstractMembership::Unsound => {
                possibly_violated.insert(check.id.clone());
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
        }
    }

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

fn push_unique_obstruction(
    obstructions: &mut Vec<FixpointObstruction>,
    obstruction: FixpointObstruction,
) {
    if !obstructions.contains(&obstruction) {
        obstructions.push(obstruction);
    }
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn ids<const N: usize>(values: [&str; N]) -> BTreeSet<Id> {
        values.into_iter().map(id).collect()
    }

    /// Acyclic chain a -> b -> c with gen sets; fixpoint converges to exact per-node states.
    #[test]
    fn acyclic_graph_converges_to_expected_states() {
        let graph = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([
            AbstractGraphNode::new(
                id("node/a"),
                [AbstractEdge::new(id("node/b")).with_gen_definite([id("cell/b")])],
            ),
            AbstractGraphNode::new(
                id("node/b"),
                [AbstractEdge::new(id("node/c")).with_gen_possible([id("cell/c")])],
            ),
            AbstractGraphNode::new(id("node/c"), []),
        ])
        .with_seeds([NodeSeed::exact(id("node/a"), [id("cell/a")])]);

        let checks = [
            MembershipCheck::new(id("check/a-in-c"), id("node/c"), id("cell/a")),
            MembershipCheck::new(id("check/b-in-c"), id("node/c"), id("cell/b")),
            MembershipCheck::new(id("check/c-in-c"), id("node/c"), id("cell/c")),
            MembershipCheck::new(id("check/missing-in-c"), id("node/c"), id("cell/zzz")),
        ];

        let report = run_fixpoint(
            id("analysis/acyclic"),
            &graph,
            &FixpointOptions::new(),
            &checks,
        )
        .expect("fixpoint runs");

        assert!(report.reached_fixpoint);
        assert!(report.is_conclusive());
        assert!(report.widening_events.is_empty());

        // node/a is the exact seed.
        let state_a = report.state(&id("node/a")).expect("state a");
        assert_eq!(state_a.definite_concrete_ids, ids(["cell/a"]));
        assert_eq!(state_a.possible_concrete_ids, ids(["cell/a"]));

        // node/b: a's exact state transferred with gen_definite cell/b.
        let state_b = report.state(&id("node/b")).expect("state b");
        assert_eq!(state_b.definite_concrete_ids, ids(["cell/a", "cell/b"]));
        assert_eq!(state_b.possible_concrete_ids, ids(["cell/a", "cell/b"]));

        // node/c: b transferred with gen_possible cell/c (only the may-set grows).
        let state_c = report.state(&id("node/c")).expect("state c");
        assert_eq!(state_c.definite_concrete_ids, ids(["cell/a", "cell/b"]));
        assert_eq!(
            state_c.possible_concrete_ids,
            ids(["cell/a", "cell/b", "cell/c"])
        );

        // Definite vs possible classification.
        assert_eq!(
            report.definitely_satisfied_check_ids,
            vec![id("check/a-in-c"), id("check/b-in-c")]
        );
        assert_eq!(report.possibly_violated_check_ids, vec![id("check/c-in-c")]);
        // cell/zzz is soundly excluded (sound domain): neither satisfied nor possibly violated.
        assert!(!report
            .possibly_violated_check_ids
            .contains(&id("check/missing-in-c")));
        assert_eq!(
            report
                .state(&id("node/c"))
                .unwrap()
                .classify(&id("cell/zzz")),
            AbstractMembership::Excluded
        );
    }

    /// Diamond a -> {b, c} -> d; join at d must intersect definites and union possibles.
    #[test]
    fn merge_node_joins_via_abstract_join() {
        let graph = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([
            AbstractGraphNode::new(
                id("node/a"),
                [
                    AbstractEdge::new(id("node/b")).with_gen_definite([id("cell/b")]),
                    AbstractEdge::new(id("node/c")).with_gen_definite([id("cell/c")]),
                ],
            ),
            AbstractGraphNode::new(id("node/b"), [AbstractEdge::new(id("node/d"))]),
            AbstractGraphNode::new(id("node/c"), [AbstractEdge::new(id("node/d"))]),
            AbstractGraphNode::new(id("node/d"), []),
        ])
        .with_seeds([NodeSeed::exact(id("node/a"), [id("cell/a")])]);

        let report = run_fixpoint(id("analysis/diamond"), &graph, &FixpointOptions::new(), &[])
            .expect("fixpoint runs");

        assert!(report.reached_fixpoint);
        let state_d = report.state(&id("node/d")).expect("state d");
        // Definite: intersection of {a,b} and {a,c} = {a}.
        assert_eq!(state_d.definite_concrete_ids, ids(["cell/a"]));
        // Possible: union = {a,b,c}.
        assert_eq!(
            state_d.possible_concrete_ids,
            ids(["cell/a", "cell/b", "cell/c"])
        );
    }

    /// Self-loop that keeps adding to the may-set; only terminates via widening.
    #[test]
    fn cyclic_graph_terminates_via_widening() {
        // node/loop has an edge to itself that introduces a fresh possible id each pass.
        // Without widening the may-set would keep changing relative to a growing top;
        // here gen_possible re-adds the same id, but widening is still exercised by the
        // revisit threshold and the configured top universe.
        let graph = AbstractGraph::new(AbstractDomain::RegionMembership, SoundnessStatus::Sound)
            .with_nodes([AbstractGraphNode::new(
                id("node/loop"),
                [AbstractEdge::new(id("node/loop")).with_gen_possible([id("cell/x")])],
            )])
            .with_seeds([NodeSeed::possible(id("node/loop"), [id("cell/seed")])]);

        let options = FixpointOptions::new()
            .with_widen_threshold(1)
            .with_widen_top([id("cell/top1"), id("cell/top2")])
            .with_max_iterations(50);

        let report =
            run_fixpoint(id("analysis/cyclic"), &graph, &options, &[]).expect("fixpoint runs");

        // It terminated within the bound (did not hit iteration_limit_exceeded).
        assert!(report.reached_fixpoint, "must converge via widening");
        assert!(
            report
                .obstructions
                .iter()
                .all(|o| o.obstruction_type != FixpointObstructionType::IterationLimitExceeded),
            "must not exhaust the iteration bound"
        );

        // A widening event was recorded for node/loop.
        assert!(!report.widening_events.is_empty(), "widening must occur");
        let event = report
            .widening_events
            .iter()
            .find(|e| e.node_id == id("node/loop"))
            .expect("widening event for loop node");
        assert!(
            event.widened_possible_ids.contains(&id("cell/top1"))
                || event.widened_possible_ids.contains(&id("cell/top2")),
            "widening relaxed toward the configured top"
        );

        // Final may-set includes the seed, the looped gen id, and the top universe.
        let state = report.state(&id("node/loop")).expect("loop state");
        assert!(state.possible_concrete_ids.contains(&id("cell/seed")));
        assert!(state.possible_concrete_ids.contains(&id("cell/x")));
        assert!(state.possible_concrete_ids.contains(&id("cell/top1")));
        assert!(state.possible_concrete_ids.contains(&id("cell/top2")));

        // Information loss is declared.
        assert!(report
            .information_loss
            .iter()
            .any(|line| line.contains("widened toward top")));
    }

    /// Widening that converts an exclusion into a possible membership is an obstruction.
    #[test]
    fn widening_lost_required_distinction_obstruction() {
        // A self-loop grows the may-set and forces revisits; widening then relaxes the
        // may-set toward `ctx/secret`, an id that is otherwise excluded. A check on
        // `ctx/secret` flips from Excluded to Possible solely because of widening, which
        // is exactly a lost required distinction.
        let graph = AbstractGraph::new(AbstractDomain::ContextMembership, SoundnessStatus::Sound)
            .with_nodes([
                AbstractGraphNode::new(id("node/a"), [AbstractEdge::new(id("node/m"))]),
                AbstractGraphNode::new(
                    id("node/m"),
                    [AbstractEdge::new(id("node/m")).with_gen_possible([id("ctx/grow")])],
                ),
            ])
            .with_seeds([NodeSeed::exact(id("node/a"), [id("ctx/a")])]);

        let options = FixpointOptions::new()
            .with_widen_threshold(1)
            .with_widen_top([id("ctx/secret")]);
        let check = MembershipCheck::new(id("check/secret-at-m"), id("node/m"), id("ctx/secret"));

        let report = run_fixpoint(id("analysis/widen-loss"), &graph, &options, &[check])
            .expect("fixpoint runs");

        assert!(report.reached_fixpoint);
        let event = report
            .widening_events
            .iter()
            .find(|e| e.node_id == id("node/m"))
            .expect("widening event for node/m");
        assert!(event.widened_possible_ids.contains(&id("ctx/secret")));
        // The check is now only possible (not excluded) at a widened node.
        assert!(report
            .possibly_violated_check_ids
            .contains(&id("check/secret-at-m")));
        assert!(report.obstructions.iter().any(
            |o| o.obstruction_type == FixpointObstructionType::WideningLostRequiredDistinction
        ));
    }

    /// An unsound seed produces an unsound_abstraction obstruction and never excludes.
    #[test]
    fn unsound_seed_blocks_absence_proofs() {
        let graph = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Unsound,
        )
        .with_nodes([AbstractGraphNode::new(id("node/u"), [])])
        .with_seeds([NodeSeed::possible(id("node/u"), [id("cell/a")])]);

        let check = MembershipCheck::new(id("check/missing"), id("node/u"), id("cell/absent"));

        let report = run_fixpoint(
            id("analysis/unsound"),
            &graph,
            &FixpointOptions::new(),
            &[check],
        )
        .expect("fixpoint runs");

        assert!(report
            .obstructions
            .iter()
            .any(|o| o.obstruction_type == FixpointObstructionType::UnsoundAbstraction));
        // Absence is NOT excluded under an unsound abstraction; it is possibly violated.
        assert!(report
            .possibly_violated_check_ids
            .contains(&id("check/missing")));
        assert!(report.unknown_region_node_ids.contains(&id("node/u")));
    }

    /// An Unknown-soundness state turns a missing id into unknown_region_requires_witness.
    #[test]
    fn unknown_soundness_requires_witness() {
        let graph = AbstractGraph::new(AbstractDomain::ContextMembership, SoundnessStatus::Unknown)
            .with_nodes([AbstractGraphNode::new(id("node/k"), [])])
            .with_seeds([NodeSeed::possible(id("node/k"), [id("ctx/a")])]);

        let check = MembershipCheck::new(id("check/k"), id("node/k"), id("ctx/missing"));

        let report = run_fixpoint(
            id("analysis/unknown"),
            &graph,
            &FixpointOptions::new(),
            &[check],
        )
        .expect("fixpoint runs");

        assert!(report.possibly_violated_check_ids.contains(&id("check/k")));
        assert!(report
            .obstructions
            .iter()
            .any(|o| o.obstruction_type == FixpointObstructionType::UnknownRegionRequiresWitness));
    }

    /// Hitting the iteration bound reports iteration_limit_exceeded and incompleteness.
    #[test]
    fn iteration_limit_is_reported() {
        // A self-loop graph with a zero iteration budget must report the bound on the
        // first pop rather than iterate. `max_iterations == 0` makes the guard fire
        // immediately and deterministically.
        let graph = AbstractGraph::new(AbstractDomain::RegionMembership, SoundnessStatus::Sound)
            .with_nodes([AbstractGraphNode::new(
                id("node/loop"),
                [AbstractEdge::new(id("node/loop")).with_gen_possible([id("cell/x")])],
            )])
            .with_seeds([NodeSeed::possible(id("node/loop"), [id("cell/seed")])]);

        let options = FixpointOptions::new()
            .with_widen_threshold(usize::MAX)
            .with_max_iterations(0);

        let report =
            run_fixpoint(id("analysis/limit"), &graph, &options, &[]).expect("fixpoint runs");

        assert!(!report.reached_fixpoint);
        assert!(report
            .obstructions
            .iter()
            .any(|o| o.obstruction_type == FixpointObstructionType::IterationLimitExceeded));
        assert!(report
            .information_loss
            .iter()
            .any(|line| line.contains("iteration bound")));
    }

    /// Different domains at a merge node produce missing_join_operation.
    ///
    /// The public `AbstractGraph` enforces a single domain, so this exercises the
    /// internal guard directly to prove the obstruction is wired.
    #[test]
    fn different_domains_report_missing_join_operation() {
        let mut solver = Solver {
            states: BTreeMap::new(),
            revisits: BTreeMap::new(),
            widening_events: Vec::new(),
            obstructions: Vec::new(),
            information_loss: BTreeSet::new(),
        };
        let mut worklist: BTreeSet<Id> = BTreeSet::new();
        solver.states.insert(
            id("node/m"),
            AbstractElement::new(
                id("node/m"),
                AbstractDomain::DependencyReachability,
                [id("cell/a")],
                SoundnessStatus::Sound,
            ),
        );
        let incoming = AbstractElement::new(
            id("node/m"),
            AbstractDomain::ContextMembership,
            [id("ctx/a")],
            SoundnessStatus::Sound,
        );

        propagate(
            &mut solver,
            &mut worklist,
            id("node/m"),
            incoming,
            &FixpointOptions::new(),
            1,
        )
        .expect("propagate handles domain mismatch");

        assert!(solver
            .obstructions
            .iter()
            .any(|o| o.obstruction_type == FixpointObstructionType::MissingJoinOperation));
    }

    /// Unknown edge targets and seed nodes are rejected at normalization.
    #[test]
    fn malformed_graph_is_rejected() {
        let bad_edge = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([AbstractGraphNode::new(
            id("node/a"),
            [AbstractEdge::new(id("node/missing"))],
        )]);
        assert_eq!(
            run_fixpoint(id("analysis/bad"), &bad_edge, &FixpointOptions::new(), &[])
                .expect_err("unknown edge target")
                .code(),
            "malformed_field"
        );

        let bad_seed = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([AbstractGraphNode::new(id("node/a"), [])])
        .with_seeds([NodeSeed::possible(id("node/missing"), [id("cell/a")])]);
        assert_eq!(
            run_fixpoint(id("analysis/bad2"), &bad_seed, &FixpointOptions::new(), &[])
                .expect_err("unknown seed node")
                .code(),
            "malformed_field"
        );

        let bad_subset = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([AbstractGraphNode::new(id("node/a"), [])])
        .with_seeds([NodeSeed::possible(id("node/a"), [id("cell/a")])
            .with_definite_concrete_ids([id("cell/not-possible")])]);
        assert_eq!(
            run_fixpoint(
                id("analysis/bad3"),
                &bad_subset,
                &FixpointOptions::new(),
                &[]
            )
            .expect_err("definite outside possible")
            .code(),
            "malformed_field"
        );
    }

    /// Equal inputs produce byte-identical serialized reports (determinism).
    #[test]
    fn fixpoint_is_deterministic() {
        let graph = AbstractGraph::new(
            AbstractDomain::DependencyReachability,
            SoundnessStatus::Sound,
        )
        .with_nodes([
            AbstractGraphNode::new(
                id("node/a"),
                [
                    AbstractEdge::new(id("node/c")).with_gen_definite([id("cell/c")]),
                    AbstractEdge::new(id("node/b")).with_gen_definite([id("cell/b")]),
                ],
            ),
            AbstractGraphNode::new(id("node/b"), [AbstractEdge::new(id("node/d"))]),
            AbstractGraphNode::new(id("node/c"), [AbstractEdge::new(id("node/d"))]),
            AbstractGraphNode::new(id("node/d"), []),
        ])
        .with_seeds([NodeSeed::exact(id("node/a"), [id("cell/a")])]);
        // Two checks on the definite id cell/a, supplied out of id order, must come back
        // sorted; one check on cell/b (only-possible after the diamond join) is separated.
        let checks = [
            MembershipCheck::new(id("check/2"), id("node/d"), id("cell/a")),
            MembershipCheck::new(id("check/3"), id("node/d"), id("cell/b")),
            MembershipCheck::new(id("check/1"), id("node/d"), id("cell/a")),
        ];

        let first = run_fixpoint(id("analysis/det"), &graph, &FixpointOptions::new(), &checks)
            .expect("first run");
        let second = run_fixpoint(id("analysis/det"), &graph, &FixpointOptions::new(), &checks)
            .expect("second run");

        let first_json = serde_json::to_string(&first).expect("serialize first");
        let second_json = serde_json::to_string(&second).expect("serialize second");
        assert_eq!(first_json, second_json);
        // Output id lists are sorted regardless of input order.
        assert_eq!(
            first.definitely_satisfied_check_ids,
            vec![id("check/1"), id("check/2")]
        );
        assert_eq!(first.possibly_violated_check_ids, vec![id("check/3")]);
        // node_states are sorted by node id.
        let node_order: Vec<Id> = first
            .node_states
            .iter()
            .map(|entry| entry.node_id.clone())
            .collect();
        assert_eq!(
            node_order,
            vec![id("node/a"), id("node/b"), id("node/c"), id("node/d")]
        );
    }

    /// The report round-trips through JSON without loss.
    #[test]
    fn report_json_round_trip() {
        let graph = AbstractGraph::new(AbstractDomain::RegionMembership, SoundnessStatus::Sound)
            .with_nodes([AbstractGraphNode::new(
                id("node/loop"),
                [AbstractEdge::new(id("node/loop")).with_gen_possible([id("cell/x")])],
            )])
            .with_seeds([NodeSeed::possible(id("node/loop"), [id("cell/seed")])]);
        let options = FixpointOptions::new()
            .with_widen_threshold(1)
            .with_widen_top([id("cell/top")]);
        let check = MembershipCheck::new(id("check/x"), id("node/loop"), id("cell/x"));

        let report =
            run_fixpoint(id("analysis/rt"), &graph, &options, &[check]).expect("fixpoint runs");

        let json = serde_json::to_string(&report).expect("serialize report");
        let decoded: AbstractInterpretationReport =
            serde_json::from_str(&json).expect("deserialize report");
        assert_eq!(decoded, report);

        // Inputs also round-trip.
        let graph_json = serde_json::to_string(&graph).expect("serialize graph");
        let decoded_graph: AbstractGraph =
            serde_json::from_str(&graph_json).expect("deserialize graph");
        assert_eq!(decoded_graph, graph);

        let options_json = serde_json::to_string(&options).expect("serialize options");
        let decoded_options: FixpointOptions =
            serde_json::from_str(&options_json).expect("deserialize options");
        assert_eq!(decoded_options, options);
    }
}
