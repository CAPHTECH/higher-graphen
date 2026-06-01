use super::*;
use crate::space::{Cell, ComplexType, InMemorySpaceStore, Incidence, IncidenceOrientation, Space};
use higher_graphen_core::CoreError;

#[test]
fn explicit_pullback_constructs_clean_fiber_product() {
    let (left, right, left_cells, right_cells, left_incidences, right_incidences) =
        clean_pullback_inputs();

    let PullbackOutcome::Constructed {
        construction,
        report,
    } = construct_explicit_pullback(PullbackInputs {
        left,
        right,
        candidate_space_id: id("space/pullback"),
        candidate_space_name: "Pullback".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_source_cells: left_cells,
        right_source_cells: right_cells,
        left_source_incidences: left_incidences,
        right_source_incidences: right_incidences,
    })
    else {
        panic!("clean pullback should construct");
    };

    let a1_b1 = id("space/pullback/pullback/cell/cell/left-a1+cell/right-b1");
    let a1_b2 = id("space/pullback/pullback/cell/cell/left-a1+cell/right-b2");
    let a2_b1 = id("space/pullback/pullback/cell/cell/left-a2+cell/right-b1");
    let a2_b2 = id("space/pullback/pullback/cell/cell/left-a2+cell/right-b2");
    let relation_id = id("space/pullback/pullback/incidence/rel/left-link+rel/right-link");

    assert_eq!(
        construction
            .cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect::<Vec<_>>(),
        vec![a1_b1.clone(), a1_b2, a2_b1, a2_b2.clone()]
    );
    assert_eq!(construction.cells.len(), 4);
    assert!(construction
        .cells
        .iter()
        .all(|cell| cell.space_id == id("space/pullback")));
    assert_eq!(construction.incidences.len(), 1);
    assert_eq!(construction.incidences[0].id, relation_id);
    assert_eq!(construction.incidences[0].from_cell_id, a1_b1.clone());
    assert_eq!(construction.incidences[0].to_cell_id, a2_b2.clone());
    assert_eq!(construction.complex.max_dimension, 0);
    assert_eq!(
        construction.space.cell_ids,
        construction
            .cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(construction.space.incidence_ids, vec![relation_id]);
    assert_eq!(
        construction.space.complex_ids,
        vec![id("space/pullback/pullback/complex")]
    );
    assert_eq!(construction.cell_matches, report.cell_matches);
    assert_eq!(construction.relation_matches, report.relation_matches);
    assert!(report.obstructions.is_empty());
}

#[test]
fn explicit_pullback_blocks_incompatible_fiber_cell_dimension() {
    let left = fixture_morphism(
        "left",
        "space/left",
        "space/target",
        [("cell/left-a", "cell/shared")],
        [],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/right",
        "space/target",
        [("cell/right-a", "cell/shared")],
        [],
        [],
    );

    let report = blocked_report(construct_explicit_pullback(PullbackInputs {
        left,
        right,
        candidate_space_id: id("space/pullback"),
        candidate_space_name: "Pullback".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_source_cells: vec![Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex")],
        right_source_cells: vec![Cell::new(id("cell/right-a"), id("space/right"), 1, "edge")],
        left_source_incidences: Vec::new(),
        right_source_incidences: Vec::new(),
    }));

    assert_obstruction(&report, PullbackObstructionType::IncompatibleFiber);
}

#[test]
fn explicit_pullback_blocks_incompatible_target_space() {
    let left = fixture_morphism("left", "space/left", "space/target-a", [], [], []);
    let right = fixture_morphism("right", "space/right", "space/target-b", [], [], []);

    let report = blocked_report(empty_pullback_outcome(left, right));

    assert_obstruction(&report, PullbackObstructionType::IncompatibleTargetSpace);
}

#[test]
fn explicit_pullback_construction_is_deterministic() {
    let first = serde_json::to_string(&deterministic_outcome()).expect("serialize first outcome");
    let second = serde_json::to_string(&deterministic_outcome()).expect("serialize second outcome");

    assert_eq!(first, second);
}

#[test]
fn explicit_pullback_outcome_round_trips_json() {
    let outcome = deterministic_outcome();
    let encoded = serde_json::to_string(&outcome).expect("serialize outcome");
    let decoded: PullbackOutcome = serde_json::from_str(&encoded).expect("deserialize outcome");

    assert_eq!(decoded, outcome);
}

#[test]
fn store_construct_pullback_matches_direct_explicit_construction() {
    let (store, left, right) = clean_pullback_store();
    let candidate_space_id = id("space/pullback");

    let store_outcome = store
        .construct_pullback(
            &left,
            &right,
            candidate_space_id.clone(),
            "Pullback".to_owned(),
            ComplexType::CellComplex,
        )
        .expect("store pullback should construct");
    let direct_outcome = construct_explicit_pullback(PullbackInputs {
        left: left.clone(),
        right: right.clone(),
        candidate_space_id,
        candidate_space_name: "Pullback".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_source_cells: gathered_cells(&store, &left.source_space_id),
        right_source_cells: gathered_cells(&store, &right.source_space_id),
        left_source_incidences: gathered_incidences(&store, &left.source_space_id),
        right_source_incidences: gathered_incidences(&store, &right.source_space_id),
    });

    assert_eq!(store_outcome, direct_outcome);
}

#[test]
fn store_construct_pullback_errors_when_source_space_is_missing() {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space/right"), "Right"))
        .expect("insert right space");
    let left = fixture_morphism("left", "space/missing-left", "space/target", [], [], []);
    let right = fixture_morphism("right", "space/right", "space/target", [], [], []);

    let error = store
        .construct_pullback(
            &left,
            &right,
            id("space/pullback"),
            "Pullback".to_owned(),
            ComplexType::CellComplex,
        )
        .expect_err("missing source space should fail");

    assert!(matches!(
        error,
        CoreError::MalformedField { ref field, ref reason }
            if field == "left" && reason.contains("space/missing-left")
    ));
}

#[test]
fn store_construct_pullback_leaves_store_unchanged() {
    let (store, left, right) = clean_pullback_store();
    let counts = (
        store.cells.len(),
        store.incidences.len(),
        store.spaces.len(),
        store.complexes.len(),
    );

    let outcome = store
        .construct_pullback(
            &left,
            &right,
            id("space/pullback"),
            "Pullback".to_owned(),
            ComplexType::CellComplex,
        )
        .expect("store pullback should construct");

    assert!(matches!(outcome, PullbackOutcome::Constructed { .. }));
    assert_eq!(
        counts,
        (
            store.cells.len(),
            store.incidences.len(),
            store.spaces.len(),
            store.complexes.len(),
        )
    );
}

fn deterministic_outcome() -> PullbackOutcome {
    let (left, right, left_cells, right_cells, left_incidences, right_incidences) =
        clean_pullback_inputs();
    construct_explicit_pullback(PullbackInputs {
        left,
        right,
        candidate_space_id: id("space/pullback"),
        candidate_space_name: "Pullback".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_source_cells: left_cells,
        right_source_cells: right_cells,
        left_source_incidences: left_incidences,
        right_source_incidences: right_incidences,
    })
}

fn clean_pullback_inputs() -> (
    Morphism,
    Morphism,
    Vec<Cell>,
    Vec<Cell>,
    Vec<Incidence>,
    Vec<Incidence>,
) {
    let left = fixture_morphism(
        "left",
        "space/left",
        "space/target",
        [
            ("cell/left-a1", "cell/target-shared"),
            ("cell/left-a2", "cell/target-shared"),
        ],
        [("rel/left-link", "rel/target-link")],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/right",
        "space/target",
        [
            ("cell/right-b1", "cell/target-shared"),
            ("cell/right-b2", "cell/target-shared"),
        ],
        [("rel/right-link", "rel/target-link")],
        [],
    );
    let left_cells = vec![
        Cell::new(id("cell/left-a1"), id("space/left"), 0, "vertex").with_label("left-a1"),
        Cell::new(id("cell/left-a2"), id("space/left"), 0, "vertex").with_context(id("ctx/a")),
    ];
    let right_cells = vec![
        Cell::new(id("cell/right-b1"), id("space/right"), 0, "vertex").with_label("right-b1"),
        Cell::new(id("cell/right-b2"), id("space/right"), 0, "vertex").with_context(id("ctx/b")),
    ];
    let left_incidences = vec![Incidence::new(
        id("rel/left-link"),
        id("space/left"),
        id("cell/left-a1"),
        id("cell/left-a2"),
        "links",
        IncidenceOrientation::Directed,
    )];
    let right_incidences = vec![Incidence::new(
        id("rel/right-link"),
        id("space/right"),
        id("cell/right-b1"),
        id("cell/right-b2"),
        "links",
        IncidenceOrientation::Directed,
    )];

    (
        left,
        right,
        left_cells,
        right_cells,
        left_incidences,
        right_incidences,
    )
}

fn clean_pullback_store() -> (InMemorySpaceStore, Morphism, Morphism) {
    let (left, right, left_cells, right_cells, left_incidences, right_incidences) =
        clean_pullback_inputs();
    let mut store = InMemorySpaceStore::new();
    for (space_id, name) in [("space/left", "Left"), ("space/right", "Right")] {
        store
            .insert_space(Space::new(id(space_id), name))
            .expect("insert space");
    }
    for cell in left_cells.into_iter().chain(right_cells) {
        store.insert_cell(cell).expect("insert cell");
    }
    for incidence in left_incidences.into_iter().chain(right_incidences) {
        store.insert_incidence(incidence).expect("insert incidence");
    }
    (store, left, right)
}

fn gathered_cells(store: &InMemorySpaceStore, space_id: &Id) -> Vec<Cell> {
    let mut cells = store
        .cells
        .values()
        .filter(|cell| &cell.space_id == space_id)
        .cloned()
        .collect::<Vec<_>>();
    cells.sort_by(|left, right| left.id.cmp(&right.id));
    cells
}

fn gathered_incidences(store: &InMemorySpaceStore, space_id: &Id) -> Vec<Incidence> {
    let mut incidences = store
        .incidences
        .values()
        .filter(|incidence| &incidence.space_id == space_id)
        .cloned()
        .collect::<Vec<_>>();
    incidences.sort_by(|left, right| left.id.cmp(&right.id));
    incidences
}

fn empty_pullback_outcome(left: Morphism, right: Morphism) -> PullbackOutcome {
    construct_explicit_pullback(PullbackInputs {
        left,
        right,
        candidate_space_id: id("space/pullback"),
        candidate_space_name: "Pullback".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_source_cells: Vec::new(),
        right_source_cells: Vec::new(),
        left_source_incidences: Vec::new(),
        right_source_incidences: Vec::new(),
    })
}

fn blocked_report(outcome: PullbackOutcome) -> ExplicitPullbackReport {
    let PullbackOutcome::Blocked { report } = outcome else {
        panic!("expected blocked pullback outcome");
    };
    report
}

fn assert_obstruction(report: &ExplicitPullbackReport, obstruction_type: PullbackObstructionType) {
    assert!(
        report
            .obstructions
            .iter()
            .any(|obstruction| obstruction.obstruction_type == obstruction_type),
        "expected obstruction {obstruction_type:?}, got {:?}",
        report.obstructions
    );
}
