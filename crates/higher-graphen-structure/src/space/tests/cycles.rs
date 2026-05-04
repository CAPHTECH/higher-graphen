use super::super::*;
use super::{id, insert_edge, layered_store};

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
