use super::*;
use crate::space::{Cell, ComplexType, Space};

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn seeded_store() -> InMemorySpaceStore {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space-a"), "Topology test space"))
        .expect("insert space");
    store
}

fn triangle_1_skeleton_store() -> (InMemorySpaceStore, Id) {
    let mut store = seeded_store();
    insert_triangle_cells(&mut store);
    let complex = store
        .construct_complex(
            id("complex-triangle"),
            id("space-a"),
            "Triangle 1-skeleton",
            ComplexType::SimplicialComplex,
            [
                id("point-a"),
                id("point-b"),
                id("point-c"),
                id("edge-ab"),
                id("edge-bc"),
                id("edge-ca"),
            ],
            [],
        )
        .expect("construct complex");
    (store, complex.id)
}

fn filled_triangle_store() -> (InMemorySpaceStore, Id) {
    let mut store = seeded_store();
    insert_triangle_cells(&mut store);
    insert_triangle_face(&mut store);
    let complex = store
        .construct_complex(
            id("complex-filled-triangle"),
            id("space-a"),
            "Filled triangle",
            ComplexType::SimplicialComplex,
            [
                id("point-a"),
                id("point-b"),
                id("point-c"),
                id("edge-ab"),
                id("edge-bc"),
                id("edge-ca"),
                id("face-abc"),
            ],
            [],
        )
        .expect("construct filled triangle complex");
    (store, complex.id)
}

fn insert_triangle_cells(store: &mut InMemorySpaceStore) {
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert point a");
    store
        .insert_cell(Cell::new(id("point-b"), id("space-a"), 0, "point"))
        .expect("insert point b");
    store
        .insert_cell(Cell::new(id("point-c"), id("space-a"), 0, "point"))
        .expect("insert point c");
    store
        .insert_cell(
            Cell::new(id("edge-ab"), id("space-a"), 1, "edge")
                .with_boundary_cell(id("point-a"))
                .with_boundary_cell(id("point-b")),
        )
        .expect("insert edge ab");
    store
        .insert_cell(
            Cell::new(id("edge-bc"), id("space-a"), 1, "edge")
                .with_boundary_cell(id("point-b"))
                .with_boundary_cell(id("point-c")),
        )
        .expect("insert edge bc");
    store
        .insert_cell(
            Cell::new(id("edge-ca"), id("space-a"), 1, "edge")
                .with_boundary_cell(id("point-c"))
                .with_boundary_cell(id("point-a")),
        )
        .expect("insert edge ca");
}

fn insert_triangle_face(store: &mut InMemorySpaceStore) {
    store
        .insert_cell(
            Cell::new(id("face-abc"), id("space-a"), 2, "face")
                .with_boundary_cell(id("edge-ab"))
                .with_boundary_cell(id("edge-bc"))
                .with_boundary_cell(id("edge-ca")),
        )
        .expect("insert face abc");
}

fn triangle_filtration_stages() -> Vec<FiltrationStage> {
    vec![
        FiltrationStage::new(
            id("stage-vertices"),
            [id("point-a"), id("point-b"), id("point-c")],
        ),
        FiltrationStage::new(
            id("stage-edge-ab"),
            [id("point-a"), id("point-b"), id("point-c"), id("edge-ab")],
        ),
        FiltrationStage::new(
            id("stage-edge-bc"),
            [
                id("point-a"),
                id("point-b"),
                id("point-c"),
                id("edge-ab"),
                id("edge-bc"),
            ],
        ),
        FiltrationStage::new(
            id("stage-cycle"),
            [
                id("point-a"),
                id("point-b"),
                id("point-c"),
                id("edge-ab"),
                id("edge-bc"),
                id("edge-ca"),
            ],
        ),
    ]
}

fn filled_triangle_filtration_stages() -> Vec<FiltrationStage> {
    let mut stages = triangle_filtration_stages();
    stages.push(FiltrationStage::new(
        id("stage-face"),
        [
            id("point-a"),
            id("point-b"),
            id("point-c"),
            id("edge-ab"),
            id("edge-bc"),
            id("edge-ca"),
            id("face-abc"),
        ],
    ));
    stages
}

#[test]
fn summarizes_connected_components_and_simple_cycle_indicator() {
    let (store, complex_id) = triangle_1_skeleton_store();

    let summary = summarize_complex(&store, &complex_id).expect("summarize topology");

    assert_eq!(summary.vertex_count, 3);
    assert_eq!(summary.graph_edge_count, 3);
    assert_eq!(summary.component_count, 1);
    assert!(summary.is_connected());
    assert_eq!(summary.first_betti_number, 1);
    assert_eq!(summary.simple_hole_count, 1);
    assert_eq!(summary.homology.betti_number(0), 1);
    assert_eq!(summary.homology.betti_number(1), 1);
    assert_eq!(summary.homology.euler_characteristic, 0);
    assert!(summary.has_simple_cycle);
    assert_eq!(summary.simple_cycles.len(), 1);
    assert_eq!(summary.simple_cycles[0].witness_edge_id, id("edge-ca"));
    assert_eq!(
        summary.connected_components[0].vertex_cell_ids,
        vec![id("point-a"), id("point-b"), id("point-c")]
    );
    assert!(summary.findings.is_empty());

    let json = serde_json::to_string(&summary).expect("serialize summary");
    let roundtrip: TopologySummary = serde_json::from_str(&json).expect("deserialize summary");
    assert_eq!(roundtrip, summary);
}

#[test]
fn filled_triangle_homology_kills_simple_cycle() {
    let (store, complex_id) = filled_triangle_store();

    let summary = summarize_complex(&store, &complex_id).expect("summarize topology");

    assert_eq!(summary.vertex_count, 3);
    assert_eq!(summary.graph_edge_count, 3);
    assert!(summary.has_simple_cycle);
    assert_eq!(summary.simple_cycles.len(), 1);
    assert_eq!(summary.first_betti_number, 0);
    assert_eq!(summary.simple_hole_count, 0);
    assert_eq!(summary.homology.betti_number(0), 1);
    assert_eq!(summary.homology.betti_number(1), 0);
    assert_eq!(summary.homology.betti_number(2), 0);
    assert_eq!(summary.homology.euler_characteristic, 1);
    assert!(summary.findings.is_empty());
}

#[test]
fn reports_uncovered_boundary_and_invalid_chain_findings() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert point a");
    store
        .insert_cell(Cell::new(id("point-b"), id("space-a"), 0, "point"))
        .expect("insert point b");
    store
        .insert_cell(
            Cell::new(id("edge-ab"), id("space-a"), 1, "edge")
                .with_boundary_cell(id("point-a"))
                .with_boundary_cell(id("point-b")),
        )
        .expect("insert edge");
    store
        .insert_cell(
            Cell::new(id("face-ab"), id("space-a"), 2, "face").with_boundary_cell(id("edge-ab")),
        )
        .expect("insert face");
    let complex = store
        .construct_complex(
            id("complex-partial"),
            id("space-a"),
            "Partial complex",
            ComplexType::CellComplex,
            [id("point-a"), id("edge-ab"), id("face-ab")],
            [],
        )
        .expect("construct partial complex");

    let summary = summarize_complex(&store, &complex.id).expect("summarize topology");

    assert_eq!(summary.vertex_count, 1);
    assert_eq!(summary.graph_edge_count, 0);
    assert_eq!(summary.findings.len(), 2);
    assert!(summary.findings.iter().any(|finding| {
        finding.finding_type == TopologyFindingKind::ExternalBoundaryCell
            && finding.obstruction_type.as_deref() == Some(UNCOVERED_REGION_OBSTRUCTION_TYPE)
    }));
    assert!(summary.findings.iter().any(|finding| {
        finding.finding_type == TopologyFindingKind::BoundaryOperatorCompositionNonZero
    }));
}

#[test]
fn filtration_tracks_component_merges_and_hole_births() {
    let (store, complex_id) = triangle_1_skeleton_store();
    let stages = triangle_filtration_stages();

    let summary = summarize_filtration(&store, &complex_id, &stages).expect("summarize");

    let component_counts = summary
        .stages
        .iter()
        .map(|stage| stage.topology.component_count)
        .collect::<Vec<_>>();
    let hole_counts = summary
        .stages
        .iter()
        .map(|stage| stage.topology.simple_hole_count)
        .collect::<Vec<_>>();
    assert_eq!(component_counts, vec![3, 2, 1, 1]);
    assert_eq!(hole_counts, vec![0, 0, 0, 1]);
    assert_eq!(summary.open_component_count, 1);
    assert_eq!(summary.open_hole_count, 1);

    let h0_deaths = summary
        .intervals
        .iter()
        .filter(|interval| interval.dimension == 0)
        .filter_map(|interval| interval.death_stage_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(h0_deaths, vec![id("stage-edge-ab"), id("stage-edge-bc")]);
    let h1 = summary
        .intervals
        .iter()
        .find(|interval| interval.dimension == 1)
        .expect("h1 interval");
    assert_eq!(h1.birth_stage_id, id("stage-cycle"));
    assert_eq!(h1.generator_cell_ids, vec![id("edge-ca")]);
    assert!(h1.is_open());
}

#[test]
fn filtration_kills_hole_when_filling_face_appears() {
    let (store, complex_id) = filled_triangle_store();
    let stages = filled_triangle_filtration_stages();

    let summary = summarize_filtration(&store, &complex_id, &stages).expect("summarize");

    let hole_counts = summary
        .stages
        .iter()
        .map(|stage| stage.topology.simple_hole_count)
        .collect::<Vec<_>>();
    assert_eq!(hole_counts, vec![0, 0, 0, 1, 0]);
    assert_eq!(summary.open_hole_count, 0);

    let h1 = summary
        .intervals
        .iter()
        .find(|interval| interval.dimension == 1)
        .expect("h1 interval");
    assert_eq!(h1.birth_stage_id, id("stage-cycle"));
    assert_eq!(h1.death_stage_id, Some(id("stage-face")));
    assert_eq!(h1.generator_cell_ids, vec![id("edge-ca")]);
    assert!(!h1.is_open());
}

#[test]
fn summarizes_two_sphere_second_homology() {
    let (store, complex_id) = tetrahedron_surface_store();

    let summary = summarize_complex(&store, &complex_id).expect("summarize sphere");

    assert_eq!(summary.homology.betti_number(0), 1);
    assert_eq!(summary.homology.betti_number(1), 0);
    assert_eq!(summary.homology.betti_number(2), 1);
    assert_eq!(summary.homology.euler_characteristic, 2);
    assert_eq!(summary.first_betti_number, 0);
    assert!(summary.findings.is_empty());
}

#[test]
fn persistence_threshold_filters_short_lived_intervals() {
    let (store, complex_id) = triangle_1_skeleton_store();
    let stages = triangle_filtration_stages();

    let summary = summarize_filtration_with_options(
        &store,
        &complex_id,
        &stages,
        PersistenceOptions::new().with_min_lifetime_stages(2),
    )
    .expect("summarize with threshold");

    assert_eq!(summary.intervals.len(), 4);
    assert_eq!(summary.persistent_intervals.len(), 2);
    assert!(summary
        .persistent_intervals
        .iter()
        .all(|interval| interval.dimension == 0));
}

#[test]
fn rejects_malformed_filtrations() {
    let (store, complex_id) = triangle_1_skeleton_store();

    let boundary_error = summarize_filtration(
        &store,
        &complex_id,
        &[FiltrationStage::new(id("stage-edge"), [id("edge-ab")])],
    )
    .expect_err("edge before boundary should fail");
    assert_eq!(boundary_error.code(), "malformed_field");

    let non_cumulative_error = summarize_filtration(
        &store,
        &complex_id,
        &[
            FiltrationStage::new(id("stage-a"), [id("point-a"), id("point-b")]),
            FiltrationStage::new(id("stage-b"), [id("point-a")]),
        ],
    )
    .expect_err("non-cumulative stage should fail");
    assert_eq!(non_cumulative_error.code(), "malformed_field");
}

fn tetrahedron_surface_store() -> (InMemorySpaceStore, Id) {
    let mut store = seeded_store();
    for point in ["point-a", "point-b", "point-c", "point-d"] {
        store
            .insert_cell(Cell::new(id(point), id("space-a"), 0, "point"))
            .expect("insert point");
    }
    insert_edge(&mut store, "edge-ab", "point-a", "point-b");
    insert_edge(&mut store, "edge-ac", "point-a", "point-c");
    insert_edge(&mut store, "edge-ad", "point-a", "point-d");
    insert_edge(&mut store, "edge-bc", "point-b", "point-c");
    insert_edge(&mut store, "edge-bd", "point-b", "point-d");
    insert_edge(&mut store, "edge-cd", "point-c", "point-d");
    insert_face(&mut store, "face-abc", ["edge-ab", "edge-ac", "edge-bc"]);
    insert_face(&mut store, "face-abd", ["edge-ab", "edge-ad", "edge-bd"]);
    insert_face(&mut store, "face-acd", ["edge-ac", "edge-ad", "edge-cd"]);
    insert_face(&mut store, "face-bcd", ["edge-bc", "edge-bd", "edge-cd"]);

    let complex = store
        .construct_complex(
            id("complex-tetrahedron-surface"),
            id("space-a"),
            "Tetrahedron surface",
            ComplexType::SimplicialComplex,
            [
                id("point-a"),
                id("point-b"),
                id("point-c"),
                id("point-d"),
                id("edge-ab"),
                id("edge-ac"),
                id("edge-ad"),
                id("edge-bc"),
                id("edge-bd"),
                id("edge-cd"),
                id("face-abc"),
                id("face-abd"),
                id("face-acd"),
                id("face-bcd"),
            ],
            [],
        )
        .expect("construct tetrahedron surface");

    (store, complex.id)
}

fn insert_edge(store: &mut InMemorySpaceStore, edge_id: &str, left: &str, right: &str) {
    store
        .insert_cell(
            Cell::new(id(edge_id), id("space-a"), 1, "edge")
                .with_boundary_cell(id(left))
                .with_boundary_cell(id(right)),
        )
        .expect("insert edge");
}

fn insert_face(store: &mut InMemorySpaceStore, face_id: &str, edges: [&str; 3]) {
    let face = edges.into_iter().fold(
        Cell::new(id(face_id), id("space-a"), 2, "face"),
        |cell, edge_id| cell.with_boundary_cell(id(edge_id)),
    );
    store.insert_cell(face).expect("insert face");
}
