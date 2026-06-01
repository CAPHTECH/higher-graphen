use super::*;
use crate::morphism::{
    construct_explicit_pushout, Morphism, MorphismType, PushoutConstruction, PushoutInputs,
    PushoutObstructionType, PushoutOutcome,
};
use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};
use std::collections::BTreeMap;

#[test]
fn construct_pushout_matches_direct_explicit_construction() {
    let (store, left, right) = clean_pushout_store();
    let candidate_space_id = id("space/pushout");
    let store_outcome = store
        .construct_pushout(
            &left,
            &right,
            candidate_space_id.clone(),
            "Pushout".to_owned(),
            ComplexType::CellComplex,
        )
        .expect("store pushout should construct");

    let left_cells = gathered_cells(&store, &left.target_space_id);
    let right_cells = gathered_cells(&store, &right.target_space_id);
    let left_incidences = gathered_incidences(&store, &left.target_space_id);
    let right_incidences = gathered_incidences(&store, &right.target_space_id);
    let direct_outcome = construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id,
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &left_incidences,
        right_incidences: &right_incidences,
    });

    let store_construction = constructed(&store_outcome);
    let direct_construction = constructed(&direct_outcome);
    assert_eq!(store_construction.cells, direct_construction.cells);
    assert_eq!(
        store_construction.incidences,
        direct_construction.incidences
    );
    assert_eq!(
        store_construction.complex.max_dimension,
        direct_construction.complex.max_dimension
    );
    assert_eq!(
        store_construction.space.cell_ids,
        direct_construction.space.cell_ids
    );
    assert_eq!(
        store_construction.space.incidence_ids,
        direct_construction.space.incidence_ids
    );
    assert_eq!(
        store_construction.space.complex_ids,
        direct_construction.space.complex_ids
    );
    assert_eq!(
        store_construction.space.context_ids,
        direct_construction.space.context_ids
    );
}

#[test]
fn construct_pushout_errors_when_target_space_is_missing() {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space/source"), "Source"))
        .expect("insert source space");
    store
        .insert_space(Space::new(id("space/right"), "Right"))
        .expect("insert right space");
    let left = fixture_morphism("left", "space/source", "space/missing-left", [], []);
    let right = fixture_morphism("right", "space/source", "space/right", [], []);

    let error = store
        .construct_pushout(
            &left,
            &right,
            id("space/pushout"),
            "Pushout".to_owned(),
            ComplexType::CellComplex,
        )
        .expect_err("missing target space should fail");

    assert!(matches!(
        error,
        CoreError::MalformedField { ref field, ref reason }
            if field == "left" && reason.contains("space/missing-left")
    ));
}

#[test]
fn construct_pushout_passes_blocked_outcome_through() {
    let mut store = InMemorySpaceStore::new();
    for (space_id, name) in [
        ("space/source-left", "Source left"),
        ("space/source-right", "Source right"),
        ("space/left", "Left"),
        ("space/right", "Right"),
    ] {
        store
            .insert_space(Space::new(id(space_id), name))
            .expect("insert space");
    }
    let left = fixture_morphism("left", "space/source-left", "space/left", [], []);
    let right = fixture_morphism("right", "space/source-right", "space/right", [], []);

    let outcome = store
        .construct_pushout(
            &left,
            &right,
            id("space/pushout"),
            "Pushout".to_owned(),
            ComplexType::CellComplex,
        )
        .expect("blocked pushout should be returned as outcome");

    let PushoutOutcome::Blocked { report } = outcome else {
        panic!("expected blocked pushout outcome");
    };
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == PushoutObstructionType::IncompatibleSourceSpace
    }));
}

#[test]
fn construct_pushout_is_deterministic_across_insertion_orders() {
    let (ordered, left, right) = deterministic_store(false);
    let (reversed, _, _) = deterministic_store(true);

    let ordered_json = serde_json::to_string(
        &ordered
            .construct_pushout(
                &left,
                &right,
                id("space/pushout"),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect("ordered store pushout"),
    )
    .expect("serialize ordered outcome");
    let reversed_json = serde_json::to_string(
        &reversed
            .construct_pushout(
                &left,
                &right,
                id("space/pushout"),
                "Pushout".to_owned(),
                ComplexType::CellComplex,
            )
            .expect("reversed store pushout"),
    )
    .expect("serialize reversed outcome");

    assert_eq!(ordered_json, reversed_json);
}

#[test]
fn construct_pushout_leaves_store_unchanged() {
    let (store, left, right) = clean_pushout_store();
    let counts = (
        store.cells.len(),
        store.incidences.len(),
        store.spaces.len(),
        store.complexes.len(),
    );

    let outcome = store
        .construct_pushout(
            &left,
            &right,
            id("space/pushout"),
            "Pushout".to_owned(),
            ComplexType::CellComplex,
        )
        .expect("store pushout should construct");

    assert!(matches!(outcome, PushoutOutcome::Constructed { .. }));
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

fn clean_pushout_store() -> (InMemorySpaceStore, Morphism, Morphism) {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [
            ("cell/source-a", "cell/left-a"),
            ("cell/source-b", "cell/left-b"),
        ],
        [("rel/source-a", "rel/left-a")],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [
            ("cell/source-a", "cell/right-a"),
            ("cell/source-b", "cell/right-b"),
        ],
        [("rel/source-a", "rel/right-a")],
    );
    let mut store = base_spaces();
    insert_cell(
        &mut store,
        Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex").with_label("shared-a"),
    );
    insert_cell(
        &mut store,
        Cell::new(id("cell/left-b"), id("space/left"), 0, "vertex").with_context(id("ctx/b")),
    );
    insert_cell(
        &mut store,
        Cell::new(id("cell/left-private"), id("space/left"), 1, "edge")
            .with_boundary_cell(id("cell/left-a")),
    );
    insert_cell(
        &mut store,
        Cell::new(id("cell/right-b"), id("space/right"), 0, "vertex").with_context(id("ctx/b")),
    );
    insert_cell(
        &mut store,
        Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex").with_label("shared-a"),
    );
    insert_incidence(
        &mut store,
        Incidence::new(
            id("rel/left-a"),
            id("space/left"),
            id("cell/left-a"),
            id("cell/left-b"),
            "attaches",
            IncidenceOrientation::Directed,
        ),
    );
    insert_incidence(
        &mut store,
        Incidence::new(
            id("rel/right-a"),
            id("space/right"),
            id("cell/right-a"),
            id("cell/right-b"),
            "attaches",
            IncidenceOrientation::Directed,
        ),
    );

    (store, left, right)
}

fn deterministic_store(reverse: bool) -> (InMemorySpaceStore, Morphism, Morphism) {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [("cell/source-a", "cell/left-a")],
        [("rel/source-a", "rel/left-a")],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [("cell/source-a", "cell/right-a")],
        [("rel/source-a", "rel/right-a")],
    );
    let mut store = base_spaces();
    let left_cells = [
        Cell::new(id("cell/left-z"), id("space/left"), 0, "vertex"),
        Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex"),
    ];
    let right_cells = [
        Cell::new(id("cell/right-z"), id("space/right"), 0, "vertex"),
        Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex"),
    ];
    if reverse {
        for cell in left_cells
            .into_iter()
            .rev()
            .chain(right_cells.into_iter().rev())
        {
            insert_cell(&mut store, cell);
        }
    } else {
        for cell in left_cells.into_iter().chain(right_cells) {
            insert_cell(&mut store, cell);
        }
    }

    let left_incidence = Incidence::new(
        id("rel/left-a"),
        id("space/left"),
        id("cell/left-a"),
        id("cell/left-z"),
        "attaches",
        IncidenceOrientation::Directed,
    )
    .with_weight(2.0);
    let right_incidence = Incidence::new(
        id("rel/right-a"),
        id("space/right"),
        id("cell/right-a"),
        id("cell/right-z"),
        "attaches",
        IncidenceOrientation::Directed,
    )
    .with_weight(2.0);
    if reverse {
        insert_incidence(&mut store, right_incidence);
        insert_incidence(&mut store, left_incidence);
    } else {
        insert_incidence(&mut store, left_incidence);
        insert_incidence(&mut store, right_incidence);
    }

    (store, left, right)
}

fn base_spaces() -> InMemorySpaceStore {
    let mut store = InMemorySpaceStore::new();
    for (space_id, name) in [
        ("space/source", "Source"),
        ("space/left", "Left"),
        ("space/right", "Right"),
    ] {
        store
            .insert_space(Space::new(id(space_id), name))
            .expect("insert space");
    }
    store
}

fn fixture_morphism<const C: usize, const R: usize>(
    morphism_id: &str,
    source_space_id: &str,
    target_space_id: &str,
    cell_pairs: [(&str, &str); C],
    relation_pairs: [(&str, &str); R],
) -> Morphism {
    Morphism {
        id: id(morphism_id),
        source_space_id: id(source_space_id),
        target_space_id: id(target_space_id),
        name: morphism_id.to_owned(),
        morphism_type: MorphismType::Translation,
        cell_mapping: mapping(cell_pairs),
        relation_mapping: mapping(relation_pairs),
        preserved_invariant_ids: Vec::new(),
        lost_structure: Vec::new(),
        distortion: Vec::new(),
        composable_with: Vec::new(),
        provenance: provenance(),
    }
}

fn mapping<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<Id, Id> {
    pairs
        .into_iter()
        .map(|(source_id, target_id)| (id(source_id), id(target_id)))
        .collect()
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

fn constructed(outcome: &PushoutOutcome) -> &PushoutConstruction {
    let PushoutOutcome::Constructed { construction, .. } = outcome else {
        panic!("expected constructed pushout");
    };
    construction
}

fn insert_cell(store: &mut InMemorySpaceStore, cell: Cell) {
    store.insert_cell(cell).expect("insert cell");
}

fn insert_incidence(store: &mut InMemorySpaceStore, incidence: Incidence) {
    store.insert_incidence(incidence).expect("insert incidence");
}

fn provenance() -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::custom("pushout-store-test").expect("valid source kind")),
        Confidence::ONE,
    )
    .with_review_status(ReviewStatus::Accepted)
}

fn id(value: impl AsRef<str>) -> Id {
    Id::new(value.as_ref()).expect("valid id")
}
