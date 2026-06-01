use super::solver::{propagate, Solver};
use super::*;
use higher_graphen_core::Id;
use std::collections::{BTreeMap, BTreeSet};

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

    let report = run_fixpoint(id("analysis/cyclic"), &graph, &options, &[]).expect("fixpoint runs");

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

    let report =
        run_fixpoint(id("analysis/widen-loss"), &graph, &options, &[check]).expect("fixpoint runs");

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
    assert!(report
        .obstructions
        .iter()
        .any(|o| o.obstruction_type == FixpointObstructionType::WideningLostRequiredDistinction));
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

    let report = run_fixpoint(id("analysis/limit"), &graph, &options, &[]).expect("fixpoint runs");

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
