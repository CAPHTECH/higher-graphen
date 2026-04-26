use super::*;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid test id")
}

fn seeded_store() -> InMemorySpaceStore {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space-a"), "Abstract space"))
        .expect("insert space");
    store
}

#[test]
fn inserts_cells_and_queries_by_space_type_dimension_and_context() {
    let mut store = seeded_store();
    let context_id = id("context-alpha");
    let cell = Cell::new(id("cell-1"), id("space-a"), 0, "entity")
        .with_label("Entity")
        .with_context(context_id.clone());
    store.insert_cell(cell).expect("insert cell");

    let query = CellQuery::new()
        .in_space(id("space-a"))
        .of_type("entity")
        .with_dimension(0)
        .in_context(context_id.clone());
    let results = store.query_cells(&query);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, id("cell-1"));
    assert_eq!(
        store.space(&id("space-a")).expect("space").context_ids,
        vec![context_id]
    );
}

#[test]
fn space_insertion_rejects_prepopulated_store_owned_memberships() {
    let mut store = InMemorySpaceStore::new();
    let mut space = Space::new(id("space-a"), "Abstract space");
    space.cell_ids.push(id("dangling-cell"));

    let error = store
        .insert_space(space)
        .expect_err("prepopulated cell membership should fail");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn cell_insertion_updates_boundary_coboundary_membership() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert boundary cell");
    let edge = Cell::new(id("edge-a"), id("space-a"), 1, "edge").with_boundary_cell(id("point-a"));
    store.insert_cell(edge).expect("insert edge");

    let point = store.cell(&id("point-a")).expect("point cell");
    let edge = store.cell(&id("edge-a")).expect("edge cell");
    assert_eq!(point.coboundary, vec![id("edge-a")]);
    assert_eq!(edge.boundary, vec![id("point-a")]);
}

#[test]
fn cell_insertion_updates_coboundary_boundary_membership() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("edge-a"), id("space-a"), 1, "edge"))
        .expect("insert coboundary cell");
    let point = Cell::new(id("point-a"), id("space-a"), 0, "point");
    let mut point = point;
    point.coboundary.push(id("edge-a"));

    store.insert_cell(point).expect("insert point");

    let point = store.cell(&id("point-a")).expect("point cell");
    let edge = store.cell(&id("edge-a")).expect("edge cell");
    assert_eq!(point.coboundary, vec![id("edge-a")]);
    assert_eq!(edge.boundary, vec![id("point-a")]);
}

#[test]
fn incidence_insertion_requires_cells_from_the_same_space() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("source"), id("space-a"), 0, "node"))
        .expect("insert source");
    store
        .insert_cell(Cell::new(id("target"), id("space-a"), 0, "node"))
        .expect("insert target");

    let incidence = Incidence::new(
        id("incidence-a"),
        id("space-a"),
        id("source"),
        id("target"),
        "relates",
        IncidenceOrientation::Directed,
    )
    .with_weight(1.0);
    store.insert_incidence(incidence).expect("insert incidence");

    let space = store.space(&id("space-a")).expect("space");
    assert_eq!(space.incidence_ids, vec![id("incidence-a")]);
    assert!(store.incidence(&id("incidence-a")).is_some());
}

#[test]
fn incidence_insertion_rejects_non_finite_weight() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("source"), id("space-a"), 0, "node"))
        .expect("insert source");
    store
        .insert_cell(Cell::new(id("target"), id("space-a"), 0, "node"))
        .expect("insert target");

    for weight in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = store
            .insert_incidence(
                Incidence::new(
                    Id::new(format!("incidence-{weight}")).expect("valid incidence id"),
                    id("space-a"),
                    id("source"),
                    id("target"),
                    "relates",
                    IncidenceOrientation::Directed,
                )
                .with_weight(weight),
            )
            .expect_err("non-finite weight should fail");
        assert_eq!(error.code(), "malformed_field");
    }
}

#[test]
fn complex_construction_validates_membership_and_computes_max_dimension() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert point");
    store
        .insert_cell(Cell::new(id("edge-a"), id("space-a"), 1, "edge"))
        .expect("insert edge");
    store
        .insert_incidence(Incidence::new(
            id("incidence-a"),
            id("space-a"),
            id("point-a"),
            id("edge-a"),
            "bounds",
            IncidenceOrientation::Directed,
        ))
        .expect("insert incidence");

    let complex = store
        .construct_complex(
            id("complex-a"),
            id("space-a"),
            "Typed graph",
            ComplexType::TypedGraph,
            [id("point-a"), id("edge-a")],
            [id("incidence-a")],
        )
        .expect("construct complex");

    assert_eq!(complex.max_dimension, 1);
    assert_eq!(
        store.space(&id("space-a")).expect("space").complex_ids,
        vec![id("complex-a")]
    );
}

#[test]
fn complex_rejects_incidence_when_endpoint_cells_are_not_included() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert point");
    store
        .insert_cell(Cell::new(id("edge-a"), id("space-a"), 1, "edge"))
        .expect("insert edge");
    store
        .insert_incidence(Incidence::new(
            id("incidence-a"),
            id("space-a"),
            id("point-a"),
            id("edge-a"),
            "bounds",
            IncidenceOrientation::Directed,
        ))
        .expect("insert incidence");

    let error = store
        .construct_complex(
            id("complex-a"),
            id("space-a"),
            "Incomplete complex",
            ComplexType::TypedGraph,
            [id("point-a")],
            [id("incidence-a")],
        )
        .expect_err("incidence endpoint outside cell_ids should fail");

    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn custom_complex_type_is_normalized_and_rejects_empty_extension() {
    let mut store = seeded_store();
    let custom = ComplexType::custom("  domain_shape  ").expect("custom complex type");
    let complex = store
        .construct_complex(
            id("complex-a"),
            id("space-a"),
            "Custom complex",
            custom,
            [],
            [],
        )
        .expect("insert custom complex");

    assert_eq!(
        complex.complex_type,
        ComplexType::Custom("domain_shape".to_owned())
    );
    assert_eq!(
        ComplexType::custom("   ")
            .expect_err("empty custom complex type")
            .code(),
        "malformed_field"
    );

    let error = store
        .insert_complex(Complex::new(
            id("complex-b"),
            id("space-a"),
            "Invalid custom complex",
            ComplexType::Custom("   ".to_owned()),
        ))
        .expect_err("empty custom complex type should fail");
    assert_eq!(error.code(), "malformed_field");

    let direct_custom = store
        .insert_complex(Complex::new(
            id("complex-c"),
            id("space-a"),
            "Direct custom complex",
            ComplexType::Custom("  direct_domain_shape  ".to_owned()),
        ))
        .expect("insert directly constructed custom complex");
    assert_eq!(
        direct_custom.complex_type,
        ComplexType::Custom("direct_domain_shape".to_owned())
    );
}

#[test]
fn reachability_finds_shortest_directed_path_with_witness() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "depends_on");

    let result = store
        .reachable(&ReachabilityQuery::new(
            id("space-a"),
            id("cell-a"),
            id("cell-c"),
        ))
        .expect("reachability should run");

    let path = result.shortest_path.expect("reachable path");
    assert!(result.reachable);
    assert_eq!(
        path.cell_ids(),
        vec![id("cell-a"), id("cell-b"), id("cell-c")]
    );
    assert_eq!(path.steps[0].relation_type, "depends_on");
}

#[test]
fn reachability_reports_frontier_when_depth_blocks_target() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "depends_on");
    let query = ReachabilityQuery::new(id("space-a"), id("cell-a"), id("cell-c")).with_options(
        TraversalOptions::new()
            .with_relation_type("depends_on")
            .with_max_depth(1),
    );

    let result = store.reachable(&query).expect("reachability should run");

    assert!(!result.reachable);
    assert_eq!(result.visited_cell_ids, vec![id("cell-a"), id("cell-b")]);
    assert_eq!(result.frontier_cell_ids, vec![id("cell-b")]);
}

#[test]
fn traversal_direction_controls_directed_incidences() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    let outgoing = ReachabilityQuery::new(id("space-a"), id("cell-b"), id("cell-a"));
    let incoming = outgoing
        .clone()
        .with_options(TraversalOptions::new().in_direction(TraversalDirection::Incoming));

    assert!(!store.reachable(&outgoing).expect("outgoing run").reachable);
    assert!(store.reachable(&incoming).expect("incoming run").reachable);
}

#[test]
fn walk_paths_returns_bounded_simple_paths() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "relates");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "relates");
    insert_edge(&mut store, "incidence-ad", "cell-a", "cell-d", "relates");
    insert_edge(&mut store, "incidence-dc", "cell-d", "cell-c", "relates");
    insert_edge(&mut store, "incidence-ba", "cell-b", "cell-a", "relates");
    let query = ReachabilityQuery::new(id("space-a"), id("cell-a"), id("cell-c"))
        .with_options(TraversalOptions::new().with_max_depth(2).with_max_paths(4));

    let paths = store.walk_paths(&query).expect("path walking should run");
    let cell_paths: Vec<Vec<Id>> = paths.iter().map(GraphPath::cell_ids).collect();

    assert_eq!(cell_paths.len(), 2);
    assert!(cell_paths.contains(&vec![id("cell-a"), id("cell-b"), id("cell-c")]));
    assert!(cell_paths.contains(&vec![id("cell-a"), id("cell-d"), id("cell-c")]));
}

#[test]
fn path_pattern_matches_layer_chain() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "maps_to");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "implements");
    let pattern = PathPattern::new(
        id("space-a"),
        CellPattern::any().of_type("layer.requirement"),
    )
    .then(
        PathPatternSegment::new(CellPattern::any().of_type("layer.design"))
            .with_relation_type("maps_to"),
    )
    .then(
        PathPatternSegment::new(CellPattern::any().of_type("layer.implementation"))
            .with_relation_type("implements"),
    );

    let matches = store
        .matches_path_pattern(&pattern)
        .expect("pattern matching should run");

    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0].matched_cell_ids,
        vec![id("cell-a"), id("cell-b"), id("cell-c")]
    );
}

#[test]
fn path_pattern_rejects_empty_segments() {
    let store = layered_store();
    let pattern = PathPattern::new(id("space-a"), CellPattern::any());

    let error = store
        .matches_path_pattern(&pattern)
        .expect_err("empty pattern should fail");

    assert_eq!(error.code(), "malformed_field");
}

fn layered_store() -> InMemorySpaceStore {
    let mut store = seeded_store();
    insert_cell(&mut store, "cell-a", "layer.requirement");
    insert_cell(&mut store, "cell-b", "layer.design");
    insert_cell(&mut store, "cell-c", "layer.implementation");
    insert_cell(&mut store, "cell-d", "layer.test");
    store
}

fn insert_cell(store: &mut InMemorySpaceStore, cell_id: &str, cell_type: &str) {
    store
        .insert_cell(Cell::new(id(cell_id), id("space-a"), 0, cell_type))
        .expect("insert test cell");
}

fn insert_edge(
    store: &mut InMemorySpaceStore,
    incidence_id: &str,
    from_cell_id: &str,
    to_cell_id: &str,
    relation_type: &str,
) {
    store
        .insert_incidence(Incidence::new(
            id(incidence_id),
            id("space-a"),
            id(from_cell_id),
            id(to_cell_id),
            relation_type,
            IncidenceOrientation::Directed,
        ))
        .expect("insert test incidence");
}

#[test]
fn invalid_references_return_core_malformed_field_errors() {
    let mut store = seeded_store();
    let error = store
        .insert_cell(Cell::new(id("orphan"), id("missing-space"), 0, "node"))
        .expect_err("missing space should fail");

    assert_eq!(error.code(), "malformed_field");
}
