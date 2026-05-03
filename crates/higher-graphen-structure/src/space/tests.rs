use super::*;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid test id")
}

#[test]
fn greedy_coverage_selects_candidates_until_universe_is_covered() {
    let selection =
        GreedyCoverageSelector::new([id("element:alpha"), id("element:beta"), id("element:gamma")])
            .with_candidates([
                CoverageCandidate::new(
                    id("candidate:one"),
                    [id("element:alpha"), id("element:beta")],
                )
                .with_priority(1),
                CoverageCandidate::new(
                    id("candidate:two"),
                    [id("element:beta"), id("element:gamma")],
                )
                .with_priority(1),
                CoverageCandidate::new(id("candidate:three"), [id("element:gamma")])
                    .with_priority(9),
            ])
            .select();

    assert_eq!(
        selection.selected_ids,
        vec![id("candidate:one"), id("candidate:three")]
    );
    assert_eq!(
        selection.covered_ids,
        vec![id("element:alpha"), id("element:beta"), id("element:gamma")]
    );
    assert!(selection.uncovered_ids.is_empty());
}

#[test]
fn greedy_coverage_reports_uncovered_universe_under_budget() {
    let selection =
        GreedyCoverageSelector::new([id("element:alpha"), id("element:beta"), id("element:gamma")])
            .with_candidates([
                CoverageCandidate::new(id("candidate:one"), [id("element:alpha")]),
                CoverageCandidate::new(id("candidate:two"), [id("element:beta")]),
            ])
            .with_budget(1)
            .select();

    assert_eq!(selection.selected_ids, vec![id("candidate:one")]);
    assert_eq!(selection.covered_ids, vec![id("element:alpha")]);
    assert_eq!(
        selection.uncovered_ids,
        vec![id("element:beta"), id("element:gamma")]
    );
}

#[test]
fn weighted_coverage_prefers_higher_weighted_uncovered_elements() {
    let selection = WeightedCoverageSelector::new([
        WeightedUniverseElement::new(id("element:alpha"), 10),
        WeightedUniverseElement::new(id("element:beta"), 1),
        WeightedUniverseElement::new(id("element:gamma"), 1),
    ])
    .with_candidates([
        CoverageCandidate::new(
            id("candidate:one"),
            [id("element:beta"), id("element:gamma")],
        ),
        CoverageCandidate::new(id("candidate:two"), [id("element:alpha")]),
    ])
    .select();

    assert_eq!(
        selection.selected_ids,
        vec![id("candidate:two"), id("candidate:one")]
    );
    assert_eq!(selection.covered_weight, 12);
    assert_eq!(selection.uncovered_weight, 0);
}

#[test]
fn dominance_analysis_reports_candidates_with_subset_coverage_and_no_better_profile() {
    let report = DominanceAnalysis::new([
        CoverageCandidate::new(
            id("candidate:broad"),
            [id("element:alpha"), id("element:beta")],
        )
        .with_priority(3)
        .with_cost(1),
        CoverageCandidate::new(id("candidate:narrow"), [id("element:alpha")])
            .with_priority(2)
            .with_cost(1),
        CoverageCandidate::new(id("candidate:expensive"), [id("element:alpha")])
            .with_priority(2)
            .with_cost(5),
    ])
    .analyze();

    assert_eq!(
        report.dominated_ids,
        vec![id("candidate:expensive"), id("candidate:narrow")]
    );
    assert!(report.relations.iter().any(|relation| {
        relation.dominant_id == id("candidate:broad")
            && relation.dominated_id == id("candidate:narrow")
            && relation.covered_ids == vec![id("element:alpha")]
    }));
    assert!(report.relations.iter().any(|relation| {
        relation.dominant_id == id("candidate:narrow")
            && relation.dominated_id == id("candidate:expensive")
    }));
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
fn complex_boundary_coboundary_and_closure_report_external_cells() {
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
            Cell::new(id("face-alpha"), id("space-a"), 2, "face").with_boundary_cell(id("edge-ab")),
        )
        .expect("insert face");

    let complex = store
        .construct_complex(
            id("complex-a"),
            id("space-a"),
            "Partial cell complex",
            ComplexType::CellComplex,
            [id("point-a"), id("edge-ab")],
            [],
        )
        .expect("construct partial complex");

    let closure = store.complex_closure(&complex.id).expect("compute closure");
    assert_eq!(
        closure.cell_ids,
        vec![id("edge-ab"), id("point-a"), id("point-b")]
    );

    let validation = store
        .validate_complex_closure(&complex.id)
        .expect("validate closure");
    assert!(!validation.is_closed());
    assert_eq!(validation.missing_boundary_cell_ids, vec![id("point-b")]);
    assert_eq!(
        validation.violations,
        vec![ComplexClosureViolation {
            cell_id: id("edge-ab"),
            missing_boundary_cell_ids: vec![id("point-b")]
        }]
    );

    let boundary = store
        .complex_boundary(&complex.id)
        .expect("compute boundary");
    assert_eq!(boundary.cell_ids, vec![id("point-a")]);
    assert_eq!(boundary.external_cell_ids, vec![id("point-b")]);

    let coboundary = store
        .complex_coboundary(&complex.id)
        .expect("compute coboundary");
    assert_eq!(coboundary.cell_ids, vec![id("edge-ab")]);
    assert_eq!(coboundary.external_cell_ids, vec![id("face-alpha")]);
}

#[test]
fn complex_neighborhood_returns_closed_star_and_link_shell() {
    let (store, complex_id) = triangle_complex_store();

    let neighborhood = store
        .complex_neighborhood(&complex_id, [id("point-a")])
        .expect("compute neighborhood");

    assert_eq!(neighborhood.seed_cell_ids, vec![id("point-a")]);
    assert_eq!(neighborhood.seed_closure_cell_ids, vec![id("point-a")]);
    assert_eq!(
        neighborhood.coface_cell_ids,
        vec![id("edge-ab"), id("edge-ca"), id("face-abc"), id("point-a")]
    );
    assert_eq!(
        neighborhood.star_cell_ids,
        vec![
            id("edge-ab"),
            id("edge-bc"),
            id("edge-ca"),
            id("face-abc"),
            id("point-a"),
            id("point-b"),
            id("point-c")
        ]
    );
    assert_eq!(
        neighborhood.link_cell_ids,
        vec![id("edge-bc"), id("point-b"), id("point-c")]
    );
}

#[test]
fn covered_region_expands_cells_to_boundary_closure_and_reports_gaps() {
    let (mut store, complex_id) = triangle_complex_store();
    store
        .insert_cell(Cell::new(id("point-d"), id("space-a"), 0, "point"))
        .expect("insert external point");
    store
        .insert_cell(
            Cell::new(id("edge-outside"), id("space-a"), 1, "edge")
                .with_boundary_cell(id("point-d")),
        )
        .expect("insert external edge");

    let complete = store
        .covered_region(&complex_id, [id("face-abc")])
        .expect("compute complete coverage");
    assert!(complete.is_complete());
    assert_eq!(
        complete.covered_cell_ids,
        vec![
            id("edge-ab"),
            id("edge-bc"),
            id("edge-ca"),
            id("face-abc"),
            id("point-a"),
            id("point-b"),
            id("point-c")
        ]
    );
    assert!(complete.uncovered_cell_ids.is_empty());

    let partial = store
        .covered_region(
            &complex_id,
            [id("edge-ab"), id("edge-ab"), id("edge-outside")],
        )
        .expect("compute partial coverage");
    assert!(!partial.is_complete());
    assert_eq!(
        partial.requested_cell_ids,
        vec![id("edge-ab"), id("edge-outside")]
    );
    assert_eq!(partial.duplicate_cell_ids, vec![id("edge-ab")]);
    assert_eq!(partial.external_cell_ids, vec![id("edge-outside")]);
    assert_eq!(
        partial.covered_cell_ids,
        vec![id("edge-ab"), id("point-a"), id("point-b")]
    );
    assert_eq!(
        partial.uncovered_cell_ids,
        vec![id("edge-bc"), id("edge-ca"), id("face-abc"), id("point-c")]
    );
    assert_eq!(
        store
            .uncovered_region(&complex_id, [id("edge-ab")])
            .expect("compute uncovered region"),
        partial.uncovered_cell_ids
    );
}

#[test]
fn cell_insertion_rejects_non_lower_dimensional_boundary() {
    let mut store = seeded_store();
    store
        .insert_cell(Cell::new(id("point-a"), id("space-a"), 0, "point"))
        .expect("insert point");

    let error = store
        .insert_cell(
            Cell::new(id("point-b"), id("space-a"), 0, "point").with_boundary_cell(id("point-a")),
        )
        .expect_err("same-dimension boundary should fail");

    assert_eq!(error.code(), "malformed_field");
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
fn find_simple_cycles_reports_directed_three_node_cycle() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "depends_on");
    insert_edge(&mut store, "incidence-ca", "cell-c", "cell-a", "depends_on");

    let cycles = store
        .find_simple_cycles(&id("space-a"), &CycleSearchOptions::new())
        .expect("cycle search should run");

    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].witness_edge_id, id("incidence-ca"));
    assert_eq!(
        cycles[0].vertex_cell_ids,
        vec![id("cell-a"), id("cell-b"), id("cell-c")]
    );
    assert_eq!(
        cycles[0].edge_cell_ids,
        vec![id("incidence-ab"), id("incidence-bc"), id("incidence-ca")]
    );
}

#[test]
fn find_simple_cycles_returns_empty_for_dag() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "depends_on");
    insert_edge(&mut store, "incidence-ad", "cell-a", "cell-d", "depends_on");

    let cycles = store
        .find_simple_cycles(&id("space-a"), &CycleSearchOptions::new())
        .expect("cycle search should run");

    assert!(cycles.is_empty());
}

#[test]
fn find_simple_cycles_returns_deterministic_multi_cycle_output() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "relates");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "relates");
    insert_edge(&mut store, "incidence-ca", "cell-c", "cell-a", "relates");
    insert_edge(&mut store, "incidence-ad", "cell-a", "cell-d", "relates");
    insert_edge(&mut store, "incidence-da", "cell-d", "cell-a", "relates");

    let cycles = store
        .find_simple_cycles(&id("space-a"), &CycleSearchOptions::new())
        .expect("cycle search should run");

    let vertex_cycles: Vec<Vec<Id>> = cycles
        .iter()
        .map(|cycle| cycle.vertex_cell_ids.clone())
        .collect();
    assert_eq!(
        vertex_cycles,
        vec![
            vec![id("cell-a"), id("cell-b"), id("cell-c")],
            vec![id("cell-a"), id("cell-d")]
        ]
    );
}

#[test]
fn find_simple_cycles_detects_disjoint_cycles_once_each() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "relates");
    insert_edge(&mut store, "incidence-ba", "cell-b", "cell-a", "relates");
    insert_edge(&mut store, "incidence-cd", "cell-c", "cell-d", "relates");
    insert_edge(&mut store, "incidence-dc", "cell-d", "cell-c", "relates");

    let cycles = store
        .find_simple_cycles(&id("space-a"), &CycleSearchOptions::new())
        .expect("cycle search should run");

    let vertex_cycles: Vec<Vec<Id>> = cycles
        .iter()
        .map(|cycle| cycle.vertex_cell_ids.clone())
        .collect();
    assert_eq!(
        vertex_cycles,
        vec![
            vec![id("cell-a"), id("cell-b")],
            vec![id("cell-c"), id("cell-d")]
        ]
    );
}

#[test]
fn find_simple_cycles_respects_relation_filter() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "depends_on");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "depends_on");
    insert_edge(&mut store, "incidence-ca", "cell-c", "cell-a", "test_only");

    let unfiltered = store
        .find_simple_cycles(&id("space-a"), &CycleSearchOptions::new())
        .expect("cycle search should run");
    let excluded = store
        .find_simple_cycles(
            &id("space-a"),
            &CycleSearchOptions::new().with_relation_type("depends_on"),
        )
        .expect("cycle search should run");

    assert_eq!(unfiltered.len(), 1);
    assert!(excluded.is_empty());
}

#[test]
fn find_simple_cycles_respects_cycle_count_and_length_bounds() {
    let mut store = layered_store();
    insert_edge(&mut store, "incidence-ab", "cell-a", "cell-b", "relates");
    insert_edge(&mut store, "incidence-bc", "cell-b", "cell-c", "relates");
    insert_edge(&mut store, "incidence-ca", "cell-c", "cell-a", "relates");
    insert_edge(&mut store, "incidence-ad", "cell-a", "cell-d", "relates");
    insert_edge(&mut store, "incidence-da", "cell-d", "cell-a", "relates");

    let limited_count = store
        .find_simple_cycles(
            &id("space-a"),
            &CycleSearchOptions::new().with_max_cycles(1),
        )
        .expect("cycle search should run");
    let limited_length = store
        .find_simple_cycles(
            &id("space-a"),
            &CycleSearchOptions::new().with_max_path_length(2),
        )
        .expect("cycle search should run");

    assert_eq!(limited_count.len(), 1);
    assert_eq!(
        limited_length
            .iter()
            .map(|cycle| cycle.vertex_cell_ids.clone())
            .collect::<Vec<_>>(),
        vec![vec![id("cell-a"), id("cell-d")]]
    );
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

fn triangle_complex_store() -> (InMemorySpaceStore, Id) {
    let mut store = seeded_store();
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
    store
        .insert_cell(
            Cell::new(id("face-abc"), id("space-a"), 2, "face")
                .with_boundary_cell(id("edge-ab"))
                .with_boundary_cell(id("edge-bc"))
                .with_boundary_cell(id("edge-ca")),
        )
        .expect("insert face");
    let complex = store
        .construct_complex(
            id("complex-triangle"),
            id("space-a"),
            "Triangle",
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
        .expect("construct triangle complex");

    (store, complex.id)
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
